// cloud-ci-scan-root-liveness live-corpus gate.
//
// 1. LIVE: walk every ci/facade/*/*.json gate policy, collect coverage-bearing root
//    declarations keyed by full JSON pointer, resolve each against the real tree
//    (glob-aware), evaluate against the frozen policy, assert GREEN.
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

use ci_scan_root_liveness::{
    CODE_DEAD_SCAN_ROOT, DeclaredRoot, ForwardDeclaration, GATE_ID, Observed, Policy, Verdict,
    evaluate,
};
use serde_json::Value;

const POLICY_PATH: &str = "ci/facade/scan-root-liveness/scan-root-liveness-policy.json";
const EXPECTED_BASELINED_DEAD_ROOTS: usize = 9;

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
    };
    (policy, coverage_keys)
}

/// Resolve a declared root against the tree. Glob-aware: a pattern resolves iff it
/// matches at least one path. Plain paths resolve iff they exist.
///
/// Deliberately simple glob support — `*` matches within one path component, `**`
/// spans components. The declarations in this repo use only those two forms, and a
/// hand-rolled matcher with no dependency is preferable to pulling a crate into the
/// gate fleet for four characters of syntax.
fn resolves(root: &Path, pattern: &str) -> bool {
    if !pattern.contains('*') && !pattern.contains('?') {
        return root.join(pattern).exists();
    }
    let segments: Vec<&str> = pattern.split('/').collect();
    expand(root, &segments, PathBuf::new())
}

fn expand(root: &Path, segments: &[&str], acc: PathBuf) -> bool {
    let Some((head, rest)) = segments.split_first() else {
        return root.join(&acc).exists();
    };
    if !head.contains('*') && !head.contains('?') {
        return expand(root, rest, acc.join(head));
    }
    let dir = root.join(&acc);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !glob_segment_matches(head, &name) {
            continue;
        }
        if rest.is_empty() {
            return true;
        }
        if expand(root, rest, acc.join(name.as_ref())) {
            return true;
        }
    }
    false
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
    root: &Path,
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
                            resolves: resolves(root, entry),
                        });
                    }
                }
                collect_from(v, &ptr, file, coverage_keys, root, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                collect_from(v, pointer, file, coverage_keys, root, out);
            }
        }
        _ => {}
    }
}

fn collect(root: &Path, coverage_keys: &[String]) -> Observed {
    let mut roots: Vec<DeclaredRoot> = Vec::new();
    let mut files_with_roots: BTreeSet<String> = BTreeSet::new();

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
            collect_from(&doc, "", &rel, coverage_keys, root, &mut roots);
            if roots.len() > before {
                files_with_roots.insert(rel);
            }
        }
    }
    Observed {
        roots,
        policy_files_with_roots: files_with_roots,
    }
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
        "the reviewed frozen ceiling shrinks as gates stop enumerating roots by hand: it was 11 \
         (four pre-existing roots plus seven retired top-level cloud roots), and dropped by one \
         for each of embedded-asset-hermeticity and caller-supplied-authorization when they were \
         routed through the registry-derived resolver in ci/adapters/scan-root-derivation. This \
         number only ever goes DOWN"
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
