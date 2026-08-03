// cloud-ci-crate-reference-integrity live-corpus gate.
//
// 1. LIVE: collect the REAL known-name census (workspace members ∪ lockfile packages) and every
//    structural reference site the policy declares, evaluate against the frozen shrink-only
//    baseline, and assert GREEN.
// 2. RED FIXTURE: the in-repo fixture under fixtures/red MUST fail, scanned with the fixture dir
//    as the scan root, an EMPTY baseline and EMPTY exclusions. A gate never demonstrated failing
//    is not evidence.
// 3. BASELINE FIDELITY: every frozen key still produces its exact live finding, so tolerated debt
//    can never outlive the defect it tolerates.
// 4. FLOOR: the collector must see the real corpus, so a collection bug cannot present as clean.
//
// This file owns ALL of the I/O — the tracked-file boundary (`git ls-files`, the same boundary
// crate-catalog-coverage uses, so an untracked scratch file cannot influence the verdict), the
// workspace resolution, and the file reads. The kernel in src/lib.rs stays provable with no
// filesystem at all.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_crate_reference_integrity::{
    CODE_DANGLING_CRATE_REFERENCE, ExclusionPolicy, GATE_ID, Observed, Policy, Resolution,
    RuleObservation, RulePolicy, Site, Verdict, evaluate, extract_sites, finding_key,
    lock_package_names, package_name_from_manifest, path_glob_matches,
};
use serde_json::Value;

const POLICY_PATH: &str = "ci/facade/crate-reference-integrity/crate-reference-integrity-policy.json";
const BASELINE_PATH: &str =
    "ci/facade/crate-reference-integrity/crate-reference-integrity-baseline.json";
const FIXTURE_DIR: &str = "ci/facade/crate-reference-integrity/fixtures/red";

/// Walk up from the test's working directory to the repo root (the dir holding the policy).
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
    panic!("failed to locate repo root (the dir holding {POLICY_PATH}) from the test current_dir");
}

fn read_json(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// The tracked file list, from git.
fn tracked_files(dir: &Path) -> Vec<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["ls-files"])
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed in {}", dir.display());
    String::from_utf8(out.stdout)
        .expect("utf8")
        .lines()
        .map(str::to_owned)
        .collect()
}

/// Live workspace package names ∪ lockfile package names.
///
/// The root manifest holds only glob patterns after expansion, so membership is resolved at the
/// canonical kernel rather than parsed textually. The lockfile union is what keeps a documented
/// `-p serde` from reading as a dangling reference.
fn known_names(root: &Path) -> BTreeSet<String> {
    let member_dirs = oya_workspace_members_kernel::resolve_member_dirs(root)
        .expect("workspace member resolution must not fail closed silently");
    let mut names: BTreeSet<String> = BTreeSet::new();
    for dir in &member_dirs {
        let manifest = root.join(dir).join("Cargo.toml");
        if let Some(name) = std::fs::read_to_string(&manifest)
            .ok()
            .and_then(|text| package_name_from_manifest(&text))
        {
            names.insert(name);
        }
    }
    let workspace_packages = names.len();
    if let Ok(lock) = std::fs::read_to_string(root.join("Cargo.lock")) {
        names.extend(lock_package_names(&lock));
    }
    eprintln!(
        "{GATE_ID}: census — {workspace_packages} workspace packages, {} names after the lockfile union",
        names.len()
    );
    names
}

fn glob_list(rule: &Value, key: &str) -> Vec<String> {
    rule[key]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

/// Does a build-graph label resolve? `//path:target` resolves iff `path/<build_file>` is a tracked
/// file — the tracked set, not the filesystem, so an untracked scratch BUCK cannot revive a label.
fn label_resolves(label: &str, build_file: &str, tracked: &BTreeSet<String>) -> bool {
    let Some(rest) = label.strip_prefix("//") else {
        return false;
    };
    let Some((dir, _target)) = rest.split_once(':') else {
        return false;
    };
    tracked.contains(&format!("{dir}/{build_file}"))
}

fn collect(root: &Path, policy_doc: &Value, tracked: &[String]) -> Observed {
    Observed {
        rules: collect_rules(root, policy_doc, tracked),
        known_names: known_names(root),
    }
}

/// Site collection only. Split from the census so the RED fixture — a directory with no
/// workspace and no lockfile — can be scanned without resolving one.
fn collect_rules(root: &Path, policy_doc: &Value, tracked: &[String]) -> Vec<RuleObservation> {
    let tracked_set: BTreeSet<String> = tracked.iter().cloned().collect();
    let exclusion_globs: Vec<String> = policy_doc["exclusions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|e| e["glob"].as_str())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let mut rules: Vec<RuleObservation> = Vec::new();
    for rule in policy_doc["rules"].as_array().expect("rules array") {
        let rule_id = rule["rule_id"].as_str().expect("rule_id").to_owned();
        let build_file = rule["build_file"].as_str().unwrap_or("BUCK").to_owned();
        let is_label_rule = rule["subject"].as_str() == Some("build_target_label");

        let mut observation = RuleObservation {
            rule_id: rule_id.clone(),
            ..RuleObservation::default()
        };
        let mut files: BTreeSet<String> = BTreeSet::new();
        for glob in glob_list(rule, "file_globs") {
            let matched: Vec<&String> = tracked
                .iter()
                .filter(|path| path_glob_matches(&glob, path))
                .collect();
            if matched.is_empty() {
                observation.globs_matching_nothing.push(glob);
                continue;
            }
            files.extend(matched.into_iter().cloned());
        }

        for file in files {
            let Ok(content) = std::fs::read_to_string(root.join(&file)) else {
                continue;
            };
            let outcome = extract_sites(rule, &content);
            observation.non_name_values_ignored += outcome.non_name_values_ignored;
            let excluded_by = exclusion_globs
                .iter()
                .find(|glob| path_glob_matches(glob, &file))
                .cloned();
            for subject in outcome.subjects {
                let resolution = if is_label_rule {
                    Resolution::Prevalidated(label_resolves(&subject, &build_file, &tracked_set))
                } else {
                    Resolution::AgainstKnownNames
                };
                observation.sites.push(Site {
                    rule_id: rule_id.clone(),
                    file: file.clone(),
                    subject,
                    resolution,
                    excluded_by: excluded_by.clone(),
                });
            }
        }
        rules.push(observation);
    }
    rules
}

fn load_policy(policy_doc: &Value, baseline_doc: &Value, tracked: &[String]) -> Policy {
    let rules = policy_doc["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .map(|rule| RulePolicy {
            rule_id: rule["rule_id"].as_str().expect("rule_id").to_owned(),
            min_sites: rule["min_sites"].as_u64().expect("min_sites") as usize,
        })
        .collect();
    let exclusions = policy_doc["exclusions"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|entry| {
                    let glob = entry["glob"].as_str().unwrap_or_default().to_owned();
                    let tracked_files_matched = tracked
                        .iter()
                        .filter(|path| path_glob_matches(&glob, path))
                        .count();
                    ExclusionPolicy {
                        glob,
                        reason: entry["reason"].as_str().unwrap_or_default().to_owned(),
                        tracked_files_matched,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    Policy {
        rules,
        exclusions,
        baseline: baseline_doc["dangling"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default(),
        min_known_names: policy_doc["min_known_names"].as_u64().unwrap_or_default() as usize,
    }
}

/// Every dangling key the live scan produces, ignoring the frozen baseline.
fn live_dangling_keys(observed: &Observed) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for obs in &observed.rules {
        for site in &obs.sites {
            if site.excluded_by.is_some() {
                continue;
            }
            let live = match site.resolution {
                Resolution::AgainstKnownNames => observed.known_names.contains(&site.subject),
                Resolution::Prevalidated(resolves) => resolves,
            };
            if !live {
                keys.insert(finding_key(&site.rule_id, &site.file, &site.subject));
            }
        }
    }
    keys
}

#[test]
fn live_corpus_is_green_against_the_frozen_baseline() {
    let root = repo_root();
    let policy_doc = read_json(&root.join(POLICY_PATH));
    let baseline_doc = read_json(&root.join(BASELINE_PATH));
    let tracked = tracked_files(&root);
    let observed = collect(&root, &policy_doc, &tracked);
    let policy = load_policy(&policy_doc, &baseline_doc, &tracked);
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
        "{GATE_ID}: GREEN — {} sites checked across {} rules; {} dangling tolerated; {} known names",
        report.sites_checked,
        observed.rules.len(),
        report.dangling_tolerated,
        report.known_name_count
    );
}

/// The in-repo RED fixture, scanned with the fixture dir as the scan root, EMPTY baseline and
/// EMPTY exclusions. If this ever passes, the gate has stopped being able to fail.
#[test]
fn red_fixture_in_repo_fails_closed() {
    let root = repo_root();
    let fixture_root = root.join(FIXTURE_DIR);
    let tracked = tracked_files(&fixture_root);
    assert!(
        !tracked.is_empty(),
        "the RED fixture must be TRACKED, or this proof silently evaluates nothing"
    );

    // Re-root the live rules at the fixture dir; drop the exclusions and the baseline so nothing
    // can suppress the failure, and drop the census floor because the fixture has no workspace.
    let policy_doc = read_json(&root.join(POLICY_PATH));
    let mut rerooted = policy_doc.clone();
    rerooted["exclusions"] = Value::Array(Vec::new());
    rerooted["min_known_names"] = Value::from(0);
    for rule in rerooted["rules"].as_array_mut().expect("rules") {
        let extension = match rule["kind"].as_str() {
            Some("cargo_package_flag" | "yaml_frontmatter_field") => "md",
            Some("build_target_label") => "*",
            _ => "json",
        };
        rule["file_globs"] = Value::from(vec![format!("*.{extension}")]);
        rule["min_sites"] = Value::from(1u64);
    }

    let observed = Observed {
        rules: collect_rules(&fixture_root, &rerooted, &tracked),
        known_names: BTreeSet::new(),
    };
    let policy = load_policy(&rerooted, &serde_json::json!({ "dangling": [] }), &tracked);
    let report = evaluate(&observed, &policy);

    assert_eq!(report.verdict, Verdict::Red, "the RED fixture MUST fail");
    let dangling: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.code == CODE_DANGLING_CRATE_REFERENCE)
        .map(|f| f.subject.as_str())
        .collect();
    assert!(
        dangling.len() >= 3,
        "expected at least one dangling finding per site kind, got {dangling:?}"
    );
    let rule_ids: BTreeSet<&str> = dangling
        .iter()
        .filter_map(|subject| subject.split("::").next())
        .collect();
    assert!(
        rule_ids.len() >= 3,
        "the fixture must fail across at least three distinct rule kinds, got {rule_ids:?}"
    );
}

/// Shrink-only fidelity. A baselined reference whose defect is GONE is STALE: the subject came
/// back to life, the reference was fixed, or the file was deleted or renamed. Either way the
/// entry must be removed in the SAME change, or the slack outlives the debt.
#[test]
fn baselined_references_are_all_still_dangling() {
    let root = repo_root();
    let policy_doc = read_json(&root.join(POLICY_PATH));
    let baseline_doc = read_json(&root.join(BASELINE_PATH));
    let tracked = tracked_files(&root);
    let observed = collect(&root, &policy_doc, &tracked);
    let live = live_dangling_keys(&observed);

    let stale: Vec<&str> = baseline_doc["dangling"]
        .as_array()
        .expect("dangling array")
        .iter()
        .filter_map(Value::as_str)
        .filter(|key| !live.contains(*key))
        .collect();
    assert!(
        stale.is_empty(),
        "{} baselined dangling reference(s) no longer reproduce; remove them in this change: {stale:#?}",
        stale.len()
    );
}

/// FALSE-GREEN FLOOR: prove the collection actually walked the tree. Without this, a broken
/// collector reports zero sites, zero findings, and a clean pass.
#[test]
fn collector_sees_the_real_corpus() {
    let root = repo_root();
    let policy_doc = read_json(&root.join(POLICY_PATH));
    let tracked = tracked_files(&root);
    let observed = collect(&root, &policy_doc, &tracked);

    let mut per_rule: BTreeMap<&str, usize> = BTreeMap::new();
    for obs in &observed.rules {
        per_rule.insert(
            obs.rule_id.as_str(),
            obs.sites.iter().filter(|s| s.excluded_by.is_none()).count(),
        );
    }
    eprintln!("{GATE_ID}: sites per rule (excluding suppressed) = {per_rule:#?}");
    assert_eq!(
        observed.rules.len(),
        policy_doc["rules"].as_array().expect("rules").len(),
        "every declared rule must produce an observation"
    );
    for (rule_id, count) in &per_rule {
        assert!(*count > 0, "rule `{rule_id}` collected nothing — the collector is broken");
    }
}

/// Emitter for the frozen baseline. Ignored by default: it WRITES the committed baseline, so it
/// runs only when a human asks for a re-freeze.
///
///   cargo test -p ci-crate-reference-integrity --test crate_reference_integrity \
///       -- --ignored --exact emit_frozen_baseline
#[test]
#[ignore = "writes the committed baseline; run explicitly to re-freeze"]
fn emit_frozen_baseline() {
    let root = repo_root();
    let policy_doc = read_json(&root.join(POLICY_PATH));
    let tracked = tracked_files(&root);
    let observed = collect(&root, &policy_doc, &tracked);
    let keys: Vec<String> = live_dangling_keys(&observed).into_iter().collect();

    let doc = serde_json::json!({
        "_comment": "FROZEN shrink-only baseline of dangling crate references present at the first commit of ci/facade/crate-reference-integrity. Removing an entry is burn-down and always allowed; adding one is not. An entry whose defect is GONE is STALE and must be removed in the SAME change, or the slack outlives the debt.",
        "gate_id": GATE_ID,
        "_provenance": {
            "frozen_at": "first commit of ci/facade/crate-reference-integrity",
            "key_format": "<rule_id>::<file>::<subject>",
            "emitted_by": "cargo test -p ci-crate-reference-integrity --test crate_reference_integrity -- --ignored --exact emit_frozen_baseline"
        },
        "dangling": keys,
    });
    let mut text = serde_json::to_string_pretty(&doc).expect("serialize baseline");
    text.push('\n');
    std::fs::write(root.join(BASELINE_PATH), text).expect("write baseline");
    eprintln!("{GATE_ID}: wrote {BASELINE_PATH} with {} entries", keys.len());
}
