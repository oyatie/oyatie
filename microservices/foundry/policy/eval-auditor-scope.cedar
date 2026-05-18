// foundry-eval µservice — auditor-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce external auditor (SOC 2 / ISO 27001 / EU AI Act CAB / HIPAA OCR /
//          KR PIPC) read-only time-boxed scope. Auditor reads of EU AI Act §15 + §17
//          evidence are themselves audit-emitted (audit-of-audits).
// Owner:   ops-security + council-privacy
// Audit:   every auditor read emits oya_foundry_eval_auditor_read_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — auditor JIT token scoped to (tenant subset, capability subset, time window)
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_eval_run_history",
    Action::"read_parity_report",
    Action::"read_replay_divergence",
    Action::"read_eu_ai_act_section_15_evidence",
    Action::"read_eu_ai_act_section_17_log",
    Action::"read_in_house_cutover_eligible",
    Action::"read_threat_model",
    Action::"read_dpia",
    Action::"read_compliance_mapping"
  ],
  resource
)
when {
  principal has audit_window_starts_at &&
  principal has audit_window_ends_at &&
  principal has tenant_id_allowlist &&
  principal has capability_id_allowlist &&
  context.now >= principal.audit_window_starts_at &&
  context.now <= principal.audit_window_ends_at &&
  (resource has tenant_id implies resource.tenant_id in principal.tenant_id_allowlist) &&
  (resource has capability_id implies resource.capability_id in principal.capability_id_allowlist)
};

// =============================================================================
// FORBID — auditor cannot pivot beyond their allowlist
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
unless {
  principal has tenant_id_allowlist &&
  (resource has tenant_id implies resource.tenant_id in principal.tenant_id_allowlist)
};

// =============================================================================
// FORBID — auditor token TTL cap (4 hours)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  principal has audit_window_starts_at &&
  principal has audit_window_ends_at &&
  (principal.audit_window_ends_at - principal.audit_window_starts_at) > 14400  // 4h in seconds
};

// =============================================================================
// FORBID — auditor cannot read SECRET-class
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
