// cloud-ci-scan-root-liveness live-corpus gate.
//
// 1. LIVE: walk every ci/facade/*/*.json gate policy, collect coverage-bearing root
//    declarations keyed by full JSON pointer, resolve each against TRACKED paths
//    (git ls-files, glob-aware), evaluate against the frozen policy, assert GREEN.
// 2. RED FIXTURE: a synthetic dead root MUST fail — the gate is proven capable of
//    failing, not merely observed passing.
// 3. BASELINE FIDELITY: the baseline equals all and only live dead non-forward roots,
//    and every forward declaration is STILL absent. Both burn down, neither drifts.
// 4. FLOOR: the collector must see the real corpus.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_scan_root_liveness::{
    CODE_DEAD_SCAN_ROOT, DeclaredRoot, ForwardDeclaration, GATE_ID, Observed, Policy, Verdict,
    evaluate,
};
use serde_json::Value;

const POLICY_PATH: &str = "ci/facade/scan-root-liveness/scan-root-liveness-policy.json";
// 9 -> 10: this PR untracks `.codex` while rust-first anti-narrowing forbids
// dropping that scan root, so the debt is recorded in the baseline.
const EXPECTED_BASELINED_DEAD_ROOTS: usize = 10;

/// The reviewed ceiling on tolerated dark gate crates.
///
/// The evaluator alone does NOT stop this list growing. A crate that is genuinely
/// dark and is listed here produces no finding, and a listed crate that is
/// genuinely dark produces no stale finding — so a PR that adds a new dark gate
/// AND baselines it in the same change satisfies both directions silently. That
/// is the laundering path this constant closes: the number cannot move without a
/// reviewer seeing it move.
///
/// Lower it in the same change that wires a gate to the live tree — or, as here, in
/// the same change that DELETES one. Both are burn-down; the constant does not care
/// which, only that the number moved under a reviewer's eye.
///
///   97 -> 94  2026-08-20  Three gate crates retired because each doctrine's successor
///                         is already live and biting: governance/check/adr-citation
///                         (adr-citation-closure emits the identical
///                         `adr_citation_rejected_authority` from a live-tree test),
///                         governance/check/supply-chain (ci/facade/supply-chain-audit
///                         scans the real lockfiles against a vendored RustSec mirror),
///                         governance/check/pre-push (its evidence sources are gone —
///                         `git ls-files bin/` is empty and no pre-push hook is tracked;
///                         the ADR-0515 protected context is the enforcement now).
///                         Their three baseline entries are struck in the same change,
///                         which is what the exact-set assertion below requires.
// 2026-08-20  94 -> 88. Lane A's three RETIREMENTS (adr-citation, pre-push, supply-chain)
// took it 97 -> 94; this integration additionally CONNECTS six doctrines to the live tree
// -- shardability, layered-architecture-discipline, cursor-pagination-coverage, data-class,
// ontology-projection-coverage, active-artifact-contract -- each of which leaves the dark
// baseline in the same change that gives it a live-corpus test. 94 - 6 = 88, and the set
// 2026-08-20  87 -> 86. no-grouping leaves the dark baseline: it gains a live-corpus test
// asserting flat single-concern microservices under specs/microservices/ with zero grouping wrappers.
// 2026-08-20  86 -> 85. benchmark leaves the dark baseline: a live-corpus test now walks
// the 14 real PRDs (docs/prds/*.md + docs/products/**/PRD*.md, doc_class-filtered) and
// freezes 4 real SectionMissing violations two-sided.
// 2026-08-20  85 -> 84. retired-vocabulary leaves the dark baseline: it gains a live-corpus test
// asserting no live documentation mentions retired CLI surfaces, scripts, or hooks.
// 2026-08-20  84 -> 64. deleting 45 zero-importer libs/ crates removes 20 baselined dark-gate
// crates outright (their gate logic is gone, not connected); the baseline shrinks with them.
const EXPECTED_BASELINED_DARK_GATE_CRATES: usize = 64;

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

fn load_policy(root: &Path) -> (Policy, Vec<String>) {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: Value = serde_json::from_str(&raw).expect("policy parses");

    let coverage_keys: Vec<String> = doc["coverage_bearing_keys"]
        .as_array()
        .expect("coverage_bearing_keys")
        .iter()
        .map(|v| v.as_str().expect("string").to_owned())
        .collect();

    let mut forward = BTreeMap::new();
    if let Some(obj) = doc["forward_declarations"].as_object() {
        for (k, v) in obj {
            forward.insert(
                k.clone(),
                ForwardDeclaration {
                    value: v["value"].as_str().unwrap_or_default().to_owned(),
                    reason: v["reason"].as_str().unwrap_or_default().to_owned(),
                },
            );
        }
    }
    let mut exempt = BTreeMap::new();
    if let Some(obj) = doc["exempt_policy_files"].as_object() {
        for (k, v) in obj {
            exempt.insert(k.clone(), v.as_str().unwrap_or_default().to_owned());
        }
    }
    let policy = Policy {
        registered_policy_files: doc["registered_policy_files"]
            .as_array()
            .expect("registered_policy_files")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .collect(),
        exempt_policy_files: exempt,
        forward_declarations: forward,
        baselined_dead_roots: doc["baselined_dead_roots"]
            .as_array()
            .expect("baselined_dead_roots")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .collect(),
        min_expected_roots: doc["min_expected_roots"].as_u64().expect("floor") as usize,
        baselined_dark_gate_crates: doc["baselined_dark_gate_crates"]
            .as_array()
            .expect("baselined_dark_gate_crates")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .collect(),
        min_expected_gate_crates: doc["min_expected_gate_crates"]
            .as_u64()
            .expect("gate-crate floor") as usize,
        exempt_gate_crates: doc["exempt_gate_crates"]
            .as_object()
            .expect("exempt_gate_crates")
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().expect("exemption reason").to_owned()))
            .collect(),
    };
    (policy, coverage_keys)
}

/// Tracked-path universe (`git ls-files`). Working-tree `exists()` is the wrong
/// boundary: a gitignored overlay (`.codex/` after untrack) is present on some
/// clones and absent on CI, which would make the frozen exact-set host-dependent.
fn tracked_files(root: &Path) -> BTreeSet<String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .expect("git ls-files");
    assert!(
        out.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("git ls-files stdout was not UTF-8")
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_tracked_root(tracked: &BTreeSet<String>, pattern: &str) -> bool {
    tracked.contains(pattern)
        || tracked.iter().any(|path| {
            path.starts_with(pattern) && path.as_bytes().get(pattern.len()) == Some(&b'/')
        })
}

fn glob_matches_tracked(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let segs: Vec<&str> = path.split('/').collect();
    if segs.len() < pat.len() {
        return false;
    }
    pat.iter()
        .zip(segs.iter())
        .all(|(p, s)| glob_segment_matches(p, s))
}

/// Resolve a declared root against TRACKED paths. Glob-aware: a pattern resolves
/// iff it matches at least one tracked path (as that path, or as a directory
/// prefix of one). Plain paths resolve iff they are tracked or have a tracked
/// child.
///
/// Deliberately simple glob support — `*` matches within one path component, `**`
/// spans components. The declarations in this repo use only those two forms, and a
/// hand-rolled matcher with no dependency is preferable to pulling a crate into the
/// gate fleet for four characters of syntax.
fn resolves(tracked: &BTreeSet<String>, pattern: &str) -> bool {
    if !pattern.contains('*') && !pattern.contains('?') {
        return is_tracked_root(tracked, pattern);
    }
    tracked
        .iter()
        .any(|path| glob_matches_tracked(pattern, path))
}

/// Match one path segment against one glob segment (`*` = any run, `?` = one char).
fn glob_segment_matches(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (mut pi, mut ni, mut star, mut mark) = (0usize, 0usize, usize::MAX, 0usize);
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = pi;
            mark = ni;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Walk one JSON document, emitting a DeclaredRoot per coverage-bearing entry,
/// keyed by full JSON pointer.
fn collect_from(
    value: &Value,
    pointer: &str,
    file: &str,
    coverage_keys: &[String],
    tracked: &BTreeSet<String>,
    out: &mut Vec<DeclaredRoot>,
) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let ptr = format!("{pointer}/{k}");
                if coverage_keys.iter().any(|c| c == k)
                    && let Some(arr) = v.as_array()
                {
                    for entry in arr.iter().filter_map(Value::as_str) {
                        out.push(DeclaredRoot {
                            policy_file: file.to_owned(),
                            key: ptr.clone(),
                            value: entry.to_owned(),
                            resolves: resolves(tracked, entry),
                        });
                    }
                }
                collect_from(v, &ptr, file, coverage_keys, tracked, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                collect_from(v, pointer, file, coverage_keys, tracked, out);
            }
        }
        _ => {}
    }
}

fn collect(root: &Path, coverage_keys: &[String]) -> Observed {
    let mut roots: Vec<DeclaredRoot> = Vec::new();
    let mut files_with_roots: BTreeSet<String> = BTreeSet::new();
    let tracked = tracked_files(root);

    let facade = root.join("ci/facade");
    let Ok(gate_dirs) = std::fs::read_dir(&facade) else {
        return Observed::default();
    };
    for gate in gate_dirs.flatten() {
        let Ok(entries) = std::fs::read_dir(gate.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            // Generated faces are producer output, not declarations.
            if rel.contains("generated") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(doc) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            let before = roots.len();
            collect_from(&doc, "", &rel, coverage_keys, &tracked, &mut roots);
            if roots.len() > before {
                files_with_roots.insert(rel);
            }
        }
    }
    Observed {
        roots,
        policy_files_with_roots: files_with_roots,
        gate_crates: collect_gate_crates(root),
    }
}

/// Substrings that mean a test reads the REAL repository rather than a fixture: walking up
/// to the repo root, resolving the manifest dir, reading the scm-facts face, or shelling to
/// the producer with `--repo-root`. A test containing none of these cannot be looking at
/// this tree.
const LIVE_CORPUS_MARKERS: [&str; 5] = [
    "repo_root",
    "CARGO_MANIFEST_DIR",
    "scm-facts",
    "--repo-root",
    "PRODUCER_ENV",
];

/// The crate-path prefixes that make a crate a GATE crate. Kept here rather than in the
/// policy because it is the definition of the gate's universe, not tunable debt.
const GATE_CRATE_PREFIXES: [&str; 4] = [
    "ci/facade/",
    "governance/check/",
    "libs/oya-check-",
    "libs/oya-governance-",
];

/// Map every gate crate to whether ANY of its test code reads the real tree.
fn collect_gate_crates(root: &Path) -> BTreeMap<String, bool> {
    let mut out = BTreeMap::new();
    for prefix in GATE_CRATE_PREFIXES {
        let (dir, stem) = match prefix.rsplit_once('/') {
            Some((d, s)) if !s.is_empty() => (root.join(d), Some(s.to_owned())),
            _ => (root.join(prefix.trim_end_matches('/')), None),
        };
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(stem) = &stem
                && !name.starts_with(stem)
            {
                continue;
            }
            if !path.join("Cargo.toml").is_file() {
                continue;
            }
            out.insert(name, crate_has_live_corpus_test(&path));
        }
    }
    out
}

fn crate_has_live_corpus_test(crate_dir: &Path) -> bool {
    let mut stack = vec![crate_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) != Some("target") {
                    stack.push(path);
                }
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let is_test_code = path.components().any(|c| c.as_os_str() == "tests")
                || text.contains("#[test]")
                || text.contains("#[cfg(test)]");
            if is_test_code && LIVE_CORPUS_MARKERS.iter().any(|m| text.contains(m)) {
                return true;
            }
        }
    }
    false
}

#[test]
fn live_corpus_is_green_against_the_frozen_policy() {
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);
    let report = evaluate(&observed, &policy);

    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the live tree MUST be GREEN. Findings:\n{}",
        report
            .findings
            .iter()
            .map(|f| format!("  [{}] {}\n      {}", f.code, f.subject, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    eprintln!(
        "{GATE_ID}: GREEN — {} declared roots across {} policy files; {} dead tolerated, {} forward",
        report.roots_checked,
        observed.policy_files_with_roots.len(),
        report.dead_tolerated,
        policy.forward_declarations.len()
    );
}

#[test]
fn red_fixture_dead_root_fails_closed() {
    let observed = Observed {
        roots: vec![DeclaredRoot {
            policy_file: "p.json".to_owned(),
            key: "/scan/roots".to_owned(),
            value: "definitely/not/a/real/path".to_owned(),
            resolves: false,
        }],
        policy_files_with_roots: ["p.json".to_owned()].into_iter().collect(),
        ..Default::default()
    };
    let policy = Policy {
        registered_policy_files: ["p.json".to_owned()].into_iter().collect(),
        min_expected_roots: 0,
        ..Policy::default()
    };
    let report = evaluate(&observed, &policy);
    assert_eq!(report.verdict, Verdict::Red);
    assert_eq!(report.findings[0].code, CODE_DEAD_SCAN_ROOT);
}

#[test]
fn baselined_dead_roots_are_all_still_dead() {
    // Shrink-only fidelity: a baselined root that came back to life must be removed
    // from the baseline in the same change, or the slack outlives the debt.
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);
    let live: BTreeMap<String, bool> = observed
        .roots
        .iter()
        .map(|r| {
            (
                format!("{}::{}::{}", r.policy_file, r.key, r.value),
                r.resolves,
            )
        })
        .collect();

    for key in &policy.baselined_dead_roots {
        match live.get(key) {
            Some(true) => panic!(
                "baselined dead root `{key}` now RESOLVES — remove it from baselined_dead_roots"
            ),
            Some(false) => {}
            None => panic!(
                "baselined dead root `{key}` is no longer declared anywhere — remove it from \
                 baselined_dead_roots"
            ),
        }
    }
}

#[test]
fn frozen_baseline_is_exactly_the_live_non_forward_debt_set() {
    // Exact-set fidelity closes both directions: a missing row stays RED in the
    // evaluator, while an extra row cannot be laundered into the frozen baseline.
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);
    let live_dead_non_forward: BTreeSet<String> = observed
        .roots
        .iter()
        .filter(|declared| !declared.resolves)
        .map(|declared| {
            format!(
                "{}::{}::{}",
                declared.policy_file, declared.key, declared.value
            )
        })
        .filter(|key| !policy.forward_declarations.contains_key(key))
        .collect();

    assert_eq!(
        policy.baselined_dead_roots, live_dead_non_forward,
        "the frozen baseline must equal all and only live dead non-forward roots"
    );
    assert_eq!(
        policy.baselined_dead_roots.len(),
        EXPECTED_BASELINED_DEAD_ROOTS,
        "the reviewed frozen ceiling: routing a gate through the registry-derived resolver \
         LOWERS it; a reorg that empties a scan root the anti-narrowing ratchet forbids removing \
         MAY raise it under review. Confirm the constant comment. Exact-set comparison above is \
         the laundering backstop."
    );
}

#[test]
fn the_dark_gate_baseline_is_exactly_the_live_dark_set_and_cannot_grow() {
    // Mirrors `frozen_baseline_is_exactly_the_live_non_forward_debt_set`, which the
    // dead-roots list has had all along and this list has not.
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);

    let live_dark: BTreeSet<String> = observed
        .gate_crates
        .iter()
        .filter(|(_, has_live_test)| !**has_live_test)
        .map(|(krate, _)| krate.clone())
        .filter(|krate| {
            // An exempted crate is not a gate at all, so it is not dark debt.
            // A blank reason is not an exemption — the evaluator refuses those,
            // and this filter must agree with it or the two would disagree about
            // what the debt set is.
            !policy
                .exempt_gate_crates
                .get(krate)
                .is_some_and(|reason| !reason.trim().is_empty())
        })
        .collect();

    assert_eq!(
        policy.baselined_dark_gate_crates, live_dark,
        "the frozen baseline must equal all and only the live dark gate crates"
    );
    assert_eq!(
        policy.baselined_dark_gate_crates.len(),
        EXPECTED_BASELINED_DARK_GATE_CRATES,
        "the reviewed dark-gate ceiling moved. Wiring a gate to the live tree LOWERS \
         it — lower the constant in the same change. If this went UP, a new dark gate \
         was added and baselined in one step, which is the laundering path the \
         constant exists to refuse."
    );
}

#[test]
fn forward_declarations_are_all_still_absent() {
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);
    for (key, fwd) in &policy.forward_declarations {
        let found = observed
            .roots
            .iter()
            .find(|r| format!("{}::{}::{}", r.policy_file, r.key, r.value) == *key);
        match found {
            Some(r) if r.resolves => panic!(
                "forward declaration `{}` has LANDED ({}); retire it from forward_declarations",
                key, fwd.value
            ),
            Some(_) => {}
            None => {
                panic!("forward declaration `{key}` is no longer declared in any policy; retire it")
            }
        }
    }
}

/// The broken-collector guard, keyed on a SET rather than a corpus size.
///
/// It used to be two counts: `roots.len() >= min_expected_roots` (floor 150, against 152 declared)
/// and `policy_files_with_roots.len() >= 5` (against exactly 5). Both were within one routing of
/// going red, and they would have gone red on PROGRESS: every gate that stops hand-enumerating its
/// roots and derives them from the capability registry removes 30-odd declarations from this
/// corpus, which is the outcome this fleet wants. A floor that a successful burn-down trips is the
/// shape that killed `min_expected_unpackaged_yaml_files` after six lowerings.
///
/// The replacement does not decay: EVERY file in `registered_policy_files` must yield at least one
/// declared root. A registered file yielding none means one of two specific things, and the message
/// says which to check — either the file was routed through the registry-derived resolver and its
/// entry belongs in `exempt_policy_files`, or the collector broke on it. `min_expected_roots` is
/// kept only as a coarse "the walk returned something at all" tripwire and is now far below the
/// live count on purpose; it is LOWERED as gates route, never raised.
#[test]
fn every_registered_policy_file_yields_declared_roots() {
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);

    for file in &policy.registered_policy_files {
        assert!(
            observed.policy_files_with_roots.contains(file),
            "registered policy file `{file}` declares NO coverage-bearing root. Either it now \
             derives its roots (move it to exempt_policy_files with the reason) or the collector \
             broke on it — this is not a file that can legitimately declare nothing."
        );
    }
    assert!(
        observed.roots.len() >= policy.min_expected_roots,
        "collected only {} declared roots (tripwire floor {}) — the collector is broken",
        observed.roots.len(),
        policy.min_expected_roots
    );
}

#[test]
fn glob_matcher_handles_the_declared_forms() {
    // The declared forms in this repo are `libs/oya-*`, `oya/*/crates/oya-*`,
    // `policy/*/*`. Pin the segment matcher rather than trusting it by inspection.
    assert!(glob_segment_matches("oya-*", "oya-check-thing"));
    assert!(glob_segment_matches("*", "anything"));
    assert!(!glob_segment_matches("oya-*", "cloud-thing"));
    assert!(glob_segment_matches("a*c", "abc"));
    assert!(glob_segment_matches("a*c", "ac"));
    assert!(!glob_segment_matches("a*c", "abd"));
    assert!(glob_segment_matches("?", "x"));
    assert!(!glob_segment_matches("?", "xy"));
}

#[test]
fn resolution_is_tracked_membership_not_working_tree_existence() {
    let tracked = BTreeSet::from(["ci/facade/scan-root-liveness/src/lib.rs".to_owned()]);
    assert!(resolves(&tracked, "ci"));
    assert!(resolves(&tracked, "ci/*/*"));
    assert!(!resolves(&tracked, ".codex"));
    assert!(!resolves(&tracked, "cloud/*/crates/oya-*"));
}

#[test]
fn gitignored_overlay_directory_does_not_resolve() {
    // Mutation: a leftover/regenerated `.codex/` is the documented local overlay
    // (gitignored). exists() would call that live; tracked membership must not.
    let root = repo_root();
    let overlay = root.join(".codex");
    let created = if overlay.exists() {
        false
    } else {
        std::fs::create_dir_all(overlay.join("hooks")).expect("mkdir gitignored overlay");
        std::fs::write(overlay.join("hooks.json"), "{}\n").expect("write gitignored overlay");
        true
    };
    struct RemoveDirOnDrop(PathBuf);
    impl Drop for RemoveDirOnDrop {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = created.then(|| RemoveDirOnDrop(overlay.clone()));
    assert!(
        overlay.exists(),
        "mutation must leave a working-tree overlay so Path::exists would have been true"
    );
    let tracked = tracked_files(&root);
    assert!(
        !resolves(&tracked, ".codex"),
        "a gitignored `.codex` overlay must not count as a live scan root"
    );
    assert!(
        !tracked
            .iter()
            .any(|path| path == ".codex" || path.starts_with(".codex/")),
        "git ls-files must not list the overlay"
    );
}
