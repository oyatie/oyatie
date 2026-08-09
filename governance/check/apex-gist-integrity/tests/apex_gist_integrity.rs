//! Live-tree caller for the apex gist integrity gate.
//!
//! The kernel is pure; this walks the REAL apex and archive corpus and hands it observations as
//! DATA. Parse failures are ERRORS, never omitted observations: an apex dropped from the census
//! because its frontmatter failed to parse would quietly shrink every finding count, and the
//! counts are pinned by equality, so a silent skip reads as a repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use check_apex_gist_integrity::{
    ApexDoc, Block, CODE_MEMBER_WITHOUT_GIST, CODE_MEMBER_WITHOUT_RESIDUAL, CODE_TOPIC_DROPPED,
    CODE_TRUNCATED_MIDWORD, CODE_UNBALANCED_TITLE, CODE_UNCLOSED_FENCE, CODE_VACUOUS_SCAN, Policy,
    Site, Topic, Verdict, evaluate, normalize_id,
};

const POLICY_PATH: &str = "governance/check/apex-gist-integrity/apex-gist-integrity-policy.json";
const APEX_DIR: &str = "docs/decisions";
const ARCHIVE_DIR: &str = "docs/adr-archive";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {POLICY_PATH})");
}

struct Config {
    policy: Policy,
    topics: Vec<Topic>,
    frozen: BTreeMap<String, usize>,
    census: BTreeMap<String, usize>,
}

fn load_config(root: &Path) -> Config {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let num = |v: &serde_json::Value, key: &str| -> usize {
        usize::try_from(
            v[key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy field {key} missing or not a number")),
        )
        .expect("fits usize")
    };
    let topics = doc["topics"]
        .as_array()
        .expect("policy.topics is an array")
        .iter()
        .map(|t| Topic {
            name: t["name"].as_str().expect("topic.name").to_owned(),
            needles: t["needles"]
                .as_array()
                .expect("topic.needles")
                .iter()
                .map(|n| n.as_str().expect("needle is a string").to_lowercase())
                .collect(),
        })
        .collect();
    let measured = &doc["measured"];
    let grab = |keys: &[&str]| -> BTreeMap<String, usize> {
        keys.iter()
            .map(|k| ((*k).to_owned(), num(measured, k)))
            .collect()
    };
    Config {
        policy: Policy {
            min_live_apexes: num(&doc, "min_live_apexes"),
            min_members: num(&doc, "min_members"),
            min_blocks: num(&doc, "min_blocks"),
            min_archived_members_read: num(&doc, "min_archived_members_read"),
            min_topics: num(&doc, "min_topics"),
        },
        topics,
        frozen: grab(&[
            CODE_TRUNCATED_MIDWORD,
            CODE_UNCLOSED_FENCE,
            CODE_UNBALANCED_TITLE,
            CODE_MEMBER_WITHOUT_GIST,
            CODE_MEMBER_WITHOUT_RESIDUAL,
            CODE_TOPIC_DROPPED,
            CODE_VACUOUS_SCAN,
        ]),
        census: grab(&["live_apexes", "members", "blocks", "archived_members_read"]),
    }
}

/// The single-line `supersedes: [ADR-0011, ADR-0017]` frontmatter list.
///
/// Returns None only when the key is absent. A key that is present but unparseable PANICS rather
/// than yielding an empty list, because an empty list makes every member-coverage finding vanish.
fn parse_supersedes(front: &str) -> Vec<String> {
    for line in front.lines() {
        let Some(rest) = line.strip_prefix("supersedes:") else {
            continue;
        };
        let rest = rest.trim();
        let inner = rest
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .unwrap_or_else(|| {
                panic!("supersedes: is not a single-line [..] list, refusing to guess: {rest}")
            });
        return inner
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                normalize_id(s).unwrap_or_else(|| panic!("supersedes entry not an ADR id: {s}"))
            })
            .collect();
    }
    Vec::new()
}

/// Strip `- **ADR-N** (title): `, returning the member id, the carried text, and whether the
/// title's parentheses were left OPEN.
///
/// Balanced matching is the primary path, and splitting on the first `): ` would be wrong for the
/// well-formed majority: real titles contain parentheses, e.g. "Rename `application` layer to
/// `usecase` (amends ADR-0105)", and the first `): ` lands inside the title. That would make the
/// observed text shorter than reality and change which blocks look truncated — the gate would be
/// measuring its own parse bug.
///
/// The FALLBACK exists because balanced matching does not terminate on 22 real bullets whose title
/// was itself cut mid-parenthesis. Returning None for those was the first version of this function
/// and it was WRONG in the most dangerous available direction: the blocks vanished from the census,
/// their body truncations went uncounted, and the resulting number was smaller and looked like a
/// healthier corpus. A gate that drops what it cannot parse reports its own blind spot as a repair.
/// So the text is recovered at the first `): ` — correct for exactly this shape, since the
/// unbalanced `(` always precedes it — and the block is reported as a defect in its own right.
fn strip_gist_leadin(line: &str) -> Option<(String, String, bool)> {
    let rest = line.strip_prefix("- **ADR-")?;
    let close = rest.find("**")?;
    let id = normalize_id(&rest[..close])?;
    let after = rest[close + 2..].trim_start();
    let mut chars = after.char_indices();
    if chars.next()?.1 != '(' {
        return None;
    }
    let mut depth = 1usize;
    for (i, c) in chars {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let tail = after[i + 1..].strip_prefix(':')?;
                    return Some((id, tail.trim().to_owned(), false));
                }
            }
            _ => {}
        }
    }
    let cut = after.find("): ")?;
    Some((id, after[cut + "): ".len()..].trim().to_owned(), true))
}

fn read_apexes(root: &Path) -> Vec<ApexDoc> {
    let dir = root.join(APEX_DIR);
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read docs/decisions")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ADR-07") && n.ends_with("-live-apex.md"))
        })
        .collect();
    entries.sort();

    entries
        .iter()
        .map(|path| {
            let text = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .into_owned();
            let name = path.file_name().and_then(|n| n.to_str()).expect("name");
            let id = normalize_id(name.strip_prefix("ADR-").unwrap_or("").get(..4).unwrap_or(""))
                .unwrap_or_else(|| panic!("apex id not parseable from {name}"));

            let front = text.split("---").nth(1).unwrap_or_else(|| {
                panic!("{} has no --- delimited frontmatter", path.display())
            });
            let supersedes = parse_supersedes(front);
            assert!(
                !supersedes.is_empty(),
                "{} parsed an EMPTY supersedes list; an empty list silently clears every \
                 member-coverage finding, so it is treated as a parse failure",
                path.display()
            );

            let mut blocks = Vec::new();
            let mut pending_residual: Option<String> = None;
            for (idx, line) in text.lines().enumerate() {
                let line_no = idx + 1;
                if let Some((member, body, title_unbalanced)) = strip_gist_leadin(line) {
                    blocks.push(Block {
                        member_id: member,
                        site: Site::Gist,
                        line: line_no,
                        text: body,
                        title_unbalanced,
                    });
                    continue;
                }
                if let Some(rest) = line.strip_prefix("### ADR-") {
                    if let Some(head) = rest.strip_suffix(" residual") {
                        pending_residual = normalize_id(head);
                    }
                    continue;
                }
                if let Some(member) = pending_residual.clone() {
                    if line.starts_with("**") {
                        if let Some(pos) = line.find("** — ") {
                            blocks.push(Block {
                                member_id: member,
                                site: Site::Residual,
                                line: line_no,
                                text: line[pos + "** — ".len()..].trim().to_owned(),
                                title_unbalanced: false,
                            });
                            pending_residual = None;
                        }
                    }
                }
            }

            ApexDoc {
                id,
                path: rel,
                supersedes,
                blocks,
                body_lower: text.to_lowercase(),
            }
        })
        .collect()
}

fn read_archive(root: &Path) -> BTreeMap<String, String> {
    let dir = root.join(ARCHIVE_DIR);
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&dir).expect("read docs/adr-archive") {
        let path = entry.expect("dir entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("ADR-") || !name.ends_with(".md") {
            continue;
        }
        let digits: String = name["ADR-".len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let Some(id) = normalize_id(&digits) else {
            continue;
        };
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        out.insert(id, text.to_lowercase());
    }
    out
}

struct Observed {
    verdict: Verdict,
    census: BTreeMap<String, usize>,
}

fn observed() -> &'static Observed {
    static CELL: OnceLock<Observed> = OnceLock::new();
    CELL.get_or_init(|| {
        let root = repo_root();
        let cfg = load_config(&root);
        let apexes = read_apexes(&root);
        let archive = read_archive(&root);
        let census = BTreeMap::from([
            ("live_apexes".to_owned(), apexes.len()),
            (
                "members".to_owned(),
                apexes.iter().map(|a| a.supersedes.len()).sum(),
            ),
            (
                "blocks".to_owned(),
                apexes.iter().map(|a| a.blocks.len()).sum(),
            ),
            ("archived_members_read".to_owned(), archive.len()),
        ]);
        let verdict = evaluate(&apexes, &archive, &cfg.topics, &cfg.policy);
        Observed { verdict, census }
    })
}

/// The anti-vacuity anchor, asserted BEFORE any finding count.
///
/// A repaired corpus and a collapsed walk both report fewer findings; only the census tells them
/// apart. Pinned by EQUALITY for the same reason the sibling citation-closure gate pins its own:
/// a `<=` ceiling can be satisfied by narrowing the walk.
/// Compare an observed map against its frozen map, reporting EVERY mismatch in one failure.
///
/// Deliberately not a per-key `assert_eq!` in a loop. An abort-on-first-mismatch ratchet teaches
/// the author exactly ONE number per run, so re-freezing six values costs six builds — the CI
/// round-trip waste this gate exists inside a workflow to remove. Re-derive by RUNNING the gate
/// and reading these lines; never by arithmetic on the old values.
fn assert_frozen(observed: &BTreeMap<String, usize>, frozen: &BTreeMap<String, usize>, why: &str) {
    let drift: Vec<String> = frozen
        .iter()
        .filter_map(|(key, want)| {
            let seen = observed.get(key).copied().unwrap_or(0);
            (seen != *want).then(|| format!("  {key}: observed {seen}, frozen {want}"))
        })
        .collect();
    assert!(drift.is_empty(), "{why}\n{}", drift.join("\n"));
}

#[test]
fn census_equals_the_frozen_values() {
    let root = repo_root();
    let cfg = load_config(&root);
    assert_frozen(
        &observed().census,
        &cfg.census,
        "census drift — re-derive by RUNNING this gate and attribute the move in the same commit; \
         a narrowed walk and a genuine corpus change move this number the same way and only one \
         of them is legitimate:",
    );
}

#[test]
fn findings_equal_the_frozen_ceilings() {
    let root = repo_root();
    let cfg = load_config(&root);
    let seen: BTreeMap<String, usize> = cfg
        .frozen
        .keys()
        .map(|code| (code.clone(), observed().verdict.count(code)))
        .collect();
    assert_frozen(
        &seen,
        &cfg.frozen,
        "finding-count drift — a NEW truncation exceeds the ceiling and a REPAIRED one falls \
         below it; both fail, which forces the ceiling down in the same change that repairs the \
         corpus:",
    );
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the real corpus, not just the fixtures.
///
/// A green gate proves nothing on its own — this injects each defect shape into a copy of a REAL
/// apex observation and asserts the count rises. Without it, a parser that silently produced zero
/// blocks would pass every assertion above by reporting a perfectly clean corpus.
#[test]
fn injecting_each_defect_shape_into_the_real_corpus_reddens_the_gate() {
    let root = repo_root();
    let cfg = load_config(&root);
    let archive = read_archive(&root);
    let base = read_apexes(&root);
    let baseline = evaluate(&base, &archive, &cfg.topics, &cfg.policy);

    // 1. A block cut mid-word.
    let mut mutated = base.clone();
    mutated[0].blocks.push(Block {
        member_id: "999999".to_owned(),
        site: Site::Gist,
        line: 0,
        text: "this sentence stops inside a wor".to_owned(),
        title_unbalanced: false,
    });
    let v = evaluate(&mutated, &archive, &cfg.topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_TRUNCATED_MIDWORD),
        baseline.count(CODE_TRUNCATED_MIDWORD) + 1,
        "a mid-word cut injected into a real apex did not raise the count"
    );

    // 2. A block that leaves a fence open.
    let mut mutated = base.clone();
    mutated[0].blocks.push(Block {
        member_id: "999999".to_owned(),
        site: Site::Residual,
        line: 0,
        text: "example follows ```rust //".to_owned(),
        title_unbalanced: false,
    });
    let v = evaluate(&mutated, &archive, &cfg.topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_UNCLOSED_FENCE),
        baseline.count(CODE_UNCLOSED_FENCE) + 1,
        "an unclosed fence injected into a real apex did not raise the count"
    );

    // 3. A bullet whose title never closes its parentheses.
    let mut mutated = base.clone();
    mutated[0].blocks.push(Block {
        member_id: "999999".to_owned(),
        site: Site::Gist,
        line: 0,
        text: "the body here finishes normally.".to_owned(),
        title_unbalanced: true,
    });
    let v = evaluate(&mutated, &archive, &cfg.topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_UNBALANCED_TITLE),
        baseline.count(CODE_UNBALANCED_TITLE) + 1,
        "an unbalanced title injected into a real apex did not raise the count"
    );

    // 4. A member named in supersedes that the apex never carries.
    let mut mutated = base.clone();
    mutated[0].supersedes.push("999999".to_owned());
    let v = evaluate(&mutated, &archive, &cfg.topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_MEMBER_WITHOUT_GIST),
        baseline.count(CODE_MEMBER_WITHOUT_GIST) + 1
    );
    assert_eq!(
        v.count(CODE_MEMBER_WITHOUT_RESIDUAL),
        baseline.count(CODE_MEMBER_WITHOUT_RESIDUAL) + 1
    );

    // 5. A topic a member carries and the apex never mentions.
    let mut archive_plus = archive.clone();
    let victim = base[0].supersedes[0].clone();
    archive_plus.insert(
        victim,
        "this member decides everything about zzqqxx-substrate".to_owned(),
    );
    let mut topics = cfg.topics.clone();
    topics.push(Topic {
        name: "injected-probe".to_owned(),
        needles: vec!["zzqqxx-substrate".to_owned()],
    });
    let v = evaluate(&base, &archive_plus, &topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_TOPIC_DROPPED),
        baseline.count(CODE_TOPIC_DROPPED) + 1,
        "a topic carried only by a member did not register as dropped"
    );
}

/// The parser observes what is actually on disk, not an empty set.
///
/// This is the guard the census cannot give: the census counts blocks, but a lead-in stripper that
/// cut at the first `): ` would still yield the right NUMBER of blocks while making each one
/// shorter than reality.
#[test]
fn the_gist_leadin_stripper_survives_a_title_containing_parentheses() {
    let (id, body, unbalanced) = strip_gist_leadin(
        "- **ADR-106** (Rename `application` layer to `usecase` (amends ADR-0105)): Rename the \
         `application` layer to **`usecase`**.",
    )
    .expect("parses");
    assert_eq!(id, "106");
    assert!(
        body.starts_with("Rename the `application` layer"),
        "leadin stripper cut into the title: {body}"
    );
    assert!(!unbalanced, "a well-formed nested title is not an unbalanced one");

    // And the shape that forced the fallback to exist: a title cut mid-parenthesis. The block must
    // still yield its body — dropping it would shrink the census and read as a healthier corpus.
    let (id, body, unbalanced) = strip_gist_leadin(
        "- **ADR-213** (Ecosystem-as-a-Service architecture — Plugin/App Store substrate \
         (third-party de): Oyatie ships a plugin substrate.",
    )
    .expect("recovers rather than dropping the block");
    assert_eq!(id, "213");
    assert!(unbalanced, "an unclosed title paren must be reported");
    assert!(body.starts_with("Oyatie ships"), "body not recovered: {body}");
}

#[test]
fn every_apex_contributed_blocks_to_the_census() {
    let root = repo_root();
    let apexes = read_apexes(&root);
    for apex in &apexes {
        assert!(
            !apex.blocks.is_empty(),
            "{} contributed no blocks; a per-file parse failure hides inside a healthy total",
            apex.path
        );
    }
}
