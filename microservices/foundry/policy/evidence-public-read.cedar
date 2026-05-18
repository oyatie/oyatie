// foundry-evidence µservice — public-read policy fragment
// Cedar v4
//
// Purpose: define the (very small) public-read surface — transparency of
//          schema versions + claim-matrix introspection per ADR-0133.
//          Tenant-private data is NEVER public-read.
// Owner:   ops-security
// Audit:   every public-read emits oya_foundry_evidence_public_read_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — Anonymous read of schema versions + claim-matrix metadata
// =============================================================================

permit (
  principal,
  action in [
    Action::"foundry_evidence_read_schema_version",
    Action::"foundry_evidence_read_claim_matrix",
    Action::"foundry_evidence_read_framework_profiles",
    Action::"foundry_evidence_read_slo_targets",
    Action::"foundry_evidence_read_capability_catalog"
  ],
  resource in PublicEndpoint::?r
);

// =============================================================================
// FORBID — Public-read of tenant data, ever
// =============================================================================

forbid (
  principal,
  action in [
    Action::"foundry_evidence_query",
    Action::"foundry_evidence_read_payload_hash",
    Action::"foundry_evidence_read_pack_metadata",
    Action::"foundry_evidence_read_plaintext"
  ],
  resource
)
unless {
  principal has spiffe_id || principal has tenant_token
};

// =============================================================================
// FORBID — Public-read of regulator-export bundles
// =============================================================================

forbid (
  principal,
  action in [
    Action::"foundry_evidence_read_regulator_export_metadata",
    Action::"foundry_evidence_regulator_export"
  ],
  resource
)
unless {
  principal has engagement_id || principal has tenant_id
};
