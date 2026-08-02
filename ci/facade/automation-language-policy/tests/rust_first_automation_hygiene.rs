#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::{fs::PermissionsExt, process::CommandExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use oya_cloud_ci_rust_first_automation_hygiene_app::{
    Finding, Verdict, collect_observed_cli_package_authority,
    collect_observed_forbidden_workflow_uses, collect_observed_interpreter_command_authority,
    collect_observed_non_rust_automation, collect_observed_workflow_inline_shell, evaluate,
    evaluate_cli_package_authority, evaluate_forbidden_workflow_uses,
    evaluate_interpreter_command_authority, evaluate_keyed,
    evaluate_non_rust_exception_baseline_keyed, evaluate_workflow_inline_shell_keyed,
    load_non_rust_exception_baseline_from_merge_base, load_scan_from_merge_base,
    load_workflow_inline_shell_baseline_from_merge_base,
    validate_non_rust_exception_baseline_ceiling, validate_scan_scope_ceiling,
    validate_workflow_inline_shell_baseline_ceiling,
};
use serde_json::{Value, json};
use serde_yaml::Value as YamlValue;

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

fn gate_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/automation-language-policy")
}

fn policy_path(root: &Path) -> PathBuf {
    gate_dir(root).join("rust-first-automation-policy.json")
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn load_policy(root: &Path) -> Value {
    load_json(&policy_path(root))
}

fn run_git<const N: usize>(root: &Path, args: [&str; N]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git fixture command");
    assert!(
        output.status.success(),
        "git fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git fixture output is UTF-8")
}

fn named_workflow_step<'a>(workflow: &'a YamlValue, job_id: &str, name: &str) -> &'a YamlValue {
    let steps = workflow
        .get("jobs")
        .and_then(|jobs| jobs.get(job_id))
        .and_then(|job| job.get("steps"))
        .and_then(YamlValue::as_sequence)
        .unwrap_or_else(|| panic!("workflow job {job_id} must contain a steps sequence"));
    let matches = steps
        .iter()
        .filter(|step| step.get("name").and_then(YamlValue::as_str) == Some(name))
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one workflow step {name:?} in job {job_id:?}, found {}",
        matches.len()
    );
    matches[0]
}

#[test]
fn named_workflow_step_requires_an_exact_name_boundary() {
    let workflow = "\
jobs:
  gate:
    steps:
      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
        run: echo wrong
      - name: Materialize cloud-ci generated faces
        run: echo right
";

    let workflow: YamlValue = serde_yaml::from_str(workflow).expect("parse workflow fixture");
    let step = named_workflow_step(&workflow, "gate", "Materialize cloud-ci generated faces");
    assert_eq!(
        step.get("run").and_then(YamlValue::as_str),
        Some("echo right")
    );
}

#[test]
fn named_workflow_step_is_scoped_to_the_requested_job() {
    let workflow = "\
jobs:
  unrelated:
    steps:
      - name: Materialize cloud-ci generated faces
        run: echo wrong-job
  gate-affected-target-set:
    steps:
      - name: Materialize cloud-ci generated faces
        run: echo right-job
";

    let workflow: YamlValue = serde_yaml::from_str(workflow).expect("parse workflow fixture");
    let step = named_workflow_step(
        &workflow,
        "gate-affected-target-set",
        "Materialize cloud-ci generated faces",
    );
    assert_eq!(
        step.get("run").and_then(YamlValue::as_str),
        Some("echo right-job")
    );
}

#[test]
fn named_workflow_step_rejects_duplicate_exact_names_inside_the_requested_job() {
    let workflow: YamlValue = serde_yaml::from_str(
        "\
jobs:
  producer-regen:
    steps:
      - name: Materialize cloud-ci generated faces
        run: echo one
      - name: Materialize cloud-ci generated faces
        run: echo two
",
    )
    .expect("parse workflow fixture");

    let result = std::panic::catch_unwind(|| {
        named_workflow_step(
            &workflow,
            "producer-regen",
            "Materialize cloud-ci generated faces",
        )
    });
    assert!(
        result.is_err(),
        "duplicate exact step names must fail closed"
    );
}

fn keys_for(findings: &BTreeSet<Finding>, code: &str) -> BTreeSet<String> {
    findings
        .iter()
        .filter(|finding| finding.code == code)
        .map(|finding| finding.key.clone())
        .collect()
}

#[test]
fn live_repo_non_rust_automation_is_explicitly_exceptioned() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = collect_observed_non_rust_automation(&root, &policy)
        .expect("read-only repo scan should not need temp files or cleanup");
    let count = observed["rows"].as_array().expect("rows").len();
    assert!(
        count > 0,
        "expected non-empty live non-Rust automation inventory"
    );

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "Rust-first automation hygiene gate found violations over {count} observed paths: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
}

#[test]
fn fixture_proves_unregistered_script_fails_closed() {
    let policy = json!({
        "gate_id": "cloud-ci-rust-first-automation-hygiene",
        "exceptions": []
    });
    let observed = json!({"rows": [{"path": "scripts/new-local-shell.sh"}]});
    let findings = evaluate_keyed(&policy, &observed);
    assert!(findings.iter().any(|finding| {
        finding.code == "rust_first_automation_unregistered_non_rust_automation"
            && finding.key == "scripts/new-local-shell.sh"
    }));
}

#[test]
fn thirdparty_python_overlay_is_retired_into_owned_rust() {
    let root = repo_root();
    let policy = load_policy(&root);
    let paths = [
        "tools/buck/apply-thirdparty-patches.py",
        "tools/buck/tests/test_apply_thirdparty_patches.py",
    ];
    let exceptions = policy["exceptions"].as_array().expect("exceptions array");
    let baseline = policy["non_rust_exception_baseline"]["codes"]
        ["rust_first_automation_unbaselined_non_rust_exception"]
        .as_array()
        .expect("non-Rust exception baseline array");

    for path in paths {
        assert!(
            !root.join(path).exists(),
            "retired Python overlay surface must be absent: {path}"
        );
        assert!(
            !exceptions
                .iter()
                .any(|row| row["path"].as_str() == Some(path)),
            "retired Python overlay must not remain exceptioned: {path}"
        );
        assert!(
            !baseline.iter().any(|value| value.as_str() == Some(path)),
            "retired Python overlay must shrink from the frozen baseline: {path}"
        );
    }
    assert!(
        root.join("ci/facade/dependency-automation/src/third_party_overlay.rs")
            .is_file(),
        "the semantic overlay must live in the owned Rust dependency-automation capability"
    );
    let wrapper = exceptions
        .iter()
        .find(|row| row["path"].as_str() == Some("scripts/ci/regen-third-party.sh"))
        .expect("remaining Reindeer wrapper exception");
    assert!(
        wrapper["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("owned Rust/Buck2")),
        "remaining wrapper debt must distinguish the Rust-owned overlay: {wrapper:#?}"
    );
}

/// The two Python BUCK generators are retired, and the one whose JOB is still live has a NAMED
/// owned-Rust successor. This is the pointer: `scripts/emit_rust_tests.py` mirrored a
/// `rust_library` into a `<name>-unittest` `rust_test`, which is exactly what the ADR-0540
/// generator `//tools/oya-buck-test-wiring-app` does — and does strictly better, because the
/// Python version carried the same blanket `already has a rust_test` skip that #1496 removed from
/// the Rust tool (it hid every crate whose unit test was wired but whose `tests/*.rs` were not).
/// `scripts/gen_first_party_buck.py` has no successor because it has no job: all 868 workspace
/// members already carry a BUCK file, so its only non-no-op mode was `--force`, which clobbers
/// hand-edited BUCK.
#[test]
fn python_buck_generators_are_retired_with_the_wiring_successor_named() {
    let root = repo_root();
    let policy = load_policy(&root);
    let paths = ["scripts/emit_rust_tests.py", "scripts/gen_first_party_buck.py"];
    let exceptions = policy["exceptions"].as_array().expect("exceptions array");
    let baseline = policy["non_rust_exception_baseline"]["codes"]
        ["rust_first_automation_unbaselined_non_rust_exception"]
        .as_array()
        .expect("non-Rust exception baseline array");

    for path in paths {
        assert!(
            !root.join(path).exists(),
            "retired Python BUCK generator must be absent: {path}"
        );
        assert!(
            !exceptions
                .iter()
                .any(|row| row["path"].as_str() == Some(path)),
            "retired Python BUCK generator must not remain exceptioned: {path}"
        );
        assert!(
            !baseline.iter().any(|value| value.as_str() == Some(path)),
            "retired Python BUCK generator must shrink from the frozen baseline: {path}"
        );
    }

    assert!(
        root.join("tools/oya-buck-test-wiring-app/src/lib.rs").is_file(),
        "the rust_test wiring job must live in the owned-Rust ADR-0540 generator \
         //tools/oya-buck-test-wiring-app — retiring the Python emitter without it would drop a \
         live job, not a dead one"
    );
}

// ───────────────────────── workflow-inline-shell dimension (pipeline-glue(a)) ─────────────────────

/// The `.github/workflows` surface MUST be in the gate's scan scope: the policy declares it as a
/// workflow-inline-shell root, the dimension is enabled, and the scanner actually finds steps there.
/// This is the explicit "blind spot closed" assertion.
#[test]
fn workflow_inline_shell_dimension_covers_dot_github_workflows() {
    let root = repo_root();
    let policy = load_policy(&root);
    let block = &policy["scan"]["workflow_inline_shell"];
    assert_eq!(
        block["enabled"].as_bool(),
        Some(true),
        "workflow_inline_shell dimension must be enabled in policy DATA"
    );
    let roots: Vec<&str> = block["roots"]
        .as_array()
        .expect("workflow_inline_shell.roots")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        roots.contains(&".github/workflows"),
        ".github/workflows must be a declared workflow-inline-shell scan root; got {roots:?}"
    );

    let observed = collect_observed_workflow_inline_shell(&root, &policy)
        .expect("read-only workflow scan should not need temp files or cleanup");
    let steps = observed["steps"].as_array().expect("steps");
    assert!(
        !steps.is_empty()
            && steps.iter().all(|s| s["file"]
                .as_str()
                .is_some_and(|f| f.starts_with(".github/workflows/"))),
        "scanner must surface inline-shell steps under .github/workflows; got {} steps",
        steps.len()
    );
}

#[test]
fn live_scan_scope_does_not_narrow_the_immutable_merge_base_configuration() {
    let root = repo_root();
    let policy = load_policy(&root);
    let protected_scan =
        load_scan_from_merge_base(&root).expect("read immutable merge-base scan configuration");
    let findings = validate_scan_scope_ceiling(&policy["scan"], &protected_scan);
    assert!(
        findings.is_empty(),
        "candidate scan scope must not narrow the immutable merge-base configuration: {findings:#?}"
    );
}

#[test]
fn retirement_event_transport_delegates_provider_tuple_to_rust_materializer_without_shell_topology()
{
    let root = repo_root();
    let workflow_path = root.join(".github/workflows/oya-ci-required.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", workflow_path.display()));
    let workflow_doc: YamlValue = serde_yaml::from_str(&workflow)
        .unwrap_or_else(|e| panic!("parse {}: {e}", workflow_path.display()));

    assert!(
        workflow.contains("workflow_dispatch:"),
        "manual reruns remain an explicitly declared workflow surface"
    );

    let env = workflow_doc
        .get("env")
        .unwrap_or_else(|| panic!("workflow must declare one inherited provider-event env"));
    for (key, binding) in [
        ("EVENT_EVALUATED_SHA", "${{ github.sha }}"),
        (
            "EVENT_PULL_REQUEST_BASE_SHA",
            "${{ github.event.pull_request.base.sha || '' }}",
        ),
        ("EVENT_PUSH_BEFORE_SHA", "${{ github.event.before || '' }}"),
        ("EVENT_PUSH_AFTER_SHA", "${{ github.event.after || '' }}"),
        (
            "EVENT_MERGE_GROUP_BASE_SHA",
            "${{ github.event.merge_group.base_sha || '' }}",
        ),
        (
            "EVENT_MERGE_GROUP_HEAD_SHA",
            "${{ github.event.merge_group.head_sha || '' }}",
        ),
        (
            "EVENT_PULL_REQUEST_HEAD_SHA",
            "${{ github.event.pull_request.head.sha || '' }}",
        ),
        ("EVENT_NAME", "${{ github.event_name }}"),
        ("EVENT_REF", "${{ github.ref }}"),
        (
            "EVENT_PULL_REQUEST_BASE_REF",
            "${{ github.event.pull_request.base.ref || '' }}",
        ),
        (
            "EVENT_MERGE_GROUP_BASE_REF",
            "${{ github.event.merge_group.base_ref || '' }}",
        ),
    ] {
        assert_eq!(
            env.get(key).and_then(YamlValue::as_str),
            Some(binding),
            "workflow provider-event env binding drifted for {key}"
        );
    }
    let materialize = named_workflow_step(
        &workflow_doc,
        "producer-regen",
        "Materialize cloud-ci generated faces",
    );
    let run = materialize
        .get("run")
        .and_then(YamlValue::as_str)
        .expect("producer materializer must be a Rust-owned run step");
    assert!(
        run.contains("buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root . --github-event"),
        "the one-line Rust materializer must own provider-tuple interpretation"
    );
    for forbidden in ["if [", "case ", "git ", "HEAD^", "rev-list", "cat-file"] {
        assert!(
            !run.contains(forbidden),
            "producer shell must not own branching or git topology command {forbidden:?}"
        );
    }

    let census_gate = named_workflow_step(&workflow_doc, "gate", "buck2 test ${{ matrix.crate }}");
    let census_gate_run = census_gate
        .get("run")
        .and_then(YamlValue::as_str)
        .expect("scm-facts census receipt gate must be a Rust-owned run step");
    assert!(
        census_gate_run.contains(
            "buck2 run //ci/facade/scm-facts-snapshot:adr-census-epoch-receipt-gate-bin -- --repo-root . --github-event"
        ),
        "the live scm-facts census receipt gate must retain provider-event identity"
    );

    for line in workflow.lines().filter(|line| {
        (line.contains("oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .")
            && !line.contains("--help")
            && !line.contains("historical_retirement_args"))
            || line.contains("\"${freshness_bin}\" --repo-root .")
            || line.contains("\"${materializer_bin}\" --repo-root .")
    }) {
        assert!(
            line.contains("--github-event"),
            "every live candidate regeneration must delegate event identity to Rust: {line}"
        );
    }
}

#[test]
fn retirement_control_plane_has_a_dedicated_owners_boundary() {
    let root = repo_root();
    let control_plane_root = root.join("registry/history-only-retirement");

    assert!(
        control_plane_root.join("control-plane.json").is_file(),
        "the retirement control plane must live in its dedicated registry subtree"
    );
    assert!(
        control_plane_root.join("OWNERS").is_file(),
        "the retirement control plane must have a nearest-ancestor OWNERS boundary"
    );
    assert!(
        !root.join("registry/OWNERS").exists(),
        "the retirement control plane must not introduce registry-root blanket ownership"
    );
}

/// The live-Postgres lanes used to ARCHIVE a bootstrap-provenance JSON blob whose real intent
/// was to show the RLS tests were not vacuous — that they ran as a NOSUPERUSER/NOBYPASSRLS role
/// and so were capable of failing. Nothing ever downloaded it and no ADR governed it, so the
/// intent moved to where it can actually fail the build: a fail-closed assertion in each live
/// path. This test keeps that from silently rotting back into an unasserted artifact —
/// `oya-data-sql-adapter-sqlx` had no such assertion at all until the archive was removed.
#[test]
fn every_live_postgres_path_asserts_rls_is_enforceable() {
    let root = repo_root();
    // Each source below backs one target the gate-live-postgres-* jobs run, and must carry
    // either its own rolsuper/rolbypassrls probe or the shared boot guard.
    for relative_path in [
        "libs/oya-data-sql-adapter-sqlx/src/lib.rs",
        "libs/oya-data-outbox-adapter-postgres/src/lib.rs",
        "tenancy/adapters/tenant-lifecycle-store-postgres/src/lib.rs",
        "iam/adapters/identity-scim-store-postgres/src/lib.rs",
        "tenancy/facade/tenant-lifecycle-app/src/lib.rs",
        "iam/facade/identity-service/src/server.rs",
    ] {
        let path = root.join(relative_path);
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            source.contains("rolbypassrls") || source.contains("assert_rls_enforceable"),
            "{relative_path} drives a live-Postgres CI target but asserts nothing about RLS \
             bypass; without it the cross-tenant-deny tests can pass vacuously"
        );
    }
}

/// The live workflow corpus inline-shell key set must equal the committed frozen baseline EXACTLY
/// (set equality, mirroring the embedded-asset hermeticity gate's shrink-only contract): a NEW
/// inline `run:` step beyond baseline is born-blocking, a RETIRED step must shrink the baseline in
/// the same PR. With the corpus == baseline, the dimension is GREEN today.
#[test]
fn live_workflow_inline_shell_matches_frozen_baseline_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    let protected_baseline = load_workflow_inline_shell_baseline_from_merge_base(&root).expect(
        "read the workflow inline-shell baseline from the immutable merge-base policy tree",
    );
    let candidate_baseline = &policy["workflow_inline_shell_baseline"];

    let observed = collect_observed_workflow_inline_shell(&root, &policy)
        .expect("read-only workflow scan should not need temp files or cleanup");
    let steps = observed["steps"].as_array().expect("steps").len();
    assert!(
        steps > 0,
        "expected a non-empty live workflow inline-shell inventory (the blind spot is real)"
    );

    let mut findings =
        validate_workflow_inline_shell_baseline_ceiling(candidate_baseline, &protected_baseline);
    findings.extend(evaluate_workflow_inline_shell_keyed(
        &observed,
        candidate_baseline,
    ));
    assert!(
        findings.is_empty(),
        "workflow inline-shell dimension found shrink-only violations over {steps} observed steps: \
         {findings:#?}\n  unbaselined (new beyond baseline) = {:?}\n  stale (baselined but gone) = {:?}",
        keys_for(
            &findings,
            "rust_first_automation_unbaselined_workflow_inline_shell"
        ),
        keys_for(
            &findings,
            "rust_first_automation_workflow_inline_shell_baseline_stale"
        ),
    );

    // The committed keys_total provenance must match the actual baseline key array length — a cheap
    // tripwire against a hand-edited baseline whose provenance count drifts from its `codes` array.
    let codes_len =
        candidate_baseline["codes"]["rust_first_automation_unbaselined_workflow_inline_shell"]
            .as_array()
            .expect("baseline codes array")
            .len();
    let provenance_total = candidate_baseline["_provenance"]["keys_total"]
        .as_u64()
        .expect("baseline _provenance.keys_total") as usize;
    assert_eq!(
        codes_len, provenance_total,
        "baseline _provenance.keys_total ({provenance_total}) must equal the codes array length \
         ({codes_len})"
    );
}

/// RED FIXTURE (mandatory, proves non-inert): injecting a NEW inline `run:` step beyond the frozen
/// baseline must make the dimension RED with the NEW key surfaced under the unbaselined code.
#[test]
fn new_inline_shell_beyond_baseline_is_born_blocking_red() {
    let baseline = json!({
        "codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": ".github/workflows/oya-ci-required.yml::buck2::Named shell", "shell_lines": 1}
            ]
        }
    });
    // Baselined key is present (accepted) + a NEW unbaselined key injected.
    let observed = json!({"steps": [
        {"key": ".github/workflows/oya-ci-required.yml::buck2::Named shell", "shell_lines": 1},
        {"key": ".github/workflows/oya-ci-required.yml::buck2::New shell", "shell_lines": 1}
    ]});
    let findings = evaluate_workflow_inline_shell_keyed(&observed, &baseline);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_unbaselined_workflow_inline_shell"
                && finding.key == ".github/workflows/oya-ci-required.yml::buck2::New shell"
        }),
        "a new inline-shell step beyond baseline must be born-blocking with its key surfaced; got \
         {findings:#?}"
    );
    // The accepted baselined key must NOT be flagged (no false positive).
    assert!(
        !findings.iter().any(|finding| {
            finding.key == ".github/workflows/oya-ci-required.yml::buck2::Named shell"
        }),
        "an accepted baselined key must not be flagged"
    );
}

/// A baselined key that no longer exists in the corpus must surface the stale code (forces the
/// baseline to shrink in the same PR, mirroring the file-scan `exception_stale` contract).
#[test]
fn retired_baselined_inline_shell_is_stale_red() {
    let baseline = json!({
        "codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": ".github/workflows/docs-graph-drift.yml::docs-graph-drift::Named shell", "shell_lines": 1}
            ]
        }
    });
    let observed = json!({"steps": []});
    let findings = evaluate_workflow_inline_shell_keyed(&observed, &baseline);
    assert!(findings.iter().any(|finding| {
        finding.code == "rust_first_automation_workflow_inline_shell_baseline_stale"
            && finding.key
                == ".github/workflows/docs-graph-drift.yml::docs-graph-drift::Named shell"
    }));
}

// ─────────────────── non-Rust-exception SHRINK-ONLY dimension ───────────────────

/// The live exceptions[] allowlist must equal the frozen review-visible baseline EXACTLY: a NEW
/// non-Rust bridge beyond baseline is born-blocking, a removed bridge must shrink the baseline in
/// the same PR. With the allowlist == baseline, the dimension is GREEN today, and the
/// _provenance.keys_total tripwire must match the baseline array length.
#[test]
fn live_non_rust_exceptions_match_frozen_baseline_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    let protected_baseline = load_non_rust_exception_baseline_from_merge_base(&root)
        .expect("read the non-Rust exception baseline from the immutable merge-base policy tree");
    let candidate_baseline = &policy["non_rust_exception_baseline"];

    let mut findings =
        validate_non_rust_exception_baseline_ceiling(candidate_baseline, &protected_baseline);
    findings.extend(evaluate_non_rust_exception_baseline_keyed(
        &policy,
        candidate_baseline,
    ));
    assert!(
        findings.is_empty(),
        "non-Rust-exception dimension found shrink-only violations: \
         {findings:#?}\n  unbaselined (new beyond baseline) = {:?}\n  stale (baselined but gone) = {:?}",
        keys_for(
            &findings,
            "rust_first_automation_unbaselined_non_rust_exception"
        ),
        keys_for(
            &findings,
            "rust_first_automation_non_rust_exception_baseline_stale"
        ),
    );

    let codes_len =
        candidate_baseline["codes"]["rust_first_automation_unbaselined_non_rust_exception"]
            .as_array()
            .expect("baseline codes array")
            .len();
    let provenance_total = candidate_baseline["_provenance"]["keys_total"]
        .as_u64()
        .expect("baseline _provenance.keys_total") as usize;
    assert_eq!(
        codes_len, provenance_total,
        "baseline _provenance.keys_total ({provenance_total}) must equal the codes array length \
         ({codes_len})"
    );
}

/// Regression for #1190: a candidate must not add both an exception and that exception to its
/// local baseline.  The baseline is loaded by Git object ID from the merge-base tree, so only a
/// separately protected change can admit an additional exception.
#[test]
fn candidate_policy_cannot_self_waive_new_exception_by_editing_its_baseline() {
    let temp = TestDir::new("automation-policy-merge-base");
    let root = temp.path();
    let path = policy_path(root);
    fs::create_dir_all(path.parent().expect("policy parent")).expect("create policy directory");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);

    let protected_policy = json!({
        "exceptions": [{"path": "scripts/accepted.sh"}],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": ["scripts/accepted.sh"]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&protected_policy).expect("serialize protected policy"),
    )
    .expect("write protected policy");
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "protected policy"]);
    let protected_commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        [
            "update-ref",
            "refs/remotes/origin/dev",
            protected_commit.trim(),
        ],
    );

    let candidate_policy = json!({
        "exceptions": [
            {"path": "scripts/accepted.sh"},
            {"path": "scripts/candidate-self-waiver.sh"}
        ],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": [
                "scripts/accepted.sh",
                "scripts/candidate-self-waiver.sh"
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&candidate_policy).expect("serialize candidate policy"),
    )
    .expect("write candidate policy");

    let vulnerable_candidate_baseline = &candidate_policy["non_rust_exception_baseline"];
    assert!(
        evaluate_non_rust_exception_baseline_keyed(
            &candidate_policy,
            vulnerable_candidate_baseline
        )
        .is_empty(),
        "control: the candidate-controlled baseline demonstrates the pre-#1190 self-waiver"
    );

    let baseline = load_non_rust_exception_baseline_from_merge_base(root)
        .expect("load immutable protected-base baseline");
    let findings = validate_non_rust_exception_baseline_ceiling(
        &candidate_policy["non_rust_exception_baseline"],
        &baseline,
    );
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_unbaselined_non_rust_exception"
                && finding.key == "scripts/candidate-self-waiver.sh"
        }),
        "candidate policy plus candidate baseline must not self-waive: {findings:#?}"
    );
}

/// Regression for #1190: workflow inline shell debt is equally candidate-controlled when its
/// baseline comes from the candidate policy. The baseline must instead be read from the immutable
/// merge-base policy tree, so a candidate workflow plus a matching candidate baseline cannot waive
/// its own new shell step.
#[test]
fn candidate_workflow_shell_cannot_self_waive_by_editing_its_baseline() {
    let temp = TestDir::new("automation-policy-workflow-merge-base");
    let root = temp.path();
    let path = policy_path(root);
    let workflows = root.join(".github/workflows");
    fs::create_dir_all(path.parent().expect("policy parent")).expect("create policy directory");
    fs::create_dir_all(&workflows).expect("create workflow directory");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);

    let scan = json!({
        "workflow_inline_shell": {
            "enabled": true,
            "roots": [".github/workflows"],
            "extensions": [".yml"]
        }
    });
    let accepted_key = ".github/workflows/required.yml::gate::Accepted shell";
    let protected_policy = json!({
        "scan": scan,
        "workflow_inline_shell_baseline": {"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": accepted_key, "shell_lines": 1}
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&protected_policy).expect("serialize protected policy"),
    )
    .expect("write protected policy");
    fs::write(
        workflows.join("required.yml"),
        "jobs:\n  gate:\n    steps:\n      - name: Accepted shell\n        run: echo accepted\n",
    )
    .expect("write protected workflow");
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "protected policy"]);
    let protected_commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        [
            "update-ref",
            "refs/remotes/origin/dev",
            protected_commit.trim(),
        ],
    );

    let candidate_key = ".github/workflows/required.yml::gate::Candidate self waiver";
    let candidate_policy = json!({
        "scan": protected_policy["scan"],
        "workflow_inline_shell_baseline": {"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": accepted_key, "shell_lines": 1},
                {"key": candidate_key, "shell_lines": 1}
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&candidate_policy).expect("serialize candidate policy"),
    )
    .expect("write candidate policy");
    fs::write(
        workflows.join("required.yml"),
        "jobs:\n  gate:\n    steps:\n      - name: Accepted shell\n        run: echo accepted\n      - name: Candidate self waiver\n        run: echo candidate\n",
    )
    .expect("write candidate workflow");

    let observed = collect_observed_workflow_inline_shell(root, &candidate_policy)
        .expect("scan candidate workflow");
    assert!(
        evaluate_workflow_inline_shell_keyed(
            &observed,
            &candidate_policy["workflow_inline_shell_baseline"]
        )
        .is_empty(),
        "control: the candidate-controlled baseline demonstrates the pre-#1190 self-waiver"
    );

    let baseline = load_workflow_inline_shell_baseline_from_merge_base(root)
        .expect("load immutable protected-base workflow baseline");
    let findings = validate_workflow_inline_shell_baseline_ceiling(
        &candidate_policy["workflow_inline_shell_baseline"],
        &baseline,
    );
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_unbaselined_workflow_inline_shell"
                && finding.key == candidate_key
        }),
        "candidate workflow plus candidate baseline must not self-waive: {findings:#?}"
    );
}

#[test]
fn immutable_baseline_loader_fails_closed_when_ref_or_policy_object_is_missing() {
    let temp = TestDir::new("automation-policy-missing-frozen-object");
    let root = temp.path();
    fs::write(root.join("README"), "unrelated commit\n").expect("write unrelated file");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "unrelated commit"]);

    assert!(
        load_workflow_inline_shell_baseline_from_merge_base(root).is_err(),
        "missing origin/dev must fail closed"
    );

    let commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        ["update-ref", "refs/remotes/origin/dev", commit.trim()],
    );
    assert!(
        load_workflow_inline_shell_baseline_from_merge_base(root).is_err(),
        "missing merge-base policy object must fail closed"
    );
}

/// Regression for #1190: candidate-owned scan scope must not hide both a new workflow inline
/// shell and a non-Rust script while candidate baselines shrink to match the narrowed scan.
#[test]
fn candidate_scan_scope_cannot_hide_new_action_shell_or_non_rust_script() {
    let temp = TestDir::new("automation-policy-scan-scope-merge-base");
    let root = temp.path();
    let path = policy_path(root);
    fs::create_dir_all(path.parent().expect("policy parent")).expect("create policy directory");
    fs::create_dir_all(root.join(".github/actions")).expect("create action directory");
    fs::create_dir_all(root.join("scripts")).expect("create script directory");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);

    let protected_policy = json!({
        "scan": {
            "roots": ["scripts"],
            "exclude_prefixes": [],
            "non_rust_extensions": [".sh"],
            "workflow_inline_shell": {
                "enabled": true,
                "roots": [".github/actions"],
                "extensions": [".yml"]
            }
        },
        "exceptions": [],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": []
        }},
        "workflow_inline_shell_baseline": {"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": []
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&protected_policy).expect("serialize protected policy"),
    )
    .expect("write protected policy");
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "protected policy"]);
    let protected_commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        [
            "update-ref",
            "refs/remotes/origin/dev",
            protected_commit.trim(),
        ],
    );

    let candidate_policy = json!({
        "scan": {
            "roots": [],
            "exclude_prefixes": ["scripts/"],
            "non_rust_extensions": [],
            "workflow_inline_shell": {
                "enabled": false,
                "roots": [],
                "extensions": []
            }
        },
        "exceptions": [],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": []
        }},
        "workflow_inline_shell_baseline": {"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": []
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&candidate_policy).expect("serialize candidate policy"),
    )
    .expect("write candidate policy");
    fs::write(
        root.join(".github/actions/hidden.yml"),
        "runs:\n  steps:\n    - name: Hidden shell\n      run: echo hidden\n",
    )
    .expect("write hidden action");
    fs::write(root.join("scripts/hidden.sh"), "#!/bin/sh\necho hidden\n")
        .expect("write hidden script");

    let observed_scripts = collect_observed_non_rust_automation(root, &candidate_policy)
        .expect("candidate scan succeeds while hiding script");
    let observed_shell = collect_observed_workflow_inline_shell(root, &candidate_policy)
        .expect("candidate scan succeeds while hiding action shell");
    assert!(
        evaluate_non_rust_exception_baseline_keyed(
            &candidate_policy,
            &candidate_policy["non_rust_exception_baseline"]
        )
        .is_empty()
            && evaluate_workflow_inline_shell_keyed(
                &observed_shell,
                &candidate_policy["workflow_inline_shell_baseline"]
            )
            .is_empty()
            && observed_scripts["rows"]
                .as_array()
                .is_some_and(Vec::is_empty),
        "control: candidate-owned scope plus narrowed baselines hides both additions"
    );

    let protected_scan =
        load_scan_from_merge_base(root).expect("load immutable protected-base scan configuration");
    let findings = validate_scan_scope_ceiling(&candidate_policy["scan"], &protected_scan);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_scan_scope_narrowing"
                && finding.key == "scan.workflow_inline_shell.enabled"
        }) && findings.iter().any(|finding| {
            finding.code == "rust_first_automation_scan_scope_narrowing"
                && finding.key == "scan.roots"
        }) && findings.iter().any(|finding| {
            finding.code == "rust_first_automation_scan_scope_narrowing"
                && finding.key == "scan.exclude_prefixes"
        }),
        "immutable scope must reject the candidate self-waiver: {findings:#?}"
    );
}

#[test]
fn candidate_scan_scope_may_broaden_coverage() {
    let protected_scan = json!({
        "roots": ["scripts"],
        "exclude_prefixes": ["target/"],
        "non_rust_extensions": [".sh"],
        "workflow_inline_shell": {
            "enabled": true,
            "roots": [".github/workflows"],
            "extensions": [".yml"]
        }
    });
    let candidate_scan = json!({
        "roots": ["scripts", "tools"],
        "exclude_prefixes": [],
        "non_rust_extensions": [".sh", ".py"],
        "workflow_inline_shell": {
            "enabled": true,
            "roots": [".github/workflows", ".github/actions"],
            "extensions": [".yml", ".yaml"]
        }
    });
    let findings = validate_scan_scope_ceiling(&candidate_scan, &protected_scan);
    assert!(
        findings.is_empty(),
        "candidate scan broadening must remain permitted: {findings:#?}"
    );
}

/// A candidate may retire a non-Rust exception and shrink its matching baseline in the same PR.
/// The merge-base baseline is only an anti-expansion ceiling, not a requirement to retain debt.
#[test]
fn candidate_non_rust_baseline_removal_below_merge_base_ceiling_is_green() {
    let temp = TestDir::new("automation-policy-removal-below-merge-base");
    let root = temp.path();
    let path = policy_path(root);
    fs::create_dir_all(path.parent().expect("policy parent")).expect("create policy directory");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);

    let protected_policy = json!({
        "exceptions": [
            {"path": "scripts/accepted.sh"},
            {"path": "scripts/retired.sh"}
        ],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": [
                "scripts/accepted.sh", "scripts/retired.sh"
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&protected_policy).expect("serialize protected policy"),
    )
    .expect("write protected policy");
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "protected policy"]);
    let protected_commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        [
            "update-ref",
            "refs/remotes/origin/dev",
            protected_commit.trim(),
        ],
    );

    let candidate_policy = json!({
        "exceptions": [{"path": "scripts/accepted.sh"}],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": ["scripts/accepted.sh"]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&candidate_policy).expect("serialize candidate policy"),
    )
    .expect("write candidate policy");

    let protected_baseline = load_non_rust_exception_baseline_from_merge_base(root)
        .expect("load immutable protected-base baseline");
    let ceiling_findings = validate_non_rust_exception_baseline_ceiling(
        &candidate_policy["non_rust_exception_baseline"],
        &protected_baseline,
    );
    let synchronization_findings = evaluate_non_rust_exception_baseline_keyed(
        &candidate_policy,
        &candidate_policy["non_rust_exception_baseline"],
    );
    assert!(
        ceiling_findings.is_empty() && synchronization_findings.is_empty(),
        "synchronized candidate baseline removal must be admitted: ceiling={ceiling_findings:#?}, \
         synchronization={synchronization_findings:#?}"
    );
}

/// A candidate may reduce a workflow shell block and lower its matching baseline line count in the
/// same PR. The immutable merge-base count is an anti-regrowth ceiling, not a retention floor.
#[test]
fn candidate_workflow_line_count_reduction_below_merge_base_ceiling_is_green() {
    let temp = TestDir::new("automation-policy-workflow-reduction-below-merge-base");
    let root = temp.path();
    let path = policy_path(root);
    let workflows = root.join(".github/workflows");
    fs::create_dir_all(path.parent().expect("policy parent")).expect("create policy directory");
    fs::create_dir_all(&workflows).expect("create workflow directory");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);

    let scan = json!({"workflow_inline_shell": {
        "enabled": true,
        "roots": [".github/workflows"],
        "extensions": [".yml"]
    }});
    let key = ".github/workflows/required.yml::gate::Reduced shell";
    let protected_policy = json!({
        "scan": scan,
        "workflow_inline_shell_baseline": {"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": key, "shell_lines": 2}
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&protected_policy).expect("serialize protected policy"),
    )
    .expect("write protected policy");
    fs::write(
        workflows.join("required.yml"),
        "jobs:\n  gate:\n    steps:\n      - name: Reduced shell\n        run: |\n          echo first\n          echo second\n",
    )
    .expect("write protected workflow");
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "protected policy"]);
    let protected_commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        [
            "update-ref",
            "refs/remotes/origin/dev",
            protected_commit.trim(),
        ],
    );

    let candidate_policy = json!({
        "scan": protected_policy["scan"],
        "workflow_inline_shell_baseline": {"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": key, "shell_lines": 1}
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&candidate_policy).expect("serialize candidate policy"),
    )
    .expect("write candidate policy");
    fs::write(
        workflows.join("required.yml"),
        "jobs:\n  gate:\n    steps:\n      - name: Reduced shell\n        run: echo first\n",
    )
    .expect("write candidate workflow");

    let protected_baseline = load_workflow_inline_shell_baseline_from_merge_base(root)
        .expect("load immutable protected-base workflow baseline");
    let observed = collect_observed_workflow_inline_shell(root, &candidate_policy)
        .expect("scan candidate workflow");
    let ceiling_findings = validate_workflow_inline_shell_baseline_ceiling(
        &candidate_policy["workflow_inline_shell_baseline"],
        &protected_baseline,
    );
    let synchronization_findings = evaluate_workflow_inline_shell_keyed(
        &observed,
        &candidate_policy["workflow_inline_shell_baseline"],
    );
    assert!(
        ceiling_findings.is_empty() && synchronization_findings.is_empty(),
        "synchronized line-count reduction must be admitted: ceiling={ceiling_findings:#?}, \
         synchronization={synchronization_findings:#?}"
    );
}

#[test]
fn exception_added_by_prior_protected_change_is_accepted() {
    let temp = TestDir::new("automation-policy-prior-protected-change");
    let root = temp.path();
    let path = policy_path(root);
    fs::create_dir_all(path.parent().expect("policy parent")).expect("create policy directory");
    run_git(root, ["init"]);
    run_git(
        root,
        ["config", "user.email", "automation-policy@example.test"],
    );
    run_git(root, ["config", "user.name", "automation policy test"]);

    let protected_policy = json!({
        "exceptions": [
            {"path": "scripts/accepted.sh"},
            {"path": "scripts/prior-protected-change.sh"}
        ],
        "non_rust_exception_baseline": {"codes": {
            "rust_first_automation_unbaselined_non_rust_exception": [
                "scripts/accepted.sh",
                "scripts/prior-protected-change.sh"
            ]
        }}
    });
    fs::write(
        &path,
        serde_json::to_vec(&protected_policy).expect("serialize protected policy"),
    )
    .expect("write protected policy");
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "-m", "prior protected policy"]);
    let protected_commit = run_git(root, ["rev-parse", "HEAD"]);
    run_git(
        root,
        [
            "update-ref",
            "refs/remotes/origin/dev",
            protected_commit.trim(),
        ],
    );

    let baseline = load_non_rust_exception_baseline_from_merge_base(root)
        .expect("load prior protected baseline");
    let findings = evaluate_non_rust_exception_baseline_keyed(&protected_policy, &baseline);
    assert!(
        findings.is_empty(),
        "an exception admitted by the prior protected tree must be accepted: {findings:#?}"
    );
}

/// RED FIXTURE (mandatory, proves non-inert): a NEW exceptions[] path beyond the frozen baseline
/// must make the dimension RED with the new path surfaced under the unbaselined code, while a
/// baselined path is not flagged.
#[test]
fn new_non_rust_exception_beyond_baseline_is_born_blocking_red() {
    let baseline = json!({
        "codes": {
            "rust_first_automation_unbaselined_non_rust_exception": [
                "scripts/tests/cloud_control_plane_operation_contract_check.py"
            ]
        }
    });
    let policy = json!({
        "exceptions": [
            { "path": "scripts/tests/cloud_control_plane_operation_contract_check.py" },
            { "path": "scripts/tests/cell_002_promotion_automation_check.py" }
        ]
    });
    let findings = evaluate_non_rust_exception_baseline_keyed(&policy, &baseline);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_unbaselined_non_rust_exception"
                && finding.key == "scripts/tests/cell_002_promotion_automation_check.py"
        }),
        "a new non-Rust bridge beyond baseline must be born-blocking with its path surfaced; got \
         {findings:#?}"
    );
    assert!(
        !findings.iter().any(|finding| {
            finding.key == "scripts/tests/cloud_control_plane_operation_contract_check.py"
        }),
        "an accepted baselined exception must not be flagged"
    );
}

/// A baselined exception no longer declared must surface the stale code (forces the baseline to
/// shrink in the same PR when a bridge is retired).
#[test]
fn retired_baselined_non_rust_exception_is_stale_red() {
    let baseline = json!({
        "codes": {
            "rust_first_automation_unbaselined_non_rust_exception": [
                "scripts/tests/cloud_control_plane_operation_contract_check.py"
            ]
        }
    });
    let policy = json!({ "exceptions": [] });
    let findings = evaluate_non_rust_exception_baseline_keyed(&policy, &baseline);
    assert!(findings.iter().any(|finding| {
        finding.code == "rust_first_automation_non_rust_exception_baseline_stale"
            && finding.key == "scripts/tests/cloud_control_plane_operation_contract_check.py"
    }));
}

// ───────────────────────── forbidden workflow `uses:` dimension ─────────────────────────────

#[test]
fn workflow_forbidden_uses_dimension_covers_dot_github_workflows() {
    let root = repo_root();
    let policy = load_policy(&root);
    let block = &policy["scan"]["workflow_forbidden_uses"];
    assert_eq!(
        block["enabled"].as_bool(),
        Some(true),
        "workflow_forbidden_uses dimension must be enabled in policy DATA"
    );
    let roots: Vec<&str> = block["roots"]
        .as_array()
        .expect("workflow_forbidden_uses.roots")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        roots.contains(&".github/workflows"),
        ".github/workflows must be a declared forbidden-uses scan root; got {roots:?}"
    );
    let forbidden: Vec<&str> = block["forbidden_uses_substrings"]
        .as_array()
        .expect("workflow_forbidden_uses.forbidden_uses_substrings")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        forbidden.contains(&"setup-buck2"),
        "setup-buck2 marketplace action residue must be a forbidden uses substring; got {forbidden:?}"
    );
}

#[test]
fn live_workflow_uses_do_not_reintroduce_setup_buck2_action() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = collect_observed_forbidden_workflow_uses(&root, &policy)
        .expect("read-only workflow uses scan should not need temp files or cleanup");
    let findings = evaluate_forbidden_workflow_uses(&observed);
    assert!(
        findings.is_empty(),
        "workflow uses scan found forbidden Buck2 setup action(s): {findings:#?}. \
         Keep infra/ci/install-buck2.sh as the repo-owned path that downloads the official \
         facebook/buck2 release asset with pinned SHA-256."
    );
}

#[test]
fn docs_graph_drift_consumes_the_installer_reported_buck2_path() {
    let root = repo_root();
    let workflow_path = root.join(".github/workflows/docs-graph-drift.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", workflow_path.display()));
    let workflow_doc: YamlValue = serde_yaml::from_str(&workflow)
        .unwrap_or_else(|error| panic!("parse {}: {error}", workflow_path.display()));
    let step = named_workflow_step(
        &workflow_doc,
        "docs-graph-drift",
        "Materialize de-committed inputs, build + test the generator",
    );
    let run = step
        .get("run")
        .and_then(YamlValue::as_str)
        .expect("docs graph drift materializer must be a run step");
    let installer = run
        .find("infra/ci/install-buck2.sh")
        .expect("docs graph drift must invoke the repo-owned Buck2 installer");
    let path_binding = run
        .find(r#"PATH="$(tail -n1 "${GITHUB_PATH}"):${PATH}"; export PATH"#)
        .expect("the same step must consume the installer's final GITHUB_PATH entry");

    assert!(
        installer < path_binding,
        "the workflow must invoke the installer before consuming its reported path"
    );
    assert!(
        !run.lines().any(|line| {
            let line = line.trim_start();
            (line.starts_with("PATH=") || line.starts_with("export PATH="))
                && line.contains("/sha256-")
        }),
        "same-step Buck2 PATH binding must not duplicate a digest-qualified installer path"
    );
}

#[test]
fn fixture_proves_setup_buck2_action_fails_closed() {
    let observed = json!({"uses": [{
        "key": ".github/workflows/ci.yml::build::step-0::some/setup-buck2@v1",
        "uses": "some/setup-buck2@v1"
    }]});
    let findings = evaluate_forbidden_workflow_uses(&observed);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_forbidden_workflow_action"
                && finding.key == ".github/workflows/ci.yml::build::step-0::some/setup-buck2@v1"
        }),
        "forbidden setup-buck2 action must fail closed; got {findings:#?}"
    );
}

// ───────────────────── interpreter command authority dimension (G006) ─────────────────────

#[test]
fn interpreter_command_authority_dimension_is_enabled_for_rust_automation_sources() {
    let root = repo_root();
    let policy = load_policy(&root);
    let block = &policy["scan"]["interpreter_command_authority"];
    assert_eq!(
        block["enabled"].as_bool(),
        Some(true),
        "interpreter_command_authority dimension must be enabled in policy DATA"
    );
    let roots: Vec<&str> = block["roots"]
        .as_array()
        .expect("interpreter_command_authority.roots")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        roots.contains(&"cloud/cloud-ci") && roots.contains(&"libs") && roots.contains(&"tools"),
        "Rust automation/gate source roots must be declared for interpreter-command authority scan; got {roots:?}"
    );
    let excluded: Vec<&str> = block["exclude_prefixes"]
        .as_array()
        .expect("interpreter_command_authority.exclude_prefixes")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        excluded.contains(&"cloud/cloud-os/"),
        "cloud/cloud-os is fenced to the later kernel/OS story for task-4; got {excluded:?}"
    );
}

#[test]
fn live_rust_automation_sources_do_not_spawn_retired_interpreters() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = collect_observed_interpreter_command_authority(&root, &policy)
        .expect("read-only interpreter-command scan should not need temp files or cleanup");
    let findings = evaluate_interpreter_command_authority(&observed);
    assert!(
        findings.is_empty(),
        "Rust automation/gate source contains retired interpreter command authority: {findings:#?}"
    );
}

#[test]
fn fixture_proves_python_node_and_mjs_command_authority_fail_closed() {
    let observed = json!({"commands": [
        {"key": "tools/example/src/main.rs:10::python3", "command": "python3"},
        {"key": "libs/example/src/lib.rs:20::node", "command": "node"},
        {"key": "cloud/cloud-ci/example/src/lib.rs:30::bin/check.mjs", "command": "bin/check.mjs"}
    ]});
    let findings = evaluate_interpreter_command_authority(&observed);
    for key in [
        "tools/example/src/main.rs:10::python3",
        "libs/example/src/lib.rs:20::node",
        "cloud/cloud-ci/example/src/lib.rs:30::bin/check.mjs",
    ] {
        assert!(
            findings.iter().any(|finding| {
                finding.code == "rust_first_automation_interpreter_command_authority"
                    && finding.key == key
            }),
            "retired interpreter command authority key {key} must fail closed; got {findings:#?}"
        );
    }
}

// ───────────────────────── CLI package authority dimension ─────────────────────────

#[test]
fn cli_package_authority_dimension_is_enabled_for_infrastructure_roots() {
    let root = repo_root();
    let policy = load_policy(&root);
    let block = &policy["scan"]["cli_package_authority"];
    assert_eq!(
        block["enabled"].as_bool(),
        Some(true),
        "cli_package_authority dimension must be enabled in policy DATA"
    );
    let roots: Vec<&str> = block["roots"]
        .as_array()
        .expect("cli_package_authority.roots")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        roots.contains(&"cloud") && roots.contains(&"infra") && roots.contains(&"tools"),
        "infrastructure/cloud/tooling roots must be scanned for CLI-first package births; got {roots:?}"
    );
}

#[test]
fn live_infrastructure_roots_do_not_add_cli_first_packages() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = collect_observed_cli_package_authority(&root, &policy)
        .expect("read-only CLI package scan should not need temp files or cleanup");
    let findings = evaluate_cli_package_authority(&observed);
    assert!(
        findings.is_empty(),
        "infrastructure/cloud/tooling roots contain CLI-first package authority: {findings:#?}"
    );
}

#[test]
fn fixture_proves_infrastructure_cli_package_fails_closed() {
    let observed = json!({"packages": [{
        "key": "infra/example/Cargo.toml::infra-fix-cli",
        "path": "infra/example/Cargo.toml",
        "package_name": "infra-fix-cli"
    }]});
    let findings = evaluate_cli_package_authority(&observed);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_cli_package_authority"
                && finding.key == "infra/example/Cargo.toml::infra-fix-cli"
        }),
        "new infrastructure CLI package must fail closed; got {findings:#?}"
    );
}

// ───────────────────────── Buck2 installer transaction regressions ─────────────────────────

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "oya-buck2-installer-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    let mut permissions = fs::metadata(path)
        .unwrap_or_else(|error| panic!("metadata {}: {error}", path.display()))
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .unwrap_or_else(|error| panic!("chmod {}: {error}", path.display()));
}

fn sha256(path: &Path) -> String {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("run sha256sum");
    assert!(output.status.success(), "sha256sum failed: {output:?}");
    String::from_utf8(output.stdout)
        .expect("sha256sum UTF-8")
        .split_whitespace()
        .next()
        .expect("sha256sum digest")
        .to_owned()
}

fn find_command(name: &str) -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").expect("PATH"))
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| panic!("required command not found: {name}"))
}

fn shim_path(bin: &Path) -> std::ffi::OsString {
    let mut paths = vec![bin.to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH"),
    ));
    std::env::join_paths(paths).expect("join PATH")
}

fn installer_command(
    root: &Path,
    bin: &Path,
    install_dir: &Path,
    asset: &str,
    digest: &str,
) -> Command {
    let mut command = Command::new(root.join("infra/ci/install-buck2.sh"));
    command
        .env("PATH", shim_path(bin))
        .env("BUCK2_INSTALL_DIR", install_dir)
        .env("BUCK2_RELEASE", "fixture")
        .env("BUCK2_ASSET", asset)
        .env("BUCK2_SHA256", digest)
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "15");
    command
}

fn installer_content_dir(install_dir: &Path, digest: &str) -> PathBuf {
    install_dir.join(format!("sha256-{}", digest.to_ascii_lowercase()))
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "installer failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn install_host_shims(bin: &Path, zstd_body: &str, curl_body: &str) {
    write_executable(
        &bin.join("uname"),
        "#!/usr/bin/env bash\ncase \"$1\" in -s) echo Linux ;; -m) echo x86_64 ;; *) exit 2 ;; esac\n",
    );
    write_executable(&bin.join("zstd"), zstd_body);
    write_executable(&bin.join("curl"), curl_body);
    if cfg!(target_os = "macos") {
        write_executable(
            &bin.join("flock"),
            "#!/usr/bin/env bash\nset -euo pipefail\ntimeout=\"\"; fd=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -x) shift ;; -w) timeout=\"$2\"; shift 2 ;; *) fd=\"$1\"; shift ;; esac; done\nexec /usr/bin/lockf -s -t \"$timeout\" \"$fd\"\n",
        );
    }
}

fn wait_for_path(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while !path.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(path.exists(), "timed out waiting for {}", path.display());
}

fn wait_for_numeric_pid(path: &Path, timeout: Duration) -> u32 {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(contents) = fs::read_to_string(path)
            && let Ok(pid) = contents.trim().parse::<u32>()
            && pid > 0
        {
            return pid;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for numeric PID in {}", path.display());
}

fn process_is_alive(pid: u32) -> bool {
    let pid = pid.to_string();
    Command::new("/bin/kill")
        .args(["-0", &pid])
        .output()
        .expect("probe process")
        .status
        .success()
}

fn process_exits_within(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !process_is_alive(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    !process_is_alive(pid)
}

struct HolderProcessGuard {
    installer: Option<Child>,
    worker_pid: Option<u32>,
    release_path: PathBuf,
}

impl HolderProcessGuard {
    fn new(installer: Child, release_path: PathBuf) -> Self {
        Self {
            installer: Some(installer),
            worker_pid: None,
            release_path,
        }
    }

    fn record_worker(&mut self, worker_pid: u32) {
        self.worker_pid = Some(worker_pid);
    }

    fn terminate_installer(&mut self) {
        let installer = self.installer.as_mut().expect("live installer holder");
        installer.kill().expect("kill installer shell");
        installer.wait().expect("wait for killed installer shell");
        self.installer = None;
    }

    fn release_worker(&mut self) {
        let worker_pid = self.worker_pid.expect("recorded holder worker");
        fs::write(&self.release_path, b"release").expect("release holder worker");
        assert!(
            process_exits_within(worker_pid, Duration::from_secs(5)),
            "holder worker {worker_pid} remained alive after release"
        );
        self.worker_pid = None;
    }
}

impl Drop for HolderProcessGuard {
    fn drop(&mut self) {
        let _ = fs::write(&self.release_path, b"release");
        if let Some(installer) = self.installer.as_mut() {
            let _ = installer.kill();
            let _ = installer.wait();
        }
        if let Some(worker_pid) = self.worker_pid {
            if !process_exits_within(worker_pid, Duration::from_secs(5)) {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", &worker_pid.to_string()])
                    .output();
                let _ = process_exits_within(worker_pid, Duration::from_secs(5));
            }
        }
    }
}

struct CapturedChildGuard {
    child: Option<Child>,
    process_group: u32,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

impl CapturedChildGuard {
    fn spawn(command: &mut Command, output_dir: &Path, label: &str) -> Self {
        let stdout_path = output_dir.join(format!("{label}.stdout"));
        let stderr_path = output_dir.join(format!("{label}.stderr"));
        let stdout = fs::File::create(&stdout_path)
            .unwrap_or_else(|error| panic!("create {label} stdout: {error}"));
        let stderr = fs::File::create(&stderr_path)
            .unwrap_or_else(|error| panic!("create {label} stderr: {error}"));
        let child = command
            .process_group(0)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .unwrap_or_else(|error| panic!("spawn {label}: {error}"));
        let process_group = child.id();
        Self {
            child: Some(child),
            process_group,
            stdout_path,
            stderr_path,
        }
    }

    fn read_output(&self, status: std::process::ExitStatus, label: &str) -> Output {
        let stdout = fs::read(&self.stdout_path)
            .unwrap_or_else(|error| panic!("read {label} stdout: {error}"));
        let stderr = fs::read(&self.stderr_path)
            .unwrap_or_else(|error| panic!("read {label} stderr: {error}"));
        Output {
            status,
            stdout,
            stderr,
        }
    }

    fn signal_process_group(&mut self) {
        let status = Command::new("/bin/kill")
            .args(["-KILL", &format!("-{}", self.process_group)])
            .output();
        if !matches!(status, Ok(output) if output.status.success())
            && let Some(child) = self.child.as_mut()
        {
            let _ = child.kill();
        }
    }

    fn poll_status(&mut self, label: &str) -> Option<std::process::ExitStatus> {
        self.child
            .as_mut()
            .expect("captured child")
            .try_wait()
            .unwrap_or_else(|error| panic!("poll {label}: {error}"))
    }

    fn wait_with_output(mut self, timeout: Duration, label: &str) -> Output {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.poll_status(label) {
                let output = self.read_output(status, label);
                self.child = None;
                return output;
            }
            if Instant::now() >= deadline {
                self.signal_process_group();
                let cleanup_deadline = Instant::now() + Duration::from_secs(5);
                while Instant::now() < cleanup_deadline {
                    if let Some(status) = self.poll_status(label) {
                        let output = self.read_output(status, label);
                        self.child = None;
                        panic!(
                            "{label} exceeded {timeout:?}:\nstdout={}\nstderr={}",
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                panic!(
                    "{label} exceeded {timeout:?} and its process group {} did not exit",
                    self.process_group
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for CapturedChildGuard {
    fn drop(&mut self) {
        if self.child.is_some() {
            self.signal_process_group();
            let cleanup_deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < cleanup_deadline {
                if self
                    .child
                    .as_mut()
                    .and_then(|child| child.try_wait().ok())
                    .flatten()
                    .is_some()
                {
                    self.child = None;
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn spawn_retry_server(
    payload: Vec<u8>,
    transient_failures: usize,
    expected_requests: usize,
) -> (u16, thread::JoinHandle<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local retry fixture");
    listener
        .set_nonblocking(true)
        .expect("make retry fixture nonblocking");
    let port = listener.local_addr().expect("fixture address").port();
    let handle = thread::spawn(move || {
        let mut requests = 0;
        let deadline = Instant::now() + Duration::from_secs(20);
        while requests < expected_requests && Instant::now() < deadline {
            let (mut stream, _) = match listener.accept() {
                Ok(connection) => connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(error) => panic!("accept fixture request: {error}"),
            };
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            requests += 1;
            if requests <= transient_failures {
                stream
                    .write_all(b"HTTP/1.1 504 Gateway Timeout\r\nRetry-After: 1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .expect("write 504 fixture response");
            } else {
                let headers = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    payload.len()
                );
                stream
                    .write_all(headers.as_bytes())
                    .expect("write 200 headers");
                stream.write_all(&payload).expect("write payload");
            }
        }
        requests
    });
    (port, handle)
}

#[test]
fn buck2_installer_retries_cache_hits_and_digest_mismatches_fail_closed() {
    let root = repo_root();
    let fixture = TestDir::new("retry");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload = b"#!/usr/bin/env bash\necho fixture buck2\n".to_vec();
    let payload_path = fixture.path().join("payload");
    fs::write(&payload_path, &payload).expect("write payload");
    let digest = sha256(&payload_path);
    let (retry_port, retry_server) = spawn_retry_server(payload.clone(), 6, 7);
    let args_log = fixture.path().join("curl-args");
    install_host_shims(
        &bin,
        "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n",
        "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' \"$@\" > \"$CURL_ARGS_LOG\"\nargs=()\nfor arg in \"$@\"; do case \"$arg\" in https://github.com/facebook/buck2/releases/download/*) args+=(\"http://127.0.0.1:$BUCK2_TEST_PORT/fixture\") ;; *) args+=(\"$arg\") ;; esac; done\nexec \"$REAL_CURL\" \"${args[@]}\" --retry-delay 1\n",
    );
    let install_dir = fixture.path().join("install");
    let output = installer_command(&root, &bin, &install_dir, "fixture.zst", &digest)
        .env("BUCK2_TEST_PORT", retry_port.to_string())
        .env("CURL_ARGS_LOG", &args_log)
        .env("REAL_CURL", find_command("curl"))
        .output()
        .expect("run retry fixture");
    assert_success(output);
    assert_eq!(
        fs::read_to_string(&args_log)
            .expect("read curl args")
            .contains("--retry-delay"),
        false
    );
    let args = fs::read_to_string(&args_log).expect("read curl args");
    for required in [
        "--retry\n8\n",
        "--retry-all-errors\n",
        "--retry-max-time\n180\n",
        "--connect-timeout\n20\n",
        "--max-time\n60\n",
    ] {
        assert!(
            args.contains(required),
            "missing required curl arguments {required:?}: {args}"
        );
    }
    assert_eq!(
        retry_server.join().expect("retry server"),
        7,
        "six 504s then one successful fixture response"
    );

    let mismatch_digest = "0".repeat(64);
    let mismatch_install_dir = fixture.path().join("mismatch");
    let (mismatch_port, mismatch_server) = spawn_retry_server(payload, 0, 1);
    let mismatch = installer_command(
        &root,
        &bin,
        &mismatch_install_dir,
        "fixture.zst",
        &mismatch_digest,
    )
    .env("BUCK2_TEST_PORT", mismatch_port.to_string())
    .env("CURL_ARGS_LOG", &args_log)
    .env("REAL_CURL", find_command("curl"))
    .output()
    .expect("run digest mismatch fixture");
    assert_eq!(
        mismatch_server.join().expect("digest mismatch server"),
        1,
        "digest mismatch must be evaluated after receiving the fixture payload"
    );
    assert!(
        !mismatch.status.success(),
        "digest mismatch must fail closed"
    );
    let mismatch_content_dir = installer_content_dir(&mismatch_install_dir, &mismatch_digest);
    assert!(!mismatch_content_dir.join("fixture.zst").exists());
    assert!(
        fs::read_dir(&mismatch_content_dir)
            .expect("mismatch dir")
            .all(|entry| !entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .contains(".part."))
    );
    write_executable(&bin.join("curl"), "#!/usr/bin/env bash\nexit 99\n");
    let cache = installer_command(&root, &bin, &install_dir, "fixture.zst", &digest)
        .output()
        .expect("run cache-hit fixture");
    assert_success(cache);
    assert!(
        installer_content_dir(&install_dir, &digest)
            .join(".buck2-install.lock")
            .is_file()
    );
}

#[test]
fn buck2_installer_rejects_invalid_lock_timeouts_before_downloader_side_effects() {
    let root = repo_root();
    let fixture = TestDir::new("invalid-lock-timeout");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    install_host_shims(
        &bin,
        "#!/usr/bin/env bash\nexit 99\n",
        "#!/usr/bin/env bash\necho called > \"$CURL_MARKER\"\nexit 99\n",
    );
    let install_dir = fixture.path().join("install");
    fs::create_dir_all(&install_dir).expect("create install dir");
    fs::write(install_dir.join("buck2"), b"prior-binary").expect("seed prior binary");
    for invalid in ["0", "", "-1", "not-a-number"] {
        let marker = fixture.path().join(format!("curl-{invalid:?}"));
        let output = installer_command(&root, &bin, &install_dir, "fixture.zst", &"0".repeat(64))
            .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", invalid)
            .env("CURL_MARKER", &marker)
            .output()
            .expect("run invalid timeout fixture");
        assert!(
            !output.status.success(),
            "invalid timeout {invalid:?} must fail"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("must be a positive integer"));
        assert!(!marker.exists(), "invalid timeout {invalid:?} invoked curl");
        assert_eq!(
            fs::read(install_dir.join("buck2")).expect("prior binary"),
            b"prior-binary"
        );
    }
}

#[test]
fn buck2_installer_rejects_malformed_digests_before_creating_content_paths() {
    let root = repo_root();
    let fixture = TestDir::new("invalid-digest");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let install_dir = fixture.path().join("install");
    let marker = fixture.path().join("curl-called");
    install_host_shims(
        &bin,
        "#!/usr/bin/env bash\nexit 99\n",
        "#!/usr/bin/env bash\ntouch \"$CURL_MARKER\"\nexit 99\n",
    );
    for invalid in ["", "../escape", &"a".repeat(63), &"g".repeat(64)] {
        let output = installer_command(&root, &bin, &install_dir, "fixture.zst", invalid)
            .env("CURL_MARKER", &marker)
            .output()
            .expect("run invalid digest fixture");
        assert!(!output.status.success(), "invalid digest must fail");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("must be exactly 64 hexadecimal characters")
        );
        assert!(!marker.exists(), "invalid digest invoked curl");
        assert!(
            !install_dir.exists(),
            "invalid digest created install content"
        );
    }
}

#[test]
fn buck2_installer_rejects_malformed_assets_before_creating_content_paths() {
    let root = repo_root();
    let fixture = TestDir::new("invalid-asset");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let install_dir = fixture.path().join("install");
    let marker = fixture.path().join("curl-called");
    install_host_shims(
        &bin,
        "#!/usr/bin/env bash\nexit 99\n",
        "#!/usr/bin/env bash\ntouch \"$CURL_MARKER\"\nexit 99\n",
    );
    let digest = "0".repeat(64);
    for invalid in [
        "",
        ".",
        "..",
        "../escape.zst",
        "asset*.zst",
        "asset name.zst",
    ] {
        let output = installer_command(&root, &bin, &install_dir, invalid, &digest)
            .env("CURL_MARKER", &marker)
            .output()
            .expect("run invalid asset fixture");
        assert!(!output.status.success(), "invalid asset must fail");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("must be a safe release-asset filename")
        );
        assert!(!marker.exists(), "invalid asset invoked curl");
        assert!(
            !install_dir.exists(),
            "invalid asset created install content"
        );
    }
}

#[test]
fn buck2_installer_serializes_same_digest_and_preserves_prior_binary_on_zstd_failure() {
    let root = repo_root();
    let fixture = TestDir::new("same-digest");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload = fixture.path().join("payload");
    fs::write(&payload, b"#!/usr/bin/env bash\necho binary-a\n").expect("write payload");
    let digest = sha256(&payload);
    let curl_body = "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) shift ;; esac; done\nmkdir \"$CRITICAL_DIR/curl\" || exit 75\nsleep 1\ncp \"$PAYLOAD\" \"$out\"\nrmdir \"$CRITICAL_DIR/curl\"\n";
    let zstd_body = "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\nmkdir \"$CRITICAL_DIR/zstd\" || exit 76\nsleep 1\ncp \"$input\" \"$output\"\nrmdir \"$CRITICAL_DIR/zstd\"\n";
    install_host_shims(&bin, zstd_body, curl_body);
    let install_dir = fixture.path().join("install");
    let critical = fixture.path().join("critical");
    fs::create_dir_all(&critical).expect("create critical fixture dir");
    let mut first = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    first
        .env("PAYLOAD", &payload)
        .env("CRITICAL_DIR", &critical);
    let mut first = first.spawn().expect("spawn first installer");
    thread::sleep(Duration::from_millis(100));
    let mut second = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    second
        .env("PAYLOAD", &payload)
        .env("CRITICAL_DIR", &critical);
    let second = second.spawn().expect("spawn second installer");
    assert!(
        first.wait().expect("wait first").success(),
        "first installer must succeed"
    );
    assert!(
        second
            .wait_with_output()
            .expect("wait second")
            .status
            .success(),
        "second installer must succeed"
    );
    let content_dir = installer_content_dir(&install_dir, &digest);
    assert_eq!(
        fs::read(content_dir.join("buck2")).expect("promoted binary"),
        fs::read(&payload).expect("read payload")
    );
    assert!(content_dir.join(".buck2-install.lock").is_file());
    assert!(
        fs::read_dir(&content_dir)
            .expect("install dir")
            .all(|entry| !entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .contains(".part."))
    );

    write_executable(&bin.join("zstd"), "#!/usr/bin/env bash\nexit 70\n");
    fs::write(content_dir.join("buck2"), b"previous-binary").expect("seed prior binary");
    let failed = installer_command(&root, &bin, &install_dir, "asset.zst", &digest)
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("PAYLOAD", &payload)
        .env("CRITICAL_DIR", &critical)
        .output()
        .expect("run zstd failure fixture");
    assert!(
        !failed.status.success(),
        "zstd failure must fail the installer"
    );
    assert_eq!(
        fs::read(content_dir.join("buck2")).expect("prior binary"),
        b"previous-binary"
    );
    assert!(
        fs::read_dir(&content_dir)
            .expect("install dir")
            .all(|entry| !entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .contains("buck2.part."))
    );
    assert!(
        !content_dir.join(".buck2-install.lock.d").exists(),
        "failed no-flock installation must release its shared lock"
    );

    write_executable(
        &bin.join("zstd"),
        "#!/usr/bin/env bash\nset -euo pipefail\noutput=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; *) shift ;; esac; done\nprintf '#!/usr/bin/env bash\\nexit 41\\n' > \"$output\"\n",
    );
    let invalid_candidate = installer_command(&root, &bin, &install_dir, "asset.zst", &digest)
        .env("PAYLOAD", &payload)
        .env("CRITICAL_DIR", &critical)
        .output()
        .expect("run invalid candidate fixture");
    assert!(
        !invalid_candidate.status.success(),
        "candidate failing its version probe must fail the installer"
    );
    assert_eq!(
        fs::read(content_dir.join("buck2")).expect("prior binary"),
        b"previous-binary"
    );
    assert!(
        fs::read_dir(&content_dir)
            .expect("install dir")
            .all(|entry| !entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .contains("buck2.part."))
    );
}

#[test]
fn buck2_installer_serializes_same_digest_without_flock() {
    let root = repo_root();
    let fixture = TestDir::new("same-digest-no-flock");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload = fixture.path().join("payload");
    fs::write(&payload, b"#!/usr/bin/env bash\necho binary-a\n").expect("write payload");
    let digest = sha256(&payload);
    let curl_body = "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) shift ;; esac; done\nmkdir \"$CRITICAL_DIR/curl\" || exit 75\nsleep 1\ncp \"$PAYLOAD\" \"$out\"\nrmdir \"$CRITICAL_DIR/curl\"\n";
    let zstd_body = "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\nmkdir \"$CRITICAL_DIR/zstd\" || exit 76\nsleep 1\ncp \"$input\" \"$output\"\nrmdir \"$CRITICAL_DIR/zstd\"\n";
    install_host_shims(&bin, zstd_body, curl_body);
    let install_dir = fixture.path().join("install");
    let critical = fixture.path().join("critical");
    fs::create_dir_all(&critical).expect("create critical fixture dir");
    let mut first = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    first
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("PAYLOAD", &payload)
        .env("CRITICAL_DIR", &critical);
    let mut second = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    second
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("PAYLOAD", &payload)
        .env("CRITICAL_DIR", &critical);
    let first = first.spawn().expect("spawn first installer");
    let second = second.spawn().expect("spawn second installer");
    assert_success(first.wait_with_output().expect("wait first"));
    assert_success(second.wait_with_output().expect("wait second"));
    assert_eq!(
        fs::read(installer_content_dir(&install_dir, &digest).join("buck2"))
            .expect("promoted binary"),
        fs::read(&payload).expect("read payload")
    );
}

#[test]
fn buck2_installer_allows_different_digests_to_install_concurrently() {
    let root = repo_root();
    let fixture = TestDir::new("different-digests");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload_a = fixture.path().join("payload-a");
    let payload_b = fixture.path().join("payload-b");
    fs::write(&payload_a, b"#!/usr/bin/env bash\necho binary-a\n").expect("write payload a");
    fs::write(&payload_b, b"#!/usr/bin/env bash\necho binary-b\n").expect("write payload b");
    let digest_a = sha256(&payload_a);
    let digest_b = sha256(&payload_b);
    let curl_body = "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"; url=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) url=\"$1\"; shift ;; esac; done\ncase \"$url\" in *asset-a*) id=a; payload=\"$PAYLOAD_A\" ;; *asset-b*) id=b; payload=\"$PAYLOAD_B\" ;; *) exit 78 ;; esac\ntouch \"$BARRIER/$id\"\nattempt=0\nuntil [ -f \"$BARRIER/a\" ] && [ -f \"$BARRIER/b\" ]; do attempt=$((attempt + 1)); [ \"$attempt\" -lt 500 ] || exit 77; sleep 0.01; done\ncp \"$payload\" \"$out\"\n";
    let zstd_body = "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n";
    install_host_shims(&bin, zstd_body, curl_body);
    let install_dir = fixture.path().join("install");
    let barrier = fixture.path().join("barrier");
    fs::create_dir_all(&barrier).expect("create barrier");

    let mut first = installer_command(&root, &bin, &install_dir, "asset-a.zst", &digest_a);
    first
        .env("PAYLOAD_A", &payload_a)
        .env("PAYLOAD_B", &payload_b)
        .env("BARRIER", &barrier);
    let mut second = installer_command(&root, &bin, &install_dir, "asset-b.zst", &digest_b);
    second
        .env("PAYLOAD_A", &payload_a)
        .env("PAYLOAD_B", &payload_b)
        .env("BARRIER", &barrier);
    let first = first.spawn().expect("spawn first digest installer");
    let second = second.spawn().expect("spawn second digest installer");
    assert_success(
        first
            .wait_with_output()
            .expect("wait first digest installer"),
    );
    assert_success(
        second
            .wait_with_output()
            .expect("wait second digest installer"),
    );
    assert_eq!(
        fs::read(installer_content_dir(&install_dir, &digest_a).join("buck2"))
            .expect("digest A binary"),
        fs::read(&payload_a).expect("payload A")
    );
    assert_eq!(
        fs::read(installer_content_dir(&install_dir, &digest_b).join("buck2"))
            .expect("digest B binary"),
        fs::read(&payload_b).expect("payload B")
    );
}

#[test]
fn buck2_installer_times_out_without_writes_then_recovers_after_holder_is_killed() {
    let root = repo_root();
    let fixture = TestDir::new("crash-recovery");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload = fixture.path().join("payload");
    fs::write(&payload, b"#!/usr/bin/env bash\necho recovered\n").expect("write payload");
    let digest = sha256(&payload);
    let marker_dir = fixture.path().join("markers");
    fs::create_dir_all(&marker_dir).expect("create marker directory");
    let curl_body = "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) shift ;; esac; done\ntouch \"$MARKER_DIR/curl-$INSTANCE\"\nif [ \"$INSTANCE\" = holder ]; then echo \"$$\" > \"$MARKER_DIR/holder-child-pid\"; while [ ! -f \"$MARKER_DIR/release-holder\" ]; do sleep 0.01; done; exit 70; fi\ncp \"$PAYLOAD\" \"$out\"\n";
    let zstd_body = "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n";
    install_host_shims(&bin, zstd_body, curl_body);
    let install_dir = fixture.path().join("install");
    let content_dir = installer_content_dir(&install_dir, &digest);
    fs::create_dir_all(&content_dir).expect("create content directory");
    fs::write(content_dir.join("buck2"), b"prior-binary").expect("seed prior binary");

    let mut holder = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    holder
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("INSTANCE", "holder")
        .env("MARKER_DIR", &marker_dir)
        .env("PAYLOAD", &payload);
    let holder = holder.spawn().expect("spawn lock holder");
    let mut holder = HolderProcessGuard::new(holder, marker_dir.join("release-holder"));
    wait_for_path(&marker_dir.join("curl-holder"), Duration::from_secs(15));
    let holder_child_pid = wait_for_numeric_pid(
        &marker_dir.join("holder-child-pid"),
        Duration::from_secs(15),
    );
    holder.record_worker(holder_child_pid);

    let mut contender_command = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    contender_command
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "1")
        .env("INSTANCE", "contender")
        .env("MARKER_DIR", &marker_dir)
        .env("PAYLOAD", &payload);
    let contender =
        CapturedChildGuard::spawn(&mut contender_command, fixture.path(), "timed-contender")
            .wait_with_output(Duration::from_secs(5), "timed contender");
    assert!(
        !contender.status.success(),
        "live-owner contender must time out"
    );
    assert!(
        String::from_utf8_lossy(&contender.stderr)
            .contains("Timed out waiting for Buck2 installer lock")
    );
    assert!(!marker_dir.join("curl-contender").exists());
    assert_eq!(
        fs::read(content_dir.join("buck2")).expect("prior binary"),
        b"prior-binary"
    );

    holder.terminate_installer();
    let mut successor_command = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    successor_command
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "2")
        .env("INSTANCE", "successor")
        .env("MARKER_DIR", &marker_dir)
        .env("PAYLOAD", &payload);
    let successor = CapturedChildGuard::spawn(
        &mut successor_command,
        fixture.path(),
        "crash-recovery-successor",
    );
    wait_for_path(&marker_dir.join("curl-successor"), Duration::from_secs(5));
    holder.release_worker();
    let successor = successor.wait_with_output(Duration::from_secs(15), "crash-recovery successor");
    assert_success(successor);
    assert!(marker_dir.join("curl-successor").exists());
    assert_eq!(
        fs::read(content_dir.join("buck2")).expect("recovered binary"),
        fs::read(&payload).expect("payload")
    );
    assert!(
        fs::read_dir(&content_dir)
            .expect("content directory")
            .all(|entry| !entry
                .expect("content entry")
                .file_name()
                .to_string_lossy()
                .contains(".part."))
    );
}

#[test]
fn buck2_installer_recovers_an_ownerless_no_flock_lock_directory() {
    let root = repo_root();
    let fixture = TestDir::new("ownerless-no-flock-lock");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload = fixture.path().join("payload");
    fs::write(&payload, b"#!/usr/bin/env bash\necho recovered\n").expect("write payload");
    let digest = sha256(&payload);
    install_host_shims(
        &bin,
        "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n",
        "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) shift ;; esac; done\ncp \"$PAYLOAD\" \"$out\"\n",
    );
    // Cross the deadline between loop observations: two separate reads let the old
    // implementation skip ownerless reaping and then time out in the same iteration.
    write_executable(
        &bin.join("date"),
        "#!/usr/bin/env bash\nset -euo pipefail\n[ \"$#\" -eq 1 ] && [ \"$1\" = \"+%s\" ]\ncalls=0\nif [ -f \"$CLOCK_STATE\" ]; then calls=\"$(cat \"$CLOCK_STATE\")\"; fi\ncalls=$((calls + 1))\nprintf '%s\\n' \"$calls\" > \"$CLOCK_STATE\"\nif [ \"$calls\" -le 2 ]; then printf '100\\n'; else printf '101\\n'; fi\n",
    );
    write_executable(&bin.join("sleep"), "#!/usr/bin/env bash\nexit 0\n");
    let install_dir = fixture.path().join("install");
    let content_dir = installer_content_dir(&install_dir, &digest);
    let clock_state = fixture.path().join("clock-state");
    fs::create_dir_all(content_dir.join(".buck2-install.lock.d"))
        .expect("construct ownerless lock directory");

    let output = installer_command(&root, &bin, &install_dir, "asset.zst", &digest)
        .env("BUCK2_INSTALL_FORCE_NO_FLOCK", "1")
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "1")
        .env("CLOCK_STATE", &clock_state)
        .env("PAYLOAD", &payload)
        .output()
        .expect("run ownerless-lock recovery fixture");
    assert_success(output);
    assert_eq!(
        fs::read(content_dir.join("buck2")).expect("recovered binary"),
        fs::read(&payload).expect("payload")
    );
    assert!(
        !content_dir.join(".buck2-install.lock.d").exists(),
        "recovered no-flock installation must release the ownerless lock"
    );
}

#[test]
fn buck2_installer_writes_a_native_windows_path_for_github_path() {
    let root = repo_root();
    let fixture = TestDir::new("windows-github-path");
    let bin = fixture.path().join("bin");
    fs::create_dir_all(&bin).expect("create shim bin");
    let payload = fixture.path().join("buck2.exe.fixture");
    fs::write(&payload, b"#!/usr/bin/env bash\necho fixture buck2.exe\n")
        .expect("write fixture Buck2 executable");
    let digest = sha256(&payload);
    let github_path = fixture.path().join("github-path");
    let runner_temp_posix = fixture.path().join("runner-temp");

    write_executable(
        &bin.join("uname"),
        "#!/usr/bin/env bash\ncase \"$1\" in -s) echo MINGW64_NT-10.0 ;; -m) echo x86_64 ;; *) exit 2 ;; esac\n",
    );
    write_executable(
        &bin.join("curl"),
        "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) shift ;; esac; done\ncp \"$PAYLOAD\" \"$out\"\n",
    );
    write_executable(
        &bin.join("zstd"),
        "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n",
    );
    write_executable(
        &bin.join("cygpath"),
        r#"#!/usr/bin/env bash
set -euo pipefail
mode="$1"; shift
[ "$1" = -- ] && shift
case "$mode" in
  -u)
    [ "$1" = "$RUNNER_TEMP" ] || exit 88
    printf '%s\n' "$RUNNER_TEMP_POSIX"
    ;;
  -w)
    case "$1" in
      "$RUNNER_TEMP_POSIX"/*)
        suffix="${1#"$RUNNER_TEMP_POSIX"/}"
        win_suffix="$(printf '%s' "$suffix" | sed 's|/|\\|g')"
        printf '%s\\%s\n' "$RUNNER_TEMP_WIN" "$win_suffix"
        ;;
      *) exit 88 ;;
    esac
    ;;
  *) exit 88 ;;
esac
"#,
    );

    let output = Command::new(root.join("infra/ci/install-buck2.sh"))
        .env("PATH", shim_path(&bin))
        .env("BUCK2_RELEASE", "fixture")
        .env("BUCK2_ASSET", "fixture.zst")
        .env("BUCK2_SHA256", &digest)
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "15")
        .env("PAYLOAD", &payload)
        .env("RUNNER_TEMP", r"D:\a\_temp")
        .env("RUNNER_TEMP_POSIX", &runner_temp_posix)
        .env("RUNNER_TEMP_WIN", r"D:\a\_temp")
        .env("GITHUB_PATH", &github_path)
        .output()
        .expect("run Windows installer fixture");
    assert_success(output);

    let content_dir = runner_temp_posix
        .join("oya-ci-buck2-fixture")
        .join(format!("sha256-{digest}"));
    assert_eq!(
        fs::read(content_dir.join("buck2.exe")).expect("installed Windows Buck2 executable"),
        fs::read(&payload).expect("fixture Buck2 executable"),
    );
    assert_eq!(
        fs::read_to_string(&github_path).expect("read GitHub PATH file"),
        format!("D:\\a\\_temp\\oya-ci-buck2-fixture\\sha256-{digest}\n"),
        "native Windows consumers must receive a Win32 path, never Git Bash's POSIX path",
    );
}
