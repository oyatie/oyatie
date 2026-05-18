// shorts µservice — CI-scope policy fragment
// Cedar v4.2 LTS
//
// Purpose: scope CI lane reads to promotion-readiness + conformance data only;
//          forbid reads of customer-tenant per-user shorts data.
// Owner:   axis-foundry + axis-shorts + ops-security

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — oya-ci reads promotion-eligibility verdicts for shorts
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
    resource.metric_name like "oya_shorts_internal_*"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT 2 — oya-ci reads HG-SHORTS conformance signals
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_conformance",
  resource in ConformanceSignal::?c
)
when {
  resource has gate_id &&
  resource.gate_id == "HG-SHORTS"
};

// =============================================================================
// PERMIT 3 — synthetic-probe tenant for shorts smoke tests
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action in [
    Action::"upload_video",
    Action::"publish_video",
    Action::"edit_video",
    Action::"delete_video",
    Action::"like_video",
    Action::"watch_video",
    Action::"render_own_feed",
    Action::"file_copyright_claim",
    Action::"file_counter_notice",
    Action::"open_appeal",
    Action::"render_chronological_feed"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-shorts-synthetic"
};

// =============================================================================
// FORBID — oya-ci cannot read customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_video",
    Action::"watch_video",
    Action::"read_video_metadata",
    Action::"read_audio_track",
    Action::"read_creator_analytics_aggregate"
  ],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self", "tenant:oya-shorts-synthetic"])
};

// =============================================================================
// FORBID — oya-ci can never read age-attestation or parental-link tables
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_age_attestation",
    Action::"read_parental_link"
  ],
  resource
);

// =============================================================================
// FORBID — oya-ci can never write to fingerprint corpus
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action == Action::"write_fingerprint_corpus",
  resource
);

// =============================================================================
// FORBID — oya-ci can never issue DRM licenses
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action == Action::"issue_drm_license",
  resource
);
