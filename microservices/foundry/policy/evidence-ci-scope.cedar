// foundry-evidence µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope what CI lanes are permitted to do against foundry-evidence.
// Owner:   ops-security + axis-governance
// Audit:   every deny emits oya_foundry_evidence_ci_denied_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI runner records to test-tenant fixtures only
// =============================================================================

permit (
  principal in CIRunner::?c,
  action == Action::"foundry_evidence_record",
  resource in EvidencePackEnvelope::?e
)
when {
  resource has tenant_id &&
  resource.tenant_id like "tenant:0000000000000000\\d+"  // test-tenant prefix
};

// =============================================================================
// PERMIT 2 — CI runner runs schema-validation reads (no tenant data)
// =============================================================================

permit (
  principal in CIRunner::?c,
  action in [
    Action::"foundry_evidence_schema_introspect",
    Action::"foundry_evidence_capability_dry_run",
    Action::"foundry_evidence_contract_test"
  ],
  resource
);

// =============================================================================
// PERMIT 3 — CI runner reloads policies in test partition only
// =============================================================================

permit (
  principal in CIRunner::?c,
  action == Action::"foundry_evidence_policy_reload",
  resource in PolicyPartition::?p
)
when {
  resource has partition &&
  resource.partition == "test"
};

// =============================================================================
// FORBID — CI runner reads production tenant data
// =============================================================================

forbid (
  principal in CIRunner::?c,
  action in [
    Action::"foundry_evidence_query",
    Action::"foundry_evidence_read_plaintext",
    Action::"foundry_evidence_read_pack_metadata"
  ],
  resource in EvidencePack::?e
)
unless {
  resource has tenant_id &&
  resource.tenant_id like "tenant:0000000000000000\\d+"
};

// =============================================================================
// FORBID — CI runner triggers regulator-export or retention-cascade
// =============================================================================

forbid (
  principal in CIRunner::?c,
  action in [
    Action::"foundry_evidence_regulator_export",
    Action::"foundry_evidence_retention_override",
    Action::"foundry_evidence_pack_rebuild",
    Action::"foundry_evidence_archive_migrate"
  ],
  resource
);
