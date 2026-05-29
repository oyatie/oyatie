// messenger µservice — personal-DM-scope policy fragment
// Cedar v4
//
// Purpose: enforce E2E posture + admin-disclosure prohibition for personal-context
//          DirectConversation resources. Decoupled from channel-scope.cedar
//          (Professional channels) per dual-context-isolation.md DCI-06.
// Owner:   axis-messenger + council-privacy + ops-security
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
// PERMIT 1 — DM peers read + write their conversation
// =============================================================================

permit (
  principal,
  action in [
    Action::"read_dm",
    Action::"list_dm_messages",
    Action::"post_dm_message",
    Action::"upload_dm_attachment",
    Action::"add_dm_reaction"
  ],
  resource in DirectConversation::?d
)
when {
  resource has peers &&
  principal has user_ref &&
  principal.user_ref in resource.peers
};

// =============================================================================
// PERMIT 2 — author edits/deletes own DM message within edit window
// =============================================================================

permit (
  principal,
  action in [Action::"edit_dm_message", Action::"delete_dm_message"],
  resource
)
when {
  resource has author_ref &&
  principal has user_ref &&
  resource.author_ref == principal.user_ref &&
  resource has posted_at &&
  context has now &&
  (context.now - resource.posted_at) < 86400
};

// =============================================================================
// FORBID — tenant-admin disclosure on personal DM
// =============================================================================
//
// Per parallel ADR-0238: personal-context DMs are never disclosed to tenant
// admin, oyatie operator, or auditor. They are E2E and stored as ciphertext
// only. There is no four-eyes path for Personal context.

forbid (
  principal,
  action in [
    Action::"disclose_dm_body",
    Action::"admin_decrypt_dm",
    Action::"read_dm_plaintext"
  ],
  resource
);

// =============================================================================
// FORBID — non-peer reads
// =============================================================================

forbid (
  principal,
  action,
  resource in DirectConversation::?d
)
when {
  resource has peers &&
  principal has user_ref &&
  !(principal.user_ref in resource.peers)
};

// =============================================================================
// FORBID — context drift (a Professional principal acting on a Personal DM)
// =============================================================================

forbid (
  principal,
  action,
  resource in DirectConversation::?d
)
when {
  principal has active_context &&
  principal.active_context != "Personal"
};

// =============================================================================
// FORBID — eDiscovery hold cannot reach personal DMs
// =============================================================================

forbid (
  principal,
  action in [
    Action::"apply_ediscovery_hold",
    Action::"export_for_ediscovery"
  ],
  resource in DirectConversation::?d
);

// =============================================================================
// FORBID — server-side capability execution on personal-DM bodies
// =============================================================================
//
// T1/T2 capabilities (summary, classify, translate) cannot run server-side
// on personal DMs because plaintext is unavailable. They MAY run client-side
// in the SDK. This rule is the defence-in-depth check.

forbid (
  principal in CapabilityExecutor::?c,
  action in [
    Action::"summarize",
    Action::"classify",
    Action::"translate",
    Action::"extract_action_items"
  ],
  resource in DirectConversation::?d
);
