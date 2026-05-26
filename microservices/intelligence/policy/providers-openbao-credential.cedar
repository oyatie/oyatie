// foundry-providers µservice — OpenBao credential-access policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: enforce that only an adapter pod with the matching tenant + pack +
//          vendor binding may resolve a SecretReference for that credential.
// Owner:   ops-security
// Audit:   every resolve emits CredentialResolved event; every deny emits
//          oya_foundry_providers_credential_resolve_denied_total
// References: credential-isolation.md CI-INV-05; threat-model.md T-01
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every credential resolution is refused unless an explicit
// permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — adapter pod resolves credentials for its bound (tenant, vendor)
// =============================================================================
//
// A pod with SPIFFE identity
// `spiffe://oyatie.dev/foundry-providers/adapter-<vendor>/pack/<pack>/tenant/<tenant>`
// may resolve credentials under
// `openbao://<pack>/<tenant>/providers/<vendor>/<credential-name>`.

permit (
  principal in AdapterPod::?p,
  action == Action::"openbao_resolve",
  resource in SecretReference::?r
)
when {
  principal has spiffe_id &&
  resource has secret_path &&
  resource.secret_path like "openbao://*/*/providers/*/*" &&
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has pack &&
  resource has pack &&
  principal.pack == resource.pack &&
  principal has vendor &&
  resource has vendor &&
  principal.vendor == resource.vendor
};

// =============================================================================
// PERMIT 2 — credential-rotation runner may read + write during rotation window
// =============================================================================
//
// Per runbooks/credential-rotation.md: the rotation runner (an ops-security
// principal with the rotation role) may read existing + write new credentials
// during the rotation drill. 2-person rule enforced at PR.

permit (
  principal in RotationRunner::?p,
  action in [
    Action::"openbao_resolve",
    Action::"openbao_write_new_version",
    Action::"openbao_set_overlap_window"
  ],
  resource in SecretReference::?r
)
when {
  principal has roles &&
  "ops-security-rotation" in principal.roles &&
  principal has approval_count &&
  principal.approval_count >= 2
};

// =============================================================================
// FORBID — non-adapter principals cannot resolve credentials
// =============================================================================

forbid (
  principal,
  action == Action::"openbao_resolve",
  resource in SecretReference::?r
)
unless {
  principal in AdapterPod::"*" ||
  principal in RotationRunner::"*"
};

// =============================================================================
// FORBID — no `list` / `delete` / `metadata` on provider credentials
// =============================================================================
//
// Per credential-isolation.md CI-INV-05: pod-bound tokens have read-only scope.

forbid (
  principal in AdapterPod::?p,
  action in [
    Action::"openbao_list",
    Action::"openbao_delete",
    Action::"openbao_metadata",
    Action::"openbao_write"
  ],
  resource in SecretReference::?r
);

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal in AdapterPod::?p,
  action == Action::"openbao_resolve",
  resource in SecretReference::?r
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — credential resolution outside lease window
// =============================================================================
//
// Adapter pods request fresh leases; resolution outside an active lease emits
// a P0 incident signal.

forbid (
  principal in AdapterPod::?p,
  action == Action::"openbao_resolve",
  resource in SecretReference::?r
)
when {
  principal has lease_active &&
  principal.lease_active == false
};
