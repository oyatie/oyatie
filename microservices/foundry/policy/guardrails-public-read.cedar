// foundry-guardrails µservice — public-read policy fragment
// Cedar v4
//
// Purpose: define the SMALL set of anonymous-allowed read operations.
//          NO tenant data; NO secret data; NO decision data. Only schema
//          validation, public health probes, and competitive-parity-matrix
//          aggregates (DP-noise-protected for cross-tenant aggregates).
// Owner: axis-foundry-guardrails + ops-security
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
// PERMIT 1 — public schema validation (anyone can validate Cedar overlay YAML / JSON)
// =============================================================================
//
// Validates a candidate Cedar overlay against the schema; does NOT register or
// deploy. Returns schema errors only. No state mutation.

permit (
  principal,
  action == Action::"validate_cedar_overlay_schema",
  resource is SchemaValidationRequest
);

// =============================================================================
// PERMIT 2 — public health + readiness probes (k8s)
// =============================================================================

permit (
  principal,
  action in [Action::"health_probe", Action::"readiness_probe"],
  resource is ProbeEndpoint
);

// =============================================================================
// PERMIT 3 — public block-reason taxonomy (the closed enum used in REST responses)
// =============================================================================
//
// Tenant SDKs need to enumerate the BlockReason values; this is non-sensitive
// metadata published at /api/v1/block-reasons (anonymous).

permit (
  principal,
  action == Action::"read_block_reason_taxonomy",
  resource == EnumDescriptor::"BlockReason"
);

// =============================================================================
// FORBID — never expose anything else publicly
// =============================================================================

forbid (
  principal,
  action,
  resource
)
unless {
  context has authenticated &&
  context.authenticated == true
};
