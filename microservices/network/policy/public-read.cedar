// network µservice — public-read policy fragment
// Cedar v4.2 LTS — default-deny per ADR-0140
//
// Purpose: govern anonymous / unauthenticated access. `network` allows some
//          public reads (public-visibility Professional profiles + public posts
//          + public Pages) because Professional networks are public-by-default
//          when the user opts in (LinkedIn / Xing precedent). This fragment
//          authorises:
//            (a) public health / status probes
//            (b) public OAuth callback endpoints for Professional onboarding
//            (c) public OpenAPI / AsyncAPI / proto schema endpoints (no data)
//            (d) read of public-visibility Professional posts + public profiles
//                + public Pages + public Events + public Skills taxonomy
//          network does NOT federate in P01, so there is no federation-inbox
//          permit (cf. social policy/public-read.cedar §PERMIT 5).
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)
// Owner:   ops-security + gtm-marketing + axis-network

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
// PERMIT 3 — anonymous OAuth callback for Professional onboarding
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"oauth_callback",
  resource == PublicEndpoint::"oauth-professional-onboard"
);

// =============================================================================
// PERMIT 4 — anonymous read of public-visibility Professional posts
// =============================================================================
//
// network is public-by-default platform; public-visibility Professional posts
// are readable without auth (with caller-IP-scoped rate limits enforced at
// gateway). Connection-only / group-only / private posts are NOT readable
// anonymously.

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
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has anonymous_read_allowed &&
  resource.anonymous_read_allowed == true
};

// =============================================================================
// PERMIT 5 — anonymous read of public Professional profile (Page or person)
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_profile",
  resource in Profile::?p
)
when {
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  resource has visibility &&
  resource.visibility == "public" &&
  (!(resource has is_protected) || resource.is_protected == false) &&
  (!(resource has minor_protect) || resource.minor_protect == false)
};

// =============================================================================
// PERMIT 6 — anonymous read of public Page + public Event + Skills taxonomy
// =============================================================================

permit (
  principal == User::"anonymous",
  action in [
    Action::"read_page",
    Action::"read_event",
    Action::"read_skill_taxonomy_entry"
  ],
  resource
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 7 — anonymous job-listing browse (public-visibility job postings)
// =============================================================================

permit (
  principal == User::"anonymous",
  action in [
    Action::"read_job_posting",
    Action::"search_jobs"
  ],
  resource in JobPosting::?j
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// FORBID — anonymous cannot read connection-graph, endorsements, recommendations
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_connection_graph",
    Action::"read_endorsement",
    Action::"read_recommendation",
    Action::"read_inmail_thread",
    Action::"read_skill_assessment",
    Action::"read_salary_insight"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot read followers-only / connections-only / private posts
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
    Action::"send_connection_request",
    Action::"follow",
    Action::"unfollow",
    Action::"block",
    Action::"restrict",
    Action::"disconnect",
    Action::"upload_media",
    Action::"upload_document",
    Action::"open_ediscovery_hold",
    Action::"disclose_inmail_body",
    Action::"report_abuse",
    Action::"endorse_skill",
    Action::"publish_recommendation",
    Action::"send_inmail",
    Action::"submit_skill_assessment",
    Action::"request_profile_verification",
    Action::"invoke_recruiter_search"
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
// FORBID — minor-account never anonymous-readable under any path
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has minor_protect &&
  resource.minor_protect == true
};

// =============================================================================
// FORBID — non-Professional context resources never anonymous-readable
// =============================================================================
//
// Belt-and-suspenders: network resources are all Professional-only by type
// (PCI-01), but this Cedar forbid catches any future regression.

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has context_kind &&
  resource.context_kind != "Professional"
};
