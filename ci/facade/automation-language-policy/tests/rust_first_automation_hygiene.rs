#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
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

    let materialize = named_workflow_step(
        &workflow_doc,
        "producer-regen",
        "Materialize cloud-ci generated faces",
    );
    let env = materialize
        .get("env")
        .unwrap_or_else(|| panic!("producer materializer must declare env"));
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
            "producer materializer env binding drifted for {key}"
        );
    }
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

#[test]
fn live_postgres_lane_emits_redacted_bootstrap_provenance_artifact() {
    let root = repo_root();
    let workflow_path = root.join(".github/workflows/oya-ci-required.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", workflow_path.display()));

    let artifact_id = "\"artifact_id\": \"live-postgres-bootstrap-provenance\"";
    let start = workflow
        .find(artifact_id)
        .expect("live-postgres bootstrap provenance artifact must be emitted");
    let rest = &workflow[start..];
    let end = rest
        .find("\n          JSON")
        .expect("bootstrap provenance heredoc must terminate");
    let provenance_block = &rest[..end];

    assert!(
        provenance_block.contains("\"image\": \"postgres:16\"")
            && provenance_block.contains("0000_runtime_role.sql")
            && provenance_block.contains("0001_identity_scim_store.sql")
            && provenance_block.contains("\"source_revision\": \"${GITHUB_SHA}\""),
        "bootstrap provenance must include image, ordered migrations, and source revision: {provenance_block}"
    );
    assert!(
        !provenance_block.contains("postgres://")
            && !provenance_block.contains("postgres:postgres")
            && !provenance_block.contains("oya_app:app"),
        "bootstrap provenance must not emit DSNs or credentials: {provenance_block}"
    );
    assert!(
        workflow.contains("name: live-postgres-adapters-bootstrap-provenance")
            && workflow.contains("name: live-postgres-facades-bootstrap-provenance")
            && workflow.contains("retention-days: 30"),
        "bootstrap provenance must be uploaded as a durable retained operator artifact"
    );
}

/// The live workflow corpus inline-shell key set must equal the committed frozen baseline EXACTLY
/// (set equality, mirroring the embedded-asset hermeticity gate's shrink-only contract): a NEW
/// inline `run:` step beyond baseline is born-blocking, a RETIRED step must shrink the baseline in
/// the same PR. With the corpus == baseline, the dimension is GREEN today.
#[test]
fn live_workflow_inline_shell_matches_frozen_baseline_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    // The frozen keyed baseline is folded INTO the policy (`workflow_inline_shell_baseline`) so it
    // lives in an already-accounted gate face and adds zero new tracked accounting rows.
    let baseline = &policy["workflow_inline_shell_baseline"];

    let observed = collect_observed_workflow_inline_shell(&root, &policy)
        .expect("read-only workflow scan should not need temp files or cleanup");
    let steps = observed["steps"].as_array().expect("steps").len();
    assert!(
        steps > 0,
        "expected a non-empty live workflow inline-shell inventory (the blind spot is real)"
    );

    let findings = evaluate_workflow_inline_shell_keyed(&observed, &baseline);
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
    let codes_len = baseline["codes"]["rust_first_automation_unbaselined_workflow_inline_shell"]
        .as_array()
        .expect("baseline codes array")
        .len();
    let provenance_total = baseline["_provenance"]["keys_total"]
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
    let baseline = &policy["non_rust_exception_baseline"];

    let findings = evaluate_non_rust_exception_baseline_keyed(&policy, baseline);
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

    let codes_len = baseline["codes"]["rust_first_automation_unbaselined_non_rust_exception"]
        .as_array()
        .expect("baseline codes array")
        .len();
    let provenance_total = baseline["_provenance"]["keys_total"]
        .as_u64()
        .expect("baseline _provenance.keys_total") as usize;
    assert_eq!(
        codes_len, provenance_total,
        "baseline _provenance.keys_total ({provenance_total}) must equal the codes array length \
         ({codes_len})"
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
    worker_termination_signaled: bool,
}

impl HolderProcessGuard {
    fn new(installer: Child) -> Self {
        Self {
            installer: Some(installer),
            worker_pid: None,
            worker_termination_signaled: false,
        }
    }

    fn record_worker(&mut self, worker_pid: u32) {
        self.worker_pid = Some(worker_pid);
        self.worker_termination_signaled = false;
    }

    fn terminate_installer(&mut self) {
        let installer = self.installer.as_mut().expect("live installer holder");
        installer.kill().expect("kill installer shell");
        installer.wait().expect("wait for killed installer shell");
        self.installer = None;
    }

    fn signal_worker_termination(&mut self) {
        let worker_pid = self.worker_pid.expect("recorded holder worker");
        let status = Command::new("/bin/kill")
            .args(["-KILL", &worker_pid.to_string()])
            .status()
            .expect("terminate holder worker");
        assert!(
            status.success(),
            "holder worker must still be live until explicit cleanup"
        );
        self.worker_termination_signaled = true;
    }

    fn confirm_worker_exit(&mut self) {
        let worker_pid = self.worker_pid.expect("recorded holder worker");
        assert!(
            process_exits_within(worker_pid, Duration::from_secs(5)),
            "holder worker {worker_pid} remained alive after termination"
        );
        self.worker_pid = None;
        self.worker_termination_signaled = false;
    }
}

impl Drop for HolderProcessGuard {
    fn drop(&mut self) {
        if let Some(installer) = self.installer.as_mut() {
            let _ = installer.kill();
            let _ = installer.wait();
        }
        if let Some(worker_pid) = self.worker_pid {
            if !self.worker_termination_signaled {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", &worker_pid.to_string()])
                    .output();
            }
            let _ = process_exits_within(worker_pid, Duration::from_secs(5));
        }
    }
}

struct CapturedChildGuard {
    child: Option<Child>,
}

impl CapturedChildGuard {
    fn spawn(command: &mut Command, label: &str) -> Self {
        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|error| panic!("spawn {label}: {error}"));
        Self { child: Some(child) }
    }

    fn wait_with_output(mut self, timeout: Duration, label: &str) -> Output {
        let deadline = Instant::now() + timeout;
        loop {
            let status = self
                .child
                .as_mut()
                .expect("captured child")
                .try_wait()
                .unwrap_or_else(|error| panic!("poll {label}: {error}"));
            if let Some(status) = status {
                let mut child = self.child.take().expect("completed captured child");
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                child
                    .stdout
                    .take()
                    .expect("captured stdout")
                    .read_to_end(&mut stdout)
                    .unwrap_or_else(|error| panic!("read {label} stdout: {error}"));
                child
                    .stderr
                    .take()
                    .expect("captured stderr")
                    .read_to_end(&mut stderr)
                    .unwrap_or_else(|error| panic!("read {label} stderr: {error}"));
                return Output {
                    status,
                    stdout,
                    stderr,
                };
            }
            if Instant::now() >= deadline {
                let mut child = self.child.take().expect("timed captured child");
                let _ = child.kill();
                let output = child
                    .wait_with_output()
                    .unwrap_or_else(|error| panic!("collect timed-out {label}: {error}"));
                panic!(
                    "{label} exceeded {timeout:?}:\nstdout={}\nstderr={}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for CapturedChildGuard {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn spawn_retry_server(payload: Vec<u8>) -> (u16, thread::JoinHandle<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local retry fixture");
    listener
        .set_nonblocking(true)
        .expect("make retry fixture nonblocking");
    let port = listener.local_addr().expect("fixture address").port();
    let handle = thread::spawn(move || {
        let mut requests = 0;
        let deadline = Instant::now() + Duration::from_secs(20);
        while requests < 8 && Instant::now() < deadline {
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
            if requests <= 6 {
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
    let (port, server) = spawn_retry_server(payload);
    let args_log = fixture.path().join("curl-args");
    install_host_shims(
        &bin,
        "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n",
        "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' \"$@\" > \"$CURL_ARGS_LOG\"\nargs=()\nfor arg in \"$@\"; do case \"$arg\" in https://github.com/facebook/buck2/releases/download/*) args+=(\"http://127.0.0.1:$BUCK2_TEST_PORT/fixture\") ;; *) args+=(\"$arg\") ;; esac; done\nexec \"$REAL_CURL\" \"${args[@]}\" --retry-delay 1\n",
    );
    let install_dir = fixture.path().join("install");
    let output = installer_command(&root, &bin, &install_dir, "fixture.zst", &digest)
        .env("BUCK2_TEST_PORT", port.to_string())
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

    let mismatch_digest = "0".repeat(64);
    let mismatch_install_dir = fixture.path().join("mismatch");
    let mismatch = installer_command(
        &root,
        &bin,
        &mismatch_install_dir,
        "fixture.zst",
        &mismatch_digest,
    )
    .env("BUCK2_TEST_PORT", port.to_string())
    .env("CURL_ARGS_LOG", &args_log)
    .env("REAL_CURL", find_command("curl"))
    .output()
    .expect("run digest mismatch fixture");
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
    assert_eq!(
        server.join().expect("retry server"),
        8,
        "six 504s then two successful fixture responses"
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
    let curl_body = "#!/usr/bin/env bash\nset -euo pipefail\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) out=\"$2\"; shift 2 ;; *) shift ;; esac; done\ntouch \"$MARKER_DIR/curl-$INSTANCE\"\nif [ \"$INSTANCE\" = holder ]; then echo \"$$\" > \"$MARKER_DIR/holder-child-pid\"; exec sleep 30; fi\ncp \"$PAYLOAD\" \"$out\"\n";
    let zstd_body = "#!/usr/bin/env bash\nset -euo pipefail\ninput=\"\"; output=\"\"\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in -o) output=\"$2\"; shift 2 ;; -d|-f) shift ;; *) input=\"$1\"; shift ;; esac; done\ncp \"$input\" \"$output\"\n";
    install_host_shims(&bin, zstd_body, curl_body);
    let install_dir = fixture.path().join("install");
    let content_dir = installer_content_dir(&install_dir, &digest);
    fs::create_dir_all(&content_dir).expect("create content directory");
    fs::write(content_dir.join("buck2"), b"prior-binary").expect("seed prior binary");

    let mut holder = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    holder
        .env("INSTANCE", "holder")
        .env("MARKER_DIR", &marker_dir)
        .env("PAYLOAD", &payload);
    let holder = holder.spawn().expect("spawn lock holder");
    let mut holder = HolderProcessGuard::new(holder);
    wait_for_path(&marker_dir.join("curl-holder"), Duration::from_secs(15));
    let holder_child_pid = wait_for_numeric_pid(
        &marker_dir.join("holder-child-pid"),
        Duration::from_secs(15),
    );
    holder.record_worker(holder_child_pid);

    let mut contender_command = installer_command(&root, &bin, &install_dir, "asset.zst", &digest);
    contender_command
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "1")
        .env("INSTANCE", "contender")
        .env("MARKER_DIR", &marker_dir)
        .env("PAYLOAD", &payload);
    let contender = CapturedChildGuard::spawn(&mut contender_command, "timed contender")
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
        .env("BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS", "2")
        .env("INSTANCE", "successor")
        .env("MARKER_DIR", &marker_dir)
        .env("PAYLOAD", &payload);
    let successor = CapturedChildGuard::spawn(&mut successor_command, "crash-recovery successor");
    wait_for_path(&marker_dir.join("curl-successor"), Duration::from_secs(5));
    holder.signal_worker_termination();
    let successor = successor.wait_with_output(Duration::from_secs(15), "crash-recovery successor");
    holder.confirm_worker_exit();
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
