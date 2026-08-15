// governance-check-adr-citation-closure live-tree gate.
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Walk failures are errors, never omitted observations: a file dropped from the census
// because its metadata or contents failed to read would quietly shrink the violation count.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_adr_citation_closure::{
    AdrRecord, CODE_AMBIGUOUS_CLOSURE, CODE_ASYMMETRY, CODE_CITATION_MISMATCH,
    CODE_DANGLING_CITATION, CODE_DUPLICATE_ID, CODE_REJECTED_AUTHORITY, CODE_UNRESOLVABLE,
    CODE_VACUOUS_SCAN, CitationLine, Oracle, Policy, Resolution, Verdict, evaluate,
    parse_supersession, scan_line, undeclared_authority_surface,
};

const POLICY_PATH: &str = "governance/check/adr-citation-closure/adr-citation-closure-policy.json";
const APEX_DIR: &str = "docs/decisions";
const ARCHIVE_DIR: &str = "docs/adr-archive";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Config {
    policy: Policy,
    authority_surfaces: Vec<String>,
    authority_surface_marker: String,
    exempt_prefixes: Vec<String>,
    scan_extensions: Vec<String>,
}

struct Observed {
    records: Vec<AdrRecord>,
    citations: Vec<CitationLine>,
    files_scanned: usize,
    /// Files whose own frontmatter declares them authority surfaces while the policy list omits
    /// them. Collected during the SAME walk that produces the census, so the check costs no second
    /// pass and cannot drift out of sync with the corpus it is checking.
    undeclared_surfaces: Vec<String>,
}

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

fn load_config(root: &Path) -> Config {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let number = |key: &str| -> usize {
        usize::try_from(
            doc[key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy field {key} missing or not a number")),
        )
        .expect("policy number fits usize")
    };
    let strings = |key: &str| -> Vec<String> {
        doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("policy field {key} missing or not an array"))
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .unwrap_or_else(|| panic!("policy field {key} holds a non-string"))
                    .to_owned()
            })
            .collect()
    };
    Config {
        policy: Policy {
            min_adr_records: number("min_adr_records"),
            min_live_apexes: number("min_live_apexes"),
            min_archived_with_successor: number("min_archived_with_successor"),
            min_citation_lines: number("min_citation_lines"),
            min_authority_surfaces: number("min_authority_surfaces"),
        },
        authority_surfaces: strings("authority_surfaces"),
        authority_surface_marker: doc["authority_surface_marker"]
            .as_str()
            .expect("policy field authority_surface_marker missing or not a string")
            .to_owned(),
        exempt_prefixes: strings("exempt_path_prefixes"),
        scan_extensions: strings("scan_extensions"),
    }
}

/// The id a filename declares, from its LEADING DIGITS rather than a fixed four.
///
/// `get(..4)` on `ADR-335-foo.md` yields `335-`, which `normalize_id` correctly rejects, and
/// `read_adrs` then skipped the file without a word — while the kernel doc says the archive carries
/// both spellings and `normalize_id` exists precisely to reconcile them. Caller and kernel
/// disagreed, and the caller silently won.
fn adr_id(name: &str) -> Option<String> {
    let rest = name.strip_prefix("ADR-")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    check_adr_citation_closure::normalize_id(&digits)
}

/// The tracked file list, from git — the same boundary every other gate in this repo uses.
///
/// The walk used to read the WORKING TREE, skipping only dot-dirs and a few build directories.
/// `.gitignore` covers paths that match `scan_extensions` (root `*.txt`, `test-results/`,
/// `/bominal/`), so a developer with any such content on disk measured a different corpus than CI
/// did — and with the ceilings pinned by EQUALITY that is a red gate CI cannot reproduce.
fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("git ls-files failed to start: {e}"))?;
    if !out.status.success() {
        return Err(format!("git ls-files exited with {}", out.status));
    }
    let text =
        String::from_utf8(out.stdout).map_err(|e| format!("git ls-files output not UTF-8: {e}"))?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Does `relative` lie inside an exempt prefix, on a PATH BOUNDARY?
///
/// A bare `starts_with` exempted `docs/adr-archive-2026/` and `docs/adr-archiveX.md` along with
/// `docs/adr-archive/`, silently dropping whole trees from the scan on a name collision.
fn exempt(relative: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| {
        let prefix = prefix.trim_end_matches('/');
        relative == prefix
            || relative
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// Is any DIRECTORY component of `relative` one the scan has always excluded?
///
/// The working-tree walk skipped dot-directories and the build/vendor roots. Moving the census onto
/// `git ls-files` must not silently widen that component rule. Repository-local runtime overlays are
/// now ignored and untracked, while other tracked dot-directories remain machine/process state rather
/// than live citation surfaces.
///
/// The dot rule applies to DIRECTORY components only, exactly as the previous walk applied it — a
/// dotfile at a scanned location is still scanned.
fn in_excluded_dir(relative: &str) -> bool {
    let mut components: Vec<&str> = relative.split('/').collect();
    components.pop(); // the filename itself is not subject to the directory rules
    components.iter().any(|component| {
        component.starts_with('.')
            || matches!(*component, "buck-out" | "target" | "node_modules" | "vendor")
    })
}

fn read_adrs(root: &Path, dir: &str, live: bool) -> Result<Vec<AdrRecord>, String> {
    let full = root.join(dir);
    let entries =
        std::fs::read_dir(&full).map_err(|e| format!("read_dir {} failed: {e}", full.display()))?;
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("entry in {dir} failed: {e}"))?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("non-UTF-8 name under {dir}"))?
            .to_owned();
        if !name.ends_with(".md") {
            continue;
        }
        let Some(id) = adr_id(&name) else { continue };
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {} failed: {e}", path.display()))?;
        let record = parse_supersession(&id, &format!("{dir}/{name}"), live, &text)
            .map_err(|e| format!("{e:?}"))?;
        records.push(record);
    }
    records.sort();
    Ok(records)
}

fn observe(root: &Path, config: &Config) -> Result<Observed, String> {
    let mut records = read_adrs(root, APEX_DIR, true)?;
    records.extend(read_adrs(root, ARCHIVE_DIR, false)?);

    let mut citations = Vec::new();
    let mut files_scanned = 0usize;
    let mut undeclared_surfaces = Vec::new();
    for relative in tracked_files(root)? {
        if in_excluded_dir(&relative) || exempt(&relative, &config.exempt_prefixes) {
            continue;
        }
        if !config
            .scan_extensions
            .iter()
            .any(|ext| relative.ends_with(ext.as_str()))
        {
            continue;
        }
        let path = root.join(&relative);
        // Every failure below is an ERROR, never an omitted observation. A file dropped from the
        // census because its metadata or contents failed to read would quietly shrink the violation
        // count, and the ceilings are pinned by equality, so a shrink reads as repair.
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("metadata {relative} failed: {e} (tracked but unreadable)"))?;
        if !metadata.is_file() {
            return Err(format!("{relative} is tracked with a scanned extension but is not a file"));
        }
        if metadata.len() > MAX_SCANNED_BYTES {
            return Err(format!(
                "{relative} is {} bytes, over the {MAX_SCANNED_BYTES}-byte scan cap — raise the cap \
                 deliberately rather than dropping the file from the census",
                metadata.len()
            ));
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("read {relative} failed: {e}"))?;
        // LOSSY, never skipped. The old code skipped any payload that was not valid UTF-8 on the
        // stated grounds that it "carries no ADR citation by construction" — which is false:
        // latin-1 and UTF-16 both carry ASCII `ADR-NNNN` perfectly well.
        let text = String::from_utf8_lossy(&bytes);
        files_scanned += 1;
        if let Some(missing) = undeclared_authority_surface(
            &relative,
            &text,
            &config.authority_surface_marker,
            &config.authority_surfaces,
        ) {
            undeclared_surfaces.push(missing);
        }
        let authority_surface = config.authority_surfaces.iter().any(|s| s == &relative);
        for (index, line) in text.lines().enumerate() {
            let (cited, context) = scan_line(line);
            if cited.is_empty() && !(authority_surface && !context.is_empty()) {
                continue;
            }
            citations.push(CitationLine {
                path: relative.clone(),
                line: index + 1,
                cited,
                context,
                authority_surface,
            });
        }
    }
    citations.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
    undeclared_surfaces.sort();
    Ok(Observed {
        records,
        citations,
        files_scanned,
        undeclared_surfaces,
    })
}

/// The live walk, done ONCE for the whole binary.
///
/// Six tests each re-walked ~16.5k files and rebuilt the oracle. The walk is a pure function of the
/// tree, so every one of them was recomputing the same answer; caching it costs no independence,
/// because each test still evaluates whatever it likes from the shared observations.
fn live() -> &'static (Config, Observed, Verdict) {
    static LIVE: OnceLock<(Config, Observed, Verdict)> = OnceLock::new();
    LIVE.get_or_init(|| {
        let root = repo_root();
        let config = load_config(&root);
        let observed = observe(&root, &config).expect("live walk");
        let verdict = evaluate(&observed.records, &observed.citations, &config.policy);
        (config, observed, verdict)
    })
}

fn report(observed: &Observed, verdict: &Verdict) -> String {
    let mut out = format!(
        "census: {} ADR records ({} live apexes, {} archived, {} declaring a successor, {} \
         rejected), {} citation lines over {} files, {} authority surfaces\n",
        verdict.census.adr_records,
        verdict.census.live_apexes,
        verdict.census.archived,
        verdict.census.archived_with_successor,
        verdict.census.rejected,
        verdict.census.citation_lines,
        observed.files_scanned,
        verdict.census.authority_surfaces,
    );
    // Sampled PER CODE, never as one truncated list: a single `take(n)` over findings sorted by
    // ADR id hides whole categories behind whichever one happens to be numerous.
    for code in [
        CODE_CITATION_MISMATCH,
        CODE_DANGLING_CITATION,
        CODE_REJECTED_AUTHORITY,
        CODE_DUPLICATE_ID,
        CODE_ASYMMETRY,
        CODE_UNRESOLVABLE,
        CODE_AMBIGUOUS_CLOSURE,
        CODE_VACUOUS_SCAN,
    ] {
        let matching: Vec<_> = verdict.findings.iter().filter(|f| f.code == code).collect();
        out.push_str(&format!("{code}: {}\n", matching.len()));
        for finding in matching.iter().take(12) {
            out.push_str(&format!("    {} — {}\n", finding.subject, finding.detail));
        }
        if matching.len() > 12 {
            out.push_str(&format!("    … {} more\n", matching.len() - 12));
        }
    }
    out
}

// THE GATE, as a SHRINK-ONLY RATCHET on *semantic finding* ceilings (equality).
//
// Absolute census equality (`files_scanned` / `citation_lines` / `adr_records`) was DELETED as a
// merge blocker (PROCESS_TAX / audit 79f76050). Hand re-freeze of those counters is not tip-entitled.
// Anti-vacuity for the walk is machine-derived (`every_tracked_scannable_file_is_counted…`,
// vacuous-scan refuse, `min_*` floors). Finding ceilings stay: a NEW finding exceeds the ceiling
// and fails; a REPAIRED finding falls below it and ALSO fails, forcing the ceiling down same-change.
#[test]
fn live_tree_findings_equal_the_frozen_ceilings() {
    let raw = std::fs::read_to_string(repo_root().join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let ceiling = |key: &str| -> usize {
        usize::try_from(
            doc["measured"][key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy measured.{key} missing or not a number")),
        )
        .expect("policy number fits usize")
    };

    let (_, observed, verdict) = live();
    let count = |code: &str| verdict.findings.iter().filter(|f| f.code == code).count();

    // PROCESS_TAX DELETE (audit 79f76050 / Fail-class law): hand equality pins on absolute
    // census (`files_scanned` / `citation_lines` / `adr_records`) are NOT merge blockers.
    // Anti-vacuity lives in machine oracles instead:
    //   - `every_tracked_scannable_file_is_counted_and_nothing_untracked_is` (git ls-files ∩ scan)
    //   - vacuous-scan refuse below
    //   - policy `min_*` floors
    // Semantic finding ceilings (mismatch / rejected_authority / …) remain enforced.

    // Anti-vacuity FIRST: every count below is meaningless if the walk saw nothing.
    assert_eq!(
        count(CODE_VACUOUS_SCAN),
        0,
        "the walk collapsed — its zero findings are not evidence\n{}",
        report(&observed, &verdict)
    );
    assert!(
        observed.files_scanned > 0,
        "census walk saw zero files — refuse vacuous green\n{}",
        report(&observed, &verdict)
    );

    for (code, key) in [
        (CODE_CITATION_MISMATCH, "adr_citation_closure_mismatch"),
        (CODE_DANGLING_CITATION, "adr_citation_dangling_path"),
        (CODE_REJECTED_AUTHORITY, "adr_citation_rejected_authority"),
        (CODE_DUPLICATE_ID, "adr_duplicate_id"),
        (CODE_ASYMMETRY, "adr_closure_asymmetry"),
        (CODE_UNRESOLVABLE, "adr_closure_unresolvable"),
        (CODE_AMBIGUOUS_CLOSURE, "adr_closure_ambiguous"),
    ] {
        let observed_count = count(code);
        let frozen = ceiling(key);
        assert_eq!(
            observed_count, frozen,
            "{code}: observed {observed_count}, frozen ceiling {frozen}. Above it, a new finding was \
             introduced and must be repaired rather than admitted. Below it, findings were repaired \
             and `measured.{key}` must be lowered to {observed_count} in the SAME change so the \
             ratchet keeps biting.\n{}",
            report(&observed, &verdict)
        );
    }
}

// Evidence, always printed: the census and the finding histogram, so a reader can tell a repaired
// corpus from a collapsed walk without re-running anything.
#[test]
fn live_tree_census_and_findings_are_reported() {
    let (_, observed, verdict) = live();
    println!("{}", report(observed, verdict));
    assert!(observed.files_scanned > 0);
}

// THE META-TEST. Inject a synthetic bad citation into the LIVE record set and assert the gate
// catches it. Without this the gate could rot into always-green and nobody would notice.
#[test]
fn a_new_bad_citation_against_the_live_corpus_is_caught() {
    let (config, observed, _) = live();
    let oracle = Oracle::new(&observed.records);

    // Find a real archived ADR with an unambiguous live closure, then cite a DIFFERENT real apex.
    let (member, correct) = observed
        .records
        .iter()
        .filter(|r| !r.live)
        .find_map(|r| match oracle.resolve(&r.id) {
            Resolution::Live(apex) => Some((r.id.clone(), apex)),
            _ => None,
        })
        .expect("the live corpus must contain at least one resolvable archived ADR");
    let wrong = observed
        .records
        .iter()
        .find(|r| r.live && r.id != correct)
        .expect("the live corpus must contain a second apex")
        .id
        .clone();

    let injected = CitationLine {
        path: "synthetic/injected-defect.md".to_owned(),
        line: 1,
        cited: vec![wrong.clone()],
        context: vec![member.clone()],
        authority_surface: false,
    };
    let before = evaluate(&observed.records, &observed.citations, &config.policy)
        .count(CODE_CITATION_MISMATCH);
    let mut with_defect = observed.citations.clone();
    with_defect.push(injected);
    let after = evaluate(&observed.records, &with_defect, &config.policy)
        .count(CODE_CITATION_MISMATCH);
    assert_eq!(
        after,
        before + 1,
        "injecting a citation of {wrong} for {member} (which resolves to {correct}) must raise the \
         mismatch count by exactly one"
    );
}

// The floors must be capable of firing on the real shape, not only on an empty vector.
#[test]
fn the_live_walk_would_fail_closed_if_it_collapsed() {
    let (config, observed, _) = live();
    let verdict = evaluate(&[], &[], &config.policy);
    assert!(verdict.failed());
    assert_eq!(verdict.count(CODE_VACUOUS_SCAN), 5);

    // And the oracle floor specifically: censuses intact, edges gone.
    let mut edgeless = observed.records.clone();
    for record in &mut edgeless {
        record.supersedes.clear();
        record.superseded_by.clear();
    }
    let collapsed = evaluate(&edgeless, &observed.citations, &config.policy);
    assert_eq!(
        collapsed.count(CODE_CITATION_MISMATCH),
        0,
        "an empty oracle accuses nothing"
    );
    assert!(
        collapsed.count(CODE_VACUOUS_SCAN) >= 1,
        "so the oracle floor must catch it"
    );
}

// The measured instance the brief names: root CLAUDE.md cites ADR-0111 as merge-queue authority,
// and ADR-0111 is Rejected in the archive. If this stops firing, either the citation was repaired
// or the authority-surface list went stale — both worth a deliberate edit here.
#[test]
fn the_rejected_authority_rule_fires_on_the_real_instance() {
    let (_, _, verdict) = live();
    let hits: Vec<&str> = verdict
        .findings
        .iter()
        .filter(|f| f.code == CODE_REJECTED_AUTHORITY)
        .map(|f| f.subject.as_str())
        .collect();
    assert!(
        !hits.is_empty(),
        "no authority surface cites a Rejected ADR — verify the surface list is still accurate"
    );
    println!("rejected-authority citations: {hits:?}");
}

// FINDING 7. `get(..4)` on `ADR-335-foo.md` yields `335-`, normalize_id rejects it, and read_adrs
// dropped the file without a word — while the kernel documents both spellings as expected input.
#[test]
fn a_short_form_filename_yields_the_same_id_as_its_padded_spelling() {
    assert_eq!(adr_id("ADR-335-governance.md").as_deref(), Some("ADR-0335"));
    assert_eq!(adr_id("ADR-0335-governance.md").as_deref(), Some("ADR-0335"));
    assert_eq!(adr_id("ADR-7-early.md").as_deref(), Some("ADR-0007"));
    assert_eq!(adr_id("ADR-12345-overlong.md"), None);
    assert_eq!(adr_id("ADR-none.md"), None);
    assert_eq!(adr_id("README.md"), None);
}

// FINDING 9. A bare `starts_with` exempts any path that merely SHARES A PREFIX, so a sibling
// directory named `docs/adr-archive-2026/` would drop out of the scan entirely and silently.
#[test]
fn exempt_prefixes_match_on_a_path_boundary_not_a_substring() {
    let prefixes = vec!["docs/adr-archive".to_owned()];
    assert!(exempt("docs/adr-archive/ADR-0001-x.md", &prefixes));
    assert!(exempt("docs/adr-archive", &prefixes));
    assert!(
        !exempt("docs/adr-archive-2026/ADR-0001-x.md", &prefixes),
        "a sibling directory sharing the prefix must still be scanned"
    );
    assert!(
        !exempt("docs/adr-archiveX.md", &prefixes),
        "a file sharing the prefix must still be scanned"
    );
}

// The dot-directory rule the working-tree walk always had, preserved across the move to
// `git ls-files`. Repository-local agent overlays are ignored/untracked; other tracked dot-state is
// still outside the live citation corpus.
#[test]
fn tracked_dot_directories_and_build_roots_stay_out_of_the_scan() {
    assert!(in_excluded_dir(".grok/programs/plan.md"));
    assert!(in_excluded_dir(".claude/agents/x.md"));
    assert!(in_excluded_dir("some/path/node_modules/pkg/readme.md"));
    assert!(in_excluded_dir("target/debug/build.rs"));
    assert!(!in_excluded_dir("docs/AGENTS.md"));
    assert!(!in_excluded_dir("ci/facade/x/src/lib.rs"));
    assert!(
        !in_excluded_dir(".gitignore"),
        "the dot rule applies to DIRECTORY components, not to a file's own name"
    );
}

// FINDING 4 + FINDING 5, as one property: the census is exactly the tracked, non-exempt, scannable
// corpus — nothing extra from an untracked working tree, and nothing silently dropped.
//
// Before the fix this failed on the omission side: `entry.file_type()` does not follow symlinks, so
// all 15 tracked symlinks with scanned extensions were skipped without being counted.
#[test]
fn every_tracked_scannable_file_is_counted_and_nothing_untracked_is() {
    let root = repo_root();
    let (config, observed, _) = live();
    let expected = tracked_files(&root)
        .expect("git ls-files")
        .into_iter()
        .filter(|relative| !in_excluded_dir(relative) && !exempt(relative, &config.exempt_prefixes))
        .filter(|relative| {
            config
                .scan_extensions
                .iter()
                .any(|ext| relative.ends_with(ext.as_str()))
        })
        .count();
    assert_eq!(
        observed.files_scanned, expected,
        "the census must equal the tracked scannable corpus — a difference means either untracked \
         content leaked in or a readable file was silently dropped"
    );
}

// Anti-vacuity for the SURFACE list itself: every declared authority surface must exist and be
// scanned, otherwise the rule above silently checks nothing.
#[test]
fn every_declared_authority_surface_exists_and_was_scanned() {
    let root = repo_root();
    let (config, observed, _) = live();
    let seen: BTreeSet<&str> = observed
        .citations
        .iter()
        .filter(|c| c.authority_surface)
        .map(|c| c.path.as_str())
        .collect();
    for surface in &config.authority_surfaces {
        assert!(
            root.join(surface).is_file(),
            "declared authority surface {surface} does not exist"
        );
        assert!(
            seen.contains(surface.as_str()),
            "declared authority surface {surface} produced no ADR citation — it may have moved"
        );
    }
}

// THE OMISSION HALF of the same staleness problem, and the one that was actually costing findings.
//
// The test above iterates `authority_surfaces` and checks each entry is real. It CANNOT see a
// surface that was never listed — it is checking the very list that is incomplete, so a governance
// document nobody remembered to declare is invisible to it and to the rejected-authority rule
// behind it. That is not a mis-set value, it is a rule that structurally cannot reach its own
// strongest instances, which is the same defect class as the `context`-only scan repaired in
// `_review_remeasure_2026_08_08`: both were rules that could not see the citations that state
// doctrine most explicitly.
//
// The fix is to stop treating the hand-curated list as the definition and derive candidates from
// what each document declares about ITSELF, in frontmatter, during the walk that is already
// running. `authority_surface_marker` is policy DATA, so another repo repoints it.
//
// OBSERVED FIRING, on the live tree at this commit's parent, before
// `docs/AGENTS-OPERATING-CONTRACT.md` was added to `authority_surfaces` — this is the execution
// that makes the green below evidence rather than decoration:
//
//   thread 'a_document_declaring_itself_an_operating_contract_is_a_declared_surface' panicked at
//   governance/check/adr-citation-closure/tests/adr_citation_closure.rs:640:5:
//   assertion `left == right` failed: these files declare `doc_class: Operating-Contract` in their
//   own frontmatter but are absent from authority_surfaces, so the rejected-authority rule cannot
//   see them: ["docs/AGENTS-OPERATING-CONTRACT.md"]. […]
//   test result: FAILED. 10 passed; 1 failed
//   Tests finished: Pass 1. Fail 1. … Commands: 3 (cached: 0, remote: 0, local: 3)
//
// It fired on exactly ONE file, which is also the control that the marker is not over-matching:
// `doc_class: Operating-Contract` appears in the frontmatter head of two tracked documents, and the
// other one — `docs/AGENTS.md` — was already declared. The sibling test above passed in that same
// run, which is the direct evidence that it cannot see this class.
#[test]
fn a_document_declaring_itself_an_operating_contract_is_a_declared_surface() {
    let (config, observed, _) = live();
    let empty: Vec<String> = Vec::new();
    assert_eq!(
        observed.undeclared_surfaces, empty,
        "these files declare `{}` in their own frontmatter but are absent from \
         authority_surfaces, so the rejected-authority rule cannot see them: {:?}. Declare them \
         and re-measure BOTH numbers it moves — the citation_lines census (an authority surface \
         contributes lines whose `cited` is empty) and adr_citation_rejected_authority — in the \
         SAME change.",
        config.authority_surface_marker,
        observed.undeclared_surfaces
    );
}
