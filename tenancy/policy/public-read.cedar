// tenancy µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to non-sensitive tenancy resources
//          (status page, OpenAPI schema, public Cedar schema, public docs).
//          Anonymous users CANNOT read any tenant-scoped data; they CAN validate
//          OpenAPI conformance against tenancy's public schema endpoint.
// Owner:   ops-security + gtm-marketing
// References: threat-model.md T-I-04 (error-message leakage); rls-isolation.md
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — anonymous read of platform status (self tenant, public-class)
// =============================================================================
//
// The public status page reads aggregated tenancy availability metrics from
// the self tenant. Platform-wide health (e.g., "validate path availability
// over the last 30 days") with no per-customer attribution.

permit (
  principal == User::"anonymous",
  action == Action::"read_metrics",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:self" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has metric_name &&
  resource.metric_name like "tenancy_public_*"
};

// =============================================================================
// PERMIT 2 — anonymous read of OpenAPI schema
// =============================================================================
//
// The tenancy REST OpenAPI spec is publishable. Schema discovery does not leak
// tenant data; it leaks API surface only, which is already documented in
// contracts/openapi/tenancy.yaml.

permit (
  principal == User::"anonymous",
  action == Action::"read_openapi_schema",
  resource in PublicEndpoint::"tenancy-openapi-schema"
);

// =============================================================================
// PERMIT 3 — anonymous read of Cedar schema (for tenant-side policy authoring)
// =============================================================================
//
// Tenant-side policy authors may want to read tenancy's Cedar schema to
// understand the action / resource taxonomy. Schema is publishable.

permit (
  principal == User::"anonymous",
  action == Action::"read_cedar_schema",
  resource in PublicEndpoint::"tenancy-cedar-schema"
);

// =============================================================================
// PERMIT 4 — anonymous health + readiness probes
// =============================================================================
//
// Kubernetes liveness + readiness probes for tenancy-rest pods. No data leak.

permit (
  principal == User::"anonymous",
  action in [Action::"read_health", Action::"read_readiness"],
  resource in PublicEndpoint::"tenancy-health"
);

// =============================================================================
// PERMIT 5 — anonymous read of pack registry (non-sensitive)
// =============================================================================
//
// The list of activated packs (pack-kr active, pack-eu conditional, etc.) is
// publishable; tenants need this to know which jurisdictions oyatie can serve.

permit (
  principal == User::"anonymous",
  action == Action::"read_pack_registry",
  resource in PublicEndpoint::"tenancy-pack-registry"
);

// =============================================================================
// FORBID — anonymous cannot read any non-public class
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class == "PUBLIC")
};

// =============================================================================
// FORBID — anonymous cannot read any customer-tenant data
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:self", "tenant:aggregate"])
};

// =============================================================================
// FORBID — anonymous cannot write anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"create_tenant",
    Action::"suspend_tenant",
    Action::"resume_tenant",
    Action::"delete_tenant",
    Action::"submit_dsr_request",
    Action::"write_metrics",
    Action::"write_logs",
    Action::"write_traces",
    Action::"alter_rls_policy",
    Action::"disable_rls",
    Action::"rotate_jwt_signing_key",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot enumerate tenants
// =============================================================================
//
// Even with read-only intent, anonymous principals MUST NOT enumerate
// tenant ids. The validate path requires a JWT; absent JWT means absent
// validation; the response shape never echoes the queried tenant_id back.

forbid (
  principal == User::"anonymous",
  action in [
    Action::"list_tenants",
    Action::"enumerate_tenants",
    Action::"validate_tenant_id_existence"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot probe DSR or RLS state
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_dsr_request",
    Action::"read_proof_of_erasure",
    Action::"read_rls_policy",
    Action::"read_rls_policy_live_state",
    Action::"read_audit_chain"
  ],
  resource
);
