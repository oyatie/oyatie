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
const EXPECTED_BASELINED_DEAD_ROOTS: usize = 11;

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
        "the reviewed frozen ceiling is four pre-existing roots plus exactly seven retired \
         top-level cloud roots"
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

#[test]
fn collector_sees_the_real_corpus() {
    let root = repo_root();
    let (policy, keys) = load_policy(&root);
    let observed = collect(&root, &keys);
    assert!(
        observed.roots.len() >= policy.min_expected_roots,
        "collected only {} declared roots (floor {}) — the collector is broken",
        observed.roots.len(),
        policy.min_expected_roots
    );
    assert!(
        observed.policy_files_with_roots.len() >= 5,
        "only {} policy files declare roots — the walk is broken",
        observed.policy_files_with_roots.len()
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
