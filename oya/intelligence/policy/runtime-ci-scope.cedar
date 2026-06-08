// foundry-runtime µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the oya-ci reserved tenant + reserved system principals to runtime-internal
//          telemetry + invocation-lifecycle reads only; forbid customer-tenant data reads.
// Owner:   axis-foundry-runtime + ops-security + axis-foundry
// References: runtime-isolation.md §"Reserved tenant IDs" + §"Invariant TI-07"
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
// PERMIT 1 — CI lane reads runtime self-observability metrics
// =============================================================================
//
// CI principals (lane runners like oya-intelligence-runtime-load-latency) read the
// runtime's self-emitted metrics for promotion-readiness verdicts. They may
// not read customer-tenant per-session data.

permit (
  principal == Service::"oya-ci",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  (
    resource.metric_name like "oya_foundry_runtime_*" ||
    resource.metric_name like "oya_promotion_*"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT 2 — foundry-supervisor emits CapabilityRegistryUpdated events
// =============================================================================
//
// foundry-supervisor (SPIFFE: spiffe://oyatie/foundry/supervisor) is the sole
// authorised emitter of CapabilityRegistryUpdated targeting this runtime's mirror.

permit (
  principal == Service::"foundry-supervisor",
  action in [
    Action::"emit_capability_registry_update",
    Action::"write_capability_mirror_row"
  ],
  resource in CapabilityMirror::?r
);

// =============================================================================
// PERMIT 3 — runtime pods write self-observability metrics under oya-self
// =============================================================================

permit (
  principal in RuntimePod::?p,
  action == Action::"write_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-self" &&
  resource has metric_name &&
  resource.metric_name like "oya_foundry_runtime_*"
};

// =============================================================================
// PERMIT 4 — runtime pods emit invocation lifecycle events to audit-chain
// =============================================================================

permit (
  principal in RuntimePod::?p,
  action == Action::"emit_audit_chain_event",
  resource in AuditChainTopic::?t
)
when {
  resource has topic_name &&
  resource.topic_name in [
    "oya.foundry-runtime.invocation.started",
    "oya.foundry-runtime.invocation.step",
    "oya.foundry-runtime.invocation.completed",
    "oya.foundry-runtime.invocation.failed",
    "oya.foundry-runtime.invocation.cancelled",
    "oya.foundry-runtime.autonomy.violation",
    "oya.foundry-runtime.session.evicted"
  ]
};

// =============================================================================
// PERMIT 5 — tenancy µservice pushes TenantTierCeilingChanged events
// =============================================================================

permit (
  principal == Service::"tenancy",
  action == Action::"emit_tenant_tier_ceiling_changed",
  resource in TenantData::?r
);

// =============================================================================
// FORBID — CI principal cannot read any customer-tenant session or invocation
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_session",
    Action::"read_invocation_history",
    Action::"read_capability_descriptor"
  ],
  resource in TenantData::?r
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self", "tenant:oya-system"])
};

// =============================================================================
// FORBID — CI principal cannot write to non-reserved tenants
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"write_metrics",
    Action::"write_logs",
    Action::"dispatch_capability",
    Action::"write_capability_mirror_row"
  ],
  resource
)
unless {
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate"]
};

// =============================================================================
// FORBID — runtime pod cannot read customer-tenant data outside dispatch path
// =============================================================================
//
// Runtime pods access session-state + capability mirror via in-code adapters
// (which themselves observe tenant-prefix invariants). They MAY NOT directly
// read cross-tenant data through any side path (e.g., admin REST routes).

forbid (
  principal in RuntimePod::?p,
  action in [Action::"read_session", Action::"read_invocation_history"],
  resource in TenantData::?r
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self", "tenant:oya-system"]) &&
  !(context has bound_tenant_for_this_invocation) ||
  (context has bound_tenant_for_this_invocation && resource.tenant_id != context.bound_tenant_for_this_invocation)
};

// =============================================================================
// FORBID — sibling Foundry µservices cannot read runtime-internal session-state
// =============================================================================
//
// foundry-providers, foundry-guardrails, foundry-evidence, foundry-supervisor
// invoke runtime via mTLS but cannot themselves read runtime session-state.
// They receive the necessary scoped fields per invocation request.

forbid (
  principal in FoundrySibling::?s,
  action == Action::"read_session",
  resource in TenantData::?r
);
