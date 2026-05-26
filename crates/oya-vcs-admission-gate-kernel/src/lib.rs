//! Foundry Oya-VCS admission-gate kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-vcs-admission-gate-kernel` —
//!   v4 BNF `oya-<product:foundry>-<topic:vcs-admission-gate>-<layer:kernel>`;
//!   13-value layer-enum suffix `kernel` (innermost ring: I/O-free port +
//!   pure invariant checks per ADR-0056 §"port-in-kernel").
//! - Companion `oya-vcs-admission-gate-app` —
//!   v4 BNF `oya-<product:foundry>-<topic:vcs-admission-gate>-<layer:app>`;
//!   binary tool surface (canonical `app` suffix per ADR-0105 §"Amendment
//!   2026-05-15 — `tools/` canonical-suffix binding"), wraps the kernel for
//!   the `oya-vcs-admission` required-check.
//!
//! # Intent
//!
//! Replaces `scripts/check-oya-vcs-admission.sh` (Wave 3 of shell/python →
//! Rust replacement program; audit `evidence/audits/shell-python-replacement-audit-2026-05-15.md`
//! row B-3). The check proves the Oya VCS replacement core, authority
//! cutover metadata, multispectrum wiring, and CI branch-protection
//! visibility without requiring a full-workspace run.
//!
//! # Algorithm (kernel — I/O-free)
//!
//! Runners load:
//!   1. `specs/root-hub-pointers.json`
//!   2. `specs/master-plan-sequencing.json`
//!   3. `specs/multispectrum-review.json`
//!   4. `specs/gitops-vcs-replacement.json`
//!   5. Provider-evidence JSON referenced from (4)
//!   6. Provider-execution-proof JSON referenced from (4)
//!   7. `.github/branch-protection.yaml` (read as text)
//!   8. `infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy` (read as text;
//!      the Jenkins-native pipeline that retired pr-tests.yml per ADR-0361)
//!   9. same Jenkins pipeline source (retired oya-governance-supply-chain.yml)
//!  10. `cargo metadata --no-deps --format-version 1` output
//!  11. `evidence/audit-chain.jsonl` (read as text)
//!  12. Each JSON file under `evidence/multispectrum/`
//!
//! …and pass them as typed [`AdmissionInputs`] into [`validate_admission`].
//! The kernel returns an [`AdmissionReport`] with zero violations on
//! success, or one [`AdmissionViolation`] per failed invariant.

#![forbid(unsafe_code)]
// ADR-0083 Tier 1 (kernel): no `.unwrap()` / `.expect()` / `panic!()` in
// non-test code. Tests use the cfg(test) exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use serde_json::Value;
use std::collections::BTreeSet;

/// Typed inputs for the admission check. All payloads are I/O-loaded by
/// the runner; the kernel does no filesystem or subprocess work.
#[derive(Clone, Debug)]
pub struct AdmissionInputs<'a> {
    /// Parsed `specs/root-hub-pointers.json`.
    /// data_class: INTERNAL_ONLY
    pub root_hub_pointers: &'a Value, // data_class: INTERNAL_ONLY
    /// Parsed `specs/master-plan-sequencing.json`.
    /// data_class: INTERNAL_ONLY
    pub master_plan_sequencing: &'a Value, // data_class: INTERNAL_ONLY
    /// Parsed `specs/multispectrum-review.json`.
    /// data_class: INTERNAL_ONLY
    pub multispectrum_review: &'a Value, // data_class: INTERNAL_ONLY
    /// Parsed `specs/gitops-vcs-replacement.json`.
    /// data_class: INTERNAL_ONLY
    pub gitops_vcs_replacement: &'a Value, // data_class: INTERNAL_ONLY
    /// Path written in `gitops_vcs_replacement.current_ci_admission_lane.provider_evidence_ref`.
    /// data_class: INTERNAL_ONLY
    pub provider_evidence_ref: &'a str, // data_class: INTERNAL_ONLY
    /// Parsed JSON file that `provider_evidence_ref` resolves to.
    /// data_class: INTERNAL_ONLY
    pub provider_evidence: &'a Value, // data_class: INTERNAL_ONLY
    /// Path written in `gitops_vcs_replacement.current_ci_admission_lane.provider_execution_proof_ref`.
    /// data_class: INTERNAL_ONLY
    pub provider_execution_proof_ref: &'a str, // data_class: INTERNAL_ONLY
    /// Parsed JSON file that `provider_execution_proof_ref` resolves to.
    /// data_class: INTERNAL_ONLY
    pub provider_execution_proof: &'a Value, // data_class: INTERNAL_ONLY
    /// Raw text of `.github/branch-protection.yaml`.
    /// data_class: INTERNAL_ONLY
    pub branch_protection_yaml: &'a str, // data_class: INTERNAL_ONLY
    /// Raw text of `.github/workflows/pr-tests.yml`.
    /// data_class: INTERNAL_ONLY
    pub pr_tests_workflow: &'a str, // data_class: INTERNAL_ONLY
    /// Raw text of `.github/workflows/oya-governance-supply-chain.yml`.
    /// data_class: INTERNAL_ONLY
    pub supply_chain_workflow: &'a str, // data_class: INTERNAL_ONLY
    /// Crate names enumerated by `cargo metadata --no-deps`.
    /// data_class: INTERNAL_ONLY
    pub workspace_packages: &'a [String], // data_class: INTERNAL_ONLY
    /// Raw text of `evidence/audit-chain.jsonl`.
    /// data_class: INTERNAL_ONLY
    pub audit_chain_jsonl: &'a str, // data_class: INTERNAL_ONLY
    /// One `(path, parsed_json)` per file under `evidence/multispectrum/*.json`.
    /// data_class: INTERNAL_ONLY
    pub multispectrum_evidence: &'a [(String, Value)], // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionViolation {
    pub code: &'static str, // data_class: INTERNAL_ONLY
    pub detail: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionReport {
    pub violations: Vec<AdmissionViolation>, // data_class: INTERNAL_ONLY
}

impl AdmissionReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Pure validator. Returns the full violation list (no short-circuit) so
/// runners surface every failure mode in one CI cycle (surface-all-failures
/// posture per M01-P09-IP-007).
pub fn validate_admission(inputs: &AdmissionInputs<'_>) -> AdmissionReport {
    let mut violations = Vec::new();

    check_root_hub_pointers(inputs.root_hub_pointers, &mut violations);
    check_master_plan_sequencing(inputs.master_plan_sequencing, &mut violations);
    check_multispectrum_agentic_flow(inputs.multispectrum_review, &mut violations);
    check_gitops_vcs_replacement(inputs.gitops_vcs_replacement, &mut violations);
    check_provider_evidence(
        inputs.provider_evidence_ref,
        inputs.provider_evidence,
        &mut violations,
    );
    check_provider_execution_proof(
        inputs.provider_execution_proof_ref,
        inputs.provider_execution_proof,
        &mut violations,
    );
    check_branch_protection(inputs.branch_protection_yaml, &mut violations);
    check_pr_tests_workflow(inputs.pr_tests_workflow, &mut violations);
    check_supply_chain_workflow(inputs.supply_chain_workflow, &mut violations);
    check_workspace_packages(inputs.workspace_packages, &mut violations);
    check_audit_chain_coverage(
        inputs.audit_chain_jsonl,
        inputs.multispectrum_evidence,
        &mut violations,
    );

    AdmissionReport { violations }
}

fn push(violations: &mut Vec<AdmissionViolation>, code: &'static str, detail: impl Into<String>) {
    violations.push(AdmissionViolation {
        code,
        detail: detail.into(),
    });
}

fn check_root_hub_pointers(value: &Value, violations: &mut Vec<AdmissionViolation>) {
    let quick = value.get("agent_quick_start_protocol");
    let quick_text = quick.map(|q| q.to_string()).unwrap_or_default();
    if !quick_text.contains("Oya VCS")
        || !quick_text.contains("claim")
        || !quick_text.contains("promote")
    {
        push(
            violations,
            "ROOT_QUICK_START_MISSING_OYA_VCS",
            "root quick-start must make Oya VCS claim/work/verify/done/promote discoverable",
        );
    }
    if let Some(quick_obj) = quick.and_then(|q| q.as_object())
        && quick_obj.contains_key("step_3_grit_state_transition")
    {
        push(
            violations,
            "ROOT_QUICK_START_RETAINS_GRIT_STEP_3",
            "root quick-start still exposes grit as the primary step_3 state transition",
        );
    }
}

fn check_master_plan_sequencing(value: &Value, violations: &mut Vec<AdmissionViolation>) {
    let sequence: Vec<String> = value
        .get("sequence")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let sequence_set: BTreeSet<&str> = sequence.iter().map(String::as_str).collect();

    if !sequence_set.contains("claim_with_oya_vcs")
        || !sequence_set.contains("oya_vcs_promote_or_record_blocker")
    {
        push(
            violations,
            "MASTER_PLAN_SEQUENCE_MISSING_OYA_VCS",
            "master-plan sequence must route claim and promotion through Oya VCS",
        );
    }
    let legacy: [&str; 3] = [
        "claim_with_grit_or_scaffold_lock",
        "close_claim_with_done_bundle",
        "store_icm_completion_summary",
    ];
    let leaking: Vec<&str> = legacy
        .iter()
        .copied()
        .filter(|step| sequence_set.contains(step))
        .collect();
    if !leaking.is_empty() {
        push(
            violations,
            "MASTER_PLAN_SEQUENCE_RETAINS_LEGACY",
            format!("master-plan sequence still contains legacy authority steps: {leaking:?}"),
        );
    }

    let must_have: BTreeSet<String> = value
        .get("implementation_plan_changeset_contract")
        .and_then(|c| c.get("must_have"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if !must_have.contains("oya_vcs_claim_scope") || !must_have.contains("vcs_completion_payload") {
        push(
            violations,
            "IMPL_PLAN_CONTRACT_MISSING_OYA_VCS_FIELDS",
            "ImplementationPlan contract must require Oya VCS claim scope and VCS completion payload",
        );
    }
    if must_have.contains("grit_claim_or_scaffold_lock_scope")
        || must_have.contains("icm_completion_payload")
    {
        push(
            violations,
            "IMPL_PLAN_CONTRACT_RETAINS_LEGACY_FIELDS",
            "ImplementationPlan contract still requires legacy grit/icm fields",
        );
    }
}

fn check_multispectrum_agentic_flow(value: &Value, violations: &mut Vec<AdmissionViolation>) {
    let agentic = value
        .get("enforcement_scopes")
        .and_then(|s| s.get("agentic_flow"));
    let agentic_text = agentic.map(|a| a.to_string()).unwrap_or_default();
    if !agentic_text.contains("Oya VCS") {
        push(
            violations,
            "MULTISPECTRUM_AGENTIC_MISSING_OYA_VCS",
            "multispectrum agentic_flow must name Oya VCS as the enforcing state machine",
        );
    }
    if agentic_text.contains("grit done") {
        push(
            violations,
            "MULTISPECTRUM_AGENTIC_RETAINS_GRIT_DONE",
            "multispectrum agentic_flow still treats grit done as promotion authority",
        );
    }
}

fn check_gitops_vcs_replacement(value: &Value, violations: &mut Vec<AdmissionViolation>) {
    let admission_ids: BTreeSet<String> = value
        .get("gitops_pipeline_integration")
        .and_then(|p| p.get("admission_gates"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|gate| gate.get("id").and_then(Value::as_str).map(String::from))
                .collect()
        })
        .unwrap_or_default();
    for required in [
        "claim-coverage",
        "policy-and-tests",
        "multispectrum-evidence",
        "controller-owned-rebase",
        "merge-queue-ownership",
        "cli-command-surface",
        "provider-evidence-slots",
        "audit-chain-coverage",
    ] {
        if !admission_ids.contains(required) {
            push(
                violations,
                "GITOPS_ADMISSION_GATE_MISSING",
                format!("Oya VCS admission gate missing {required}"),
            );
        }
    }

    let closure_authority = value
        .get("foundry_agentic_pipeline_integration_plan")
        .and_then(|p| p.get("closure_authority"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if closure_authority != "Oya VCS ChangeBundle -> Promotion -> ReleaseTrain" {
        push(
            violations,
            "GITOPS_CLOSURE_AUTHORITY_INCORRECT",
            "Foundry integration plan must declare Oya VCS closure authority",
        );
    }

    let current_lane = value.get("current_ci_admission_lane");
    let smoke_joined = current_lane
        .and_then(|c| c.get("command_surface_smoke"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    for verb in ["claim", "verify", "done", "promote"] {
        let needle = format!("oya vcs --format json {verb}");
        if !smoke_joined.contains(&needle) {
            push(
                violations,
                "GITOPS_LANE_SMOKE_MISSING_VERB",
                format!("current CI lane must smoke-test oya vcs {verb}"),
            );
        }
    }

    let provider_required: BTreeSet<String> = current_lane
        .and_then(|c| c.get("provider_evidence_required"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let expected_providers: BTreeSet<String> = ["ci", "github-actions", "trivy", "argo-gitops"]
        .into_iter()
        .map(String::from)
        .collect();
    if provider_required != expected_providers {
        let mut sorted: Vec<&String> = expected_providers.iter().collect();
        sorted.sort();
        push(
            violations,
            "GITOPS_LANE_PROVIDER_REQUIRED_MISMATCH",
            format!("current CI lane provider evidence must be {sorted:?}"),
        );
    }
}

fn check_provider_evidence(
    provider_evidence_ref: &str,
    provider_doc: &Value,
    violations: &mut Vec<AdmissionViolation>,
) {
    if provider_evidence_ref.is_empty() {
        push(
            violations,
            "PROVIDER_EVIDENCE_REF_EMPTY",
            "current CI lane must point at provider evidence",
        );
        return;
    }
    let slots = provider_doc
        .get("provider_evidence")
        .and_then(|p| p.get("slots"))
        .and_then(Value::as_array);
    let Some(slot_array) = slots else {
        push(
            violations,
            "PROVIDER_EVIDENCE_SLOTS_MISSING",
            "provider evidence document has no provider_evidence.slots array",
        );
        return;
    };
    let expected = ["ci", "github-actions", "trivy", "argo-gitops"];
    for required in expected {
        let slot = slot_array
            .iter()
            .find(|s| s.get("id").and_then(Value::as_str) == Some(required));
        let Some(slot) = slot else {
            push(
                violations,
                "PROVIDER_EVIDENCE_SLOT_MISSING",
                format!("provider evidence missing slot {required}"),
            );
            continue;
        };
        if slot.get("provider_kind").and_then(Value::as_str) != Some(required) {
            push(
                violations,
                "PROVIDER_EVIDENCE_KIND_MISMATCH",
                format!("provider evidence slot {required} has wrong provider kind"),
            );
        }
        if slot.get("availability").and_then(Value::as_str) != Some("available")
            || slot.get("decision").and_then(Value::as_str) != Some("passed")
        {
            push(
                violations,
                "PROVIDER_EVIDENCE_NOT_PASSED",
                format!("provider evidence slot {required} must be available/passed"),
            );
        }
        let proof_kind = slot.get("proof_kind").and_then(Value::as_str).unwrap_or("");
        let evidence_ref = slot
            .get("evidence_ref")
            .and_then(Value::as_str)
            .unwrap_or("");
        let live_status = slot
            .get("live_status")
            .and_then(Value::as_str)
            .unwrap_or("");
        if evidence_ref.is_empty() || proof_kind.is_empty() {
            push(
                violations,
                "PROVIDER_EVIDENCE_REF_OR_KIND_EMPTY",
                format!("provider evidence slot {required} must name evidence_ref and proof_kind"),
            );
        }
        if (required == "trivy" || required == "argo-gitops")
            && proof_kind.to_lowercase().contains("fixture")
        {
            push(
                violations,
                "PROVIDER_EVIDENCE_REGRESSED_TO_FIXTURE",
                format!(
                    "provider evidence slot {required} regressed to fixture-only proof: {proof_kind}"
                ),
            );
        }
        if live_status.contains("remote-run-not-required") {
            push(
                violations,
                "PROVIDER_EVIDENCE_REMOTE_RUN_NOT_REQUIRED",
                format!("provider evidence slot {required} still says remote-run-not-required"),
            );
        }
    }
}

fn check_provider_execution_proof(
    proof_ref: &str,
    proof_doc: &Value,
    violations: &mut Vec<AdmissionViolation>,
) {
    if proof_ref.is_empty() {
        push(
            violations,
            "PROVIDER_EXECUTION_PROOF_REF_EMPTY",
            "current CI lane must point at provider execution proof",
        );
        return;
    }
    let slots = proof_doc.get("provider_slots").and_then(Value::as_array);
    let Some(slot_array) = slots else {
        push(
            violations,
            "PROVIDER_EXECUTION_SLOTS_MISSING",
            "provider execution proof document has no provider_slots array",
        );
        return;
    };
    let expected = ["ci", "github-actions", "trivy", "argo-gitops"];
    for required in expected {
        let slot = slot_array
            .iter()
            .find(|s| s.get("id").and_then(Value::as_str) == Some(required));
        let Some(slot) = slot else {
            push(
                violations,
                "PROVIDER_EXECUTION_SLOT_MISSING",
                format!("provider execution proof missing slot {required}"),
            );
            continue;
        };
        if slot.get("provider_kind").and_then(Value::as_str) != Some(required) {
            push(
                violations,
                "PROVIDER_EXECUTION_KIND_MISMATCH",
                format!("provider execution proof slot {required} has wrong provider kind"),
            );
        }
        if slot.get("decision").and_then(Value::as_str) != Some("passed") {
            push(
                violations,
                "PROVIDER_EXECUTION_NOT_PASSED",
                format!("provider execution proof slot {required} must be passed"),
            );
        }
        if slot
            .get("execution_mode")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
        {
            push(
                violations,
                "PROVIDER_EXECUTION_MODE_MISSING",
                format!("provider execution proof slot {required} must name execution_mode"),
            );
        }
        if (required == "trivy" || required == "argo-gitops")
            && slot
                .get("evidence_digest")
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
        {
            push(
                violations,
                "PROVIDER_EXECUTION_EVIDENCE_DIGEST_MISSING",
                format!("provider execution proof slot {required} must carry evidence_digest"),
            );
        }
    }
}

fn check_branch_protection(text: &str, violations: &mut Vec<AdmissionViolation>) {
    if !text.contains("required_status_checks:") || !text.contains("- oya-vcs-admission") {
        push(
            violations,
            "BRANCH_PROTECTION_MISSING_ADMISSION",
            "branch protection must require oya-vcs-admission",
        );
    }
    if !text.contains("- oya-vcs-provider-execution") {
        push(
            violations,
            "BRANCH_PROTECTION_MISSING_PROVIDER_EXECUTION",
            "branch protection must require oya-vcs-provider-execution",
        );
    }
}

fn check_pr_tests_workflow(text: &str, violations: &mut Vec<AdmissionViolation>) {
    if !text.contains("oya-vcs-admission") || !text.contains("oya-vcs-admission-gate-app") {
        push(
            violations,
            "PR_TESTS_WORKFLOW_MISSING_ADMISSION_JOB",
            "pr-tests workflow must expose the oya-vcs-admission job invoking oya-vcs-admission-gate-app",
        );
    }
    let invokes_legacy_script = text.contains("scripts/install-trivy-ci.sh");
    let invokes_rust_installer = text.contains("supply-chain install-trivy");
    if !invokes_legacy_script && !invokes_rust_installer {
        push(
            violations,
            "PR_TESTS_WORKFLOW_MISSING_TRIVY_INSTALL",
            "pr-tests workflow must install Trivy before Oya VCS admission via the Rust supply-chain installer or compatibility shim",
        );
    }
}

fn check_supply_chain_workflow(text: &str, violations: &mut Vec<AdmissionViolation>) {
    // The required job ID is the branch-protection contract surface. The
    // body of the job may invoke either the legacy script or the Wave 3
    // Rust app (the script removal lands atomically with the workflow
    // body switch in commit 2 of the Wave 3 fan-out).
    if !text.contains("oya-vcs-provider-execution") {
        push(
            violations,
            "SUPPLY_CHAIN_WORKFLOW_MISSING_PROVIDER_EXECUTION_JOB",
            "oya-governance-supply-chain workflow must expose the oya-vcs-provider-execution job",
        );
    }
    let invokes_legacy_script =
        text.contains("scripts/check-oya-vcs-provider-execution.sh --mode ci");
    let invokes_rust_app = text.contains("oya-vcs-provider-execution-gate-app");
    if !invokes_legacy_script && !invokes_rust_app {
        push(
            violations,
            "SUPPLY_CHAIN_WORKFLOW_MISSING_PROVIDER_EXECUTION_INVOCATION",
            "oya-governance-supply-chain workflow must invoke either scripts/check-oya-vcs-provider-execution.sh --mode ci (legacy) or oya-vcs-provider-execution-gate-app (Wave 3)",
        );
    }
}

fn check_workspace_packages(packages: &[String], violations: &mut Vec<AdmissionViolation>) {
    let observed: BTreeSet<&str> = packages.iter().map(String::as_str).collect();
    let expected = [
        "oya-dev-cli",
        "oya-vcs-kernel",
        "oya-vcs-ast-index-kernel",
        "oya-vcs-lockstore-adapter",
        "oya-vcs-changebundle-kernel",
        "oya-vcs-polyglot-indexer-adapter",
        "oya-vcs-test-standard-gate-kernel",
        "oya-vcs-promotion-controller-kernel",
        "oya-vcs-review-mergequeue-kernel",
        "oya-vcs-cli-ratchet-kernel",
    ];
    let mut missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|pkg| !observed.contains(pkg))
        .collect();
    if !missing.is_empty() {
        missing.sort();
        push(
            violations,
            "WORKSPACE_MISSING_OYA_VCS_PACKAGES",
            format!("workspace missing Oya VCS packages: {missing:?}"),
        );
    }
}

fn check_audit_chain_coverage(
    audit_chain_text: &str,
    multispectrum_evidence: &[(String, Value)],
    violations: &mut Vec<AdmissionViolation>,
) {
    for (path, evidence) in multispectrum_evidence {
        let Some(change_id) = evidence.get("change_id").and_then(Value::as_str) else {
            push(
                violations,
                "MULTISPECTRUM_EVIDENCE_MISSING_CHANGE_ID",
                format!("multispectrum evidence {path} has no change_id"),
            );
            continue;
        };
        if !audit_chain_text.contains(change_id) {
            push(
                violations,
                "AUDIT_CHAIN_MISSING_CHANGE_ID",
                format!(
                    "multispectrum evidence {path} missing audit-chain coverage for {change_id}"
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[allow(clippy::too_many_arguments)]
    fn empty_inputs<'a>(
        root: &'a Value,
        seq: &'a Value,
        msr: &'a Value,
        vcs: &'a Value,
        pe: &'a Value,
        pex: &'a Value,
        bp: &'a str,
        pr: &'a str,
        sc: &'a str,
        pkgs: &'a [String],
        chain: &'a str,
        ms: &'a [(String, Value)],
    ) -> AdmissionInputs<'a> {
        AdmissionInputs {
            root_hub_pointers: root,
            master_plan_sequencing: seq,
            multispectrum_review: msr,
            gitops_vcs_replacement: vcs,
            provider_evidence_ref: "specs/provider-evidence.json",
            provider_evidence: pe,
            provider_execution_proof_ref: "evidence/gitops-vcs/provider-execution-proof.json",
            provider_execution_proof: pex,
            branch_protection_yaml: bp,
            pr_tests_workflow: pr,
            supply_chain_workflow: sc,
            workspace_packages: pkgs,
            audit_chain_jsonl: chain,
            multispectrum_evidence: ms,
        }
    }

    fn passing_root() -> Value {
        json!({
            "agent_quick_start_protocol": {
                "step_1": "Oya VCS claim",
                "step_2": "verify",
                "step_3": "promote",
            }
        })
    }

    fn passing_seq() -> Value {
        json!({
            "sequence": [
                "claim_with_oya_vcs",
                "oya_vcs_promote_or_record_blocker"
            ],
            "implementation_plan_changeset_contract": {
                "must_have": ["oya_vcs_claim_scope", "vcs_completion_payload"]
            }
        })
    }

    fn passing_msr() -> Value {
        json!({
            "enforcement_scopes": {
                "agentic_flow": {"authority": "Oya VCS"}
            }
        })
    }

    fn passing_vcs() -> Value {
        json!({
            "gitops_pipeline_integration": {
                "admission_gates": [
                    {"id": "claim-coverage"},
                    {"id": "policy-and-tests"},
                    {"id": "multispectrum-evidence"},
                    {"id": "controller-owned-rebase"},
                    {"id": "merge-queue-ownership"},
                    {"id": "cli-command-surface"},
                    {"id": "provider-evidence-slots"},
                    {"id": "audit-chain-coverage"}
                ]
            },
            "foundry_agentic_pipeline_integration_plan": {
                "closure_authority": "Oya VCS ChangeBundle -> Promotion -> ReleaseTrain"
            },
            "current_ci_admission_lane": {
                "command_surface_smoke": [
                    "oya vcs --format json claim",
                    "oya vcs --format json verify",
                    "oya vcs --format json done",
                    "oya vcs --format json promote"
                ],
                "provider_evidence_required": ["ci", "github-actions", "trivy", "argo-gitops"]
            }
        })
    }

    fn passing_provider_evidence() -> Value {
        json!({
            "provider_evidence": {
                "slots": [
                    {"id": "ci", "provider_kind": "ci", "availability": "available",
                     "decision": "passed", "evidence_ref": "ev", "proof_kind": "live", "live_status": "ok"},
                    {"id": "github-actions", "provider_kind": "github-actions",
                     "availability": "available", "decision": "passed",
                     "evidence_ref": "ev", "proof_kind": "live", "live_status": "ok"},
                    {"id": "trivy", "provider_kind": "trivy", "availability": "available",
                     "decision": "passed", "evidence_ref": "ev", "proof_kind": "live", "live_status": "ok"},
                    {"id": "argo-gitops", "provider_kind": "argo-gitops",
                     "availability": "available", "decision": "passed",
                     "evidence_ref": "ev", "proof_kind": "live", "live_status": "ok"}
                ]
            }
        })
    }

    fn passing_proof() -> Value {
        json!({
            "provider_slots": [
                {"id": "ci", "provider_kind": "ci", "decision": "passed",
                 "execution_mode": "live-local-or-runner"},
                {"id": "github-actions", "provider_kind": "github-actions",
                 "decision": "passed", "execution_mode": "live-runner"},
                {"id": "trivy", "provider_kind": "trivy", "decision": "passed",
                 "execution_mode": "live-local-or-runner", "evidence_digest": "sha256:abc"},
                {"id": "argo-gitops", "provider_kind": "argo-gitops",
                 "decision": "passed",
                 "execution_mode": "credentialless-desired-state-dry-run",
                 "evidence_digest": "sha256:def"}
            ]
        })
    }

    fn passing_packages() -> Vec<String> {
        [
            "oya-dev-cli",
            "oya-vcs-kernel",
            "oya-vcs-ast-index-kernel",
            "oya-vcs-lockstore-adapter",
            "oya-vcs-changebundle-kernel",
            "oya-vcs-polyglot-indexer-adapter",
            "oya-vcs-test-standard-gate-kernel",
            "oya-vcs-promotion-controller-kernel",
            "oya-vcs-review-mergequeue-kernel",
            "oya-vcs-cli-ratchet-kernel",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    #[test]
    fn clean_inputs_produce_no_violations() {
        let root = passing_root();
        let seq = passing_seq();
        let msr = passing_msr();
        let vcs = passing_vcs();
        let pe = passing_provider_evidence();
        let pex = passing_proof();
        let pkgs = passing_packages();
        let ms = vec![(
            "evidence/multispectrum/x.json".to_string(),
            json!({"change_id": "CID-1"}),
        )];
        let chain = "CID-1\n".to_string();
        let bp = "required_status_checks:\n  - oya-vcs-admission\n  - oya-vcs-provider-execution\n";
        let pr = "oya-vcs-admission ... cargo run -q -p oya-vcs-admission-gate-app ... cargo run -p oya-dev-cli -- supply-chain install-trivy";
        let sc = "oya-vcs-provider-execution ... cargo run -q -p oya-vcs-provider-execution-gate-app --mode ci";
        let inputs = empty_inputs(
            &root, &seq, &msr, &vcs, &pe, &pex, bp, pr, sc, &pkgs, &chain, &ms,
        );
        let report = validate_admission(&inputs);
        assert!(report.is_clean(), "expected clean, got {report:?}");
    }

    #[test]
    fn missing_oya_vcs_in_root_quick_start_is_flagged() {
        let root = json!({"agent_quick_start_protocol": {"step_1": "nothing"}});
        let seq = passing_seq();
        let msr = passing_msr();
        let vcs = passing_vcs();
        let pe = passing_provider_evidence();
        let pex = passing_proof();
        let pkgs = passing_packages();
        let ms: Vec<(String, Value)> = Vec::new();
        let bp = "required_status_checks:\n  - oya-vcs-admission\n  - oya-vcs-provider-execution\n";
        let pr = "oya-vcs-admission ... oya-vcs-admission-gate-app ... cargo run -p oya-dev-cli -- supply-chain install-trivy";
        let sc = "oya-vcs-provider-execution ... oya-vcs-provider-execution-gate-app";
        let inputs = empty_inputs(
            &root, &seq, &msr, &vcs, &pe, &pex, bp, pr, sc, &pkgs, "", &ms,
        );
        let report = validate_admission(&inputs);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "ROOT_QUICK_START_MISSING_OYA_VCS")
        );
    }

    #[test]
    fn legacy_grit_step_in_master_plan_is_flagged() {
        let root = passing_root();
        let seq = json!({
            "sequence": ["claim_with_oya_vcs",
                         "oya_vcs_promote_or_record_blocker",
                         "claim_with_grit_or_scaffold_lock"],
            "implementation_plan_changeset_contract": {
                "must_have": ["oya_vcs_claim_scope", "vcs_completion_payload"]
            }
        });
        let msr = passing_msr();
        let vcs = passing_vcs();
        let pe = passing_provider_evidence();
        let pex = passing_proof();
        let pkgs = passing_packages();
        let ms: Vec<(String, Value)> = Vec::new();
        let bp = "required_status_checks:\n  - oya-vcs-admission\n  - oya-vcs-provider-execution\n";
        let pr = "oya-vcs-admission ... oya-vcs-admission-gate-app ... cargo run -p oya-dev-cli -- supply-chain install-trivy";
        let sc = "oya-vcs-provider-execution ... oya-vcs-provider-execution-gate-app";
        let inputs = empty_inputs(
            &root, &seq, &msr, &vcs, &pe, &pex, bp, pr, sc, &pkgs, "", &ms,
        );
        let report = validate_admission(&inputs);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "MASTER_PLAN_SEQUENCE_RETAINS_LEGACY")
        );
    }

    #[test]
    fn fixture_only_trivy_proof_kind_is_flagged() {
        let root = passing_root();
        let seq = passing_seq();
        let msr = passing_msr();
        let vcs = passing_vcs();
        let mut pe = passing_provider_evidence();
        pe["provider_evidence"]["slots"][2]["proof_kind"] = json!("fixture-only");
        let pex = passing_proof();
        let pkgs = passing_packages();
        let ms: Vec<(String, Value)> = Vec::new();
        let bp = "required_status_checks:\n  - oya-vcs-admission\n  - oya-vcs-provider-execution\n";
        let pr = "oya-vcs-admission ... oya-vcs-admission-gate-app ... cargo run -p oya-dev-cli -- supply-chain install-trivy";
        let sc = "oya-vcs-provider-execution ... oya-vcs-provider-execution-gate-app";
        let inputs = empty_inputs(
            &root, &seq, &msr, &vcs, &pe, &pex, bp, pr, sc, &pkgs, "", &ms,
        );
        let report = validate_admission(&inputs);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "PROVIDER_EVIDENCE_REGRESSED_TO_FIXTURE")
        );
    }

    #[test]
    fn missing_workspace_package_is_flagged() {
        let root = passing_root();
        let seq = passing_seq();
        let msr = passing_msr();
        let vcs = passing_vcs();
        let pe = passing_provider_evidence();
        let pex = passing_proof();
        let pkgs: Vec<String> = vec!["oya-dev-cli".into()];
        let ms: Vec<(String, Value)> = Vec::new();
        let bp = "required_status_checks:\n  - oya-vcs-admission\n  - oya-vcs-provider-execution\n";
        let pr = "oya-vcs-admission ... oya-vcs-admission-gate-app ... cargo run -p oya-dev-cli -- supply-chain install-trivy";
        let sc = "oya-vcs-provider-execution ... oya-vcs-provider-execution-gate-app";
        let inputs = empty_inputs(
            &root, &seq, &msr, &vcs, &pe, &pex, bp, pr, sc, &pkgs, "", &ms,
        );
        let report = validate_admission(&inputs);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "WORKSPACE_MISSING_OYA_VCS_PACKAGES")
        );
    }

    #[test]
    fn audit_chain_missing_change_id_is_flagged() {
        let root = passing_root();
        let seq = passing_seq();
        let msr = passing_msr();
        let vcs = passing_vcs();
        let pe = passing_provider_evidence();
        let pex = passing_proof();
        let pkgs = passing_packages();
        let ms = vec![(
            "evidence/multispectrum/x.json".to_string(),
            json!({"change_id": "CID-MISSING"}),
        )];
        let chain = "OTHER\n".to_string();
        let bp = "required_status_checks:\n  - oya-vcs-admission\n  - oya-vcs-provider-execution\n";
        let pr = "oya-vcs-admission ... oya-vcs-admission-gate-app ... cargo run -p oya-dev-cli -- supply-chain install-trivy";
        let sc = "oya-vcs-provider-execution ... oya-vcs-provider-execution-gate-app";
        let inputs = empty_inputs(
            &root, &seq, &msr, &vcs, &pe, &pex, bp, pr, sc, &pkgs, &chain, &ms,
        );
        let report = validate_admission(&inputs);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "AUDIT_CHAIN_MISSING_CHANGE_ID")
        );
    }
}
