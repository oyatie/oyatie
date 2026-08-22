// audit-chain µservice — tenant-scope policy fragment
// Cedar v4
//
// Purpose: enforce per-tenant scope on emission + query + verification + export reads.
// Owner:   ops-security + axis-audit-chain
// Audit:   every deny emits audit_chain_unauthorized_attempt_total
// References: threat-model.md T-I-01, T-S-01; tenant-isolation analogue from observability
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
// PERMIT 1 — Workload µservice emits AuditEvent bound to its own tenant
// =============================================================================
//
// The emitter principal carries a SPIFFE identity which maps to a (microservice,
// tenant) pair via the PrincipalResolver. The emission's claimed tenant_id
// in the payload MUST match the SPIFFE-resolved tenant binding.

permit (
  principal in WorkloadEmitter::?w,
  action == Action::"audit_emit",
  resource in AuditEvent::?e
)
when {
  principal has spiffe_id &&
  principal has bound_tenant_id &&
  resource has tenant_id &&
  resource has source_microservice &&
  resource.tenant_id == principal.bound_tenant_id &&
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack
};

// =============================================================================
// PERMIT 2 — Tenant operator reads own tenant's audit events
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"audit_query",
    Action::"audit_verify",
    Action::"audit_export_request",
    Action::"audit_read_proof"
  ],
  resource in AuditEvent::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — Tenant operator with DPA-recorded sensitive-data entitlement
// =============================================================================
//
// pack-us-healthcare PHI events require BAA; pack-kr sensitive PIPA Art. 23 events
// require explicit consent. The principal's sensitive_data_entitlements list is
// populated at tenant onboarding per the DPA / BAA.

permit (
  principal in TenantOperator::?p,
  action == Action::"audit_query",
  resource in AuditEvent::?e
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
// FORBID — Pack-pinning at emission layer
// =============================================================================
//
// An emitter bound to pack-X cannot emit events with pack-Y label. This is the
// load-bearing residency invariant.

forbid (
  principal in WorkloadEmitter::?w,
  action == Action::"audit_emit",
  resource in AuditEvent::?e
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
    Action::"audit_query",
    Action::"audit_verify",
    Action::"audit_export_request",
    Action::"audit_read_proof"
  ],
  resource in AuditEvent::?e
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — Tenant operators cannot directly emit (must go through tenancy proxy)
// =============================================================================
//
// Direct external emission by tenant operators would bypass the SPIFFE-based
// source-attribution; tenant-originated emissions must go through the tenancy
// µservice which mediates with its own SPIFFE identity.

forbid (
  principal in TenantOperator::?p,
  action == Action::"audit_emit",
  resource
);

// =============================================================================
// FORBID — Never expose SECRET-class data via tenant reads
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action,
  resource in AuditEvent::?e
)
when {
  resource has payload_data_class &&
  resource.payload_data_class == "SECRET"
};

// =============================================================================
// FORBID — Tenants cannot trigger retention or key-rotation operations
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"audit_retention_override",
    Action::"audit_key_rotate",
    Action::"audit_redact",
    Action::"audit_emergency_export"
  ],
  resource
);
