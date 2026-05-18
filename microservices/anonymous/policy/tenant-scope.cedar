// anonymous µservice — tenant-scope policy fragment
// Cedar v4.2 LTS; default-deny.
//
// Purpose: enforce per-tenant + per-affinity read/write scope on posts, votes,
//          attestations, notifications. NEVER permits cross-tenant reads.
// Owner:   axis-anonymous + ops-security
// Audit:   every deny emits oya_anonymous_tenant_unauthorized_attempt_total
// References: PRD-anonymous §"Security", I1, I2; ADR-0117 tenant-isolation

// =============================================================================
// Default deny — every action is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant member reads own tenant's posts within own affinity scope
// =============================================================================

permit (
  principal in AnonymousMember::?p,
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_comment",
    Action::"read_feed",
    Action::"read_trending",
    Action::"read_hashtags"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has affinity_bindings &&
  resource has affinity_cluster_id &&
  resource.affinity_cluster_id in principal.affinity_bindings
};

// =============================================================================
// PERMIT 2 — tenant member publishes a post bound to a verified affinity
// =============================================================================

permit (
  principal in AnonymousMember::?p,
  action in [
    Action::"publish_post",
    Action::"publish_comment",
    Action::"cast_vote",
    Action::"hard_delete_own_post",
    Action::"file_abuse_report",
    Action::"open_appeal"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has blinded_credential_present &&
  principal.blinded_credential_present == true &&
  principal has affinity_attestation_verified &&
  principal.affinity_attestation_verified == true &&
  resource has affinity_cluster_id &&
  resource.affinity_cluster_id in principal.affinity_bindings
};

// =============================================================================
// PERMIT 3 — tenant moderator acts on flagged content within own tenant
// =============================================================================

permit (
  principal in TenantModerator::?p,
  action in [
    Action::"moderate_hide",
    Action::"moderate_quarantine",
    Action::"moderate_revoke_affinity_credential",
    Action::"moderate_lock_thread",
    Action::"moderate_pin",
    Action::"moderate_resolve_appeal"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has moderator_entitlement &&
  principal.moderator_entitlement == true
};

// =============================================================================
// PERMIT 4 — anonymous-DM read/write under MLS group membership only
// =============================================================================
//
// MLS group membership is verified at the messenger µservice; this policy
// permits the platform to route ciphertexts but NEVER permits plaintext access.

permit (
  principal in AnonymousMember::?p,
  action in [
    Action::"send_dm_ciphertext",
    Action::"read_dm_ciphertext",
    Action::"join_mls_group",
    Action::"leave_mls_group"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has mls_group_membership &&
  resource has mls_group_id &&
  resource.mls_group_id in principal.mls_group_membership
};

// =============================================================================
// FORBID — cross-tenant access (defence-in-depth, deny-overrides)
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
// FORBID — never permit platform-side identity correlation (I1)
// =============================================================================
//
// This is the structural guarantee: no Cedar permit ever authorises the
// platform to correlate user_id ↔ post_id. Only legal-process-disclosure.cedar
// can authorise that, and only under dual-control + court-order conditions.

forbid (
  principal,
  action in [
    Action::"correlate_user_to_post",
    Action::"reveal_post_author",
    Action::"link_blinded_commitment_to_user_id",
    Action::"export_blinded_credential_unblinded"
  ],
  resource
);

// =============================================================================
// FORBID — Ontology Person writes from anonymous (I1 + I2 enforcement)
// =============================================================================
//
// The anonymous µservice MUST NEVER write a Person Ontology entity. Affinity
// writes are permitted via affinity-attestation.cedar; Person is structurally
// refused here.

forbid (
  principal,
  action in [
    Action::"ontology_write_person",
    Action::"ontology_write_user_identity",
    Action::"ontology_link_handle_to_person"
  ],
  resource
);

// =============================================================================
// FORBID — third-party tracker egress (I4)
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
