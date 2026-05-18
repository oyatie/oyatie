// meet µservice — CI-scope policy fragment
// Cedar v4.2
//
// Purpose: scope CI lane reads to promotion-readiness + conformance data only;
//          forbid reads of customer-tenant per-meeting data.
// Owner:   axis-foundry + axis-meet + ops-security
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
// PERMIT 1 — oya-ci reads promotion-eligibility verdicts for meet
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
    resource.metric_name like "oya_meet_internal_*"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT 2 — oya-ci reads HG-MEET conformance signals
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_conformance",
  resource in ConformanceSignal::?c
)
when {
  resource has gate_id &&
  resource.gate_id == "HG-MEET"
};

// =============================================================================
// PERMIT 3 — synthetic-probe tenant for meet smoke tests
// =============================================================================
//
// A reserved tenant:oya-meet-synthetic is used by oya-ci to drive
// end-to-end create-room + join + record drills without polluting customer data.

permit (
  principal == Service::"oya-ci",
  action in [
    Action::"create_meeting_room",
    Action::"start_meeting_instance",
    Action::"join_meeting",
    Action::"start_recording",
    Action::"end_meeting_instance",
    Action::"read_recording"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-meet-synthetic"
};

// =============================================================================
// FORBID — oya-ci cannot read customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_recording",
    Action::"read_transcript",
    Action::"read_meeting_room_metadata"
  ],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self", "tenant:oya-meet-synthetic"])
};
