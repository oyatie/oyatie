// foundry-runtime µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema (future Slice D).
//
// Purpose: enforce per-tenant read/write scope on capability dispatch, session-state read,
//          invocation-history read, and capability-descriptor read served via
//          oya-intelligence-runtime-capability-executor-rest, SDK, and Workflow Studio surface.
// Owner:   ops-security + axis-foundry-runtime
// Audit:   every deny emits oya_foundry_runtime_unauthorized_attempt_total
// References: runtime-isolation.md §"Invariant TI-04 + TI-07"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator dispatches own capability invocation
// =============================================================================
//
// A principal bound to TenantOperator::"<tenant-id>" may dispatch a capability
// whose owning tenant matches their tenant claim, at-or-below their authorised
// autonomy tier ceiling.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"dispatch_capability",
    Action::"read_session",
    Action::"read_invocation_history",
    Action::"read_capability_descriptor",
    Action::"cancel_invocation"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — workload µservice dispatches on behalf of tenant principal
// =============================================================================
//
// workflow-engine and other internal µservices invoke capabilities on behalf
// of the tenant principal. The principal's tenant claim is bound at OIDC
// boundary and the workload µservice carries the principal context via the
// (workload_spiffe, on_behalf_of_tenant) tuple.

permit (
  principal in WorkloadService::?p,
  action == Action::"dispatch_capability",
  resource in TenantData::?r
)
when {
  principal has on_behalf_of_tenant &&
  resource has tenant_id &&
  principal.on_behalf_of_tenant == resource.tenant_id &&
  principal.workload_spiffe like "spiffe://oyatie/*/*"
};

// =============================================================================
// PERMIT 3 — tenant operator reads autonomy ceiling for own tenant
// =============================================================================
//
// Tenants may inspect their own autonomy tier ceiling (for capability-authoring
// UX). They MAY NOT read another tenant's ceiling.

permit (
  principal in TenantOperator::?p,
  action == Action::"read_autonomy_ceiling",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — tenant operator with DPA-authorised sensitive-data scope reads PHI traces
// =============================================================================
//
// pack-us-healthcare PHI session reads require explicit BAA + Cedar entitlement.
// pack-kr sensitive reads require recorded PIPA Art. 23 consent in OpenBao.

permit (
  principal in TenantOperator::?p,
  action == Action::"read_session",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has sensitive_data_entitlements &&
  resource has data_class &&
  resource.data_class in principal.sensitive_data_entitlements
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================
//
// Even if a permit somehow matched a cross-tenant request, this forbid blocks it.
// Cedar's deny-overrides semantics make this the safety net per ADR-0140 (retired per ADR-0145).

forbid (
  principal,
  action in [
    Action::"dispatch_capability",
    Action::"read_session",
    Action::"read_invocation_history",
    Action::"read_capability_descriptor",
    Action::"cancel_invocation",
    Action::"read_autonomy_ceiling"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
} unless {
  principal has on_behalf_of_tenant &&
  resource has tenant_id &&
  principal.on_behalf_of_tenant == resource.tenant_id
};

// =============================================================================
// FORBID — capability dispatch above tenant autonomy tier ceiling
// =============================================================================
//
// Per ADR-0022, dispatch is refused when the capability's declared tier exceeds
// the tenant's authorised ceiling. This is the policy-layer expression of the
// AutonomyGate invariant (runtime-isolation.md TI-08). Defence-in-depth alongside
// the in-code AutonomyGate; both must agree.

forbid (
  principal,
  action == Action::"dispatch_capability",
  resource in TenantData::?r
)
when {
  resource has declared_autonomy_level &&
  principal has autonomy_ceiling &&
  resource.declared_autonomy_level > principal.autonomy_ceiling
};

// =============================================================================
// FORBID — high-risk EU AI Act capability dispatch without notified-body engagement
// =============================================================================
//
// pack-eu high-risk Annex III capabilities cannot dispatch unless the tenant's
// notified-body engagement is recorded in the principal claim.

forbid (
  principal,
  action == Action::"dispatch_capability",
  resource in TenantData::?r
)
when {
  resource has eu_ai_act_class &&
  resource.eu_ai_act_class == "high_risk_annex_iii" &&
  resource has pack &&
  resource.pack == "pack-eu" &&
  !(principal has notified_body_engagement_id)
};

// =============================================================================
// FORBID — never expose SECRET-class data via tenant reads
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
