use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::usage;

const OYA_CI_REQUIRED_CONTEXT: &str = "oya-ci-required";
const AGENT_PR_REVIEW_CONTEXT: &str = "oya-pr-review";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HyperscalerMaturityClaimsValidateArgs {
    gates_path: PathBuf,
    workflow_studio_path: PathBuf,
    workflow_path: PathBuf,
    workspace_hygiene_path: PathBuf,
    branch_protection_path: PathBuf,
    pr_review_workflow_path: PathBuf,
    ci_fix_loop_workflow_path: PathBuf,
    gitops_vcs_path: PathBuf,
    merge_queue_path: PathBuf,
    iterative_fix_loop_path: PathBuf,
    ci_fix_loop_retry_budget_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HyperscalerMaturityClaimsReport {
    pub gate_count: usize,
    pub competitor_count: usize,
    pub claim_status: String,
}

pub(crate) fn parse_hyperscaler_maturity_claims_validate_args(
    args: Vec<String>,
) -> Result<HyperscalerMaturityClaimsValidateArgs, String> {
    let mut parsed = HyperscalerMaturityClaimsValidateArgs {
        gates_path: PathBuf::from("specs/hyperscaler-gates.json"),
        workflow_studio_path: PathBuf::from("specs/microservices/workflow-studio.json"),
        workflow_path: PathBuf::from("specs/microservices/workflow.json"),
        workspace_hygiene_path: PathBuf::from("specs/workspace-hygiene.json"),
        branch_protection_path: PathBuf::from(".github/branch-protection.yaml"),
        // Current authority is the single cloud-ci/oya-ci required context. The
        // GitHub Actions workflow is a bridge producer for that context until the
        // Kubernetes-native oya-ci-controller owns status production; the retired
        // oya-dev-cli/Jenkins/local CLI surfaces must not be merge authority.
        pr_review_workflow_path: PathBuf::from(".github/workflows/oya-ci-required.yml"),
        ci_fix_loop_workflow_path: PathBuf::from(".github/workflows/oya-ci-required.yml"),
        gitops_vcs_path: PathBuf::from("specs/gitops-vcs-replacement.json"),
        merge_queue_path: PathBuf::from("specs/merge-queue-parked-pr.json"),
        iterative_fix_loop_path: PathBuf::from("specs/iterative-fix-loop.json"),
        ci_fix_loop_retry_budget_path: PathBuf::from("registry/ci-fix-loop-retry-budget.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--gates" => parsed.gates_path = PathBuf::from(path),
            "--workflow-studio" => parsed.workflow_studio_path = PathBuf::from(path),
            "--workflow" => parsed.workflow_path = PathBuf::from(path),
            "--workspace-hygiene" => parsed.workspace_hygiene_path = PathBuf::from(path),
            "--branch-protection" => parsed.branch_protection_path = PathBuf::from(path),
            "--pr-review-workflow" => parsed.pr_review_workflow_path = PathBuf::from(path),
            "--ci-fix-loop-workflow" => parsed.ci_fix_loop_workflow_path = PathBuf::from(path),
            "--gitops-vcs" => parsed.gitops_vcs_path = PathBuf::from(path),
            "--merge-queue" => parsed.merge_queue_path = PathBuf::from(path),
            "--iterative-fix-loop" => parsed.iterative_fix_loop_path = PathBuf::from(path),
            "--ci-fix-loop-retry-budget" => {
                parsed.ci_fix_loop_retry_budget_path = PathBuf::from(path)
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_hyperscaler_maturity_claims_gate(
    args: HyperscalerMaturityClaimsValidateArgs,
) -> Result<HyperscalerMaturityClaimsReport, String> {
    let gates = read_json(&args.gates_path, "hyperscaler gates")?;
    let workflow_studio = read_json(&args.workflow_studio_path, "workflow-studio product spec")?;
    let workflow = read_json(&args.workflow_path, "workflow product spec")?;
    let gitops_vcs = read_json(&args.gitops_vcs_path, "gitops-vcs replacement spec")?;
    let merge_queue = read_json(&args.merge_queue_path, "merge queue spec")?;
    let iterative_fix_loop = read_json(&args.iterative_fix_loop_path, "iterative fix-loop spec")?;
    let ci_fix_loop_retry_budget = read_json(
        &args.ci_fix_loop_retry_budget_path,
        "CI fix-loop retry-budget registry",
    )?;
    crate::validate_workspace_hygiene_gate(crate::WorkspaceHygieneValidateArgs {
        policy_path: args.workspace_hygiene_path,
        scan: false,
        strict: false,
        clean_build_artifacts: false,
        clean_temp_artifacts: false,
    })
    .map_err(|error| format!("workspace hygiene maturity prerequisite invalid: {error}"))?;
    let branch_protection = fs::read_to_string(&args.branch_protection_path).map_err(|error| {
        format!(
            "branch protection unreadable {}: {error}",
            args.branch_protection_path.display()
        )
    })?;
    let pr_review_workflow =
        fs::read_to_string(&args.pr_review_workflow_path).map_err(|error| {
            format!(
                "PR review workflow unreadable {}: {error}",
                args.pr_review_workflow_path.display()
            )
        })?;
    let ci_fix_loop_workflow =
        fs::read_to_string(&args.ci_fix_loop_workflow_path).map_err(|error| {
            format!(
                "CI fix-loop workflow unreadable {}: {error}",
                args.ci_fix_loop_workflow_path.display()
            )
        })?;

    let (gate_count, claim_status) = validate_hyperscaler_gates(&gates)?;
    let competitor_count = validate_workflow_studio(&workflow_studio)?;
    validate_workflow_engine(&workflow)?;
    validate_pr_review_pipeline(&branch_protection, &pr_review_workflow)?;
    validate_pipeline_closure(
        &branch_protection,
        &pr_review_workflow,
        &ci_fix_loop_workflow,
        &gitops_vcs,
        &merge_queue,
        &iterative_fix_loop,
        &ci_fix_loop_retry_budget,
    )?;

    Ok(HyperscalerMaturityClaimsReport {
        gate_count,
        competitor_count,
        claim_status,
    })
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{label} unreadable: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("{label} invalid JSON: {error}"))
}

fn validate_hyperscaler_gates(gates: &Value) -> Result<(usize, String), String> {
    let root = object(gates, "hyperscaler gates root")?;
    let claim_rule = object_field(root, "hyperscaler_mature_claim_rule")?;
    let exact_claim = string_field(claim_rule, "exact_claim")?;
    if exact_claim != HYPERSCALER_MATURE_CLAIM {
        return Err(format!(
            "hyperscaler claim rule exact_claim must be {HYPERSCALER_MATURE_CLAIM:?}, got {exact_claim:?}"
        ));
    }

    let claim_status = string_field(claim_rule, "claim_status")?;
    match claim_status {
        "blocked_until_required_evidence_is_green" => {
            require_non_empty_array(claim_rule, "current_blockers")?;
            require_string_array_contains(
                claim_rule,
                "forbidden_claim_patterns",
                "hyperscaler mature",
            )?;
        }
        "allowed" => {
            let blockers = array_field(claim_rule, "current_blockers")?;
            if !blockers.is_empty() {
                return Err(
                    "hyperscaler claim rule cannot be allowed while current_blockers is non-empty"
                        .into(),
                );
            }
            require_non_empty_array(claim_rule, "allowed_evidence_refs")?;
        }
        _ => {
            return Err(format!(
                "hyperscaler claim rule has unknown claim_status {claim_status:?}"
            ));
        }
    }

    for surface in REQUIRED_PRIORITY_SURFACES {
        require_string_array_contains(claim_rule, "required_priority_surfaces", surface)?;
    }

    let gate_rows = array_field(root, "gates")?;
    let mut gate_ids = BTreeSet::new();
    for (index, row) in gate_rows.iter().enumerate() {
        let gate = object(row, &format!("gates[{index}]"))?;
        let id = string_field(gate, "id")?;
        if !gate_ids.insert(id.to_owned()) {
            return Err(format!("duplicate hyperscaler gate id {id:?}"));
        }
        require_non_empty_string(gate, "name")?;
        require_non_empty_array(gate, "requires")?;
        require_non_empty_array(gate, "evidence_classes")?;
    }

    if gate_ids.contains("HG-GRIT") {
        return Err("retired HG-GRIT gate is forbidden; use HG-VCS per ADR-0116".into());
    }
    for required_id in REQUIRED_GATE_IDS {
        if !gate_ids.contains(*required_id) {
            return Err(format!("missing required hyperscaler gate {required_id}"));
        }
    }

    Ok((gate_rows.len(), claim_status.to_owned()))
}

fn validate_workflow_studio(workflow_studio: &Value) -> Result<usize, String> {
    let root = object(workflow_studio, "workflow-studio root")?;
    let identity = object_field(root, "identity")?;
    let product_id = string_field(identity, "product_id")?;
    if product_id != "workflow-studio" {
        return Err(format!(
            "workflow-studio spec identity.product_id must be workflow-studio, got {product_id:?}"
        ));
    }

    let policy = object_field(root, "competitive_claim_policy")?;
    let status = string_field(policy, "status")?;
    if status != "binding" {
        return Err(format!(
            "workflow-studio competitive_claim_policy.status must be binding, got {status:?}"
        ));
    }
    require_non_empty_array(policy, "forbidden_without_benchmark_evidence")?;
    for required_field in COMPETITOR_ROW_REQUIRED_FIELDS {
        require_string_array_contains(policy, "required_per_competitor_row", required_field)?;
    }

    validate_workflow_studio_ux(root)?;

    let rows = array_field(root, "competitive")?;
    let mut competitors = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let competitor = object(row, &format!("workflow-studio competitive[{index}]"))?;
        if competitor.contains_key("we_beat_on") || competitor.contains_key("measurable") {
            return Err(format!(
                "workflow-studio competitor row {index} uses retired unsupported benchmark fields"
            ));
        }
        let name = string_field(competitor, "competitor")?;
        competitors.insert(name.to_owned());
        require_non_empty_array(competitor, "source_evidence_refs")?;
        require_non_empty_array(competitor, "observed_strengths")?;
        require_non_empty_array(competitor, "observed_weaknesses_or_gaps")?;
        require_non_empty_array(competitor, "adopt_from_them")?;
        require_non_empty_array(competitor, "improve_beyond_them")?;
        require_non_empty_string(competitor, "claim_boundary")?;
    }
    if rows.len() < MIN_COMPETITOR_ROWS {
        return Err(format!(
            "workflow-studio competitive matrix must cover at least {MIN_COMPETITOR_ROWS} competitors"
        ));
    }

    for required_competitor in REQUIRED_WORKFLOW_STUDIO_COMPETITORS {
        if !competitors.contains(*required_competitor) {
            return Err(format!(
                "workflow-studio competitive matrix missing {required_competitor}"
            ));
        }
    }

    Ok(rows.len())
}

fn validate_workflow_studio_ux(root: &serde_json::Map<String, Value>) -> Result<(), String> {
    let ux = object_field(root, "user_experience")?;
    require_non_empty_string(ux, "accessibility_coverage")?;
    require_non_empty_string(ux, "offline_behavior")?;
    require_non_empty_string(ux, "loading_state_coverage")?;
    require_non_empty_array(ux, "journey_critical_paths")?;
    let keyboard_coverage = ux
        .get("keyboard_navigation_coverage_pct")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            "workflow-studio user_experience.keyboard_navigation_coverage_pct must be a number"
                .to_string()
        })?;
    if keyboard_coverage < 100 {
        return Err(
            "workflow-studio keyboard_navigation_coverage_pct must be 100 for maturity claim governance"
                .into(),
        );
    }
    let error_states = object_field(ux, "error_state_coverage")?;
    for state in REQUIRED_ERROR_STATES {
        require_non_empty_string(error_states, state)?;
    }
    Ok(())
}

fn validate_workflow_engine(workflow: &Value) -> Result<(), String> {
    let root = object(workflow, "workflow root")?;
    require_non_empty_string(root, "competitive_research_ref")?;
    require_non_empty_array(root, "competitive")?;
    let anti_patterns = array_field(root, "anti_patterns")?;
    let has_unsourced_benchmark_blocker = anti_patterns.iter().any(|item| {
        object(item, "workflow anti_patterns[]")
            .ok()
            .and_then(|entry| entry.get("name"))
            .and_then(Value::as_str)
            == Some("unsourced_competitor_benchmark_claim")
    });
    if !has_unsourced_benchmark_blocker {
        return Err(
            "workflow spec must forbid unsourced_competitor_benchmark_claim anti-patterns".into(),
        );
    }
    Ok(())
}

fn validate_pr_review_pipeline(
    branch_protection: &str,
    pr_review_workflow: &str,
) -> Result<(), String> {
    let protection = parse_dev_branch_protection(branch_protection)?;
    validate_oya_ci_required_branch_protection(&protection)?;
    validate_oya_ci_required_workflow(pr_review_workflow)?;
    Ok(())
}

fn validate_oya_ci_required_branch_protection(
    protection: &BranchProtectionContract,
) -> Result<(), String> {
    if !protection
        .required_status_checks
        .contains(OYA_CI_REQUIRED_CONTEXT)
    {
        return Err("dev branch protection must require the single oya-ci-required context".into());
    }
    for retired_context in [
        AGENT_PR_REVIEW_CONTEXT,
        "cargo-fmt",
        "cargo-check",
        "cargo-clippy",
        "cargo-nextest",
        "oya-governance-protection-context-match",
    ] {
        if protection.required_status_checks.contains(retired_context) {
            return Err(format!(
                "dev branch protection must not require retired/local context {retired_context}"
            ));
        }
    }
    if protection.commented_out_oya_pr_review {
        return Err(
            "branch protection must not preserve commented-out oya-pr-review authority".into(),
        );
    }
    if protection.required_approving_reviews != Some(0) {
        return Err(
            "human GitHub approving reviews must not be the pipeline merge authority".into(),
        );
    }
    if !protection.agent_review_authority_comment {
        return Err("branch protection must document automated agent review authority".into());
    }
    Ok(())
}

fn validate_oya_ci_required_workflow(workflow: &str) -> Result<(), String> {
    // This contract asserts the workflow as ADR-0716 leaves it: Cargo is the merge path and
    // Buck2 is local hermeticity plus a weekly smoke. The previous expectations still described
    // the pre-ADR-0716 shape — a `buck2 test //ci/...` merge lane, a gate matrix fan-in, a
    // Buck2 fan-in leg, and `oya-ci-required: GREEN`/`RED` terminal markers — none of which the
    // workflow has emitted since that ADR landed. It asserted a workflow that no longer exists.
    for expected in [
        "name: oya-ci-required",
        "fail-fast: false",
        "persist-credentials: false",
        "fetch-depth: 0",
        // Digest-pinned tool acquisition is the integrity anchor (ADR-0556).
        "Install pinned buck2 (digest-verified)",
        // Generated faces are materialized before the gate crates consume them.
        "Materialize generated faces",
        // The Cargo merge path itself (ADR-0716), locked so the lockfile is authoritative.
        "cargo test --locked --workspace",
        "generated-output-diff-policy",
        // The forever public status string is dual-emitted beside the legacy protected context
        // until the branch-protection flip; losing it would silently strand the cutover.
        "merge-admission-required",
    ] {
        require_contains(workflow, expected, "oya-ci-required workflow contract")?;
    }
    require_contains(
        workflow,
        "needs: [lint, test",
        "oya-ci-required fan-in must depend on the lint and test lanes",
    )?;
    for forbidden in [
        "cargo run -q -p oya-dev-cli",
        "cargo run -p oya-dev-cli",
        "./bin/oya",
        "oya gate",
        "oya verify",
        "infra/ci/jenkins/pipeline-closure-contract.md",
        "Jenkinsfile",
    ] {
        require_absent(
            workflow,
            forbidden,
            "oya-ci-required workflow must not use retired CLI/Jenkins authority",
        )?;
    }
    Ok(())
}

fn validate_pipeline_closure(
    branch_protection: &str,
    pr_review_workflow: &str,
    ci_fix_loop_workflow: &str,
    gitops_vcs: &Value,
    merge_queue: &Value,
    iterative_fix_loop: &Value,
    ci_fix_loop_retry_budget: &Value,
) -> Result<(), String> {
    let protection = parse_dev_branch_protection(branch_protection)?;
    if protection.require_linear_history != Some(true) {
        return Err("branch protection must require linear history to avoid unsafe merges".into());
    }
    if protection.disallow_force_push != Some(true) {
        return Err("branch protection must disallow force-push on protected branches".into());
    }
    validate_oya_ci_required_branch_protection(&protection)?;
    validate_oya_ci_required_workflow(pr_review_workflow)?;
    validate_oya_ci_required_workflow(ci_fix_loop_workflow)?;
    validate_merge_safety_specs(
        gitops_vcs,
        merge_queue,
        iterative_fix_loop,
        ci_fix_loop_retry_budget,
    )?;
    Ok(())
}

fn validate_merge_safety_specs(
    gitops_vcs: &Value,
    merge_queue: &Value,
    iterative_fix_loop: &Value,
    ci_fix_loop_retry_budget: &Value,
) -> Result<(), String> {
    validate_oya_vcs_merge_safety(gitops_vcs)?;
    validate_merge_queue_safety(merge_queue)?;
    validate_iterative_fix_loop_automation(iterative_fix_loop)?;
    validate_ci_fix_loop_retry_budget(ci_fix_loop_retry_budget)?;
    Ok(())
}

fn validate_oya_vcs_merge_safety(gitops_vcs: &Value) -> Result<(), String> {
    let root = object(gitops_vcs, "Oya VCS replacement spec root")?;
    let architecture = object_field(root, "architecture")?;
    let public_interface = object_field(architecture, "public_interface")?;
    require_string_array_contains(public_interface, "agent_forbidden_commands", "git")?;
    require_string_array_contains(public_interface, "agent_forbidden_commands", "gh")?;
    require_string_field_contains(public_interface, "compatibility_note", "ADR-0116")?;

    let rebase_queue = object_field(root, "rebase_and_merge_queue")?;
    require_bool_field(rebase_queue, "controller_owned", true)?;
    for forbidden in [
        "git rebase",
        "git merge",
        "git push",
        "gh pr merge",
        "manual merge queue manipulation",
    ] {
        require_string_array_contains(rebase_queue, "agent_forbidden", forbidden)?;
    }
    for operation in [
        "controller-owned rebase or equivalent patch replay in isolated workspace",
        "rerun affected tests after rebase",
        "merge queue enrollment with service identity",
        "merge queue status watch and automatic retry/backoff",
    ] {
        require_string_array_contains(rebase_queue, "operations", operation)?;
    }
    require_string_field_contains(
        rebase_queue,
        "merge_queue_policy",
        "Agents can observe queue state but cannot reorder, force, or bypass it.",
    )?;

    let pipeline = object_field(root, "gitops_pipeline_integration")?;
    for operation in [
        "controller-owned rebase/patch replay",
        "merge queue enrollment",
        "merge queue enrollment and status watch",
        "remote branch/ref mutation",
        "review/fix task creation",
    ] {
        require_string_array_contains(pipeline, "controller_owned_operations", operation)?;
    }
    Ok(())
}

fn validate_merge_queue_safety(merge_queue: &Value) -> Result<(), String> {
    let root = object(merge_queue, "merge queue spec root")?;
    let invariants = object_field(root, "invariants")?;
    require_string_field_contains(
        invariants,
        "projected_merge_state_validation",
        "projected-merge-state",
    )?;
    require_string_field_contains(
        invariants,
        "ordered_projection_forecast_required",
        "admission_position order",
    )?;
    require_string_field_contains(invariants, "forecast_before_conflict", "conflict_paths")?;
    require_string_field_contains(invariants, "safe_auto_merge", "Auto-merge")?;
    require_string_field_contains(invariants, "convergence_proof", "TickEntry")?;
    require_string_field_contains(
        invariants,
        "no_agent_owned_rebase",
        "agents NEVER own rebase/merge operations",
    )?;

    let forecast = object_field(root, "ordered_projection_forecast")?;
    for trigger in [
        "queue-enrollment",
        "post-merge-head-advance",
        "fix-loop-converged",
        "scheduler-tick",
    ] {
        require_string_array_contains(forecast, "triggered_by", trigger)?;
    }
    require_string_field_contains(forecast, "simulation_engine", "git merge-tree --write-tree")?;
    for field in [
        "conflict_paths",
        "evidence_sha_resolution_state",
        "auto_merge_state",
        "next_action",
    ] {
        require_string_array_contains(forecast, "per_entry_required_fields", field)?;
    }

    let state_machine = object_field(root, "state_machine")?;
    require_transition_with_trigger(
        state_machine,
        "parked",
        "fix-loop-converged",
        "revalidating",
    )?;
    require_transition_with_trigger(
        state_machine,
        "revalidating",
        "speculative-rebase-conflict",
        "parked",
    )?;
    require_transition_with_trigger(state_machine, "revalidating", "budget-exhausted", "evicted")?;
    require_transition_note_contains(
        state_machine,
        "revalidating",
        "budget-exhausted",
        "agent-remediation-required",
    )?;
    require_transition_note_contains(
        state_machine,
        "revalidating",
        "budget-exhausted",
        "fix-loop-exhausted",
    )?;
    require_json_absent_text(
        merge_queue,
        "human-escalation",
        "merge queue must not encode human escalation as an automated pipeline dependency",
    )?;
    Ok(())
}

fn validate_iterative_fix_loop_automation(iterative_fix_loop: &Value) -> Result<(), String> {
    let root = object(iterative_fix_loop, "iterative fix-loop spec root")?;
    let loop_state = object_field(root, "loop_state_machine")?;
    require_string_array_contains(loop_state, "states", "automated-remediation-required")?;
    require_string_array_contains(
        loop_state,
        "terminal_states",
        "automated-remediation-required",
    )?;

    let verification = object_field(root, "verification_commands_per_facet")?;
    for (facet, command) in [
        ("F1_linus", "automated peer-agent code review"),
        ("F2_hyperscaler", "automated scale review"),
        ("F5_quality", "automated code review"),
        ("F7_security", "automated OWASP class review"),
    ] {
        require_string_array_contains(verification, facet, command)?;
    }
    require_json_absent_text(
        iterative_fix_loop,
        "human-escalation",
        "iterative fix-loop must not end in human escalation",
    )?;
    require_json_absent_text(
        iterative_fix_loop,
        "manual code review",
        "iterative fix-loop review should be agent-run",
    )?;
    require_json_absent_text(
        iterative_fix_loop,
        "manual scale review",
        "iterative fix-loop scale review should be agent-run",
    )?;
    Ok(())
}

fn validate_ci_fix_loop_retry_budget(ci_fix_loop_retry_budget: &Value) -> Result<(), String> {
    let root = object(
        ci_fix_loop_retry_budget,
        "CI fix-loop retry budget registry root",
    )?;
    let meta = object_field(root, "_meta")?;
    let schema = object_field(meta, "schema")?;
    require_string_array_contains(schema, "shared_across_sources", "ci-failure")?;
    require_string_array_contains(schema, "shared_across_sources", "pr-review-fix-requested")?;
    require_string_field_contains(schema, "escalation_action", "agent-remediation-required")?;
    require_string_field_contains(schema, "escalation_action", "fix-loop-exhausted")?;
    require_json_absent_text(
        ci_fix_loop_retry_budget,
        "human-escalation",
        "CI fix-loop retry budget must not route to human escalation",
    )?;
    Ok(())
}

#[derive(Debug, Default)]
struct BranchProtectionContract {
    required_status_checks: BTreeSet<String>,
    require_linear_history: Option<bool>,
    disallow_force_push: Option<bool>,
    required_approving_reviews: Option<u64>,
    agent_review_authority_comment: bool,
    commented_out_oya_pr_review: bool,
}

fn parse_dev_branch_protection(contents: &str) -> Result<BranchProtectionContract, String> {
    let mut contract = BranchProtectionContract::default();
    let mut in_dev = false;
    let mut in_required_checks = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if !in_dev {
            if line.starts_with("  dev:") {
                in_dev = true;
            }
            continue;
        }
        if line.starts_with("  ") && !line.starts_with("    ") && trimmed.ends_with(':') {
            break;
        }

        if trimmed.contains("reviews are agent-run and fully automated")
            || trimmed.contains("Reviews are agent-run and fully automated")
        {
            contract.agent_review_authority_comment = true;
        }
        if trimmed == "# - oya-pr-review" {
            contract.commented_out_oya_pr_review = true;
        }

        if trimmed == "required_status_checks:" {
            in_required_checks = true;
            continue;
        }
        if in_required_checks {
            if let Some(check) = trimmed.strip_prefix("- ") {
                contract.required_status_checks.insert(check.to_owned());
                continue;
            }
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_required_checks = false;
            }
        }

        if let Some(value) = parse_yaml_bool(trimmed, "require_linear_history") {
            contract.require_linear_history = Some(value);
        }
        if let Some(value) = parse_yaml_bool(trimmed, "disallow_force_push") {
            contract.disallow_force_push = Some(value);
        }
        if let Some(value) = parse_yaml_u64(trimmed, "required_approving_reviews")? {
            contract.required_approving_reviews = Some(value);
        }
    }

    if !in_dev {
        return Err("branch protection missing dev branch contract".into());
    }
    Ok(contract)
}

fn parse_yaml_bool(line: &str, key: &str) -> Option<bool> {
    let value = line.strip_prefix(&format!("{key}: "))?;
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_yaml_u64(line: &str, key: &str) -> Result<Option<u64>, String> {
    let Some(value) = line.strip_prefix(&format!("{key}: ")) else {
        return Ok(None);
    };
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|error| format!("{key} must be an unsigned integer: {error}"))
}

#[derive(Debug)]
struct WorkflowContract<'a> {
    name: Option<String>,
    job_present: bool,
    job_display_name: Option<String>,
    contents: &'a str,
}

impl WorkflowContract<'_> {
    fn require_command(&self, command: &str, context: &str) -> Result<(), String> {
        require_contains(self.contents, command, context)
    }

    fn require_command_order(
        &self,
        first: &str,
        second: &str,
        context: &str,
    ) -> Result<(), String> {
        let first_index = self
            .contents
            .find(first)
            .ok_or_else(|| format!("{context}: missing first command {first:?}"))?;
        let second_index = self
            .contents
            .find(second)
            .ok_or_else(|| format!("{context}: missing second command {second:?}"))?;
        if first_index > second_index {
            Err(context.into())
        } else {
            Ok(())
        }
    }
}

fn parse_workflow_contract<'a>(
    contents: &'a str,
    job_id: &str,
) -> Result<WorkflowContract<'a>, String> {
    let mut name = None;
    let mut job_present = false;
    let mut job_display_name = None;
    let mut in_jobs = false;
    let mut in_target_job = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(value) = line.strip_prefix("name: ") {
            name = Some(value.trim().to_owned());
        }
        if trimmed == "jobs:" {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        if line.starts_with("  ") && !line.starts_with("    ") && trimmed.ends_with(':') {
            in_target_job = trimmed == format!("{job_id}:");
            job_present |= in_target_job;
            continue;
        }
        if in_target_job && line.starts_with("    name: ") {
            job_display_name = Some(trimmed.trim_start_matches("name: ").to_owned());
        }
    }

    Ok(WorkflowContract {
        name,
        job_present,
        job_display_name,
        contents,
    })
}

fn object<'a>(value: &'a Value, path: &str) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path} must be a JSON object"))
}

fn object_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing object field {field}"))?
        .as_object()
        .ok_or_else(|| format!("{field} must be a JSON object"))
}

fn array_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing array field {field}"))?
        .as_array()
        .ok_or_else(|| format!("{field} must be a JSON array"))
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing string field {field}"))?
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))
}

fn require_non_empty_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), String> {
    let value = string_field(object, field)?;
    if value.trim().is_empty() {
        Err(format!("{field} must be non-empty"))
    } else {
        Ok(())
    }
}

fn require_non_empty_array(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), String> {
    let values = array_field(object, field)?;
    if values.is_empty() {
        Err(format!("{field} must be non-empty"))
    } else {
        Ok(())
    }
}

fn require_string_array_contains(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    let values = array_field(object, field)?;
    let contains = values.iter().any(|value| value.as_str() == Some(expected));
    if contains {
        Ok(())
    } else {
        Err(format!("{field} must include {expected:?}"))
    }
}

fn require_bool_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: bool,
) -> Result<(), String> {
    let value = object
        .get(field)
        .ok_or_else(|| format!("missing bool field {field}"))?
        .as_bool()
        .ok_or_else(|| format!("{field} must be a boolean"))?;
    if value == expected {
        Ok(())
    } else {
        Err(format!("{field} must be {expected}, got {value}"))
    }
}

fn require_string_field_contains(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    let value = string_field(object, field)?;
    if value.contains(expected) {
        Ok(())
    } else {
        Err(format!("{field} must contain {expected:?}"))
    }
}

fn require_transition_with_trigger(
    state_machine: &serde_json::Map<String, Value>,
    from: &str,
    trigger_fragment: &str,
    to: &str,
) -> Result<(), String> {
    let transitions = array_field(state_machine, "transitions")?;
    let found = transitions.iter().any(|transition| {
        object(transition, "state_machine.transitions[]")
            .ok()
            .is_some_and(|entry| {
                entry.get("from").and_then(Value::as_str) == Some(from)
                    && entry
                        .get("trigger")
                        .and_then(Value::as_str)
                        .is_some_and(|trigger| trigger.contains(trigger_fragment))
                    && entry.get("to").and_then(Value::as_str) == Some(to)
            })
    });
    if found {
        Ok(())
    } else {
        Err(format!(
            "state_machine.transitions must include {from:?} --{trigger_fragment:?}--> {to:?}"
        ))
    }
}

fn require_transition_note_contains(
    state_machine: &serde_json::Map<String, Value>,
    from: &str,
    trigger_fragment: &str,
    expected_note_fragment: &str,
) -> Result<(), String> {
    let transitions = array_field(state_machine, "transitions")?;
    let found = transitions.iter().any(|transition| {
        object(transition, "state_machine.transitions[]")
            .ok()
            .is_some_and(|entry| {
                entry.get("from").and_then(Value::as_str) == Some(from)
                    && entry
                        .get("trigger")
                        .and_then(Value::as_str)
                        .is_some_and(|trigger| trigger.contains(trigger_fragment))
                    && entry
                        .get("note")
                        .and_then(Value::as_str)
                        .is_some_and(|note| note.contains(expected_note_fragment))
            })
    });
    if found {
        Ok(())
    } else {
        Err(format!(
            "state_machine.transitions note for {from:?}/{trigger_fragment:?} must contain {expected_note_fragment:?}"
        ))
    }
}

fn require_json_absent_text(value: &Value, forbidden: &str, context: &str) -> Result<(), String> {
    if json_contains_text(value, forbidden) {
        Err(format!("{context}: forbidden {forbidden:?}"))
    } else {
        Ok(())
    }
}

fn json_contains_text(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text.contains(needle),
        Value::Array(values) => values.iter().any(|value| json_contains_text(value, needle)),
        Value::Object(object) => object
            .values()
            .any(|value| json_contains_text(value, needle)),
        _ => false,
    }
}

fn require_contains(contents: &str, expected: &str, context: &str) -> Result<(), String> {
    if contents.contains(expected) {
        Ok(())
    } else {
        Err(format!("{context}: missing {expected:?}"))
    }
}

fn require_absent(contents: &str, forbidden: &str, context: &str) -> Result<(), String> {
    if contents.contains(forbidden) {
        Err(format!("{context}: forbidden {forbidden:?}"))
    } else {
        Ok(())
    }
}

const HYPERSCALER_MATURE_CLAIM: &str = "we are hyperscaler mature";

const REQUIRED_PRIORITY_SURFACES: &[&str] = &[
    "workflow-studio",
    "workflow-engine",
    "integration-connector-catalog",
    "governance-pipeline",
    "oya-vcs-admission",
    "ci-cd-toolchain",
    "development-cycle",
    "workspace-hygiene",
    "guardrails",
    "safety",
    "ease-of-use",
];

const REQUIRED_GATE_IDS: &[&str] = &[
    "HG-ARCH",
    "HG-CONTRACT",
    "HG-SECURITY",
    "HG-SAFETY",
    "HG-GUARDRAILS",
    "HG-RELIABILITY",
    "HG-OBS",
    "HG-TEST",
    "HG-SUPPLY",
    "HG-OPS",
    "HG-DOCS",
    "HG-PLAN",
    "HG-PIPELINE",
    "HG-TOOLCHAIN",
    "HG-CICD",
    "HG-DEV-CYCLE",
    "HG-HYGIENE",
    "HG-PRODUCT-DEPTH",
    "HG-UX",
    "HG-EASE",
    "HG-COMPETITIVE",
    "HG-VCS",
];

const COMPETITOR_ROW_REQUIRED_FIELDS: &[&str] = &[
    "source_evidence_refs",
    "observed_strengths",
    "observed_weaknesses_or_gaps",
    "adopt_from_them",
    "improve_beyond_them",
    "claim_boundary",
];

const MIN_COMPETITOR_ROWS: usize = 8;

const REQUIRED_WORKFLOW_STUDIO_COMPETITORS: &[&str] = &[
    "n8n",
    "Temporal",
    "Camunda Web Modeler",
    "Argo Workflows",
    "GitHub Actions",
    "Workato",
    "Make",
    "Zapier",
    "Microsoft Power Automate",
    "Linear",
];

const REQUIRED_ERROR_STATES: &[&str] = &[
    "invalid_spec",
    "collaboration_conflict",
    "network_partition",
    "policy_denied",
];
