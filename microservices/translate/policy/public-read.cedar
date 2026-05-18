// translate µservice — public-read policy fragment
// Cedar v4.2
//
// Purpose: define the limited set of resources that may be read by
//          unauthenticated principals (status page; engine capabilities;
//          supported-language list).
// Owner:   ops-security
// Audit:   every read emits oya_translate_public_read_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — status page reads (per-pack rollup; no per-tenant)
// =============================================================================

permit (
  principal,
  action == Action::"status.read",
  resource in StatusPage::?s
);

// =============================================================================
// PERMIT 2 — capability catalogue (list of supported language pairs;
// engine capabilities; placeholder/format support)
// =============================================================================

permit (
  principal,
  action == Action::"capabilities.read_public",
  resource in PublicCatalogue::?c
);

// =============================================================================
// PERMIT 3 — OpenAPI spec read (developer portal)
// =============================================================================

permit (
  principal,
  action == Action::"openapi.read",
  resource in OpenApiSpec::?o
);

// =============================================================================
// PERMIT 4 — supported-language list read (BCP 47 inventory)
// =============================================================================

permit (
  principal,
  action == Action::"supported_languages.read",
  resource in PublicCatalogue::?c
);

// =============================================================================
// FORBID — public never accesses per-tenant data
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has tenant_id
};

// =============================================================================
// FORBID — public never sees credentials, TM, termbase, audit events
// =============================================================================

forbid (
  principal,
  action in [
    Action::"credential.resolve",
    Action::"tm.lookup",
    Action::"tm.export",
    Action::"termbase.read",
    Action::"termbase.export",
    Action::"audit.read_events",
    Action::"audit.read_evidence",
    Action::"bulk.read",
    Action::"doc.read",
    Action::"stream.open",
    Action::"engine_pin.set",
    Action::"engine_pin.read"
  ],
  resource
);
