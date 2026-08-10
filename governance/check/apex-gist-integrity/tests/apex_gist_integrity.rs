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
    ApexDoc, ArchivedMember, Block, CODE_CUT_FROM_SOURCE, CODE_MEMBER_WITHOUT_GIST,
    CODE_MEMBER_WITHOUT_RESIDUAL, CODE_TITLE_UNRESOLVED, CODE_TOPIC_DROPPED, CODE_TRUNCATED_MIDWORD,
    CODE_UNBALANCED_TITLE, CODE_UNCLOSED_FENCE, CODE_VACUOUS_SCAN, Policy, Site, Topic, Verdict,
    evaluate, normalize_id,
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
    /// `"{code}@{apex}" -> count`, read WHOLE from the policy rather than by a fixed key list.
    ///
    /// Reading the whole object matters: a key list would only ever compare the pairs someone
    /// remembered to name, so a code that started firing on an apex it had never fired on before
    /// would be invisible. `assert_frozen` unions both key sets for the same reason.
    frozen_by_apex: BTreeMap<String, usize>,
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
        frozen_by_apex: measured["findings_by_apex"]
            .as_object()
            .expect("policy measured.findings_by_apex is an object")
            .iter()
            .map(|(key, value)| {
                let count = value
                    .as_u64()
                    .unwrap_or_else(|| panic!("findings_by_apex.{key} is not a number"));
                assert!(
                    KNOWN_CODES.iter().any(|code| key.starts_with(&format!("{code}@"))),
                    "findings_by_apex key `{key}` names no known finding code; a typo here \
                     freezes a pair that can never be observed and silently exempts the real one"
                );
                (key.clone(), usize::try_from(count).expect("fits usize"))
            })
            .collect(),
        census: grab(&["live_apexes", "members", "blocks", "archived_members_read"]),
    }
}

/// Every code the kernel can emit. Used to reject a mistyped frozen key, which would otherwise
/// freeze a pair that is never observed while leaving the real pair unpinned — green both ways.
const KNOWN_CODES: &[&str] = &[
    CODE_CUT_FROM_SOURCE,
    CODE_TRUNCATED_MIDWORD,
    CODE_UNCLOSED_FENCE,
    CODE_TITLE_UNRESOLVED,
    CODE_UNBALANCED_TITLE,
    CODE_MEMBER_WITHOUT_GIST,
    CODE_MEMBER_WITHOUT_RESIDUAL,
    CODE_TOPIC_DROPPED,
    CODE_VACUOUS_SCAN,
];

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

/// One parsed gist bullet: `- **ADR-N** (title): body`.
struct GistLeadIn {
    id: String,
    title: String,
    body: String,
    title_unbalanced: bool,
}

/// Strip `- **ADR-N** (title): `, returning the member id, the TITLE, the carried text, and whether
/// the title's parentheses were left OPEN.
///
/// The title is returned on BOTH branches. Returning it only on the unbalanced branch — which is
/// what this function used to do, by discarding `after[1..i]` — meant the balanced majority was
/// never title-checked at all, and since the generator emits its own closing `)` after a title it
/// already cut, balanced IS the majority: 254 gist bullets, 22 unbalanced. The 22 were the accident.
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
fn strip_gist_leadin(line: &str) -> Option<GistLeadIn> {
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
                    return Some(GistLeadIn {
                        id,
                        title: after[1..i].trim().to_owned(),
                        body: tail.trim().to_owned(),
                        title_unbalanced: false,
                    });
                }
            }
            _ => {}
        }
    }
    let cut = after.find("): ")?;
    Some(GistLeadIn {
        id,
        title: after[1..cut].trim().to_owned(),
        body: after[cut + "): ".len()..].trim().to_owned(),
        title_unbalanced: true,
    })
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
                if let Some(lead) = strip_gist_leadin(line) {
                    blocks.push(Block {
                        member_id: lead.id,
                        site: Site::Gist,
                        line: line_no,
                        text: lead.body,
                        title: lead.title,
                        title_unbalanced: lead.title_unbalanced,
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
                                // A residual section has no `(title)` lead-in to carry.
                                title: String::new(),
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

/// Every spelling of a member's title the generator could legitimately have carried, in the order
/// the corpus offers them: filename stem, frontmatter `title:`, H1 heading.
///
/// All three are needed and the measurement says so: 448 archived members all carry an H1, but only
/// 171 declare a frontmatter `title:`, and the gist bullets demonstrably carry all three forms
/// (`ADR-0038`'s bullet carries the stem, `ADR-0106`'s the frontmatter title, `ADR-0166`'s the H1).
/// Knowing about only some of them would report the rest as unresolved — a gate that fires on
/// correct data is worse than no gate, because the real findings drown.
fn member_titles(stem: &str, text: &str) -> Vec<String> {
    let mut titles = vec![stem.to_owned()];
    for line in text.lines() {
        if let Some(raw) = line.strip_prefix("title:") {
            let t = raw.trim().trim_matches('"').trim_matches('\'').trim();
            if !t.is_empty() {
                titles.push(t.to_owned());
            }
        }
        if let Some(raw) = line.strip_prefix("# ") {
            let t = raw.trim();
            if !t.is_empty() {
                titles.push(t.to_owned());
            }
        }
    }
    titles
}

fn read_archive(root: &Path) -> BTreeMap<String, ArchivedMember> {
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
        let stem = name.trim_end_matches(".md");
        out.insert(
            id,
            ArchivedMember {
                titles: member_titles(stem, &text),
                body_lower: text.to_lowercase(),
            },
        );
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
///
/// The key set is the UNION of both sides, not the frozen side alone. Iterating only the frozen
/// keys makes a NEW key invisible — a code that starts firing on an apex it had never fired on
/// before would be compared against nothing and pass — which is the same shape of hole as an
/// unratcheted count, one level down.
fn assert_frozen(observed: &BTreeMap<String, usize>, frozen: &BTreeMap<String, usize>, why: &str) {
    let keys: BTreeMap<&String, ()> = frozen.keys().chain(observed.keys()).map(|k| (k, ())).collect();
    let drift: Vec<String> = keys
        .into_keys()
        .filter_map(|key| {
            let seen = observed.get(key).copied().unwrap_or(0);
            let want = frozen.get(key).copied().unwrap_or(0);
            (seen != want).then(|| format!("  {key}: observed {seen}, frozen {want}"))
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

/// The shrink-only ratchet, pinned per `(code, apex)`.
///
/// WHAT THIS DETECTS, stated at the strength the mechanism actually has: any NET change to the
/// number of findings of a given code WITHIN A GIVEN APEX. A new truncation exceeds the pin and a
/// repaired one falls below it; both fail, which forces the pin down in the same change that
/// repairs the corpus.
///
/// WHAT IT DOES NOT DETECT: a change that is net-zero for one code inside one apex — repair one
/// `apex_gist_truncated_midword` in `ADR-0700` and introduce another there and the pin is unmoved.
/// Splitting by apex NARROWS that hole (it used to span the whole corpus); it does not close it.
/// It closes only for a code the repair drives to 0, where equality at 0 cannot be satisfied by any
/// offsetting pair. That residual is recorded in the gate's non_claims, not papered over here — and
/// this gate's own `_parse_recovery_attribution` is the standing proof that a count-only ledger has
/// already deceived this lane once, and was caught by an independent hand-derivation rather than by
/// the ratchet.
#[test]
fn findings_equal_the_frozen_ceilings() {
    let root = repo_root();
    let cfg = load_config(&root);
    assert_frozen(
        &observed().verdict.counts_by_apex,
        &cfg.frozen_by_apex,
        "finding-count drift, per (code, apex) — re-derive by RUNNING this gate and reading \
         'observed N' from these lines; never by arithmetic on the old values:",
    );
    assert_eq!(
        observed().verdict.count(CODE_VACUOUS_SCAN),
        0,
        "the scan went vacuous; every count above is then meaningless, and an empty by-apex map \
         would otherwise only show up as mass drift rather than as the census failure it is"
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
        title: String::new(),
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
        title: String::new(),
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
        title: String::new(),
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

    // 5. A block that is a verbatim PROPER PREFIX of its member — the cut the shape predicates
    //    cannot see. The injected text ends on a COMMA on purpose: `ends_midword` returns false
    //    for it, so a rise here can only have come from the source comparison.
    let mut archive_plus = archive.clone();
    archive_plus.insert(
        "999999".to_owned(),
        ArchivedMember {
            titles: vec!["zzqqxx-probe-stem".to_owned()],
            body_lower: "the carried half stops here, and the ratified remainder follows it."
                .to_owned(),
        },
    );
    let mut mutated = base.clone();
    mutated[0].blocks.push(Block {
        member_id: "999999".to_owned(),
        site: Site::Gist,
        line: 0,
        text: "The carried half stops here,".to_owned(),
        title: "zzqqxx-probe-stem".to_owned(),
        title_unbalanced: false,
    });
    let v = evaluate(&mutated, &archive_plus, &cfg.topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_CUT_FROM_SOURCE),
        baseline.count(CODE_CUT_FROM_SOURCE) + 1,
        "a verbatim proper prefix injected into a real apex did not raise the count"
    );
    assert_eq!(
        v.count(CODE_TRUNCATED_MIDWORD),
        baseline.count(CODE_TRUNCATED_MIDWORD),
        "the injected cut ends on punctuation, so it must NOT move the mid-word count; if it did, \
         the two codes are double-counting the same block"
    );

    // 6. A bullet whose title matches no spelling its member offers.
    let mut mutated = base.clone();
    mutated[0].blocks.push(Block {
        member_id: "999999".to_owned(),
        site: Site::Gist,
        line: 0,
        text: "The body here finishes normally.".to_owned(),
        title: "zzqqxx-probe-ste".to_owned(),
        title_unbalanced: false,
    });
    let v = evaluate(&mutated, &archive_plus, &cfg.topics, &cfg.policy);
    assert_eq!(
        v.count(CODE_TITLE_UNRESOLVED),
        baseline.count(CODE_TITLE_UNRESOLVED) + 1,
        "a title matching no spelling of its member did not raise the count"
    );

    // 7. A topic a member carries and the apex never mentions.
    let mut archive_plus = archive.clone();
    let victim = base[0].supersedes[0].clone();
    archive_plus.insert(
        victim,
        ArchivedMember {
            titles: Vec::new(),
            body_lower: "this member decides everything about zzqqxx-substrate".to_owned(),
        },
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
    let lead = strip_gist_leadin(
        "- **ADR-106** (Rename `application` layer to `usecase` (amends ADR-0105)): Rename the \
         `application` layer to **`usecase`**.",
    )
    .expect("parses");
    assert_eq!(lead.id, "106");
    assert!(
        lead.body.starts_with("Rename the `application` layer"),
        "leadin stripper cut into the title: {}",
        lead.body
    );
    assert!(
        !lead.title_unbalanced,
        "a well-formed nested title is not an unbalanced one"
    );
    // The title must survive the BALANCED branch. This is the regression that made ~232 of the 254
    // gist bullets un-title-checkable: the balanced branch used to discard the title entirely, so
    // only the 22 accidentally-unbalanced ones were ever looked at.
    assert_eq!(
        lead.title,
        "Rename `application` layer to `usecase` (amends ADR-0105)",
        "the balanced branch discarded or truncated the title"
    );

    // And the shape that forced the fallback to exist: a title cut mid-parenthesis. The block must
    // still yield its body — dropping it would shrink the census and read as a healthier corpus.
    let lead = strip_gist_leadin(
        "- **ADR-213** (Ecosystem-as-a-Service architecture — Plugin/App Store substrate \
         (third-party de): Oyatie ships a plugin substrate.",
    )
    .expect("recovers rather than dropping the block");
    assert_eq!(lead.id, "213");
    assert!(lead.title_unbalanced, "an unclosed title paren must be reported");
    assert!(
        lead.body.starts_with("Oyatie ships"),
        "body not recovered: {}",
        lead.body
    );
    assert!(
        lead.title.starts_with("Ecosystem-as-a-Service architecture"),
        "title not recovered on the unbalanced branch: {}",
        lead.title
    );
}

/// The title oracle reads the corpus's ACTUAL spellings, and all three of them exist.
///
/// Asserted against the real archive rather than a fixture because the claim under test is about
/// the corpus, not about the parser: if a future archive rewrite dropped H1 headings or moved to
/// frontmatter-only titles, a fixture would keep passing while the live gate started reporting
/// hundreds of correct titles as unresolved.
#[test]
fn every_archived_member_offers_at_least_a_stem_and_a_heading_to_resolve_titles_against() {
    let root = repo_root();
    let archive = read_archive(&root);
    let with_heading = archive
        .values()
        .filter(|m| m.titles.len() >= 2)
        .count();
    assert_eq!(
        with_heading,
        archive.len(),
        "some archived members offer only their filename stem; a gist bullet carrying such a \
         member's H1 would then be reported unresolved when it is correct"
    );
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
