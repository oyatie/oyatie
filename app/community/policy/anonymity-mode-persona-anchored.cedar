// community µservice — anonymity-mode: persona-anchored (TeamBlind-class)
// Cedar v4.2 LTS; default-deny.
//
// Purpose: gate posting and read actions where the principal posts under a
//          VERIFIED-EMPLOYER-ANONYMOUS persona. The platform attests employer
//          affinity (via OIDC/SAML/ZK-proof) but the post author is hidden
//          behind a rotating handle. The employer + role is visible; the user_id
//          is NEVER disclosed except under legal-process-disclosure.cedar.
//          This is the TeamBlind-class surface.
//
// Posting mode: persona-anchored
// Product surface: teamblind-style (verified-employer anonymous posting;
//                  persona = verified employer + role + anonymous handle;
//                  cross-employer industry channels)
//
// Owner:   axis-community + ops-security + council-privacy
// Audit:   every permit/deny emits community_persona_anchored_policy_eval_total
// References:
//   - ADR-0243 (Cedar universal gate)
//   - ADR-0244 (tenant scoping primitive)
//   - ADR-0300 (whistleblower + press-freedom doctrine — see fully-anonymous mode)
//   - ADR-ANON-0001 (cryptographic blinding protocol — migrated from anonymous/)
//   - ADR-ANON-0002 (affinity attestation verification — migrated from anonymous/)
//   - community/capabilities/teamblind-mode.yaml
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
// PERMIT 1 — affinity-verified member reads own tenant's content within
//            own affinity scope (employer cluster)
// =============================================================================

permit (
  principal in PersonaAnchoredMember::?p,
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_comment",
    Action::"read_feed",
    Action::"read_trending",
    Action::"read_industry_channel"
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
// PERMIT 2 — affinity-verified member publishes under blinded persona
// =============================================================================
//
// The blinded credential proves employer affinity WITHOUT revealing user_id.
// The affinity_attestation_verified flag is set only by the affinity-attestation
// BC after successful OIDC/SAML/ZK-proof verification.

permit (
  principal in PersonaAnchoredMember::?p,
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
  resource.affinity_cluster_id in principal.affinity_bindings &&
  principal has posting_mode &&
  principal.posting_mode == "persona-anchored"
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
// FORBID — platform-side identity correlation (structural I1 guarantee)
// =============================================================================
//
// The platform NEVER correlates user_id ↔ post_id in persona-anchored mode.
// Only legal-process-disclosure.cedar can authorise that under dual-control
// + court-order conditions.

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
// FORBID — Ontology Person writes (persona-anchored must not produce
//          identity-linkable graph nodes)
// =============================================================================

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
