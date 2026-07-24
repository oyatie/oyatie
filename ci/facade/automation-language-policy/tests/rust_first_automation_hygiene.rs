#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use oya_cloud_ci_rust_first_automation_hygiene_app::{
    Finding, Verdict, collect_observed_cli_package_authority,
    collect_observed_forbidden_workflow_uses, collect_observed_interpreter_command_authority,
    collect_observed_non_rust_automation, collect_observed_workflow_inline_shell, evaluate,
    evaluate_cli_package_authority, evaluate_forbidden_workflow_uses,
    evaluate_interpreter_command_authority, evaluate_keyed,
    evaluate_non_rust_exception_baseline_keyed, evaluate_workflow_inline_shell_keyed,
};
use serde_json::{Value, json};

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
fn retirement_event_transport_is_bound_to_provider_facts_without_head_parent_inference() {
    let root = repo_root();
    let workflow_path = root.join(".github/workflows/oya-ci-required.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", workflow_path.display()));

    assert!(
        workflow.contains("workflow_dispatch:"),
        "manual reruns remain an explicitly declared workflow surface"
    );

    let materialize = workflow
        .split("- name: Materialize cloud-ci generated faces")
        .nth(1)
        .expect("producer materialization step");
    let protected_sha = materialize
        .find("EVENT_PROTECTED_SHA")
        .expect("producer must declare an event-bound protected-base transport");
    let event_ref = materialize
        .find("EVENT_REF: ${{ github.ref }}")
        .expect("producer must transport the provider event ref");
    let fail_closed = materialize
        .find("if [ -z \"${EVENT_PROTECTED_SHA}\" ]; then")
        .expect("manual dispatch without an event-bound protected base must fail closed");
    let evaluated_head = materialize
        .find("evaluated_commit=\"$(git rev-parse HEAD)\"")
        .expect("evaluated commit must be resolved from checked-out HEAD");

    assert!(
        protected_sha < fail_closed && fail_closed < evaluated_head,
        "the producer must reject workflow_dispatch before using provider-bound event facts"
    );
    assert!(
        event_ref < fail_closed,
        "the provider event ref must be declared before materialization validation"
    );
    assert!(
        materialize.contains("--scm-event-ref \"${EVENT_REF}\"") && !materialize.contains("HEAD^1"),
        "the producer must forward the provider event ref and never infer protected ancestry from HEAD^1"
    );
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
                ".github/workflows/oya-ci-required.yml::buck2::step-1"
            ]
        }
    });
    // Baselined key is present (accepted) + a NEW unbaselined key injected.
    let observed = json!({"steps": [
        {"key": ".github/workflows/oya-ci-required.yml::buck2::step-1"},
        {"key": ".github/workflows/oya-ci-required.yml::buck2::step-99-NEW-SHELL"}
    ]});
    let findings = evaluate_workflow_inline_shell_keyed(&observed, &baseline);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "rust_first_automation_unbaselined_workflow_inline_shell"
                && finding.key == ".github/workflows/oya-ci-required.yml::buck2::step-99-NEW-SHELL"
        }),
        "a new inline-shell step beyond baseline must be born-blocking with its key surfaced; got \
         {findings:#?}"
    );
    // The accepted baselined key must NOT be flagged (no false positive).
    assert!(
        !findings.iter().any(|finding| {
            finding.key == ".github/workflows/oya-ci-required.yml::buck2::step-1"
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
                ".github/workflows/docs-graph-drift.yml::docs-graph-drift::step-3"
            ]
        }
    });
    let observed = json!({"steps": []});
    let findings = evaluate_workflow_inline_shell_keyed(&observed, &baseline);
    assert!(findings.iter().any(|finding| {
        finding.code == "rust_first_automation_workflow_inline_shell_baseline_stale"
            && finding.key == ".github/workflows/docs-graph-drift.yml::docs-graph-drift::step-3"
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
