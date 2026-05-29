// community µservice — public-read policy fragment
// Scope: anonymous read for spaces explicitly marked publication_visibility == "public".
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)
// Tenant must opt-in per-space; default is "private" (tenant members only).

forbid (
  principal,
  action,
  resource
);

// Anonymous principal reads from a public space
permit (
  principal == AnonymousPrincipal::"anonymous",
  action in [
    Action::"read_post",
    Action::"read_kb_article",
    Action::"read_thread",
    Action::"read_vote_tally"
  ],
  resource in CommunityResource::?r
)
when {
  resource has space_visibility &&
  resource.space_visibility == "public" &&
  resource has moderation_state &&
  resource.moderation_state == "approved"
};

// Anonymous reads of search are scoped to public-visible documents only
permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"read_search_results",
  resource in CommunityResource::?r
)
when {
  resource has space_visibility &&
  resource.space_visibility == "public"
};

// Anonymous never writes, votes, flags, or reads private spaces
forbid (
  principal == AnonymousPrincipal::"anonymous",
  action in [
    Action::"create_post",
    Action::"edit_own_post",
    Action::"create_kb_article",
    Action::"cast_vote",
    Action::"raise_flag",
    Action::"subscribe",
    Action::"read_moderation_action",
    Action::"read_audit_log"
  ],
  resource
);

// Anonymous never sees hidden / locked / pending-moderation content
forbid (
  principal == AnonymousPrincipal::"anonymous",
  action,
  resource
)
when {
  resource has moderation_state &&
  resource.moderation_state in ["hidden", "locked", "pending"]
};

// Anonymous reads observe tenant rate-limit (defense-in-depth at gateway)
forbid (
  principal == AnonymousPrincipal::"anonymous",
  action,
  resource
)
when {
  resource has anonymous_read_disabled &&
  resource.anonymous_read_disabled == true
};
