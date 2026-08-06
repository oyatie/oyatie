// messenger µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope CI lane reads to promotion-readiness + conformance data only;
//          forbid reads of customer-tenant per-channel data.
// Owner:   axis-foundry + axis-messenger + ops-security
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
// PERMIT 1 — oya-ci reads promotion-eligibility verdicts for messenger
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  (
    resource.metric_name like "oya_promotion_*" ||
    resource.metric_name like "oya_messenger_internal_*"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT 2 — oya-ci reads HG-MESSENGER conformance signals
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_conformance",
  resource in ConformanceSignal::?c
)
when {
  resource has gate_id &&
  resource.gate_id == "HG-MESSENGER"
};

// =============================================================================
// PERMIT 3 — synthetic-probe tenant exists for messenger smoke tests
// =============================================================================
//
// A reserved tenant:oya-messenger-synthetic is used by oya-ci to drive
// end-to-end channel-create-and-post drills without polluting customer data.

permit (
  principal == Service::"oya-ci",
  action in [
    Action::"post_message",
    Action::"create_channel",
    Action::"delete_channel",
    Action::"read_message"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-messenger-synthetic"
};

// =============================================================================
// FORBID — oya-ci cannot read customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_message",
    Action::"read_thread",
    Action::"read_attachment",
    Action::"read_channel_metadata"
  ],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self", "tenant:oya-messenger-synthetic"])
};
