// community µservice — anonymity-mode: pseudonymous (Reddit-class)
// Cedar v4.2 LTS; default-deny.
//
// Purpose: gate posting and read actions where the principal posts under a
//          user-chosen pseudonymous handle. No employer affinity attestation
//          required. Handle is stable across posts (unlike fully-anonymous).
//          Karma, voting, and sub-community moderation apply.
//          This is the Reddit-class surface.
//
// Posting mode: pseudonymous
// Product surface: reddit-style (topic/sub-community forums; threaded comments;
//                  moderation tools; karma; NSFW gating; ranking algorithms)
//
// Owner:   axis-community + council-privacy
// Audit:   every permit/deny emits community_pseudonymous_policy_eval_total
// References:
//   - ADR-0243 (Cedar universal gate)
//   - ADR-0244 (tenant scoping primitive)
//   - ADR-0300 (whistleblower + press-freedom doctrine — see fully-anonymous mode)
//   - community/capabilities/reddit-mode.yaml
//   - community/ARCHITECTURE.md §community-as-4-mixture-product
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
//          (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// DEFAULT DENY — all actions refused unless explicit permit matches
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — pseudonymous member reads sub-community content (public subs)
// =============================================================================

permit (
  principal in PseudonymousMember::?p,
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_comment",
    Action::"read_feed",
    Action::"read_trending",
    Action::"read_subcommunity_sidebar",
    Action::"read_wiki_page"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has visibility &&
  (resource.visibility == "public" ||
   (resource.visibility == "members_only" &&
    principal has subcommunity_membership &&
    resource has subcommunity_id &&
    resource.subcommunity_id in principal.subcommunity_membership))
};

// =============================================================================
// PERMIT 2 — pseudonymous member publishes under stable handle
// =============================================================================

permit (
  principal in PseudonymousMember::?p,
  action in [
    Action::"publish_post",
    Action::"publish_comment",
    Action::"cast_vote",
    Action::"hard_delete_own_post",
    Action::"file_abuse_report",
    Action::"open_appeal",
    Action::"create_subcommunity",
    Action::"edit_wiki_page"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has pseudonymous_handle &&
  principal has handle_verified &&
  principal.handle_verified == true &&
  principal has posting_mode &&
  principal.posting_mode == "pseudonymous" &&
  principal has account_age_days &&
  principal.account_age_days >= 0
};

// =============================================================================
// PERMIT 3 — NSFW content read (requires age-gate + explicit tenant enablement)
// =============================================================================

permit (
  principal in PseudonymousMember::?p,
  action == Action::"read_nsfw_post",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has age_gate_cleared &&
  principal.age_gate_cleared == true &&
  resource has nsfw_tenant_enabled &&
  resource.nsfw_tenant_enabled == true &&
  resource has nsfw_flag &&
  resource.nsfw_flag == true
};

// =============================================================================
// PERMIT 4 — sub-community moderator acts on content within own community
// =============================================================================

permit (
  principal in SubcommunityModerator::?p,
  action in [
    Action::"moderate_hide",
    Action::"moderate_remove",
    Action::"moderate_quarantine",
    Action::"moderate_ban_from_subcommunity",
    Action::"moderate_lock_thread",
    Action::"moderate_pin",
    Action::"moderate_resolve_appeal",
    Action::"moderate_flair_post"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has subcommunity_moderator_for &&
  resource has subcommunity_id &&
  resource.subcommunity_id in principal.subcommunity_moderator_for
};

// =============================================================================
// FORBID — cross-tenant access (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — karma manipulation attempts
// =============================================================================

forbid (
  principal in PseudonymousMember::?p,
  action == Action::"cast_vote",
  resource in TenantData::?r
)
when {
  // Self-vote prohibition
  principal has handle_id &&
  resource has author_handle_id &&
  principal.handle_id == resource.author_handle_id
};

// =============================================================================
// FORBID — third-party tracker egress
// =============================================================================

forbid (
  principal,
  action in [
    Action::"emit_to_google_analytics",
    Action::"emit_to_segment",
    Action::"emit_to_mixpanel",
    Action::"emit_to_amplitude",
    Action::"emit_to_facebook_pixel",
    Action::"emit_to_heap",
    Action::"emit_to_fullstory"
  ],
  resource
);
