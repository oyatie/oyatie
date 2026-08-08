// workflow-engine µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/workflow-engine/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-tenant scope on workflow spec submission, run start/stop, event publish/subscribe,
//          and replay-debugger reads served via workflow-engine REST + SDK.
// Owner:   ops-security + axis-workflow
// Audit:   every deny emits oya_workflow_engine_unauthorized_read_attempt_total
// References: threat-model.md §"T-I-01" (cross-tenant run state) + §"T-I-03" (cross-tenant subscription)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's workflow data
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_workflow_spec",
    Action::"read_workflow_run",
    Action::"read_step_execution",
    Action::"read_workflow_event",
    Action::"read_replay_session",
    Action::"read_run_analytics",
    Action::"list_workflow_specs",
    Action::"list_workflow_runs"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator submits workflow spec (via Studio or SDK)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"submit_workflow_spec",
    Action::"deprecate_workflow_spec"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_signer_entitlement &&
  principal.has_signer_entitlement == true
};

// =============================================================================
// PERMIT 3 — tenant operator starts/pauses/resumes/cancels own runs
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"start_workflow_run",
    Action::"pause_workflow_run",
    Action::"resume_workflow_run",
    Action::"cancel_workflow_run",
    Action::"signal_workflow_run"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — tenant operator publishes / subscribes to own tenant's events
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"publish_workflow_event",
    Action::"subscribe_workflow_event"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 5 — replay-debugger access requires DPA-authorised debugger entitlement
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"replay_workflow_run",
    Action::"step_through_run",
    Action::"inspect_step_payload"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has debugger_entitlement &&
  principal.debugger_entitlement == true
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_workflow_spec",
    Action::"read_workflow_run",
    Action::"read_step_execution",
    Action::"read_workflow_event",
    Action::"read_replay_session",
    Action::"read_run_analytics",
    Action::"list_workflow_specs",
    Action::"list_workflow_runs",
    Action::"submit_workflow_spec",
    Action::"deprecate_workflow_spec",
    Action::"start_workflow_run",
    Action::"pause_workflow_run",
    Action::"resume_workflow_run",
    Action::"cancel_workflow_run",
    Action::"signal_workflow_run",
    Action::"publish_workflow_event",
    Action::"subscribe_workflow_event",
    Action::"replay_workflow_run",
    Action::"step_through_run",
    Action::"inspect_step_payload"
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
// FORBID — production-tier run cancel requires 2-person rule
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action == Action::"cancel_workflow_run",
  resource in TenantData::?r
)
when {
  resource has environment &&
  resource.environment == "production" &&
  principal has has_two_person_rule_satisfied &&
  principal.has_two_person_rule_satisfied != true
};
