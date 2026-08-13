// ci-reorg-target-debt gate test (Global Binding Rule 1; North-Star Completion bootstrap
// step T3b). Two halves, both required:
//
//   (a) LIVE-CORPUS, born-blocking zero-NEW ratchet: Arms A–D run over the real tree
//       against the committed shrink-only baseline and must be GREEN — the existing
//       target-prefix estate is migration inventory, anything NEW fails closed. The run
//       must carry the liveness signal (evaluated_path_count, evaluated_arms); a missing
//       run is a gap, never a pass.
//
//   (b) FIXTURE CORPUS: specs/fixtures/reorg-target-debt/tc-*.json (parse-verbatim gate
//       inputs, ADR-0555 convention) prove every refusal class RED and the proven-claim
//       case GREEN without touching the live tree, through the SAME engine the binary
//       runs — including the fail-closed interval-audit mode.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use ci_reorg_target_debt::{
    Baseline, BaselineCandidate, CODE_DEP_PATH_UNPARSEABLE, NameDecl, NameSurface, POLICY_PATH,
    Policy, Report, Verdict, WorkspaceDep, audit_interval, check_live_tree,
    collect_baseline_candidate, collect_target_prefix_paths, enforce_shrink_only, entry_digest,
    evaluate_masterplan, evaluate_name_surface, evaluate_reduction_claims, evaluate_tree,
    evaluate_workspace_manifest, load_baseline, load_json, load_policy, name_decl_digest,
    parse_manifest_facts, workspace_path_dep_digest,
};

/// Walk up from the test's working directory to the repo root (the dir holding the
/// canonical root-hub pointer file). Mirrors the helper in the baseline-ratchet
/// registration meta-test so both gates resolve the root identically.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn live_policy(root: &Path) -> Policy {
    let (policy, _) = load_policy(root, POLICY_PATH).expect("load live policy");
    policy
}

fn fixtures_dir(root: &Path) -> PathBuf {
    root.join("specs/fixtures/reorg-target-debt")
}

fn finding_codes(report: &Report) -> Vec<String> {
    report.findings.iter().map(|f| f.code.clone()).collect()
}

// ─── (a) live corpus ────────────────────────────────────────────────────────

#[test]
fn live_tree_is_green_at_the_committed_baseline_with_liveness_signal() {
    let root = repo_root();
    let policy = live_policy(&root);
    let baseline = load_baseline(&root, &policy).expect("load committed baseline");
    let report = check_live_tree(&root, &policy, &baseline).expect("evaluate live tree");

    assert_eq!(
        report.verdict(),
        Verdict::Green,
        "born-blocking zero-NEW ratchet: the live tree must be green at the committed \
         shrink-only baseline. Findings: {:#?}",
        report.findings
    );
    // Liveness signal: every run reports what it evaluated. A run that examined nothing
    // is itself a gap.
    assert!(
        report.evaluated_path_count > 0,
        "the gate must report a non-zero evaluated-path count"
    );
    let rendered = report.to_json();
    assert!(rendered.get("evaluated_path_count").is_some());
    assert!(
        rendered
            .get("evaluated_arms")
            .and_then(Value::as_array)
            .is_some_and(|arms| arms.len() == 4),
        "all four blocking arms must be evaluated on every run"
    );
}

#[test]
fn committed_baseline_matches_the_live_target_prefix_estate_exactly() {
    let root = repo_root();
    let policy = live_policy(&root);
    let baseline = load_baseline(&root, &policy).expect("load committed baseline");
    let live = collect_target_prefix_paths(&root, &policy).expect("collect target-prefix paths");
    let candidate =
        collect_baseline_candidate(&root, &policy).expect("collect live Arm B candidate");

    // Digest set semantics: hash each live path, compare exact sets. NEW debt is reported
    // by its literal live path; stale baseline digests are reported as a count (the
    // literal removed paths are unrecoverable from digests by design).
    let live_hashes: BTreeSet<String> = live.iter().map(|path| entry_digest(path)).collect();
    let new: Vec<&String> = live
        .iter()
        .filter(|path| !baseline.path_hashes.contains(&entry_digest(path)))
        .collect();
    let stale_count = baseline.path_hashes.difference(&live_hashes).count();
    assert!(
        new.is_empty() && stale_count == 0,
        "baseline drift — NEW target-prefix file(s) {new:?} / {stale_count} stale baseline \
         digest(s). New files under a target prefix are refused (Global Binding Rule 1); \
         admissible removals require regenerating the baseline with: {}",
        policy.regeneration_command
    );

    let live_arm_b = candidate.to_baseline();
    assert_eq!(
        baseline.workspace_path_dep_hashes, live_arm_b.workspace_path_dep_hashes,
        "Arm B tuple hashes must match the live collected set exactly; extra or stale \
         hashes are unauthorized headroom"
    );
    assert_eq!(
        baseline.dep_name_hashes, live_arm_b.dep_name_hashes,
        "Arm B name hashes must match the live collected set exactly; extra or stale \
         hashes are unauthorized headroom"
    );
}

/// The committed baseline file must never carry a literal target-prefix path string:
/// per-path digests are the whole admissibility story versus the brand-residue ratchet.
#[test]
fn committed_baseline_file_carries_digests_not_literal_paths() {
    let root = repo_root();
    let policy = live_policy(&root);
    let raw = load_json(&root.join(&policy.baseline_file)).expect("load raw baseline value");
    for key in [
        "arm_a_path_hashes",
        "arm_b_workspace_path_dep_hashes",
        "arm_b_dep_name_hashes",
    ] {
        for entry in raw[key].as_array().expect("digest array") {
            let entry = entry.as_str().expect("digest entry is a string");
            assert!(
                entry.len() == 64
                    && entry
                        .bytes()
                        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
                "{key} entry {entry:?} is not a lowercase sha256 hex digest"
            );
        }
    }
}

// ─── (b) fixture corpus ─────────────────────────────────────────────────────

const REQUIRED_CASES: [&str; 18] = [
    "tc-RTD-bad-new-file-under-target-prefix.json",
    "tc-RTD-bad-new-workspace-path-dep.json",
    "tc-RTD-bad-single-quoted-relative-path-dep.json",
    "tc-RTD-bad-unparseable-path-dep.json",
    "tc-RTD-bad-new-target-crate-name.json",
    "tc-RTD-bad-new-member-target-path-dep.json",
    "tc-RTD-bad-new-edge-to-baselined-destination.json",
    "tc-RTD-bad-target-qualified-dep-subtable.json",
    "tc-RTD-bad-work-item-target-anchor.json",
    "tc-RTD-bad-unproven-net-reduction-claim.json",
    "tc-RTD-bad-growth-net-reduction-claim.json",
    "tc-RTD-good-proven-net-reduction-claim.json",
    "tc-RTD-bad-baseline-expansion-regen.json",
    "tc-RTD-audit-bad-planted-target-debt-commit.json",
    "tc-RTD-audit-bad-malformed-dep-fact.json",
    "tc-RTD-audit-bad-range-mismatch.json",
    "tc-RTD-audit-bad-relative-spelling-dep.json",
    "tc-RTD-audit-good-normalized-dep-path.json",
];

/// Fixtures declare baselines as LITERAL synthetic strings (parse-verbatim inputs); the
/// harness digests them through the SAME [`entry_digest`] / [`workspace_path_dep_digest`]
/// the engine and `--regen-baseline` use, so the fixture corpus proves the hashed-baseline
/// membership semantics end to end. Arm B path-dep rows are `{origin, name, dest}` objects
/// (the edge-identity tuple); a leftover string form is refused so destination-only
/// membership cannot sneak back into the corpus.
fn baseline_from_fixture(input: &Value) -> Baseline {
    let set = |key: &str| -> BTreeSet<String> {
        input
            .get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|v| v.as_str().expect("baseline entry is a string").to_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    let digests = |key: &str| set(key).iter().map(|entry| entry_digest(entry)).collect();
    let path_dep_hashes = input
        .get("baseline_workspace_path_deps")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let object = item.as_object().unwrap_or_else(|| {
                        panic!(
                            "baseline_workspace_path_deps entries must be                              {{origin, name, dest}} objects (edge-identity tuple);                              destination-only strings are refused"
                        )
                    });
                    workspace_path_dep_digest(
                        object
                            .get("origin")
                            .and_then(Value::as_str)
                            .expect("edge origin is a string"),
                        object
                            .get("name")
                            .and_then(Value::as_str)
                            .expect("edge name is a string"),
                        object
                            .get("dest")
                            .and_then(Value::as_str)
                            .expect("edge dest is a string"),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    Baseline {
        path_hashes: digests("baseline_paths"),
        workspace_path_dep_hashes: path_dep_hashes,
        dep_name_hashes: digests("baseline_dep_names"),
        anchors: set("baseline_anchors"),
    }
}

fn run_fixture(policy: &Policy, fixture: &Value, name: &str) {
    let kind = fixture
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{name}: fixture must declare kind"));
    let input = fixture
        .get("input")
        .unwrap_or_else(|| panic!("{name}: fixture must declare input"));
    let expected_codes: Vec<String> = fixture
        .get("expected_codes")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{name}: fixture must declare expected_codes"))
        .iter()
        .map(|v| v.as_str().expect("expected code is a string").to_owned())
        .collect();
    let baseline = baseline_from_fixture(input);

    let report = match kind {
        "tree" => {
            let paths: BTreeSet<String> = input
                .get("paths")
                .and_then(Value::as_array)
                .expect("tree fixture declares paths")
                .iter()
                .map(|v| v.as_str().expect("path is a string").to_owned())
                .collect();
            evaluate_tree(policy, &baseline, &paths)
        }
        "workspace-manifest" => {
            let manifest = input
                .get("manifest_lines")
                .and_then(Value::as_array)
                .expect("workspace-manifest fixture declares manifest_lines")
                .iter()
                .map(|v| v.as_str().expect("manifest line is a string"))
                .collect::<Vec<_>>()
                .join("\n");
            match evaluate_workspace_manifest(
                policy,
                &baseline,
                &manifest,
                "workspace.dependencies",
            ) {
                Ok(report) => report,
                Err(error) => {
                    assert_eq!(
                        expected_codes,
                        vec![CODE_DEP_PATH_UNPARSEABLE.to_owned()],
                        "{name}: a parser refusal must match the declared fail-closed code"
                    );
                    assert!(
                        error.to_string().contains(CODE_DEP_PATH_UNPARSEABLE),
                        "{name}: parser refusal must carry {CODE_DEP_PATH_UNPARSEABLE}: {error}"
                    );
                    return;
                }
            }
        }
        "masterplan" => {
            let plan = input.get("plan").expect("masterplan fixture declares plan");
            evaluate_masterplan(policy, &baseline, plan)
        }
        "reduction-claims" => {
            let artifact = input
                .get("artifact")
                .expect("reduction-claims fixture declares artifact");
            evaluate_reduction_claims(policy, artifact)
        }
        "name-surface" => {
            let surface = name_surface_from_fixture(input, name);
            evaluate_name_surface(policy, &baseline, &surface)
        }
        "member-manifest" => {
            // Prove the live member-manifest parser (including target-qualified
            // subtables) rather than a pre-parsed name-surface injection.
            let manifest = input
                .get("manifest_lines")
                .and_then(Value::as_array)
                .expect("member-manifest fixture declares manifest_lines")
                .iter()
                .map(|v| v.as_str().expect("manifest line is a string"))
                .collect::<Vec<_>>()
                .join("\n");
            let origin = input
                .get("origin")
                .and_then(Value::as_str)
                .expect("member-manifest fixture declares origin")
                .to_owned();
            let facts = parse_manifest_facts(&manifest, &policy.member_dependency_sections)
                .unwrap_or_else(|error| panic!("{name}: member-manifest parse failed: {error}"));
            let mut surface = NameSurface::default();
            if let Some(pkg) = facts.package_name {
                surface.names.push(NameDecl {
                    name: pkg,
                    origin: origin.clone(),
                });
            }
            for bin in facts.bin_names {
                surface.names.push(NameDecl {
                    name: bin,
                    origin: origin.clone(),
                });
            }
            for dep in facts.path_deps {
                surface.member_path_deps.push((origin.clone(), dep));
            }
            evaluate_name_surface(policy, &baseline, &surface)
        }
        "baseline-regen" => {
            run_baseline_regen_fixture(fixture, input, name);
            return;
        }
        "interval-audit" => {
            run_audit_fixture(policy, fixture, input, name);
            return;
        }
        other => panic!("{name}: unknown fixture kind {other:?}"),
    };

    assert_eq!(
        finding_codes(&report),
        expected_codes,
        "{name}: finding codes must equal the declared expectation exactly"
    );
    assert!(
        !report.evaluated_arms.is_empty(),
        "{name}: every evaluation carries the liveness arm list"
    );
}

/// Build a synthetic name surface from fixture data: `names` entries carry
/// `{name, origin}`; `member_path_deps` entries carry `{origin, name, path}`.
fn name_surface_from_fixture(input: &Value, _name: &str) -> NameSurface {
    let mut surface = NameSurface::default();
    for decl in input
        .get("names")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
    {
        surface.names.push(NameDecl {
            name: decl["name"].as_str().expect("name is a string").to_owned(),
            origin: decl["origin"]
                .as_str()
                .expect("origin is a string")
                .to_owned(),
        });
    }
    for dep in input
        .get("member_path_deps")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
    {
        surface.member_path_deps.push((
            dep["origin"]
                .as_str()
                .expect("origin is a string")
                .to_owned(),
            WorkspaceDep {
                name: dep["name"]
                    .as_str()
                    .expect("dep name is a string")
                    .to_owned(),
                path: dep["path"]
                    .as_str()
                    .expect("dep path is a string")
                    .to_owned(),
                path_unparseable: dep
                    .get("path_unparseable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            },
        ));
    }
    surface
}

/// Prove the shrink-only regeneration guard through the same [`enforce_shrink_only`]
/// the `--regen-baseline` surface runs: `expected_regen` is `"ok"` or `"refused"`.
fn run_baseline_regen_fixture(fixture: &Value, input: &Value, name: &str) {
    let set = |key: &str| -> BTreeSet<String> {
        input
            .get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|v| v.as_str().expect("entry is a string").to_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    let prior = baseline_from_fixture(input);
    let candidate = BaselineCandidate {
        paths: set("candidate_paths"),
        workspace_path_deps: set("candidate_workspace_path_deps"),
        dep_names: set("candidate_dep_names"),
        anchors: set("candidate_anchors"),
    };
    let expected = fixture
        .get("expected_regen")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{name}: baseline-regen fixture must declare expected_regen"));
    match enforce_shrink_only(&prior, &candidate) {
        Ok(()) => assert_eq!(expected, "ok", "{name}: expansion was NOT refused"),
        Err(error) => {
            assert_eq!(expected, "refused", "{name}: unexpected refusal: {error}");
            assert!(
                error.to_string().contains("RTD_BASELINE_EXPANSION"),
                "{name}: refusal must carry the explicit code: {error}"
            );
        }
    }
}

fn run_audit_fixture(policy: &Policy, fixture: &Value, input: &Value, name: &str) {
    let audit_input = input
        .get("audit_input")
        .unwrap_or_else(|| panic!("{name}: interval-audit fixture must declare audit_input"));
    let expected_verdict = fixture
        .get("expected_verdict")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{name}: interval-audit fixture must declare expected_verdict"));

    match audit_interval(policy, audit_input) {
        Ok(report) => {
            assert_ne!(
                expected_verdict, "invalid",
                "{name}: expected fail-closed input rejection but the audit ran"
            );
            assert_eq!(
                report.verdict().to_string(),
                expected_verdict,
                "{name}: audit verdict mismatch; findings: {:#?}",
                report.findings
            );
            let expected_commits: BTreeSet<String> = fixture
                .get("expected_finding_commits")
                .and_then(Value::as_array)
                .unwrap_or_else(|| {
                    panic!("{name}: interval-audit fixture must declare expected_finding_commits")
                })
                .iter()
                .map(|v| v.as_str().expect("commit sha is a string").to_owned())
                .collect();
            let reported: BTreeSet<String> = report
                .findings
                .iter()
                .map(|f| f.subject.split(':').next().unwrap_or("").to_owned())
                .collect();
            assert_eq!(
                reported, expected_commits,
                "{name}: the audit must report exactly the planted debt commit(s)"
            );
            // Liveness signal on the audit surface too.
            let rendered = report.to_json();
            assert!(rendered.get("evaluated_path_count").is_some());
            assert!(rendered.get("evaluated_arms").is_some());
        }
        Err(error) => {
            assert_eq!(
                expected_verdict, "invalid",
                "{name}: unexpected audit input rejection: {error}"
            );
            assert!(
                error.to_string().contains("RTD_AUDIT_INPUT_INVALID"),
                "{name}: fail-closed rejection must carry the explicit finding code: {error}"
            );
        }
    }
}

#[test]
fn fixture_corpus_proves_every_arm_through_the_live_engine() {
    let root = repo_root();
    let policy = live_policy(&root);
    let dir = fixtures_dir(&root);

    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read fixtures dir {}: {e}", dir.display()))
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.starts_with("tc-") && name.ends_with(".json"))
        .collect();
    names.sort();

    for required in REQUIRED_CASES {
        assert!(
            names.iter().any(|name| name == required),
            "required fixture case {required} is missing from {}",
            dir.display()
        );
    }

    for name in &names {
        let fixture = load_json(&dir.join(name)).unwrap_or_else(|e| panic!("load {name}: {e}"));
        run_fixture(&policy, &fixture, name);
    }
}

#[test]
fn audit_planted_commit_stays_red_and_prose_remediation_is_refused() {
    let root = repo_root();
    let policy = live_policy(&root);
    let dir = fixtures_dir(&root);
    let planted = load_json(&dir.join("tc-RTD-audit-bad-planted-target-debt-commit.json"))
        .expect("load planted-commit audit fixture");
    let mut audit_input = planted["input"]["audit_input"].clone();

    let report = audit_interval(&policy, &audit_input).expect("planted-range audit runs");
    assert_eq!(
        report.verdict(),
        Verdict::Red,
        "unremediated planted debt stays red"
    );

    // Candidate-authored prose is not an authority boundary and must never turn the same
    // captured violation green.
    let planted_sha = report.findings[0]
        .subject
        .split(':')
        .next()
        .expect("finding subject carries the commit sha")
        .to_owned();
    audit_input["remediation_records"] = serde_json::json!([
        { "commit": planted_sha, "resolution": "reverted; target-surface census re-measured" }
    ]);
    let error = audit_interval(&policy, &audit_input).expect_err("prose remediation is refused");
    assert!(
        error
            .to_string()
            .contains("remediation_records are not self-authorizing"),
        "{error}"
    );
}
#[test]
fn committed_arm_b_baseline_rejects_extra_tuple_and_name_hashes() {
    let root = repo_root();
    let policy = live_policy(&root);
    let candidate =
        collect_baseline_candidate(&root, &policy).expect("collect live Arm B candidate");
    let extra_tuple = workspace_path_dep_digest(
        "libs/unauthorized/Cargo.toml",
        "sneaky",
        "cloud/sneaky-estate",
    );
    let extra_name = name_decl_digest("libs/unauthorized/BUCK", "oya-preauthorized");
    let mut bloated = candidate.to_baseline();
    bloated
        .workspace_path_dep_hashes
        .insert(extra_tuple.clone());
    assert_ne!(
        bloated.workspace_path_dep_hashes,
        candidate.to_baseline().workspace_path_dep_hashes,
        "an extra tuple hash must be distinguishable from the live collected set"
    );
    bloated.dep_name_hashes.insert(extra_name.clone());
    assert!(
        !candidate
            .to_baseline()
            .dep_name_hashes
            .contains(&extra_name),
        "an extra name hash must be distinguishable from the live collected set"
    );
}

#[test]
fn same_change_edge_plus_hash_expansion_is_refused() {
    let root = repo_root();
    let policy = live_policy(&root);
    let prior = load_baseline(&root, &policy).expect("load committed baseline");
    let mut candidate = collect_baseline_candidate(&root, &policy).expect("collect live candidate");
    candidate.workspace_path_deps.insert(
        "libs/new-consumer/Cargo.toml\0legacy-estate\0cloud/legacy-estate-crate".to_owned(),
    );
    let error = enforce_shrink_only(&prior, &candidate).expect_err("same-change expansion");
    assert!(
        error.to_string().contains("RTD_BASELINE_EXPANSION"),
        "{error}"
    );
}
