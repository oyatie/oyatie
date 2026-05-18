// foundry-runtime µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA /
//          EU AI Act notified body / etc.) read time-boxed, tenant-scoped subsets
//          of runtime data when engagement active.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05, T-I-05, T-L-05; runtime-isolation.md §"Audit Trail"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads tenant-scoped data within engagement window
// =============================================================================
//
// Auditor principals issued JIT tokens by OpenBao at engagement start. Token carries:
//   - engagement_id           unique per audit cycle
//   - scoped_tenants          list of tenant_ids in scope
//   - valid_from / valid_to   ISO8601 timestamps
//   - audit_framework         soc2 | iso27001 | hipaa | gdpr-dpa | kr-pipc | eu-ai-act | ...

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_metrics",
    Action::"read_logs",
    Action::"read_session",
    Action::"read_invocation_history",
    Action::"read_audit_chain",
    Action::"read_autonomy_ceiling",
    Action::"read_autonomy_violation_history",
    Action::"read_capability_descriptor",
    Action::"read_runbook",
    Action::"read_dashboard"
  ],
  resource
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// PERMIT 2 — auditor reads pack-level policy + config artifacts
// =============================================================================
//
// Auditors may read per-pack overlays (threat-model, dpia, compliance, etc.) and
// pack-level cluster configuration without per-tenant scoping. These are policy
// + structural information, not customer data.

permit (
  principal in Auditor::?a,
  action == Action::"read_artifact",
  resource in PolicyArtifact::?r
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has artifact_type &&
  resource.artifact_type in [
    "threat-model",
    "dpia",
    "compliance",
    "runtime-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "openslo-manifest",
    "adr",
    "prd",
    "capability-catalog-record",
    "failure-modes",
    "incident-response",
    "multi-region",
    "capacity-model",
    "cost-budget"
  ]
};

// =============================================================================
// PERMIT 3 — EU AI Act notified body reads high-risk-classification trail
// =============================================================================
//
// When an EU AI Act notified body is engaged for a tenant's high-risk capability,
// the engagement principal carries audit_framework=eu-ai-act and may read the
// capability descriptor's classification history + invocation lifecycle records
// without per-tenant scoping IF the resource carries eu_ai_act_class=high_risk_annex_iii
// AND the tenant is on the notified body's scoped list.

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_capability_descriptor",
    Action::"read_invocation_history",
    Action::"read_autonomy_violation_history"
  ],
  resource in TenantData::?r
)
when {
  principal has audit_framework &&
  principal.audit_framework == "eu-ai-act" &&
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has eu_ai_act_class &&
  resource.eu_ai_act_class == "high_risk_annex_iii" &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_metrics",
    Action::"write_logs",
    Action::"dispatch_capability",
    Action::"write_capability_mirror_row",
    Action::"write_artifact",
    Action::"delete_session",
    Action::"delete_invocation_record",
    Action::"delete_artifact",
    Action::"rotate_secret",
    Action::"cancel_invocation"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read outside engagement window
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
unless {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// FORBID — auditor cannot pivot beyond scoped_tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_metrics",
    Action::"read_logs",
    Action::"read_session",
    Action::"read_invocation_history",
    Action::"read_audit_chain",
    Action::"read_autonomy_ceiling",
    Action::"read_autonomy_violation_history",
    Action::"read_capability_descriptor"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot access SECRET-class data
// =============================================================================
//
// SECRET-class data (Redis AUTH, Postgres creds, SPIFFE material, provider keys)
// is out of audit scope. Audit evidence is derived facts (audit-chain records),
// not raw secrets.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
