// application µservice — CI runner scope policy fragment
// Cedar v4
//
// Purpose: govern what the CI runner (SPIFFE-identified) may read or write
//          against the Application Shell control plane during gate execution.
// Owner:   axis-foundry + ops-security
// References: route-isolation.md §"Lane Enforcement"; threat-model.md (CI runner actor)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI may read route registry for layout + scope verification
// =============================================================================

permit (
  principal == ServiceAccount::"oya-ci",
  action in [
    Action::"read_route_registry",
    Action::"read_module_manifest",
    Action::"read_openslo_manifest",
    Action::"read_catalog"
  ],
  resource
)
when {
  resource has data_class &&
  (resource.data_class == "INTERNAL_ONLY" ||
   resource.data_class == "PUBLIC")
};

// =============================================================================
// PERMIT 2 — CI may publish module manifests via SDK (signed)
// =============================================================================
//
// Only the oya-ci-publish workflow may publish a module manifest, and only
// when accompanied by a valid Ed25519 detached signature whose key id matches
// the per-product publisher key registered in OpenBao.

permit (
  principal == ServiceAccount::"oya-ci-publish",
  action == Action::"publish_module_manifest",
  resource in Module::?m
)
when {
  principal has signing_key_id &&
  resource has expected_signer_key_id &&
  principal.signing_key_id == resource.expected_signer_key_id
};

// =============================================================================
// PERMIT 3 — CI may write to test tenants only
// =============================================================================

permit (
  principal == ServiceAccount::"oya-ci",
  action,
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id like "tenant:test-*"
};

// =============================================================================
// FORBID — CI never reads SECRET-class data
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — CI never touches production tenant data
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-ci",
  action in [
    Action::"render_route",
    Action::"load_module",
    Action::"read_audit",
    Action::"write_session"
  ],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id like "tenant:test-*")
};
