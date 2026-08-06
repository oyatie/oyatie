// Cedar policy: auditor-scope
// Microservice: tasks
// Purpose: bound external auditors (SOC 2 / ISO 27001 / GDPR DPA / HIPAA OCR / KR PIPC /
// EU AI Act notified body / EEOC bias auditor / NY DCWP AEDT auditor) to time-boxed
// read-only access scoped to specific engagement-target tenants.
// Owner: ops-security + council-privacy
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), ADR-TASKS-0006
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: auditor has NO access outside engagement window.
// =====================================================================================

forbid (
  principal in AuditorRole::"external_auditor",
  action,
  resource
)
when {
  context.engagement_active == false ||
  context.token_expires_at < context.now
};

// =====================================================================================
// REFUSE: auditor cannot write anything, ever.
// =====================================================================================

forbid (
  principal in AuditorRole::"external_auditor",
  action in [
    Action::"task_write",
    Action::"task_purge",
    Action::"task_state_change",
    Action::"task_assign",
    Action::"task_bulk_edit",
    Action::"project_write",
    Action::"task_import",
    Action::"legal_hold_open",
    Action::"legal_hold_lift",
    Action::"tenant_dek_rotate",
    Action::"foundry_classifier_promote",
    Action::"task_t2_auto_assign"
  ],
  resource
);

// =====================================================================================
// PERMIT: read-only access to engagement-target tenants + their related evidence.
// =====================================================================================

permit (
  principal in AuditorRole::"external_auditor",
  action in [
    Action::"task_read",
    Action::"project_read",
    Action::"view_render",
    Action::"search_query",
    Action::"audit_chain_read",
    Action::"evidence_read",
    Action::"compliance_doc_read"
  ],
  resource
)
when {
  context.engagement_active == true &&
  context.token_expires_at >= context.now &&
  resource.tenant_id in context.engagement_target_tenants &&
  context.mtls_pinned == true
};

// =====================================================================================
// REFUSE: cross-tenant pivot during engagement.
// =====================================================================================

forbid (
  principal in AuditorRole::"external_auditor",
  action,
  resource
)
when {
  context.engagement_active == true &&
  not (resource.tenant_id in context.engagement_target_tenants)
};

// =====================================================================================
// REFUSE: auditor reading Personal-context end-user content (Professional only).
// =====================================================================================

forbid (
  principal in AuditorRole::"external_auditor",
  action == Action::"task_read",
  resource in TaskContext::"Personal"
);

// =====================================================================================
// PERMIT: HIPAA auditor reads pack-us-healthcare PHI when audit scope includes HIPAA.
// =====================================================================================

permit (
  principal in AuditorRole::"hipaa_auditor",
  action == Action::"task_read",
  resource in DataClass::"PHI"
)
when {
  context.engagement_active == true &&
  context.engagement_kind == "hipaa" &&
  context.auditor_baa_on_file == true &&
  resource.pack_tag == "pack-us-healthcare"
};

// =====================================================================================
// PERMIT: EU AI Act notified body auditor reads T2 auto-assign audit chain.
// =====================================================================================

permit (
  principal in AuditorRole::"eu_ai_act_notified_body",
  action in [
    Action::"audit_chain_read",
    Action::"foundry_classifier_audit_read",
    Action::"fairness_metric_read"
  ],
  resource
)
when {
  context.engagement_active == true &&
  context.engagement_kind == "eu_ai_act_conformity_assessment" &&
  resource.tenant_id in context.engagement_target_tenants &&
  resource.pack_tag in ["pack-eu", "pack-us", "pack-kr"]
};

// =====================================================================================
// PERMIT: EEOC / NY DCWP AEDT bias auditor reads auto-assign decision audit chain.
// =====================================================================================

permit (
  principal in AuditorRole::"eeoc_bias_auditor",
  action in [
    Action::"audit_chain_read",
    Action::"fairness_metric_read",
    Action::"foundry_classifier_audit_read"
  ],
  resource
)
when {
  context.engagement_active == true &&
  context.engagement_kind in ["eeoc_disparate_impact_audit", "ny_local_law_144_aedt_audit"] &&
  resource.tenant_id in context.engagement_target_tenants &&
  resource.pack_tag in ["pack-us", "pack-us-healthcare"]
};

// =====================================================================================
// AUDIT EMISSION: every auditor read is itself recorded via usecase orchestrator.
// LEAN check `oya-check-auditor-read-emission` verifies.
// =====================================================================================
