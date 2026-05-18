// ontology µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to non-sensitive
//          Ontology resources (e.g., the public Object Type schema browser,
//          public OpenAPI schema endpoint, public differential-privacy
//          aggregate dashboards).
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
// PERMIT 1 — anonymous read of platform-wide Object Type schema browser
// =============================================================================
//
// The public schema-browser lists registered Object Type schemas (names +
// property descriptors + tier classification) but NOT any instances. It
// helps prospective tenants understand the entity model before signing.

permit (
  principal == User::"anonymous",
  action == Action::"read_object_type_schema",
  resource in PublicArtifact::?r
)
when {
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has visibility &&
  resource.visibility == "platform-wide-schema"
};

// =============================================================================
// PERMIT 2 — anonymous read of OpenAPI specification (for SDK integration)
// =============================================================================
//
// The OpenAPI 3.2 specification for the Ontology REST surface is published
// at /api/v1/openapi.yaml; no tenant data; no authentication required.

permit (
  principal == User::"anonymous",
  action == Action::"read_openapi_spec",
  resource in PublicEndpoint::"openapi-spec"
);

// =============================================================================
// PERMIT 3 — anonymous read of differential-privacy aggregates
// =============================================================================
//
// Cross-tenant aggregates with DP-noise injection (ε ≤ 1) may be read by
// anonymous principals. These are marketing-suitable metrics like
// "total registered Object Types platform-wide" — never per-tenant attributable.

permit (
  principal == User::"anonymous",
  action == Action::"read_function_result",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-aggregate" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has dp_epsilon &&
  resource.dp_epsilon <= 1.0
};

// =============================================================================
// PERMIT 4 — anonymous schema validation (no submission persistence)
// =============================================================================
//
// The public schema-validation endpoint accepts an Object Type schema
// proposal and returns conformance feedback. Submissions are NOT persisted,
// logged, or transmitted off-host beyond synchronous validation.

permit (
  principal == User::"anonymous",
  action == Action::"validate_object_type_schema",
  resource in PublicEndpoint::"object-type-schema-validator"
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
  !(resource.tenant_id in ["tenant:oya-self", "tenant:oya-aggregate"])
};

// =============================================================================
// FORBID — anonymous cannot write anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"write_object_type",
    Action::"write_link_type",
    Action::"invoke_action_type",
    Action::"write_object_type_schema",
    Action::"write_link_type_schema",
    Action::"write_action_type_schema",
    Action::"write_function_type_schema",
    Action::"write_cedar_fragment",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot use the agent gateway
// =============================================================================
//
// LLM agent dispatch is forbidden to anonymous principals; agent JWT
// required.

forbid (
  principal == User::"anonymous",
  action == Action::"agent_tool_call",
  resource
);
