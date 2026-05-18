// Cedar policy: ci-scope
// Microservice: tasks
// Purpose: bound what the oyatie CI runner can do against the tasks µservice
// (PR-time checks, integration tests, evidence emission). No tenant data access.
// Owner: ops-security
// Status: Accepted (2026-05-17)
// Related: ADR-0139 (SLO-gated promotion), ADR-0140 (retired per ADR-0145)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: CI principal has NO access to live tenant data.
// =====================================================================================

forbid (
  principal in CIRole::"oya-ci",
  action in [
    Action::"task_read",
    Action::"task_write",
    Action::"task_assign",
    Action::"task_bulk_edit",
    Action::"project_read",
    Action::"project_write",
    Action::"search_query",
    Action::"task_import",
    Action::"task_export",
    Action::"workflow_bridge_create_task"
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
  principal in CIRole::"oya-ci",
  action in [
    Action::"task_read",
    Action::"task_write",
    Action::"task_assign",
    Action::"task_bulk_edit",
    Action::"project_read",
    Action::"project_write",
    Action::"task_dependency_add",
    Action::"task_dependency_remove",
    Action::"view_render",
    Action::"search_query",
    Action::"task_import",
    Action::"task_export",
    Action::"recurrence_materialise"
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
  principal in CIRole::"oya-ci",
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
  principal in CIRole::"oya-ci",
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
// REFUSE: CI cannot bypass invariants, cannot perform purge, cannot escalate role.
// =====================================================================================

forbid (
  principal in CIRole::"oya-ci",
  action in [
    Action::"task_purge",
    Action::"legal_hold_lift",
    Action::"tenant_dek_rotate",
    Action::"role_escalate",
    Action::"cross_pack_transfer",
    Action::"foundry_classifier_promote"
  ],
  resource
);

// =====================================================================================
// REFUSE: CI cannot read tenant secrets even in synthetic mode.
// =====================================================================================

forbid (
  principal in CIRole::"oya-ci",
  action == Action::"secret_read",
  resource
)
unless {
  resource.kind == "ci_synthetic_test_credential" &&
  resource.scope == "ephemeral_test"
};

// =====================================================================================
// REFUSE: CI cannot execute T2 auto-assign (even in synthetic mode).
// CI may verify T2 admission gate via Cedar policy unit test only.
// =====================================================================================

forbid (
  principal in CIRole::"oya-ci",
  action == Action::"task_t2_auto_assign",
  resource
);
