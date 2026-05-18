// workflow-studio µservice — public-read policy fragment
// Cedar v4 schema
//
// Purpose: define what (if anything) is publicly readable from the workflow-studio µservice.
//          Default posture: NOTHING tenant-specific publicly readable. This file makes that explicit.
// Owner:   ops-security + council-privacy + axis-workflow + council-design-system
// References: threat-model.md §"T-I-07" (per-tenant branding mid-render anti-pattern)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — public access refused.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — anonymous fetch of static WASM bundle + spec schema + design-system primitives
// =============================================================================
//
// These assets are tenant-agnostic; required for editor bootstrap.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"fetch_static_asset",
  resource in StaticAsset::?s
)
when {
  resource has tenant_scoped &&
  resource.tenant_scoped == false
};

// =============================================================================
// PERMIT — anonymous schema validation (OpenAPI/spec-schema fetch)
// =============================================================================
//
// Spec schema validation requires the schema itself to be fetchable.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"validate_workflow_spec_schema",
  resource == SchemaValidationEndpoint::"workflow_spec_schema_v1"
);

// =============================================================================
// PERMIT — public status (Studio health, no tenant data)
// =============================================================================
//
// Health endpoint returns up/down + version + cluster-aggregate active editor sessions;
// no per-tenant data. Consumed by status.oyatie.dev.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"read_public_health",
  resource == PublicEndpoint::"studio_health"
);

// =============================================================================
// PERMIT — anonymous fetch of public node-library catalog descriptors
// =============================================================================
//
// Tenant-agnostic descriptor of node libraries available per pack; useful for
// tenant onboarding tooling. Per-tenant entitlement enforced separately at load.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"list_public_node_library_catalog",
  resource == PublicCatalog::"node_library_catalog"
);

// =============================================================================
// FORBID — explicitly nothing else publicly readable
// =============================================================================

forbid (
  principal == AnonymousPrincipal::"anonymous",
  action in [
    Action::"read_editor_session",
    Action::"read_workflow_definition_draft",
    Action::"open_editor_session",
    Action::"save_workflow_definition",
    Action::"publish_crdt_op",
    Action::"subscribe_crdt_stream",
    Action::"invoke_llm_assist",
    Action::"open_debugger_session",
    Action::"step_through_run_frontend",
    Action::"load_node_library",
    Action::"read_collab_participant_list",
    Action::"list_editor_sessions",
    Action::"list_workflow_definitions"
  ],
  resource
);

// =============================================================================
// FORBID — per-tenant branding mid-render (anti-pattern)
// =============================================================================
//
// Anti-pattern per /specs/microservices/workflow-studio.json §anti_patterns:
// per-tenant branding rendered in same DOM tree as canvas is forbidden.

forbid (
  principal,
  action == Action::"render_tenant_branding_inline",
  resource
);
