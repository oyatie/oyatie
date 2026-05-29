// ontology µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/ontology/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-tenant read scope on Object Types, Link Types,
//          Function results, Action receipts, and audit-chain entries.
// Owner:   ops-security + axis-ontology
// Audit:   every deny emits oya_ontology_tenant_unauthorized_read_attempt_total
// References: type-isolation.md §"Invariant TI-07"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every read is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's Object/Link/Function/Action data
// =============================================================================
//
// A principal bound to TenantOperator::"<tenant-id>" may read any
// resource owned by the same tenant. Scope gated on tenant_id claim
// match + max_sensitivity_level ceiling + pillar_kind match.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"read_function_result",
    Action::"read_action_receipt",
    Action::"read_audit_chain",
    Action::"read_object_type_schema",
    Action::"read_link_type_schema",
    Action::"read_action_type_schema",
    Action::"read_function_type_schema"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has property_sensitivity_level &&
  principal has max_sensitivity_level &&
  principal.max_sensitivity_level >= resource.property_sensitivity_level &&
  resource has pillar_kind &&
  principal has pillar_kind &&
  (
    principal.pillar_kind == resource.pillar_kind ||
    (
      principal has cross_pillar_grant &&
      resource.pillar_kind in principal.cross_pillar_grant.allowed_pillars &&
      context has now &&
      principal.cross_pillar_grant.expires_at >= context.now
    )
  )
};

// =============================================================================
// PERMIT 2 — tenant operator writes own Object Types + invokes Actions
// =============================================================================
//
// Write authority: same-tenant, same-pillar (or cross-pillar grant), and
// the writer's max_sensitivity_level ceiling must permit the property sensitivity level they're
// writing.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"write_object_type",
    Action::"write_link_type",
    Action::"invoke_action_type"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has property_sensitivity_level &&
  principal has max_sensitivity_level &&
  principal.max_sensitivity_level >= resource.property_sensitivity_level &&
  resource has pillar_kind &&
  principal has pillar_kind &&
  (
    principal.pillar_kind == resource.pillar_kind ||
    (
      principal has cross_pillar_grant &&
      resource.pillar_kind in principal.cross_pillar_grant.allowed_pillars &&
      context has now &&
      principal.cross_pillar_grant.expires_at >= context.now
    )
  )
};

// =============================================================================
// PERMIT 3 — DSR cascade runner reads every Object Type within scoped tenant
// =============================================================================
//
// The dsr-cascade-runner principal needs read access across every Object
// Type table to find subject identifiers. Tenant scope still enforced;
// pillar boundary bypassed only for the DSR action.

permit (
  principal == Service::"ontology-dsr-cascade-runner",
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"dsr_tombstone_object"
  ],
  resource in TenantData::?r
)
when {
  context has dsr_request_id &&
  resource has tenant_id &&
  context has scoped_tenant_id &&
  resource.tenant_id == context.scoped_tenant_id
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================
//
// Even if a permit accidentally matched a cross-tenant request,
// this forbid blocks it. Cedar's deny-overrides semantics make this
// the safety net.

forbid (
  principal,
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"read_function_result",
    Action::"read_action_receipt",
    Action::"read_audit_chain",
    Action::"write_object_type",
    Action::"write_link_type",
    Action::"invoke_action_type"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
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

// =============================================================================
// FORBID — never permit writes that exceed the principal's max_sensitivity_level
// =============================================================================
//
// A principal carrying max_sensitivity_level = Tier2Restricted cannot write a property
// declared Tier1Sensitive even within their own tenant.

forbid (
  principal,
  action in [
    Action::"write_object_type",
    Action::"write_link_type",
    Action::"invoke_action_type"
  ],
  resource
)
when {
  resource has property_sensitivity_level &&
  principal has max_sensitivity_level &&
  principal.max_sensitivity_level < resource.property_sensitivity_level
};

// =============================================================================
// FORBID — cross-tenant Link Type creation refused unless explicit grant
// =============================================================================
//
// A Link Type's src + dst Object Types each carry a tenant_id. Cross-tenant
// links are forbidden unless the principal carries a CrossTenantLinkGrant
// for both tenants within the grant's data_class_cap.

forbid (
  principal,
  action == Action::"write_link_type",
  resource is LinkInstance
)
when {
  resource has src_tenant_id &&
  resource has dst_tenant_id &&
  resource.src_tenant_id != resource.dst_tenant_id
}
unless {
  principal has cross_tenant_link_grant &&
  resource.src_tenant_id in principal.cross_tenant_link_grant.tenants &&
  resource.dst_tenant_id in principal.cross_tenant_link_grant.tenants &&
  resource has property_sensitivity_level &&
  principal.cross_tenant_link_grant.data_class_cap >= resource.property_sensitivity_level &&
  context has now &&
  principal.cross_tenant_link_grant.expires_at >= context.now
};
