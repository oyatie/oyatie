// cloud-secrets µservice — public-read policy fragment
// Cedar v4; conforms to schema.cedarschema
//
// Purpose: enumerate the (very small) surface that is publicly readable
//          without authentication. cloud-secrets is a high-sensitivity
//          µservice; almost nothing is public.
//
// Owner:   ops-security + axis-cloud-secrets
// References: secret-isolation.md §"TI-06 Default-deny"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — public health probes (k8s liveness/readiness — no payload)
// =============================================================================

permit (
  principal == Anonymous::"public",
  action in [
    Action::"get_health",
    Action::"get_ready"
  ],
  resource in HealthEndpoint::?r
);

// =============================================================================
// PERMIT 2 — public OpenAPI + AsyncAPI + Proto contract surface
// =============================================================================
//
// Contracts are themselves INTERNAL_ONLY content (no raw secrets);
// publishing the spec for client SDK generation is acceptable.

permit (
  principal == Anonymous::"public",
  action == Action::"read_contract",
  resource in Contract::?r
)
when {
  resource has visibility &&
  resource.visibility == "public"
};

// =============================================================================
// PERMIT 3 — public docs (PRD, ADR pointers, glossary)
// =============================================================================

permit (
  principal == Anonymous::"public",
  action == Action::"read_doc",
  resource in PublicDoc::?r
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource has data_class &&
  resource.data_class != "SECRET" &&
  resource.data_class != "AUDIT" &&
  resource.data_class != "SENSITIVE_PIPA_ART23"
};

// =============================================================================
// FORBID — no public read of any secret, audit event, policy fragment,
//          rotation state, namespace metadata, or attestation report
// =============================================================================

forbid (
  principal == Anonymous::"public",
  action in [
    Action::"resolve_secret_reference",
    Action::"list_secret_references",
    Action::"read_rotation_policy",
    Action::"read_namespace_metadata",
    Action::"read_attestation_report",
    Action::"read_audit_chain_export",
    Action::"read_policy_fragment",
    Action::"read_iac_manifest",
    Action::"hsm_sign",
    Action::"upload_byok"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous principals cannot read anything class > INTERNAL_ONLY
// =============================================================================

forbid (
  principal == Anonymous::"public",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class != "PUBLIC" &&
  resource.data_class != "INTERNAL_ONLY"
};
