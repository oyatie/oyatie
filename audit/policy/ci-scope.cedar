// audit-chain µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the ci reserved tenant + sealing-worker + retention-cascade-worker
//          to their authorised operations; forbid reads of customer tenant payloads.
// Owner:   axis-foundry + axis-audit-chain + ops-security
// References: threat-model.md T-E-01, T-E-02, T-E-03
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
// PERMIT 1 — emission-rest writes to WAL for any incoming SPIFFE-validated event
// =============================================================================

permit (
  principal == Service::"emission-rest",
  action in [Action::"wal_write", Action::"raw_blob_stage"],
  resource in AuditEvent::?e
)
when {
  resource has spiffe_validated &&
  resource.spiffe_validated == true
};

// =============================================================================
// PERMIT 2 — sealing-worker reads WAL + writes SealRecord + calls HSM
// =============================================================================
//
// The sealing-worker (SPIFFE: spiffe://oyatie/audit-chain/sealing-worker) is
// the sole authorised writer of SealRecord and the sole authorised caller of
// HSM signing per pack partition.

permit (
  principal == Service::"sealing-worker",
  action in [
    Action::"wal_read",
    Action::"merkle_build",
    Action::"hsm_sign",
    Action::"seal_record_write",
    Action::"raw_blob_write",
    Action::"root_publish"
  ],
  resource
)
when {
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack
};

// =============================================================================
// PERMIT 3 — retention-cascade-worker applies declared retention policy
// =============================================================================
//
// The worker enforces the policy from policy/retention-matrix.yaml; it does NOT
// have an operator-bypass path. Emergency redaction operations require a separate
// JIT-elevated principal (see PERMIT 5).

permit (
  principal == Service::"retention-cascade-worker",
  action in [Action::"retention_apply", Action::"soft_delete", Action::"hard_delete"],
  resource in AuditEvent::?e
)
when {
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack &&
  resource has retention_eligible &&
  resource.retention_eligible == true
};

// =============================================================================
// PERMIT 4 — verification-rest reads published roots + serves proofs
// =============================================================================

permit (
  principal == Service::"verification-rest",
  action in [Action::"root_read", Action::"proof_verify", Action::"keyresolver_lookup"],
  resource
);

// =============================================================================
// PERMIT 5 — JIT-elevated operator performs HSM admin / emergency redact
// =============================================================================
//
// HSM key generation, rotation, retirement, and destruction; emergency redact;
// out-of-cadence retention override. All require 2-person OpenBao JIT elevation
// (verified by Cedar via the principal's `elevated_until` claim being in the
// future + the principal having an `approved_by_principal` distinct from itself).

permit (
  principal in OperatorPrincipal::?op,
  action in [
    Action::"hsm_key_generate",
    Action::"hsm_key_rotate",
    Action::"hsm_key_retire",
    Action::"hsm_key_destroy",
    Action::"hsm_key_revoke",
    Action::"audit_emergency_redact",
    Action::"audit_retention_override"
  ],
  resource
)
when {
  principal has elevated_until &&
  context has now &&
  context.now <= principal.elevated_until &&
  principal has approved_by_principal &&
  principal.approved_by_principal != principal
};

// =============================================================================
// PERMIT 6 — slo-engine-worker reads audit-chain self-SLI from self tenant
// =============================================================================

permit (
  principal == Service::"slo-engine-worker",
  action == Action::"audit_chain_sli_read",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  resource.metric_name like "audit_chain_*" &&
  resource has tenant_id &&
  resource.tenant_id == "tenant:self"
};

// =============================================================================
// FORBID — emission-rest cannot read sealed records or other tenants' WAL
// =============================================================================

forbid (
  principal == Service::"emission-rest",
  action in [Action::"seal_record_read", Action::"wal_read"],
  resource
);

// =============================================================================
// FORBID — sealing-worker cannot read payloads outside its own pack
// =============================================================================

forbid (
  principal == Service::"sealing-worker",
  action,
  resource in AuditEvent::?e
)
unless {
  principal has bound_pack &&
  resource has pack &&
  resource.pack == principal.bound_pack
};

// =============================================================================
// FORBID — retention-cascade-worker cannot exceed the declared retention policy
// =============================================================================

forbid (
  principal == Service::"retention-cascade-worker",
  action in [Action::"soft_delete", Action::"hard_delete"],
  resource in AuditEvent::?e
)
unless {
  resource has retention_eligible &&
  resource.retention_eligible == true
};

// =============================================================================
// FORBID — verification-rest cannot mutate state
// =============================================================================

forbid (
  principal == Service::"verification-rest",
  action in [
    Action::"wal_write",
    Action::"seal_record_write",
    Action::"hsm_sign",
    Action::"retention_apply",
    Action::"audit_redact"
  ],
  resource
);

// =============================================================================
// FORBID — Operator JIT-elevated principals cannot bypass the 2-person rule
// =============================================================================
//
// approved_by_principal must differ from principal (the requester); cedar
// expression above enforces this at PERMIT 5 time. Repeat-forbid ensures no
// permit path accidentally allows self-approval.

forbid (
  principal in OperatorPrincipal::?op,
  action in [
    Action::"hsm_key_generate",
    Action::"hsm_key_rotate",
    Action::"hsm_key_retire",
    Action::"hsm_key_destroy",
    Action::"hsm_key_revoke",
    Action::"audit_emergency_redact",
    Action::"audit_retention_override"
  ],
  resource
)
when {
  principal has approved_by_principal &&
  principal.approved_by_principal == principal
};
