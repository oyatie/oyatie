// shorts µservice — public-read policy fragment
// Cedar v4.2 LTS
//
// Purpose: govern anonymous / unauthenticated access. shorts allows public
//          reads of public-visibility videos by default (consistent with
//          short-form-video industry norms) but enforces:
//          (a) anonymous reads are subject to age-classification gating
//          (b) DRM-protected videos refuse anonymous playback
//          (c) DMCA-taken-down videos refuse anonymous playback
//          (d) Personal-tier videos never readable anonymously
//          (e) public OAuth + public OpenAPI / AsyncAPI / proto endpoints OK
// Owner:   ops-security + gtm-marketing + axis-shorts
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
// PERMIT 1 — anonymous read of public health / status indicators
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_health",
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind in ["liveness", "readiness", "public-status"]
};

// =============================================================================
// PERMIT 2 — anonymous read of API schema artifacts
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_artifact",
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind in ["openapi-schema", "asyncapi-schema", "proto-descriptor"]
};

// =============================================================================
// PERMIT 3 — anonymous OAuth callback for personal-onboarding
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"oauth_callback",
  resource == PublicEndpoint::"oauth-personal-onboard"
);

// =============================================================================
// PERMIT 4 — anonymous read of public-visibility videos (general-audience only)
// =============================================================================
//
// shorts is a public-by-default platform; public-visibility general-audience
// videos are readable without auth (with caller-IP-scoped rate limits at
// gateway). Personal-tier videos NEVER readable anonymously even with
// public visibility (Personal-tier requires authenticated viewer).
// Adult / mature / restricted videos NEVER readable anonymously
// (regulatory floor per EU AVMSD + UK OSA + KR 청소년 보호법).

permit (
  principal == User::"anonymous",
  action in [
    Action::"read_video",
    Action::"watch_video",
    Action::"render_public_feed"
  ],
  resource in Video::?v
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has age_classification &&
  resource.age_classification == "general_audience" &&
  resource has anonymous_read_allowed &&
  resource.anonymous_read_allowed == true &&
  resource has dmca_taken_down &&
  resource.dmca_taken_down == false &&
  resource has drm_protected &&
  resource.drm_protected == false
};

// =============================================================================
// PERMIT 5 — anonymous OpenSearch / hashtag / sound discovery (metadata-only)
// =============================================================================

permit (
  principal == User::"anonymous",
  action in [
    Action::"search_hashtag",
    Action::"search_sound",
    Action::"search_public_video"
  ],
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind == "public-search-metadata"
};

// =============================================================================
// FORBID — anonymous cannot read Personal-context resources
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};

// =============================================================================
// FORBID — anonymous cannot read non-public videos
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource in Video::?v
)
when {
  resource has visibility &&
  !(resource.visibility == "public")
};

// =============================================================================
// FORBID — anonymous cannot read age-classified-mature / adult / restricted videos
// =============================================================================
//
// EU AVMSD 2018/1808 Art. 28b(2) + UK Online Safety Act 2023 + KR 청소년 보호법
// + COPPA require age-gating before any mature content surfaces.

forbid (
  principal == User::"anonymous",
  action,
  resource in Video::?v
)
when {
  resource has age_classification &&
  resource.age_classification in ["mature", "adult", "restricted"]
};

// =============================================================================
// FORBID — anonymous cannot watch DRM-protected videos
// =============================================================================
//
// DRM-protected videos require EME license bound to viewer device; anonymous
// cannot acquire valid EME license.

forbid (
  principal == User::"anonymous",
  action == Action::"watch_video",
  resource in Video::?v
)
when {
  resource has drm_protected &&
  resource.drm_protected == true
};

// =============================================================================
// FORBID — anonymous cannot watch DMCA-taken-down videos
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [Action::"read_video", Action::"watch_video"],
  resource in Video::?v
)
when {
  resource has dmca_taken_down &&
  resource.dmca_taken_down == true
};

// =============================================================================
// FORBID — anonymous cannot write anything (other than oauth callback)
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"upload_video",
    Action::"publish_video",
    Action::"edit_video",
    Action::"delete_video",
    Action::"like_video",
    Action::"comment_on_video",
    Action::"share_video",
    Action::"repost_stitch",
    Action::"repost_duet",
    Action::"file_copyright_claim",
    Action::"file_counter_notice",
    Action::"execute_dmca_takedown",
    Action::"disclose_video_body",
    Action::"report_abuse",
    Action::"open_appeal",
    Action::"open_ediscovery_hold",
    Action::"issue_drm_license"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot access non-public data classes
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class == "PUBLIC")
};

// =============================================================================
// FORBID — anonymous cannot access age-attestation / parental-link tables
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_age_attestation",
    Action::"read_parental_link"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot access fingerprint corpus
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_fingerprint_corpus",
    Action::"write_fingerprint_corpus"
  ],
  resource
);
