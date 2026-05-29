// social µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope CI lane reads to promotion-readiness + conformance data only;
//          forbid reads of customer-tenant per-user social data.
// Owner:   axis-foundry + axis-social + ops-security
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
// PERMIT 1 — oya-ci reads promotion-eligibility verdicts for social
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
    resource.metric_name like "oya_social_internal_*"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT 2 — oya-ci reads HG-SOCIAL conformance signals
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_conformance",
  resource in ConformanceSignal::?c
)
when {
  resource has gate_id &&
  resource.gate_id == "HG-SOCIAL"
};

// =============================================================================
// PERMIT 3 — synthetic-probe tenant for social smoke tests
// =============================================================================
//
// A reserved tenant:oya-community-social-synthetic is used by oya-ci to drive
// end-to-end profile-and-post drills without polluting customer data.

permit (
  principal == Service::"oya-ci",
  action in [
    Action::"publish_post",
    Action::"edit_post",
    Action::"delete_post",
    Action::"follow",
    Action::"unfollow",
    Action::"react_to_post",
    Action::"render_own_feed",
    Action::"read_post"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-community-social-synthetic"
};

// =============================================================================
// FORBID — oya-ci cannot read customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_media",
    Action::"read_profile",
    Action::"read_follow_graph_aggregate"
  ],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self", "tenant:oya-community-social-synthetic"])
};

// =============================================================================
// FORBID — oya-ci can never read age-attestation table
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action == Action::"read_age_attestation",
  resource
);
