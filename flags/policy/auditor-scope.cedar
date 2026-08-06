// Cedar v4.2 LTS fragment — auditor-scope
// Microservice: feature-flags
// Binding ADR: ADR-0243 + ADR-0028 (audit chain) + ADR-0263
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-05-20
//
// Governs read-only access to the sealed audit trail for compliance officers,
// QSA auditors, and regulator access.

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: compliance-officer may read audit events for their tenant
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"compliance-officer",
  action == Action::"AuditEventRead",
  resource is AuditEvent
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: QSA auditor may read audit events for the tenant under audit
// QSA access is time-bounded (audit window); enforced via context.audit_window_valid
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"qsa-auditor",
  action == Action::"AuditEventRead",
  resource is AuditEvent
)
when {
  principal.tenant_id == resource.tenant_id
  && context.audit_window_valid == true
  && principal.mfa_verified == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: regulator access (read-only; per warrant gate)
// Cross-ref: ADR-0251 §regulator-access; ADR-0276 portability
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"regulator",
  action == Action::"AuditEventRead",
  resource is AuditEvent
)
when {
  context.warrant_id != ""
  && context.warrant_valid == true
  && context.warrant_scope covers resource.tenant_id
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: platform-safety-officer may read audit events cross-tenant
// (platform-level incident investigation; Foundry-attested)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"platform-safety-officer",
  action == Action::"AuditEventRead",
  resource is AuditEvent
)
when {
  context.foundry_attestation_present == true
  && context.investigation_ticket_id != ""
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: audit events cannot be deleted or modified by any principal
// Sealed audit chain (ADR-0028) is append-only; no deletion
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"AuditEventDelete", Action::"AuditEventModify"],
  resource is AuditEvent
);

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: cross-tenant audit reads without platform-safety-officer or regulator role
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"AuditEventRead",
  resource is AuditEvent
)
unless {
  principal.tenant_id == resource.tenant_id
  || principal in Role::"platform-safety-officer"
  || principal in Role::"regulator"
};
