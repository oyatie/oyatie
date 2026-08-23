// Cedar policy: ci-scope
// Microservice: docs
// Purpose: bound what the oyatie CI runner can do against the docs µservice
// (PR-time checks, integration tests, evidence emission). No tenant data access.
// Owner: ops-security
// Status: Accepted (2026-05-17)
// Related: ADR-0139 (SLO-gated promotion), ADR-0140 (retired per ADR-0145)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: CI principal has NO access to live tenant data.
// =====================================================================================

forbid (
  principal in CIRole::"ci",
  action in [
    Action::"document_read",
    Action::"document_write",
    Action::"comment_read",
    Action::"comment_write",
    Action::"share_grant",
    Action::"share_link_issue",
    Action::"document_export",
    Action::"document_import",
    Action::"attachment_upload",
    Action::"crdt_op_publish"
  ],
  resource
)
when {
  resource.tenancy_kind == "live_tenant"
};

// =====================================================================================
// PERMIT: synthetic-tenant operations for integration tests.
// =====================================================================================

permit (
  principal in CIRole::"ci",
  action in [
    Action::"document_read",
    Action::"document_write",
    Action::"comment_read",
    Action::"comment_write",
    Action::"suggestion_read",
    Action::"suggestion_write",
    Action::"share_grant",
    Action::"share_revoke",
    Action::"share_link_issue",
    Action::"share_link_redeem",
    Action::"document_export",
    Action::"document_import",
    Action::"attachment_upload",
    Action::"attachment_download",
    Action::"embed_resolve",
    Action::"crdt_op_publish",
    Action::"crdt_op_subscribe",
    Action::"version_restore"
  ],
  resource
)
when {
  resource.tenancy_kind == "synthetic_test_tenant" &&
  resource.tenant_id in ["ci-tenant-alpha", "ci-tenant-beta", "ci-tenant-gamma"]
};

// =====================================================================================
// PERMIT: read-only access to per-changeset evidence + dashboard metrics.
// =====================================================================================

permit (
  principal in CIRole::"ci",
  action in [
    Action::"evidence_read",
    Action::"metrics_read",
    Action::"slo_read"
  ],
  resource
)
when {
  resource.kind in ["per_changeset_evidence", "slo_eligibility_verdict", "dashboard_metric"] &&
  resource.scope == "ci_observable"
};

// =====================================================================================
// PERMIT: emit per-changeset evidence + audit-chain test records.
// =====================================================================================

permit (
  principal in CIRole::"ci",
  action in [
    Action::"evidence_write",
    Action::"audit_chain_emit_test"
  ],
  resource
)
when {
  resource.kind == "per_changeset_evidence" &&
  resource.changeset_id == context.current_changeset_id
};

// =====================================================================================
// REFUSE: CI cannot bypass legal hold, cannot perform purge, cannot escalate role.
// =====================================================================================

forbid (
  principal in CIRole::"ci",
  action in [
    Action::"document_purge",
    Action::"legal_hold_lift",
    Action::"tenant_dek_rotate",
    Action::"role_escalate",
    Action::"cross_pack_transfer"
  ],
  resource
);

// =====================================================================================
// REFUSE: CI cannot read tenant secrets even in synthetic mode.
// =====================================================================================

forbid (
  principal in CIRole::"ci",
  action == Action::"secret_read",
  resource
)
unless {
  resource.kind == "ci_synthetic_test_credential" &&
  resource.scope == "ephemeral_test"
};

// =====================================================================================
// PERMIT: CI can exercise the gVisor export sandbox with synthetic content.
// =====================================================================================

permit (
  principal in CIRole::"ci",
  action in [
    Action::"gvisor_export_sandbox_invoke",
    Action::"gvisor_export_sandbox_escape_test"
  ],
  resource
)
when {
  resource.kind == "synthetic_export_sandbox_test"
};
