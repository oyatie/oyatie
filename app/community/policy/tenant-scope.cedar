// community µservice — tenant-scope policy fragment
// Cedar v4; conforms to app/community/policy/schema.cedarschema
//
// Purpose: enforce per-tenant read + write scope on community surface objects.
// Owner:   ops-security + axis-community
// Audit:   every deny emits oya_community_tenant_unauthorized_total
// References: community-isolation.md §"Invariant CI-06"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant member reads own tenant's posts + KB articles + votes
// =============================================================================

permit (
  principal in TenantMember::?p,
  action in [
    Action::"read_post",
    Action::"read_kb_article",
    Action::"read_vote_tally",
    Action::"read_thread",
    Action::"read_search_results",
    Action::"read_mention_resolution"
  ],
  resource in CommunityResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant member writes own posts + KB articles + votes
// =============================================================================

permit (
  principal in TenantMember::?p,
  action in [
    Action::"create_post",
    Action::"edit_own_post",
    Action::"delete_own_post",
    Action::"create_kb_article",
    Action::"edit_own_kb_article",
    Action::"cast_vote",
    Action::"raise_flag",
    Action::"subscribe",
    Action::"unsubscribe"
  ],
  resource in CommunityResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  (
    !(resource has author_id) ||
    principal.member_id == resource.author_id ||
    action in [Action::"create_post", Action::"create_kb_article", Action::"cast_vote", Action::"raise_flag", Action::"subscribe", Action::"unsubscribe"]
  )
};

// =============================================================================
// PERMIT 3 — tenant moderator takes moderation actions within own tenant
// =============================================================================

permit (
  principal in TenantModerator::?p,
  action in [
    Action::"hide_post",
    Action::"lock_thread",
    Action::"pin_post",
    Action::"unpin_post",
    Action::"unhide_post",
    Action::"unlock_thread",
    Action::"merge_threads",
    Action::"move_post",
    Action::"resolve_flag",
    Action::"escalate_flag"
  ],
  resource in CommunityResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has moderator_role &&
  principal.moderator_role == true
};

// =============================================================================
// PERMIT 4 — tenant_admin destructive actions (two-eyes enforced in usecase)
// =============================================================================

permit (
  principal in TenantAdmin::?p,
  action in [
    Action::"delete_post",
    Action::"purge_kb_article",
    Action::"ban_member",
    Action::"unban_member",
    Action::"promote_to_moderator",
    Action::"demote_from_moderator"
  ],
  resource in CommunityResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has admin_role &&
  principal.admin_role == true
};

// =============================================================================
// FORBID — defence-in-depth cross-tenant deny
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
// FORBID — never expose SECRET-class data via tenant reads
// =============================================================================

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
// FORBID — banned members may not write or vote
// =============================================================================

forbid (
  principal,
  action in [
    Action::"create_post",
    Action::"edit_own_post",
    Action::"create_kb_article",
    Action::"edit_own_kb_article",
    Action::"cast_vote",
    Action::"raise_flag"
  ],
  resource
)
when {
  principal has banned &&
  principal.banned == true
};
