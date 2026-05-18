// messenger µservice — channel-scope policy fragment
// Cedar v4
//
// Purpose: enforce channel-level RBAC + dual-context isolation +
//          retention/hold invariants at every read/write of a Channel
//          or Message resource.
// Owner:   axis-messenger + ops-security + council-privacy
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny (defence-in-depth FORBID)
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — channel members read messages + channel metadata
// =============================================================================
//
// A principal who is a current member of the channel (membership lookup at
// authz time) can read channel + messages within their context_kind.

permit (
  principal,
  action in [
    Action::"read_channel",
    Action::"list_messages",
    Action::"read_message",
    Action::"list_thread_replies",
    Action::"list_members"
  ],
  resource
)
when {
  resource has channel_id &&
  principal has memberships &&
  resource.channel_id in principal.memberships &&
  resource has context_kind &&
  principal has active_context &&
  resource.context_kind == principal.active_context
};

// =============================================================================
// PERMIT 2 — channel members post messages, reactions, thread replies
// =============================================================================

permit (
  principal,
  action in [
    Action::"post_message",
    Action::"add_reaction",
    Action::"remove_reaction",
    Action::"post_thread_reply",
    Action::"upload_attachment",
    Action::"mark_read"
  ],
  resource
)
when {
  resource has channel_id &&
  principal has memberships &&
  resource.channel_id in principal.memberships &&
  resource has context_kind &&
  principal has active_context &&
  resource.context_kind == principal.active_context &&
  // No hold-active blocks; hold suspends only purge, not post
  true
};

// =============================================================================
// PERMIT 3 — author edits/deletes own message within edit window
// =============================================================================

permit (
  principal,
  action in [Action::"edit_message", Action::"delete_message"],
  resource
)
when {
  resource has author_ref &&
  principal has user_ref &&
  resource.author_ref == principal.user_ref &&
  resource has posted_at &&
  context has now &&
  // Edit window: 24h after post
  (context.now - resource.posted_at) < 86400
};

// =============================================================================
// PERMIT 4 — channel admin grants/revokes membership + archives
// =============================================================================

permit (
  principal,
  action in [
    Action::"add_channel_member",
    Action::"remove_channel_member",
    Action::"change_member_role",
    Action::"archive_channel",
    Action::"set_retention_policy"
  ],
  resource
)
when {
  resource has channel_id &&
  principal has admin_channels &&
  resource.channel_id in principal.admin_channels
};

// =============================================================================
// PERMIT 5 — channel creation by tenant member
// =============================================================================

permit (
  principal,
  action == Action::"create_channel",
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has can_create_channel &&
  principal.can_create_channel == true
};

// =============================================================================
// FORBID — cross-context routing
// =============================================================================
//
// Invariant DCI-02 + DCI-06: a Personal-context principal cannot act on a
// Professional resource and vice versa.

forbid (
  principal,
  action,
  resource
)
when {
  resource has context_kind &&
  principal has active_context &&
  resource.context_kind != principal.active_context
};

// =============================================================================
// FORBID — non-member reads / writes
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_channel",
    Action::"list_messages",
    Action::"read_message",
    Action::"post_message"
  ],
  resource
)
when {
  resource has channel_id &&
  principal has memberships &&
  !(resource.channel_id in principal.memberships)
};

// =============================================================================
// FORBID — bots cannot post to channels that haven't installed them
// =============================================================================

forbid (
  principal in BotPrincipal::?b,
  action in [Action::"post_message", Action::"add_reaction"],
  resource
)
when {
  resource has channel_id &&
  principal has installed_channels &&
  !(resource.channel_id in principal.installed_channels)
};

// =============================================================================
// FORBID — destructive ops on archived channels
// =============================================================================

forbid (
  principal,
  action in [
    Action::"post_message",
    Action::"edit_message",
    Action::"add_reaction"
  ],
  resource
)
when {
  resource has archived_at &&
  resource.archived_at != null
};
