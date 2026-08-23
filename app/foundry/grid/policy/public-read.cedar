// sheets µservice — public-read policy fragment
// Cedar v4.2 LTS schema
//
// Purpose: define what (if anything) is publicly readable from the sheets µservice.
//          Default posture: NOTHING tenant-specific publicly readable. This file makes that explicit.
// Owner:   ops-security + council-privacy + axis-sheets + council-design-system
// References: threat-model.md §"T-I-06" (per-tenant branding mid-render anti-pattern)
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

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"validate_canonical_sheet_schema",
  resource == SchemaValidationEndpoint::"canonical_sheet_schema_v1"
);

// =============================================================================
// PERMIT — public status (Sheets health, no tenant data)
// =============================================================================

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"read_public_health",
  resource == PublicEndpoint::"sheets_health"
);

// =============================================================================
// PERMIT — anonymous fetch of public function-library descriptor
// =============================================================================
//
// Tenant-agnostic ≥400-function library descriptor; useful for tenant onboarding tooling
// and IDE auto-complete. Per-tenant entitlement enforced separately at invocation.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"list_public_function_library",
  resource == PublicCatalog::"function_library_catalog"
);

// =============================================================================
// FORBID — explicitly nothing else publicly readable
// =============================================================================

forbid (
  principal == AnonymousPrincipal::"anonymous",
  action in [
    Action::"read_workbook_session",
    Action::"read_workbook_metadata",
    Action::"read_cell",
    Action::"read_range",
    Action::"open_workbook_session",
    Action::"save_workbook",
    Action::"write_cell",
    Action::"write_range",
    Action::"write_formula",
    Action::"publish_crdt_op",
    Action::"subscribe_crdt_stream",
    Action::"invoke_ai_formula_draft",
    Action::"invoke_smart_fill",
    Action::"import_xlsx",
    Action::"export_xlsx",
    Action::"register_connected_query",
    Action::"refresh_connected_query",
    Action::"share_workbook",
    Action::"author_range_acl",
    Action::"open_embed_bridge",
    Action::"author_edit_trigger",
    Action::"restore_workbook_version"
  ],
  resource
);

// =============================================================================
// FORBID — per-tenant branding mid-render (anti-pattern)
// =============================================================================
//
// Anti-pattern per /specs/app/sheets.json §anti_patterns:
// per-tenant branding rendered in same DOM tree as cell-grid is forbidden.

forbid (
  principal,
  action == Action::"render_tenant_branding_inline",
  resource
);
