// gate-registration completeness meta-test (ADR-0515 D2; CICD-DESIGN-PLAN Stage 1B + Pre-mortem
// Scenario-1c "silent-skip false-green" sibling acceptance test).
//
// INVARIANT: every gate crate directory under `ci/facade/` — EXCEPT the producer
// (`oya-cloud-ci-accounting-registry-app`, the rust_binary that EMITS the faces, not a gate
// lane) — MUST be registered as a job lane in `.github/workflows/oya-ci-required.yml`, the
// single canonical `oya-ci-required` fan-in. A new gate cannot be added without registering
// it in the required workflow; an in-tree-but-unregistered gate fails this test (it would be a
// silent false-green one level below the workflow's `needs:` fan-in).
//
// It is a pure filesystem + text gate: it reads the gates dir and greps the workflow yaml. No
// network, no GitHub API — runnable in any presubmit. Keep it deterministic and surface-all
// (it collects EVERY unregistered gate, not just the first).
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The crates under `ci/facade/` that are NOT gate lanes — they are the rust_binaries
/// that EMIT the faces (registered in the workflow via a `run` step, not a `cargo test -p ...`
/// gate lane) or the on-demand born-accounting orchestrator. These are the intentional exclusions
/// from the gate-registration invariant:
///   - the accounting producer (emits the five accounting faces);
///   - the scm-facts emitter (the single out-of-graph git boundary that emits
///     scm-facts.generated.json; OYA-CI-HERMETIC-EXECUTION-DESIGN §1.5);
///   - the born-accounting register_crate ORCHESTRATOR (G011 slice 3b, ADR-0568): it MUTATES the
///     source SSOTs (OWNERS/registry/ADR/catalog/reachability) to onboard a NEW crate. It is not a
///     gate lane and not a face-emitter — it is invoked ON DEMAND to register a crate, never in the
///     required fan-in (it would have nothing to assert and would mutate the tree under presubmit).
///   - the planning-projection renderer: a pure library invoked by generated-artifact freshness to
///     materialize the untracked board projection; it has no independent admission verdict.
const PRODUCER_CRATE: &str = "artifact-inventory-registry";
const NON_GATE_CRATES: [&str; 3] = [
    "artifact-inventory-registry",
    "crate-registration",
    "planning-projection",
];

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the helper in `firewall.rs` so both meta-gates
/// resolve the root identically.
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

fn gates_dir(root: &Path) -> PathBuf {
    root.join("ci/facade")
}

fn workflow_path(root: &Path) -> PathBuf {
    root.join(".github/workflows/oya-ci-required.yml")
}

fn phase0_automation_matrix_path(root: &Path) -> PathBuf {
    root.join("specs/phase0-automation-matrix.json")
}

fn branch_protection_path(root: &Path) -> PathBuf {
    root.join(".github/branch-protection.yaml")
}

fn agent_contract_path(root: &Path) -> PathBuf {
    root.join("docs/AGENTS.md")
}

fn pr_template_path(root: &Path) -> PathBuf {
    root.join("docs/templates/pull-request-template.md")
}
fn root_pr_template_path(root: &Path) -> PathBuf {
    root.join("templates/pull-request-template.md")
}

fn pr_template_v2_path(root: &Path) -> PathBuf {
    root.join("docs/templates/pull-request-template-v2.md")
}

fn oya_ci_config_path(root: &Path) -> PathBuf {
    root.join("oya-ci.toml")
}

fn bundled_gate_disposition_path(root: &Path) -> PathBuf {
    root.join("libs/oya-ci-config/src/bundled/gate-disposition.json")
}

/// Resolve the NEW de-branded `ci/facade/<dir>` directory a moved gate crate now lives in,
/// keyed on its OLD cargo/crate name (`oya-<gate_id>-app`). The committed move-plan
/// (`specs/reorg/ci-move-plan.json`, ADR-0562/0563) is the SSOT for the ci keystone rename;
/// the required-workflow matrix `crate:` value is this NEW dir. The de-brand renamed
/// SEMANTICALLY (e.g. cloud-ci-total-accounting -> artifact-accountability), so there is no
/// textual prefix-strip from the gate id to the lane — the move-plan is the only authority.
fn ci_move_new_dir(root: &Path, old_cargo_name: &str) -> Option<String> {
    let plan: Value =
        serde_json::from_str(&read_to_string(&root.join("specs/reorg/ci-move-plan.json"))).ok()?;
    if let Some(moves) = plan.get("moves").and_then(Value::as_array) {
        for m in moves {
            if m.get("old_cargo_name").and_then(Value::as_str) == Some(old_cargo_name) {
                return m
                    .get("new_path")
                    .and_then(Value::as_str)
                    .and_then(|p| p.rsplit('/').next())
                    .map(str::to_owned);
            }
        }
    }
    if let Some(arts) = plan.get("artifacts").and_then(Value::as_array) {
        for a in arts {
            let old_tail = a
                .get("old_path")
                .and_then(Value::as_str)
                .and_then(|p| p.rsplit('/').next());
            if old_tail == Some(old_cargo_name) {
                return a
                    .get("new_path")
                    .and_then(Value::as_str)
                    .and_then(|p| p.rsplit('/').next())
                    .map(str::to_owned);
            }
        }
    }
    None
}

fn code_review_standard_path(root: &Path) -> PathBuf {
    root.join("docs/standards/code-review.md")
}

fn fixuptasks_path(root: &Path) -> PathBuf {
    root.join("registry/fixuptasks.jsonl")
}

/// Every directory directly under `ci/facade/` that is a gate lane. Most lanes have a
/// Cargo manifest, but Buck2-only productized gates are equally required workflow authority and must
/// not be silently skipped by this meta-test.
fn gate_crate_dirs(gates: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(gates)
        .unwrap_or_else(|e| panic!("read gates dir {}: {e}", gates.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.is_dir() && (p.join("Cargo.toml").is_file() || p.join("BUCK").is_file()))
        .map(|p| {
            p.file_name()
                .expect("dir file_name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// The trimmed value of a YAML list item (`- value   # comment`) → `value`, or None if the line
/// is not a list item. Strips an end-of-line comment so matrix entries / `needs:` entries that
/// carry trailing comments still compare cleanly.
fn yaml_list_item(line: &str) -> Option<String> {
    let no_comment = line.split('#').next().unwrap_or("").trim();
    no_comment.strip_prefix("- ").map(|v| v.trim().to_owned())
}

fn fan_in_block(workflow: &str) -> &str {
    let fan_in_anchor = "\n  oya-ci-required:";
    let idx = workflow
        .find(fan_in_anchor)
        .expect("fan-in job `oya-ci-required:` not found in workflow");
    &workflow[idx..]
}

fn workflow_job(workflow: &str, job_name: &str) -> String {
    let anchor = format!("  {job_name}:");
    let mut found = false;
    let mut lines = Vec::new();
    for line in workflow.lines() {
        if line == anchor {
            found = true;
        } else if found && line.starts_with("  ") && !line.starts_with("    ") {
            break;
        }
        if found {
            lines.push(line);
        }
    }
    assert!(found, "workflow job `{job_name}` not found");
    lines.join("\n")
}

fn fan_in_mentions_job(fan_in_block: &str, job: &str) -> bool {
    fan_in_block
        .lines()
        .filter_map(yaml_list_item)
        .any(|v| v == job)
        && fan_in_block.contains(&format!("needs.{job}.result"))
}

fn live_postgres_split_fan_in_is_complete(workflow: &str) -> bool {
    let block = fan_in_block(workflow);
    fan_in_mentions_job(block, "gate-live-postgres-adapters")
        && fan_in_mentions_job(block, "gate-live-postgres-facades")
        && !block.contains("needs.gate-live-postgres.result")
}

fn affected_set_long_step_telemetry_is_wired(workflow: &str) -> bool {
    workflow.contains("oya-cloud-ci-step-telemetry-bin")
        && workflow.contains("--phase derive-affected-set-tier --")
        && workflow.contains("--phase materialize-merge-base-build-health-baseline --")
        && workflow.contains("--phase binding-affected-set-build-test --")
}

/// True iff `crate_dir` is an entry in the `gate` job's `strategy.matrix` — i.e. it is a
/// homogeneous matrix gate run via `cargo test -p ${{ matrix.crate }}`. Recognizes both the
/// simple list form (`- <crate>`) and the `include`-object form (`{ crate: <crate>, label: … }`).
/// Crate names are unique and distinct from the `needs:` job names, so scanning the whole
/// workflow is safe; the per-line comment strip keeps trailing `# …` from defeating the match.
fn is_matrix_gate(workflow: &str, crate_dir: &str) -> bool {
    workflow.lines().any(|line| {
        let t = line.split('#').next().unwrap_or("").trim();
        t == format!("- {crate_dir}") || t.contains(&format!("crate: {crate_dir},"))
    })
}

fn is_buck_gate(workflow: &str, crate_dir: &str) -> bool {
    workflow.contains(&format!("//ci/facade/{crate_dir}:"))
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn fixuptask_by_id(fixuptasks_jsonl: &str, id: &str) -> Option<Value> {
    fixuptasks_jsonl.lines().find_map(|line| {
        let value: Value = serde_json::from_str(line).ok()?;
        (value.get("id").and_then(Value::as_str) == Some(id)).then_some(value)
    })
}

fn bool_field(row: &Value, key: &str) -> bool {
    row.get(key).and_then(Value::as_bool) == Some(true)
}

fn quoted_value(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

fn oya_ci_enabled_gate_specs(config: &str) -> Vec<(String, String)> {
    let mut specs = Vec::new();
    let mut in_gate = false;
    let mut id: Option<String> = None;
    let mut input_kind: Option<String> = None;

    for line in config.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed == "[[gates.enabled]]" {
            if let Some(gate_id) = id.take() {
                specs.push((gate_id, input_kind.take().unwrap_or_default()));
            }
            input_kind = None;
            in_gate = true;
            continue;
        }
        if in_gate && trimmed.starts_with('[') {
            if let Some(gate_id) = id.take() {
                specs.push((gate_id, input_kind.take().unwrap_or_default()));
            }
            input_kind = None;
            in_gate = false;
            continue;
        }
        if !in_gate {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("id = ") {
            id = Some(quoted_value(value));
        } else if let Some(value) = trimmed.strip_prefix("input_kind = ") {
            input_kind = Some(quoted_value(value));
        }
    }

    if let Some(gate_id) = id {
        specs.push((gate_id, input_kind.unwrap_or_default()));
    }

    specs
}

fn assert_pr_template_authority(root: &Path, path: &Path) {
    let text = read_to_string(path);
    for required in [
        "oya-ci-required",
        "F-PR5-06",
        "trusted server-side/cloud-ci",
        "not live cloud admission enforcement",
        "## Code Review",
    ] {
        assert!(
            text.contains(required),
            "{} must align PR authority on `{required}`.",
            path.display()
        );
    }

    for forbidden in [
        "grit",
        "icm",
        "oya-tooling-agent-read",
        "cargo nextest",
        "cargo clippy",
        "cargo deny",
        "oya verify",
        "oya gate validate",
        "guard-pr-merge-review.mjs",
        ".omc/plans",
        ".omc/scratch",
        "Grit-claim",
    ] {
        assert!(
            !text.contains(forbidden),
            "{} still carries retired local/CLI review instruction `{forbidden}`; \
             use Buck2/cloud-ci, `oya-ci-required`, and reviewer evidence instead.",
            path.display()
        );
    }

    let canonical = root.join("docs/templates/pull-request-template.md");
    if path != canonical {
        assert!(
            text.contains("docs/templates/pull-request-template.md")
                || text.contains("Canonical PR body"),
            "{} must identify the canonical docs PR-template authority.",
            path.display()
        );
    }
}

fn row_claims_live_pre_merge_review_authority(row: &Value) -> bool {
    bool_field(row, "review_authority_live")
        && (bool_field(row, "has_durable_review_evidence")
            || bool_field(row, "has_machine_verifiable_review_status"))
        && bool_field(row, "review_blocks_merge")
        && bool_field(row, "reviewer_identity_distinct_from_author")
}

fn phase0_pre_merge_review_rows(root: &Path) -> Vec<Value> {
    let path = phase0_automation_matrix_path(root);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let value: Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    let rows = value
        .get("seed_rows")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} must contain a seed_rows array", path.display()));
    let review_rows: Vec<Value> = rows
        .iter()
        .filter(|row| bool_field(row, "requires_pre_merge_review_authority"))
        .cloned()
        .collect();
    assert!(
        !review_rows.is_empty(),
        "{} must contain the pre-merge review-authority row; removing it would hide F-PR5-06",
        path.display()
    );
    review_rows
}

#[test]
fn every_gate_crate_is_registered_in_oya_ci_required_workflow() {
    let root = repo_root();
    let gates = gates_dir(&root);
    let wf = workflow_path(&root);

    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    let crates = gate_crate_dirs(&gates);
    assert!(
        !crates.is_empty(),
        "found no gate crates under {} — the gates dir layout changed",
        gates.display()
    );

    // The producer must be present in-tree (sanity: the exclusion is real, not a typo).
    assert!(
        crates.iter().any(|c| c == PRODUCER_CRATE),
        "producer crate {PRODUCER_CRATE} not found under {} — update PRODUCER_CRATE if it was \
         renamed",
        gates.display()
    );

    // Surface-all: collect EVERY unregistered gate, then assert the set is empty.
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if NON_GATE_CRATES.contains(&crate_dir.as_str()) {
            continue;
        }
        // A gate is "registered" iff the workflow runs it as a cargo lane, matrix lane, or
        // dedicated Buck target lane. Freshness is a Buck-built binary because it must use the
        // same face producer targets as the materialization boundary without mutating the tree.
        let registered = workflow.contains(&format!("-p {crate_dir}"))
            || is_matrix_gate(&workflow, crate_dir)
            || is_buck_gate(&workflow, crate_dir);
        if !registered {
            missing.push(crate_dir.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "gate crate(s) present under {} but NOT registered in {} — add the crate to the `gate` \
         job's `strategy.matrix.crate` list (homogeneous gates), give it a bespoke `-p <crate>` \
         job, or wire a dedicated `//ci/facade/<crate>:` Buck target \
         job: {:?}\n\
         An in-tree-but-unregistered gate is a silent false-green one level below the fan-in.",
        gates.display(),
        wf.display(),
        missing
    );
}

#[test]
fn fixed_census_receipt_is_a_buck_live_face_gate_not_a_cargo_only_false_green() {
    let root = repo_root();
    let workflow = read_to_string(&workflow_path(&root));
    let buck = read_to_string(&root.join("ci/facade/scm-facts-snapshot/BUCK"));
    let gate_matrix_entry = "crate: scm-facts-snapshot,";
    let download_step = workflow
        .find("Download regenerated faces")
        .expect("gate matrix must download controller-generated faces");
    let test_step = workflow
        .find("buck2 test ${{ matrix.crate }}")
        .expect("gate matrix must execute Buck tests");

    assert!(workflow.contains(gate_matrix_entry));
    assert!(
        download_step < test_step,
        "the live face must be downloaded before validation"
    );
    assert!(
        buck.contains("name = \"ci-scm-facts-snapshot-gate\"")
            && buck.contains("src/bin/adr-census-parent-receipt-gate.rs"),
        "the matrix target must execute the fixed-receipt live validator under Buck"
    );
    assert!(
        workflow.contains("//ci/facade/${{ matrix.crate }}:ci-${{ matrix.crate }}-gate"),
        "a Cargo-only test command is not required-CI authority"
    );
}

/// The review-admission gap must never be hidden behind aspirational docs or repo-local target
/// matrices. Until a trusted server-side/cloud-ci review producer exists and this test is replaced
/// with live producer evidence, the gap must stay explicitly bounded in the docs and the backlog
/// SSOT with concrete evidence from the most recent admission incident.
#[test]
fn review_admission_gap_is_bounded_by_open_ssot_debt_until_trusted_producer() {
    let root = repo_root();
    let branch_path = branch_protection_path(&root);
    let branch_protection = fs::read_to_string(&branch_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", branch_path.display()));

    let review_rows = phase0_pre_merge_review_rows(&root);

    let docs_path = agent_contract_path(&root);
    let docs = fs::read_to_string(&docs_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", docs_path.display()));
    assert!(
        docs.contains("REVIEW-ADMISSION-GAP-LIVE-BOUNDARY")
            && docs.contains("F-PR5-06")
            && docs.contains("PR #964")
            && docs.contains("not a cloud-enforced review admission gate"),
        "{} still describes review admission as live cloud enforcement while the producer is not \
         wired. Add an explicit REVIEW-ADMISSION-GAP-LIVE-BOUNDARY note with F-PR5-06 and PR #964 \
         evidence.",
        docs_path.display()
    );

    for template_path in [
        pr_template_path(&root),
        pr_template_v2_path(&root),
        root_pr_template_path(&root),
    ] {
        assert_pr_template_authority(&root, &template_path);
    }

    let standard_path = code_review_standard_path(&root);
    let standard = fs::read_to_string(&standard_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", standard_path.display()));
    assert!(
        standard.contains("F-PR5-06")
            && standard.contains("advisory only")
            && standard.contains("trusted server-side/cloud-ci review producer"),
        "{} must not present local hooks as live cloud review authority while F-PR5-06 is open.",
        standard_path.display()
    );

    assert!(
        branch_protection.contains("F-PR5-06") && branch_protection.contains("PR #964"),
        "{} must name the bounded review-admission debt while required_approving_reviews is 0 and \
         oya-pr-review is absent.",
        branch_path.display()
    );

    let fixuptasks = fs::read_to_string(fixuptasks_path(&root)).expect("read fixuptasks.jsonl");
    let task =
        fixuptask_by_id(&fixuptasks, "F-PR5-06").expect("registry/fixuptasks.jsonl has F-PR5-06");
    assert_eq!(task.get("status").and_then(Value::as_str), Some("open"));

    for row in review_rows {
        assert!(
            !row_claims_live_pre_merge_review_authority(&row),
            "F-PR5-06 is still open, so repo-local phase0 matrix rows must not be allowed to \
             false-clear the review-admission gap as live authority. Close F-PR5-06 only in the \
             same change that adds trusted server-side/cloud-ci producer evidence. Row was: {row}"
        );
    }

    let task_text = task.to_string();
    for required in [
        "PR #964",
        "reviewDecision",
        "COMMENTED",
        "required_approving_reviews: 0",
        "single oya-ci-required",
        "oya-pr-review",
        "not live",
    ] {
        assert!(
            task_text.contains(required),
            "F-PR5-06 must carry fresh bounded-debt evidence term `{required}`; row was: {task_text}"
        );
    }
}

#[test]
fn oya_ci_configured_gates_have_disposition_and_required_workflow_authority() {
    let root = repo_root();
    let workflow = read_to_string(&workflow_path(&root));
    let config = read_to_string(&oya_ci_config_path(&root));
    let configured = oya_ci_enabled_gate_specs(&config);
    assert!(
        !configured.is_empty(),
        "oya-ci.toml must declare the accounting/firewall gate set under [[gates.enabled]]"
    );

    let disposition_path = bundled_gate_disposition_path(&root);
    let disposition: Value = serde_json::from_str(&read_to_string(&disposition_path))
        .unwrap_or_else(|e| panic!("parse {}: {e}", disposition_path.display()));
    let disposition_gates = disposition
        .get("gates")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{} must contain gates object", disposition_path.display()));

    let configured_ids: BTreeSet<String> = configured
        .iter()
        .map(|(gate_id, _)| gate_id.clone())
        .collect();
    let disposition_ids: BTreeSet<String> = disposition_gates.keys().cloned().collect();

    let missing_disposition: Vec<String> = configured_ids
        .difference(&disposition_ids)
        .cloned()
        .collect();
    let extra_disposition: Vec<String> = disposition_ids
        .difference(&configured_ids)
        .cloned()
        .collect();
    assert!(
        missing_disposition.is_empty() && extra_disposition.is_empty(),
        "oya-ci.toml [[gates.enabled]] and bundled gate-disposition.json must converge exactly; \
         missing disposition for {:?}, extra disposition rows {:?}",
        missing_disposition,
        extra_disposition
    );

    let mut missing_workflow_authority = Vec::new();
    for (gate_id, input_kind) in configured {
        let has_required_lane = match input_kind.as_str() {
            // The matrix `crate:` value is the NEW de-branded ci/facade dir; resolve it via the
            // committed move-plan SSOT (the ci keystone move renamed the crates semantically).
            "producer-face" => ci_move_new_dir(&root, &format!("oya-{gate_id}-app"))
                .is_some_and(|dir| workflow.contains(&format!("crate: {dir},"))),
            "raw-corpus-collector" => {
                workflow.contains("producer-regen") && workflow.contains("gate-baseline-ratchet")
            }
            "frozen-empty-meta" => {
                gate_id == "cloud-ci-freshness"
                    && workflow.contains("gate-generated-artifact-freshness")
            }
            other => panic!("unknown gate input_kind `{other}` for {gate_id} in oya-ci.toml"),
        };
        if !has_required_lane {
            missing_workflow_authority.push(format!("{gate_id} ({input_kind})"));
        }
    }

    assert!(
        missing_workflow_authority.is_empty(),
        "oya-ci.toml enabled gate(s) lack required workflow authority in .github/workflows/oya-ci-required.yml: {:?}",
        missing_workflow_authority
    );
}

/// Defense-in-depth: every registered gate must ALSO be wired into the fan-in job's `needs:`
/// list (a `cargo test -p <crate>` lane that the `oya-ci-required` job does not depend on
/// would never gate the required context). We assert each non-producer gate's directory name
/// appears somewhere after the `oya-ci-required:` job header in a `needs:` context.
#[test]
fn every_gate_lane_is_a_dependency_of_the_fan_in_job() {
    let root = repo_root();
    let gates = gates_dir(&root);
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    let fan_in_block = fan_in_block(&workflow);
    assert!(
        fan_in_block.contains("needs:"),
        "fan-in job `oya-ci-required` has no `needs:` — it must depend on every gate lane"
    );

    // Map each non-producer gate crate to its expected job-name token in the fan-in `needs:`.
    // Job names in this workflow embed the gate identity (e.g. `gate-total-accounting`,
    // `gate-registry-drift`). We assert the gate's short identity appears in the fan-in block.
    let crates = gate_crate_dirs(&gates);
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if NON_GATE_CRATES.contains(&crate_dir.as_str()) {
            continue;
        }
        // A matrix gate runs under the single `gate` job → the fan-in must depend on `gate`.
        // A bespoke gate has its own `gate-<short>` job → the fan-in must reference that short
        // identity. (`registry-drift` is already short; others strip `oya-cloud-ci-`/`-app`.)
        let wired = if is_matrix_gate(&workflow, crate_dir) {
            fan_in_block
                .lines()
                .filter_map(yaml_list_item)
                .any(|v| v == "gate")
        } else {
            let short = crate_dir
                .strip_prefix("oya-cloud-ci-")
                .unwrap_or(crate_dir)
                .strip_suffix("-app")
                .map(str::to_owned)
                .unwrap_or_else(|| crate_dir.clone());
            fan_in_block.contains(&short)
        };
        if !wired {
            missing.push(format!(
                "{crate_dir} (matrix gate ⇒ needs `- gate`; bespoke ⇒ needs its `gate-<short>` job)"
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "gate lane(s) not wired into the `oya-ci-required` fan-in job's `needs:` in {}: {:?}\n\
         Every gate lane must be a dependency of the fan-in or its failure cannot make the \
         required context red.",
        wf.display(),
        missing
    );
}

#[test]
fn live_postgres_split_lanes_are_both_required_by_fan_in() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));
    assert!(
        live_postgres_split_fan_in_is_complete(&workflow),
        "fan-in must require both split live-postgres jobs and must not keep the retired monolithic needs token"
    );

    let without_adapters =
        workflow.replace("      - gate-live-postgres-adapters", "      # removed");
    assert!(
        !live_postgres_split_fan_in_is_complete(&without_adapters),
        "missing adapter sublane must be detected as fan-in incomplete"
    );

    let without_facades = workflow.replace("      - gate-live-postgres-facades", "      # removed");
    assert!(
        !live_postgres_split_fan_in_is_complete(&without_facades),
        "missing facade sublane must be detected as fan-in incomplete"
    );
}

#[test]
fn windows_workspace_resolver_differential_is_a_buck2_matrix_leg() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    let gate_job = workflow_job(&workflow, "gate");
    assert!(
        gate_job.contains("runs-on: ${{ matrix.os || 'ubuntu-latest' }}"),
        "the reusable gate matrix must select its runner from the matrix"
    );
    assert!(
        gate_job.contains("os: windows-latest")
            && gate_job.contains(
                "targets: \"//libs/oya-workspace-members-kernel:oya-workspace-members-kernel-cargo-differential\""
            ),
        "the Windows matrix leg must execute the workspace resolver Cargo differential Buck2 target"
    );
    assert!(
        gate_job.contains("shell: pwsh") && gate_job.contains("$IsWindows"),
        "the shared matrix step must use PowerShell, which is native on Windows and avoids Bash/MSYS rewriting"
    );
    assert!(
        gate_job.contains("[string]::IsNullOrWhiteSpace($targets)")
            && gate_job.contains("cmd.exe /d /s /c")
            && gate_job.contains("call `\"%ProgramFiles%\\Microsoft Visual Studio\\2022\\Enterprise\\Common7\\Tools\\VsDevCmd.bat`\"")
            && gate_job.contains("buck2 test $targets"),
        "the Windows-native path must reject an empty target, initialize MSVC, and run the exact Buck2 target"
    );
    assert!(
        !gate_job.contains("shell: bash\n        # Default matrix legs expand"),
        "the Windows Buck2 path must not be routed through Bash/MSYS"
    );
    assert!(
        !workflow.contains("\n  windows-workspace-member-resolver:")
            && !gate_job.contains("cargo test --locked -p oya-workspace-members-kernel"),
        "the Windows differential must stay inside the Buck2 target, not add a direct Cargo job"
    );
    assert!(
        fan_in_mentions_job(fan_in_block(&workflow), "gate"),
        "the Windows matrix leg must remain under the single oya-ci-required fan-in"
    );
}

#[test]
fn windows_buck2_toolchain_uses_prelude_msvc_defaults() {
    let root = repo_root();
    let toolchains = fs::read_to_string(root.join("toolchains/BUCK"))
        .expect("read system toolchain declarations");

    for field in ["compiler", "compiler_type", "linker", "archiver"] {
        assert!(
            toolchains.contains(&format!("{field} = None if host_info().os.is_windows else")),
            "Windows must retain the prelude MSVC {field} default instead of overriding it with a Unix path"
        );
    }
}

#[test]
fn affected_set_long_step_telemetry_wraps_long_running_phases() {
    let root = repo_root();
    let wf = workflow_path(&root);
    let workflow =
        fs::read_to_string(&wf).unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    assert!(
        affected_set_long_step_telemetry_is_wired(&workflow),
        "affected-set long-running derive/baseline/binding phases must be wrapped by the telemetry helper"
    );

    let without_baseline = workflow.replace(
        "--phase materialize-merge-base-build-health-baseline --",
        "--phase removed --",
    );
    assert!(
        !affected_set_long_step_telemetry_is_wired(&without_baseline),
        "missing baseline materialization telemetry must be detected"
    );
}
