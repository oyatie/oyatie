// messenger µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/messenger/policy/schema.cedarschema
//
// Purpose: enforce per-tenant read/write scope for messenger entities.
// Owner:   ops-security + axis-messenger
// Audit:   every deny emits oya_messenger_tenant_unauthorized_attempt_total
// References: dual-context-isolation.md, tenant-isolation conventions in observability
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
// PERMIT 1 — tenant operator reads own tenant's professional channels
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_channel",
    Action::"read_channel_metadata",
    Action::"list_channels",
    Action::"read_thread",
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
// PERMIT 2 — end-user reads/writes channels they are members of
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_message",
    Action::"post_message",
    Action::"react_message",
    Action::"edit_own_message",
    Action::"delete_own_message",
    Action::"read_thread",
    Action::"post_thread_reply",
    Action::"read_attachment",
    Action::"upload_attachment",
    Action::"read_presence",
    Action::"update_own_presence"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has channel_id &&
  principal has member_of_channels &&
  resource.channel_id in principal.member_of_channels
};

// =============================================================================
// PERMIT 3 — channel-admin manages channel ACL within their channel scope
// =============================================================================

permit (
  principal in ChannelAdmin::?a,
  action in [
    Action::"add_channel_member",
    Action::"remove_channel_member",
    Action::"set_channel_retention",
    Action::"set_channel_metadata"
  ],
  resource in Channel::?c
)
when {
  principal has channel_admin_of &&
  resource has channel_id &&
  resource.channel_id in principal.channel_admin_of
};

// =============================================================================
// PERMIT 4 — compliance-officer issues holds (requires entitlement)
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
// PERMIT 5 — four-eyes disclosure (paired action)
// =============================================================================
//
// Two distinct principals with disclosure entitlement must co-sign.
// Cedar evaluates per-attempt; the four-eyes pairing requires two
// distinct accepts within an audit-chain transaction.

permit (
  principal in DisclosureApprover::?d,
  action == Action::"disclose_message_body",
  resource in Message::?m
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
// FORBID — never permit disclosure of personal DMs
// =============================================================================
//
// Personal-context DMs are E2E-encrypted server-side; even with four-eyes,
// the body is unreadable. This Cedar block belt-and-suspenders rejects any
// attempt to invoke disclosure on a Personal resource.

forbid (
  principal,
  action == Action::"disclose_message_body",
  resource in Message::?m
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
    Action::"read_message",
    Action::"read_thread",
    Action::"read_channel",
    Action::"read_channel_metadata",
    Action::"list_channels",
    Action::"read_attachment",
    Action::"read_presence",
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
// FORBID — non-member read of channel content
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_message",
    Action::"read_thread",
    Action::"read_attachment"
  ],
  resource in Channel::?c
)
unless {
  principal has member_of_channels &&
  resource has channel_id &&
  resource.channel_id in principal.member_of_channels
};
