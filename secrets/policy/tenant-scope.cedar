// cloud-secrets µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/cloud-secrets/policy/schema.cedarschema
//
// Purpose: enforce per-tenant + per-µservice scope on secret resolution + admin reads.
// Owner:   ops-security + axis-cloud-secrets
// Audit:   every deny emits cloud_secrets_tenant_unauthorized_read_attempt_total
// References: secret-isolation.md §"TI-01 Per-tenant namespace", §"TI-02 Per-µservice scope"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's metadata (NOT raw secrets)
// =============================================================================
//
// Tenant operators can list secret references, view rotation policy, view audit
// for own tenant. They CANNOT resolve raw secrets (TI-07).

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"list_secret_references",
    Action::"read_rotation_policy",
    Action::"read_namespace_metadata",
    Action::"read_own_audit",
    Action::"read_attestation_report"
  ],
  resource in TenantNamespace::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator uploads encryption-key BYOK material (ADR-0251 §D-10)
// =============================================================================
//
// encryption-key BYOK upload requires JIT short-lived elevation + MFA (ADR-0251 §D-10). The policy fragment
// validates the operator's tenant scope; OpenBao policy validates JIT + MFA.

permit (
  principal in TenantOperator::?p,
  action == Action::"upload_byok",
  resource in TenantNamespace::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has mfa_verified &&
  principal.mfa_verified == true &&
  principal has jit_elevation_active &&
  principal.jit_elevation_active == true
};

// =============================================================================
// PERMIT 3 — in-cluster µservice consumer resolves a SecretReference
// =============================================================================
//
// SPIFFE-attested workload identity resolves secrets within its
// (tenant, microservice) scope. The principal carries spiffe_id + tenant_id +
// microservice claims. The resource carries the same trio derived from the
// secret path.

permit (
  principal in WorkloadIdentity::?p,
  action == Action::"resolve_secret_reference",
  resource in SecretReferenceResource::?r
)
when {
  principal has spiffe_id &&
  principal has tenant_id &&
  principal has microservice &&
  resource has tenant_id &&
  resource has microservice &&
  principal.tenant_id == resource.tenant_id &&
  principal.microservice == resource.microservice
};

// =============================================================================
// PERMIT 4 — shared-substrate consumer resolves a shared SecretReference
// =============================================================================
//
// Shared secrets (mTLS CA roots, mesh certs, etc.) live under `shared/<ms>/*`.
// Per-µservice scope still applies (only the µservice owning the shared bucket
// may write; consumers in the allowlist may read).

permit (
  principal in WorkloadIdentity::?p,
  action == Action::"resolve_secret_reference",
  resource in SharedSecretReferenceResource::?r
)
when {
  resource has scope &&
  resource.scope == "shared" &&
  principal has microservice &&
  resource has reader_allowlist &&
  principal.microservice in resource.reader_allowlist
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"list_secret_references",
    Action::"read_rotation_policy",
    Action::"read_namespace_metadata",
    Action::"read_own_audit",
    Action::"read_attestation_report",
    Action::"resolve_secret_reference",
    Action::"upload_byok"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — explicit cross-µservice deny (defence-in-depth)
// =============================================================================

forbid (
  principal in WorkloadIdentity::?p,
  action == Action::"resolve_secret_reference",
  resource in SecretReferenceResource::?r
)
unless {
  principal has microservice &&
  resource has microservice &&
  principal.microservice == resource.microservice
};

// =============================================================================
// FORBID — tenant operators cannot resolve raw secrets (TI-07)
// =============================================================================
//
// Even with valid OIDC + correct tenant_id, tenant operators MUST NOT receive
// raw SECRET-class data via the REST surface. Resolution is SDK-side only.

forbid (
  principal in TenantOperator::?p,
  action == Action::"resolve_secret_reference",
  resource
);

// =============================================================================
// FORBID — never expose SECRET-class material directly to any human principal
// =============================================================================

forbid (
  principal in Human::?p,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — cross-pack scope (residency invariant TI-09)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has pack &&
  resource has pack &&
  principal.pack != resource.pack
};
