// Cedar policy: tenant-scope
// Microservice: tasks
// Purpose: enforce per-tenant scoping on every task operation; refuse cross-tenant
// read/write outright (tasks has NO cross-tenant grant mechanism, unlike calendar).
// Owner: ops-security + axis-tasks
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/task-isolation.md, policy/data-residency.md, ADR-TASKS-0006
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: forbid cross-tenant reads / writes by default.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"task_read",
    Action::"task_write",
    Action::"task_state_change",
    Action::"task_assign",
    Action::"task_comment",
    Action::"task_dependency_add",
    Action::"task_dependency_remove",
    Action::"task_bulk_edit",
    Action::"task_export",
    Action::"task_import",
    Action::"project_read",
    Action::"project_write",
    Action::"view_render",
    Action::"search_query",
    Action::"recurrence_materialise",
    Action::"workflow_bridge_create_task"
  ],
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// =====================================================================================
// PERMIT: same-tenant access with role-based authorization.
// =====================================================================================

permit (
  principal in TasksRole::"tenant_operator",
  action in [
    Action::"task_read",
    Action::"task_write",
    Action::"task_state_change",
    Action::"task_assign",
    Action::"task_comment",
    Action::"task_dependency_add",
    Action::"task_dependency_remove",
    Action::"task_export",
    Action::"task_import",
    Action::"project_read",
    Action::"project_write",
    Action::"view_render",
    Action::"search_query"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.context_kind == resource.context_kind
};

// =====================================================================================
// BULK-EDIT: requires explicit per-tenant authorization + size confirmation.
// =====================================================================================

permit (
  principal in TasksRole::"tenant_operator",
  action == Action::"task_bulk_edit",
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  // Bulk edits ≤10k tasks: no second confirmation needed.
  context.bulk_size <= 10000
};

permit (
  principal in TasksRole::"tenant_operator",
  action == Action::"task_bulk_edit",
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  // Bulk edits >10k tasks: require second confirmation per PRD §"Security".
  context.bulk_size > 10000 &&
  context.second_confirmation_signed == true
};

// =====================================================================================
// WORKFLOW-BRIDGE: workflow-engine creates tasks (per ADR-TASKS-0005).
// =====================================================================================

permit (
  principal in TasksRole::"workflow_engine_runner",
  action == Action::"workflow_bridge_create_task",
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  // Workflow-creates-task records workflow run ID for attribution chain (T-S-02).
  resource has created_by_workflow_run_id &&
  resource has created_by_principal &&
  // SPIFFE-identity of workflow-engine verified.
  context.spiffe_verified == true
};

// =====================================================================================
// LEGAL HOLD: refuse hard-delete on held tasks.
// =====================================================================================

forbid (
  principal,
  action == Action::"task_purge",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == true
};

permit (
  principal in TasksRole::"compliance_officer_with_2person_rule",
  action == Action::"task_purge",
  resource
)
when {
  // Hard-delete permitted only with 2-person rule + explicit lift of hold.
  resource has legal_hold_open && resource.legal_hold_open == false &&
  context.two_person_rule_approved == true &&
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// DEPENDENCY EDGE WRITE: refuse cycle-creating writes (PRD AC-02 — 100% correctness).
// =====================================================================================

forbid (
  principal,
  action == Action::"task_dependency_add",
  resource
)
when {
  // Cycle pre-check is part of the usecase orchestrator;
  // Cedar enforces it via the `would_create_cycle` context attribute
  // computed at admission time (ADR-TASKS-0002).
  context.would_create_cycle == true
};

// =====================================================================================
// CROSS-PACK DATA TRANSFER: forbid by default; permit cross-µservice handoff with SCC.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"task_read",
    Action::"task_write"
  ],
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

permit (
  principal in TasksRole::"cross_pack_workflow_bridge",
  action == Action::"workflow_bridge_create_task",
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag &&
  context.scc_on_file == true &&
  context.tenant_dpa_includes_scc_clause == true
};

// =====================================================================================
// PROJECT OWNERSHIP TRANSFER.
// =====================================================================================

permit (
  principal in TasksRole::"project_owner",
  action == Action::"project_transfer_ownership",
  resource
)
when {
  principal.user_id == resource.owner_user_id &&
  // Transfer requires owner explicit action.
  context.transfer_acknowledged_by_owner == true
};

forbid (
  principal,
  action == Action::"project_transfer_ownership",
  resource
)
unless {
  principal.user_id == resource.owner_user_id
};
