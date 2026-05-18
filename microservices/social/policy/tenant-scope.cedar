// social µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/social/policy/schema.cedarschema
//
// Purpose: enforce per-tenant read/write scope for social entities.
// Owner:   ops-security + axis-social
// Audit:   every deny emits oya_social_tenant_unauthorized_attempt_total
// References: dual-context-isolation.md, tenant-isolation conventions in observability + messenger
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
// PERMIT 1 — tenant operator reads own tenant's professional posts + profiles
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_profile",
    Action::"read_profile_metadata",
    Action::"list_profiles",
    Action::"read_post",
    Action::"read_post_metadata",
    Action::"read_follow_graph_aggregate",
    Action::"read_audit_chain"
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
// PERMIT 2 — end-user reads / writes own profile + own posts
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_own_profile",
    Action::"update_own_profile",
    Action::"publish_own_post",
    Action::"edit_own_post",
    Action::"delete_own_post",
    Action::"react_to_post",
    Action::"comment_on_post",
    Action::"repost",
    Action::"quote_post",
    Action::"follow",
    Action::"unfollow",
    Action::"block",
    Action::"mute",
    Action::"bookmark",
    Action::"create_list",
    Action::"add_to_list",
    Action::"remove_from_list",
    Action::"render_own_feed",
    Action::"search_people",
    Action::"search_content",
    Action::"upload_media",
    Action::"report_abuse",
    Action::"open_appeal"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — end-user reads other users' public posts + profiles
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_profile",
    Action::"read_post",
    Action::"read_thread"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has visibility &&
  resource.visibility == "public"
};

// =============================================================================
// PERMIT 4 — end-user reads followers-only posts when they are a follower
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_post",
    Action::"read_thread"
  ],
  resource in Post::?p
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has visibility &&
  resource.visibility == "followers" &&
  resource has author_ref &&
  principal has follows &&
  resource.author_ref in principal.follows
};

// =============================================================================
// PERMIT 5 — tenant-moderator reads only flagged content
// =============================================================================
//
// Moderators can act only on content surfaced via AbuseReportFiled or
// classifier verdict; they cannot enumerate posts at large.

permit (
  principal in TenantModerator::?m,
  action in [
    Action::"read_post",
    Action::"read_moderation_context",
    Action::"issue_moderation_verdict",
    Action::"resolve_appeal"
  ],
  resource in Post::?p
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has flagged &&
  resource.flagged == true
};

// =============================================================================
// PERMIT 6 — compliance-officer issues holds (Professional-tier only)
// =============================================================================

permit (
  principal in ComplianceOfficer::?o,
  action in [
    Action::"open_ediscovery_hold",
    Action::"close_ediscovery_hold",
    Action::"list_held_resources"
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
// PERMIT 7 — four-eyes disclosure (Professional-tier only; paired action)
// =============================================================================

permit (
  principal in DisclosureApprover::?d,
  action == Action::"disclose_post_body",
  resource in Post::?p
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
// FORBID — never permit disclosure of Personal-tier posts
// =============================================================================

forbid (
  principal,
  action == Action::"disclose_post_body",
  resource in Post::?p
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
    Action::"read_post",
    Action::"read_thread",
    Action::"read_profile",
    Action::"read_profile_metadata",
    Action::"list_profiles",
    Action::"read_media",
    Action::"read_audit_chain"
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
// end-user personal posts even within their tenant.

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
// FORBID — non-follower read of followers-only posts
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_post",
    Action::"read_thread"
  ],
  resource in Post::?p
)
when {
  resource has visibility &&
  resource.visibility == "followers"
}
unless {
  principal has follows &&
  resource has author_ref &&
  resource.author_ref in principal.follows
};

// =============================================================================
// FORBID — non-list-member read of list-visibility posts
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_post",
    Action::"read_thread"
  ],
  resource in Post::?p
)
when {
  resource has visibility &&
  resource.visibility == "list"
}
unless {
  principal has list_memberships &&
  resource has visibility_list_id &&
  resource.visibility_list_id in principal.list_memberships
};

// =============================================================================
// FORBID — blocked users cannot read posts of blocker
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_profile",
    Action::"follow"
  ],
  resource
)
when {
  principal has principal_id &&
  resource has blocked_principals &&
  principal.principal_id in resource.blocked_principals
};

// =============================================================================
// FORBID — age-attestation table access restricted
// =============================================================================
//
// Minor-list privacy: only age_verification_reader entitlement can read.

forbid (
  principal,
  action == Action::"read_age_attestation",
  resource in AgeAttestation::?a
)
unless {
  principal has entitlements &&
  "age_verification_reader" in principal.entitlements
};
