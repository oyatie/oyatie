// foundry-runtime µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to non-sensitive aggregated
//          runtime indicators (public status page; capability template catalog
//          for the oya-system reserved tenant).
// Owner:   ops-security + gtm-marketing
// References: threat-model.md T-I-05; dpia.md §"Cross-tenant aggregations"
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
// PERMIT 1 — anonymous read of platform status (oya-self tenant, PUBLIC class)
// =============================================================================
//
// The public status page reads aggregated runtime availability metrics from the
// oya-self tenant. These are platform-wide health indicators (e.g., "runtime
// capacity utilisation by hour") with no per-customer attribution.

permit (
  principal == User::"anonymous",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-self" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has metric_name &&
  resource.metric_name like "oya_foundry_runtime_public_*"
};

// =============================================================================
// PERMIT 2 — anonymous list of oya-system capability templates (catalog browse)
// =============================================================================
//
// The oya-system reserved tenant carries oyatie-owned capability templates.
// Anonymous browsing is permitted (similar to GitHub Marketplace listings)
// for the descriptor surface (name, description, declared_autonomy_level, purpose-of-use,
// eu_ai_act_class, regulatory_packs_consumed). The actual capability invocation
// path requires authenticated principal.

permit (
  principal == User::"anonymous",
  action == Action::"read_capability_descriptor",
  resource in TenantData::?r
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-system" &&
  resource has data_class &&
  resource.data_class == "PUBLIC"
};

// =============================================================================
// PERMIT 3 — anonymous schema-validate a capability descriptor (no persistence)
// =============================================================================
//
// The public schema-validation endpoint accepts a capability-descriptor YAML and
// returns schema-conformance feedback. It does NOT persist the submission and
// does NOT reveal whether dependencies resolve in the caller's tenant.

permit (
  principal == User::"anonymous",
  action == Action::"validate_capability_descriptor_schema",
  resource in PublicEndpoint::"capability-descriptor-validator"
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
  !(resource.tenant_id in ["tenant:oya-self", "tenant:oya-aggregate", "tenant:oya-system"])
};

// =============================================================================
// FORBID — anonymous cannot dispatch any capability
// =============================================================================
//
// Dispatch always requires authenticated principal carrying a tenant_id claim.

forbid (
  principal == User::"anonymous",
  action in [
    Action::"dispatch_capability",
    Action::"cancel_invocation",
    Action::"write_metrics",
    Action::"write_capability_mirror_row",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous validation endpoint does not echo or persist submitted descriptor
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_artifact",
    Action::"write_artifact",
    Action::"delete_artifact"
  ],
  resource
);
