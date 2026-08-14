// Cedar policy: auditor-scope
// Microservice: calendar
// Purpose: bound external auditors (SOC 2 / ISO 27001 / GDPR DPA / HIPAA OCR / KR PIPC)
// to time-boxed read-only access scoped to specific engagement-target tenants.
// Owner: ops-security + council-privacy
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145)
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
    Action::"event_write",
    Action::"event_purge",
    Action::"room_book",
    Action::"invitation_send",
    Action::"caldav_put",
    Action::"caldav_delete",
    Action::"legal_hold_open",
    Action::"legal_hold_lift",
    Action::"tenant_dek_rotate"
  ],
  resource
);

// =====================================================================================
// PERMIT: read-only access to engagement-target tenants + their related evidence.
// =====================================================================================

permit (
  principal in AuditorRole::"external_auditor",
  action in [
    Action::"event_read",
    Action::"availability_read",
    Action::"room_calendar_read",
    Action::"audit_chain_read",
    Action::"evidence_read",
    Action::"compliance_doc_read"
  ],
  resource
)
when {
  context.engagement_active == true &&
  context.token_expires_at >= context.now &&
  // Auditor token scoped to specific tenant subset; cross-pivot forbidden.
  resource.tenant_id in context.engagement_target_tenants &&
  // mTLS client cert pinned to auditor firm gateway during engagement window.
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
  // Even within engagement, refuse access to tenants outside target list.
  context.engagement_active == true &&
  not (resource.tenant_id in context.engagement_target_tenants)
};

// =====================================================================================
// REFUSE: auditor reading Personal-context end-user content (limited to Professional).
// =====================================================================================

forbid (
  principal in AuditorRole::"external_auditor",
  action == Action::"event_read",
  resource in EventContext::"Personal"
);

// =====================================================================================
// PERMIT: auditor reads pack-us-healthcare PHI only when audit scope includes HIPAA.
// =====================================================================================

permit (
  principal in AuditorRole::"hipaa_auditor",
  action == Action::"event_read",
  resource in DataClass::"PHI"
)
when {
  context.engagement_active == true &&
  context.engagement_kind == "hipaa" &&
  // BAA-equivalent between auditor and oyatie must be on file.
  context.auditor_baa_on_file == true &&
  resource.pack_tag == "pack-us-healthcare"
};

// =====================================================================================
// AUDIT EMISSION: every auditor read is itself recorded.
// =====================================================================================

// (Cedar policies don't emit; the application's usecase enforces the audit-emission
//  invariant. This comment documents the contract; verification via LEAN check
//  `oya-check-auditor-read-emission`.)
