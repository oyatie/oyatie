// shorts µservice — tenant-scope policy fragment
// Cedar v4.2 LTS; conforms to microservices/shorts/policy/schema.cedarschema
//
// Purpose: enforce per-tenant read/write scope for shorts entities.
// Owner:   ops-security + axis-shorts
// Audit:   every deny emits oya_shorts_tenant_unauthorized_attempt_total
// References: dual-context-isolation.md, observability + social + messenger tenant-scope patterns

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's Professional videos + metadata
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_video_metadata",
    Action::"list_videos",
    Action::"read_audio_track",
    Action::"read_audit_chain",
    Action::"read_moderation_verdict_history",
    Action::"read_copyright_claim_history",
    Action::"read_creator_analytics_aggregate"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 2 — creator (end-user) reads / writes own videos + metadata + analytics
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"upload_video",
    Action::"publish_video",
    Action::"edit_video",
    Action::"delete_video",
    Action::"repost_stitch",
    Action::"repost_duet",
    Action::"like_video",
    Action::"comment_on_video",
    Action::"share_video",
    Action::"render_own_feed",
    Action::"render_for_you_feed",
    Action::"watch_video",
    Action::"read_video",
    Action::"upload_caption_manual",
    Action::"report_abuse",
    Action::"open_appeal",
    Action::"file_counter_notice",
    Action::"read_own_creator_analytics",
    Action::"read_own_repeat_infringer_status"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — viewer reads public-visibility videos
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_video",
    Action::"watch_video",
    Action::"render_public_feed"
  ],
  resource in Video::?v
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has visibility &&
  resource.visibility == "public"
};

// =============================================================================
// PERMIT 4 — tenant-moderator reads only flagged content
// =============================================================================

permit (
  principal in TenantModerator::?m,
  action in [
    Action::"read_video",
    Action::"read_moderation_context",
    Action::"issue_moderation_verdict",
    Action::"resolve_appeal"
  ],
  resource in Video::?v
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has flagged &&
  resource.flagged == true
};

// =============================================================================
// PERMIT 5 — compliance-officer issues holds + DMCA takedowns (Professional only)
// =============================================================================

permit (
  principal in ComplianceOfficer::?o,
  action in [
    Action::"open_ediscovery_hold",
    Action::"close_ediscovery_hold",
    Action::"list_held_resources",
    Action::"execute_dmca_takedown",
    Action::"flag_repeat_infringer"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "ediscovery_hold" in principal.entitlements &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 6 — four-eyes disclosure (Professional-tier only; paired action)
// =============================================================================

permit (
  principal in DisclosureApprover::?d,
  action == Action::"disclose_video_body",
  resource in Video::?v
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "disclosure_approver" in principal.entitlements &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  context has paired_approver_id &&
  context.paired_approver_id != principal.principal_id
};

// =============================================================================
// PERMIT 7 — rights-holder files DMCA claim (verified-business actor)
// =============================================================================

permit (
  principal in RightsHolder::?r,
  action == Action::"file_copyright_claim",
  resource in Video::?v
)
when {
  principal has business_verified &&
  principal.business_verified == true &&
  context has perjury_attestation &&
  context.perjury_attestation == true
};

// =============================================================================
// PERMIT 8 — minor (with parental-link) reads age-appropriate content
// =============================================================================
//
// Minor account: chronological-only feed; no algorithmic-recommendation;
// no adult-classified content surfacing; DM-restricted.

permit (
  principal in MinorEndUser::?m,
  action in [
    Action::"read_video",
    Action::"watch_video",
    Action::"render_chronological_feed",
    Action::"like_video",
    Action::"comment_on_video"
  ],
  resource in Video::?v
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has visibility &&
  resource.visibility == "public" &&
  resource has age_classification &&
  resource.age_classification == "general_audience" &&
  principal has age_bracket &&
  resource has age_min_required &&
  principal.age_bracket >= resource.age_min_required
};

// =============================================================================
// FORBID — never permit disclosure of Personal-tier videos
// =============================================================================

forbid (
  principal,
  action == Action::"disclose_video_body",
  resource in Video::?v
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};

// =============================================================================
// FORBID — cross-tenant access (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_video",
    Action::"watch_video",
    Action::"read_audit_chain",
    Action::"read_video_metadata",
    Action::"list_videos",
    Action::"read_moderation_verdict_history",
    Action::"read_copyright_claim_history",
    Action::"read_creator_analytics_aggregate"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — SECRET-class data never via tenant reads
// =============================================================================
//
// Per-content DRM keys + per-tenant DEKs are SECRET; never disclose.

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
// FORBID — tenant-admin / operator never reads Personal-tier resources
// =============================================================================
//
// Personal-tier privacy invariant: tenant operators must NOT pivot into
// end-user personal videos even within their tenant.

forbid (
  principal in TenantOperator::?p,
  action,
  resource
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};

// =============================================================================
// FORBID — minor accounts cannot access age-classified-adult content
// =============================================================================

forbid (
  principal in MinorEndUser::?m,
  action,
  resource in Video::?v
)
when {
  resource has age_classification &&
  resource.age_classification in ["mature", "adult", "restricted"]
};

// =============================================================================
// FORBID — minor accounts disabled by default from algorithmic-For-You ranking
// =============================================================================

forbid (
  principal in MinorEndUser::?m,
  action == Action::"render_for_you_feed",
  resource
)
unless {
  principal has algorithmic_recommendation_consented &&
  principal.algorithmic_recommendation_consented == true
};

// =============================================================================
// FORBID — minor accounts: DM-restricted by default
// =============================================================================

forbid (
  principal in MinorEndUser::?m,
  action == Action::"share_video_to_dm",
  resource
)
unless {
  principal has dm_unrestricted &&
  principal.dm_unrestricted == true
};

// =============================================================================
// FORBID — DRM-protected video playback without valid EME license
// =============================================================================

forbid (
  principal in EndUser::?u,
  action == Action::"watch_video",
  resource in Video::?v
)
when {
  resource has drm_protected &&
  resource.drm_protected == true
}
unless {
  context has eme_license_valid &&
  context.eme_license_valid == true &&
  context has license_bound_device_match &&
  context.license_bound_device_match == true
};

// =============================================================================
// FORBID — DMCA takedown subject videos cannot be replayed
// =============================================================================

forbid (
  principal,
  action in [Action::"read_video", Action::"watch_video"],
  resource in Video::?v
)
when {
  resource has dmca_taken_down &&
  resource.dmca_taken_down == true
}
unless {
  resource has counter_notice_resolved &&
  resource.counter_notice_resolved == true &&
  resource has restoration_window_elapsed &&
  resource.restoration_window_elapsed == true
};

// =============================================================================
// FORBID — age-attestation table access restricted
// =============================================================================

forbid (
  principal,
  action == Action::"read_age_attestation",
  resource in AgeAttestation::?a
)
unless {
  principal has entitlements &&
  "age_verification_reader" in principal.entitlements
};

// =============================================================================
// FORBID — parental-link table access restricted
// =============================================================================

forbid (
  principal,
  action == Action::"read_parental_link",
  resource in ParentalLink::?l
)
unless {
  principal has entitlements &&
  "parental_link_reader" in principal.entitlements
};

// =============================================================================
// FORBID — fingerprint corpus writes restricted to ops-legal
// =============================================================================

forbid (
  principal,
  action == Action::"write_fingerprint_corpus",
  resource
)
unless {
  principal has entitlements &&
  "fingerprint_corpus_writer" in principal.entitlements
};
