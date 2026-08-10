// social µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access. social DOES allow some
//          public reads (public-visibility posts + public profiles) because
//          social is a public-by-default platform when the user opts in.
//          This fragment authorises:
//            (a) public health / status probes
//            (b) public OAuth callback endpoints for personal-context onboarding
//            (c) public OpenAPI / AsyncAPI schema endpoints (no data)
//            (d) read of public-visibility posts + public profiles
//            (e) ActivityPub federation inbox (Professional-tier only)
// Owner:   ops-security + gtm-marketing + axis-social
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
// PERMIT 4 — anonymous read of public-visibility posts (server-rendered)
// =============================================================================
//
// social is a public-by-default platform; public-visibility posts are
// readable without auth (with caller-IP-scoped rate limits enforced
// at gateway). Personal-tier posts under Personal context are by
// default NOT public (user opt-in required); see policy/dual-context-isolation.md.

permit (
  principal == User::"anonymous",
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_profile",
    Action::"render_public_feed"
  ],
  resource in Post::?p
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource has anonymous_read_allowed &&
  resource.anonymous_read_allowed == true
};

// =============================================================================
// PERMIT 5 — ActivityPub federation inbox (Professional-tier only)
// =============================================================================
//
// External federation peer (HTTP Signatures verified) may POST to
// /federation/inbox. The peer is not "anonymous" in the trust sense
// but is unauthenticated from oyatie OIDC perspective. Verification
// happens at the federation-gateway layer; this Cedar permit allows
// the gateway to accept the request when HTTP Signature is valid.

permit (
  principal == User::"federation-peer",
  action == Action::"federation_inbox_deliver",
  resource in FederationEndpoint::?f
)
when {
  context has http_signature_valid &&
  context.http_signature_valid == true &&
  context has peer_allowlisted &&
  context.peer_allowlisted == true &&
  resource has accepts_personal_tier &&
  resource.accepts_personal_tier == false  // Personal-tier never federates
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
// FORBID — anonymous cannot read followers-only / list / private posts
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource in Post::?p
)
when {
  resource has visibility &&
  !(resource.visibility == "public")
};

// =============================================================================
// FORBID — anonymous cannot write anything (other than oauth callback)
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"publish_post",
    Action::"edit_post",
    Action::"delete_post",
    Action::"react_to_post",
    Action::"comment_on_post",
    Action::"follow",
    Action::"unfollow",
    Action::"block",
    Action::"mute",
    Action::"upload_media",
    Action::"open_ediscovery_hold",
    Action::"disclose_post_body",
    Action::"report_abuse"
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
// FORBID — anonymous cannot access age-attestation table
// =============================================================================

forbid (
  principal == User::"anonymous",
  action == Action::"read_age_attestation",
  resource
);

// =============================================================================
// FORBID — federation peer cannot deliver Personal-tier activities
// =============================================================================
//
// Compile-time invariant + Cedar belt-and-suspenders.

forbid (
  principal == User::"federation-peer",
  action == Action::"federation_inbox_deliver",
  resource
)
when {
  context has target_context_kind &&
  context.target_context_kind == "Personal"
};
