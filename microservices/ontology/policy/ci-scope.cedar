// ontology µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the oya-ci principal's access to schema validation +
//          governance reads; forbid customer-tenant Object Type reads.
// Owner:   axis-foundry + axis-ontology + ops-security
// References: type-isolation.md §"Reserved principals"
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
// PERMIT 1 — CI principal reads schema registry only
// =============================================================================
//
// The oya-governance-promotion-readiness CI lane (running as principal Service::"oya-ci")
// may read Object Type / Link Type / Action Type / Function Type schemas
// (not instances) for governance + cohesion validation.

permit (
  principal == Service::"oya-ci",
  action in [
    Action::"read_object_type_schema",
    Action::"read_link_type_schema",
    Action::"read_action_type_schema",
    Action::"read_function_type_schema",
    Action::"read_cedar_fragment",
    Action::"read_pillar_definition"
  ],
  resource is PolicyArtifact
);

// =============================================================================
// PERMIT 2 — schema-propagation-worker reads + writes schema registry
// =============================================================================
//
// The worker (SPIFFE: spiffe://oyatie/ontology/schema-propagation-worker)
// is the sole authorised writer for schema registry mutations originating
// from PR-reviewed schema definitions in microservices/ontology/specs/.

permit (
  principal == Service::"ontology-schema-propagation-worker",
  action in [
    Action::"read_object_type_schema",
    Action::"read_link_type_schema",
    Action::"read_action_type_schema",
    Action::"read_function_type_schema",
    Action::"write_object_type_schema",
    Action::"write_link_type_schema",
    Action::"write_action_type_schema",
    Action::"write_function_type_schema",
    Action::"emit_schema_registered_event"
  ],
  resource is SchemaArtifact
);

// =============================================================================
// PERMIT 3 — action-engine-worker reads its own action-type registry entries
// =============================================================================

permit (
  principal == Service::"ontology-action-engine-worker",
  action in [
    Action::"read_action_type_schema",
    Action::"read_cedar_fragment",
    Action::"emit_action_receipt"
  ],
  resource is ActionTypeArtifact
);

// =============================================================================
// PERMIT 4 — function-engine-worker reads function-type-registry + cedar
// =============================================================================

permit (
  principal == Service::"ontology-function-engine-worker",
  action in [
    Action::"read_function_type_schema",
    Action::"read_cedar_fragment"
  ],
  resource is FunctionTypeArtifact
);

// =============================================================================
// PERMIT 5 — audit-chain-worker reads outbox + writes audit chain
// =============================================================================

permit (
  principal == Service::"ontology-audit-chain-worker",
  action in [
    Action::"read_audit_outbox",
    Action::"write_audit_chain_node",
    Action::"emit_audit_chain_seal"
  ],
  resource is AuditChainArtifact
);

// =============================================================================
// FORBID — CI principal cannot read customer-tenant instances
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"read_function_result",
    Action::"read_action_receipt"
  ],
  resource in TenantData::?r
);

// =============================================================================
// FORBID — CI principal cannot write anything
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"write_object_type",
    Action::"write_link_type",
    Action::"invoke_action_type",
    Action::"write_object_type_schema",
    Action::"write_link_type_schema",
    Action::"write_action_type_schema",
    Action::"write_function_type_schema",
    Action::"write_cedar_fragment",
    Action::"emit_audit_chain_seal"
  ],
  resource
);

// =============================================================================
// FORBID — Workers cannot pivot beyond their declared scope
// =============================================================================
//
// Even with a worker SPIFFE identity, each worker's authority is bounded
// to its own resource type. A worker attempting actions on resources
// outside its declared scope is rejected.

forbid (
  principal in Service::"ontology-schema-propagation-worker",
  action,
  resource
)
unless {
  resource is SchemaArtifact ||
  resource is PolicyArtifact
};

forbid (
  principal in Service::"ontology-action-engine-worker",
  action,
  resource
)
unless {
  resource is ActionTypeArtifact ||
  resource is TenantData
};

forbid (
  principal in Service::"ontology-function-engine-worker",
  action,
  resource
)
unless {
  resource is FunctionTypeArtifact ||
  resource is TenantData
};

forbid (
  principal in Service::"ontology-audit-chain-worker",
  action,
  resource
)
unless {
  resource is AuditChainArtifact
};
