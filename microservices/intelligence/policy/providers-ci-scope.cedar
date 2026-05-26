// foundry-providers µservice — CI scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: scope CI runner reads to non-tenant resources during gate execution.
//          CI may inspect provider-config schema, run integration tests against
//          mocked vendor edges, and verify residency conformance — but never
//          read real tenant credentials or invocation history.
// Owner:   axis-foundry + ops-security
// Audit:   CI principal access emits CiRead event
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI runner reads schema + canonical test fixtures
// =============================================================================

permit (
  principal in CiRunner::?p,
  action in [
    Action::"ci_read_schema",
    Action::"ci_read_fixture",
    Action::"ci_read_residency_matrix",
    Action::"ci_read_capability_catalog"
  ],
  resource in CiAccessibleResource::?r
)
when {
  resource has data_class &&
  resource.data_class == "INTERNAL_ONLY" &&
  principal has lane_id &&
  principal.lane_id like "oya-*"
};

// =============================================================================
// PERMIT 2 — CI runner invokes router against synthetic tenant
// =============================================================================
//
// CI integration tests use synthetic tenant `oya-ci`; the router treats this
// tenant identically to real tenants for behavior verification, but the
// upstream HTTP call is intercepted by the mock-vendor-edge service.

permit (
  principal in CiRunner::?p,
  action in [
    Action::"router_invoke",
    Action::"router_decide",
    Action::"router_get_decision",
    Action::"capability_profile_update"
  ],
  resource in ProviderInvocation::?r
)
when {
  resource has tenant_id &&
  resource.tenant_id == "oya-ci" &&
  resource has mock_vendor_edge &&
  resource.mock_vendor_edge == true
};

// =============================================================================
// FORBID — CI cannot read real tenant credentials, invocations, or events
// =============================================================================

forbid (
  principal in CiRunner::?p,
  action,
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id != "oya-ci"
};

// =============================================================================
// FORBID — CI cannot resolve real OpenBao secrets
// =============================================================================

forbid (
  principal in CiRunner::?p,
  action == Action::"openbao_resolve",
  resource in SecretReference::?r
);

// =============================================================================
// FORBID — CI cannot write to production OpenBao paths
// =============================================================================

forbid (
  principal in CiRunner::?p,
  action in [
    Action::"openbao_write_new_version",
    Action::"openbao_set_overlap_window",
    Action::"openbao_delete"
  ],
  resource
);
