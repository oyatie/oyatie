// workflow-studio µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/workflow-studio/policy/schema.cedarschema
//
// Purpose: enforce per-tenant scope on editor session open/read/write, CRDT op publish/subscribe,
//          LLM-assist invocation, save action, replay-debugger frontend read, and node-library load
//          served via workflow-studio REST + WebSocket gateway.
// Owner:   ops-security + axis-workflow
// Audit:   every deny emits oya_workflow_studio_unauthorized_attempt_total
// References: threat-model.md §"T-I-01" (cross-tenant editor session) + §"T-I-04" (cross-definition collab)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action refused unless explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's editor sessions + drafts
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_editor_session",
    Action::"read_workflow_definition_draft",
    Action::"read_collab_participant_list",
    Action::"list_editor_sessions",
    Action::"list_workflow_definitions",
    Action::"read_jurisdiction_overlay",
    Action::"read_node_library_descriptor",
    Action::"read_debugger_session"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator opens new editor session
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"open_editor_session",
    Action::"close_editor_session",
    Action::"resume_editor_session"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_studio_seat_entitlement &&
  principal.has_studio_seat_entitlement == true
};

// =============================================================================
// PERMIT 3 — tenant operator authors + saves workflow definitions
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"publish_crdt_op",
    Action::"save_workflow_definition",
    Action::"discard_workflow_definition_draft",
    Action::"switch_jurisdiction_overlay"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — tenant operator subscribes to own tenant's collab streams
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"subscribe_crdt_stream",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has definition_id &&
  principal has subscribed_definitions &&
  resource.definition_id in principal.subscribed_definitions
};

// =============================================================================
// PERMIT 5 — tenant operator invokes LLM-assist (consent-required)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"invoke_llm_assist",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has llm_assist_consent &&
  principal.llm_assist_consent == true &&
  principal has has_studio_seat_entitlement &&
  principal.has_studio_seat_entitlement == true
};

// =============================================================================
// PERMIT 6 — tenant operator opens replay-debugger session
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"open_debugger_session",
    Action::"step_through_run_frontend",
    Action::"close_debugger_session"
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
// PERMIT 7 — tenant operator loads per-pack node library
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"load_node_library",
  resource in NodeLibrary::?l
)
when {
  resource has pack &&
  principal has packs &&
  resource.pack in principal.packs
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defense-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_editor_session",
    Action::"read_workflow_definition_draft",
    Action::"read_collab_participant_list",
    Action::"list_editor_sessions",
    Action::"list_workflow_definitions",
    Action::"read_jurisdiction_overlay",
    Action::"read_node_library_descriptor",
    Action::"read_debugger_session",
    Action::"open_editor_session",
    Action::"close_editor_session",
    Action::"resume_editor_session",
    Action::"publish_crdt_op",
    Action::"save_workflow_definition",
    Action::"discard_workflow_definition_draft",
    Action::"switch_jurisdiction_overlay",
    Action::"subscribe_crdt_stream",
    Action::"invoke_llm_assist",
    Action::"open_debugger_session",
    Action::"step_through_run_frontend",
    Action::"close_debugger_session"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — cross-pack node library load
// =============================================================================

forbid (
  principal,
  action == Action::"load_node_library",
  resource in NodeLibrary::?l
)
unless {
  resource has pack &&
  principal has packs &&
  resource.pack in principal.packs
};

// =============================================================================
// FORBID — never expose SECRET-class data
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
// FORBID — Studio never executes; only emits
// =============================================================================

forbid (
  principal,
  action in [
    Action::"execute_workflow_step",
    Action::"dispatch_step_body",
    Action::"start_workflow_run"
  ],
  resource
);

// =============================================================================
// FORBID — production-tier save requires Cedar policy preview acknowledgment
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action == Action::"save_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has environment &&
  resource.environment == "production" &&
  principal has has_policy_preview_acknowledged &&
  principal.has_policy_preview_acknowledged != true
};
