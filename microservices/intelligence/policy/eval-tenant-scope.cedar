// foundry-eval µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: enforce per-tenant read scope on eval-runs, parity reports, replay divergence
//          reports, and capability publish-gate verdicts served via eval-runner-rest,
//          parity-analyzer-rest, and SDK clients.
// Owner:   ops-security + axis-foundry
// Audit:   every deny emits oya_foundry_eval_tenant_unauthorized_read_attempt_total
// References: tenant-isolation.md §"Invariant TI-07"; threat-model.md T-I-01
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every read is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's eval verdicts + parity history
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_eval_run_history",
    Action::"read_eval_run_latest",
    Action::"read_parity_report",
    Action::"read_replay_divergence",
    Action::"read_publish_gate_verdict",
    Action::"read_dashboards"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — capability owner reads + writes own capability's eval-sets
// =============================================================================

permit (
  principal in CapabilityOwner::?p,
  action in [
    Action::"read_eval_set",
    Action::"author_eval_set",
    Action::"trigger_ad_hoc_run",
    Action::"read_eval_run_latest"
  ],
  resource in CapabilityData::?r
)
when {
  principal has capability_id_allowlist &&
  resource has capability_id &&
  resource.capability_id in principal.capability_id_allowlist
};

// =============================================================================
// PERMIT 3 — eval-runner-worker SPIFFE identity may write evidence + verdicts
// =============================================================================

permit (
  principal == ServiceIdentity::"spiffe://oyatie/foundry-eval/eval-runner-worker",
  action in [
    Action::"write_eval_run_started",
    Action::"write_eval_run_completed",
    Action::"write_eval_evidence",
    Action::"write_parity_verdict",
    Action::"write_replay_divergence",
    Action::"write_in_house_cutover_eligible"
  ],
  resource
);

// =============================================================================
// PERMIT 4 — foundry-runtime may read publish-gate verdict
// =============================================================================

permit (
  principal == ServiceIdentity::"spiffe://oyatie/foundry-runtime/admission",
  action == Action::"read_publish_gate_verdict",
  resource
);

// =============================================================================
// PERMIT 5 — foundry-providers may read parity verdict + in-house-cutover
// =============================================================================

permit (
  principal == ServiceIdentity::"spiffe://oyatie/foundry-providers/router",
  action in [
    Action::"read_parity_report",
    Action::"read_in_house_cutover_eligible"
  ],
  resource
);

// =============================================================================
// PERMIT 6 — external auditor JIT scope (time-bounded; per-tenant subset)
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_eval_run_history",
    Action::"read_parity_report",
    Action::"read_replay_divergence",
    Action::"read_eu_ai_act_section_15_evidence",
    Action::"read_eu_ai_act_section_17_log"
  ],
  resource in TenantData::?r
)
when {
  principal has audit_window_starts_at &&
  principal has audit_window_ends_at &&
  principal has tenant_id_allowlist &&
  resource has tenant_id &&
  resource.tenant_id in principal.tenant_id_allowlist &&
  context.now >= principal.audit_window_starts_at &&
  context.now <= principal.audit_window_ends_at
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth per threat-model T-I-01)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_eval_run_history",
    Action::"read_eval_run_latest",
    Action::"read_parity_report",
    Action::"read_replay_divergence"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — never expose SECRET-class data via tenant reads
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — never expose live PHI (synthetic-PHI-only policy enforcement)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "PHI" &&
  resource has phi_origin &&
  resource.phi_origin != "synthetic"
};
