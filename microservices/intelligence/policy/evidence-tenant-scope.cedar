// foundry-evidence µservice — tenant-scope policy fragment
// Cedar v4
//
// Purpose: enforce per-tenant scope on record_invocation + evidence_query +
//          regulator_export reads.
// Owner:   ops-security + axis-foundry-evidence
// Audit:   every deny emits oya_foundry_evidence_unauthorized_attempt_total
// References: threat-model.md T-I-01, T-I-02, T-I-03, T-S-01, T-S-02;
//             tenant-scope analogue from audit-chain.
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
// PERMIT 1 — Foundry workload µservice records an invocation on its own tenant
// =============================================================================
//
// The recorder principal carries a SPIFFE identity which maps to a
// (microservice ∈ foundry-*, tenant) pair via the PrincipalResolver. The
// envelope's claimed tenant_id MUST match the SPIFFE-resolved tenant binding,
// and source_microservice MUST match SPIFFE-resolved microservice.

permit (
  principal in FoundryWorkloadRecorder::?w,
  action == Action::"foundry_evidence_record",
  resource in EvidencePackEnvelope::?e
)
when {
  principal has spiffe_id &&
  principal has bound_tenant_id &&
  principal has bound_microservice &&
  resource has tenant_id &&
  resource has source_microservice &&
  resource.tenant_id == principal.bound_tenant_id &&
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack &&
  resource.source_microservice == principal.bound_microservice
};

// =============================================================================
// PERMIT 2 — Tenant operator reads own tenant's evidence packs
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"foundry_evidence_query",
    Action::"foundry_evidence_read_payload_hash",
    Action::"foundry_evidence_read_pack_metadata"
  ],
  resource in EvidencePack::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — Tenant operator with sensitive-data entitlement reads plaintext
// =============================================================================
//
// PHI / PIPA Art. 23 / SECRET payload reads require explicit per-data-class
// entitlement issued by tenancy µservice at DPA / BAA signature.

permit (
  principal in TenantOperator::?p,
  action == Action::"foundry_evidence_read_plaintext",
  resource in EvidencePack::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has sensitive_data_entitlements &&
  resource has payload_data_class &&
  resource.payload_data_class in principal.sensitive_data_entitlements
};

// =============================================================================
// PERMIT 4 — Internal forensic principal reads cross-tenant (Sev-2 incident)
// =============================================================================
//
// ops-security forensic role; entitlement-gated; every read audit-emitted; only
// during an active declared incident.

permit (
  principal in InternalForensic::?p,
  action == Action::"foundry_evidence_query",
  resource in EvidencePack::?e
)
when {
  principal has active_incident_id &&
  principal has incident_scope_packs &&
  resource has pack &&
  resource.pack in principal.incident_scope_packs
};

// =============================================================================
// FORBID — Pack-pinning at record layer (load-bearing residency invariant)
// =============================================================================

forbid (
  principal in FoundryWorkloadRecorder::?w,
  action == Action::"foundry_evidence_record",
  resource in EvidencePackEnvelope::?e
)
unless {
  principal has bound_pack &&
  resource has pack &&
  resource.pack == principal.bound_pack
};

// =============================================================================
// FORBID — Cross-tenant query refusal (defence-in-depth)
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"foundry_evidence_query",
    Action::"foundry_evidence_read_payload_hash",
    Action::"foundry_evidence_read_pack_metadata",
    Action::"foundry_evidence_read_plaintext"
  ],
  resource in EvidencePack::?e
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — Direct tenant-operator record (must go through foundry-runtime)
// =============================================================================
//
// Tenant-operator-initiated recording would bypass SPIFFE source-attribution;
// real recording must originate from a SPIFFE foundry-* identity.

forbid (
  principal in TenantOperator::?p,
  action == Action::"foundry_evidence_record",
  resource
);

// =============================================================================
// FORBID — Tenant operators cannot trigger retention overrides or rebuilds
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"foundry_evidence_retention_override",
    Action::"foundry_evidence_pack_rebuild",
    Action::"foundry_evidence_archive_migrate",
    Action::"foundry_evidence_emergency_export"
  ],
  resource
);

// =============================================================================
// FORBID — Never expose SECRET-class plaintext to tenant operators
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action == Action::"foundry_evidence_read_plaintext",
  resource in EvidencePack::?e
)
when {
  resource has payload_data_class &&
  resource.payload_data_class == "SECRET"
};

// =============================================================================
// FORBID — Restricted (Art. 18 GDPR / KR PIPA equivalent) subject read
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"foundry_evidence_query",
    Action::"foundry_evidence_read_plaintext"
  ],
  resource in EvidencePack::?e
)
when {
  resource has subject_restricted &&
  resource.subject_restricted == true
};

// =============================================================================
// Notes (TS-09): constant-time padding on tenant-not-found vs tenant-empty
// is implemented in the rest-adapter layer; Cedar evaluation is itself
// constant-time per Cedar v4 contract.
