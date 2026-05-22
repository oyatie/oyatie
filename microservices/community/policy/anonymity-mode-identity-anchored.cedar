// community µservice — anonymity-mode: identity-anchored (LinkedIn-class)
// Cedar v4.2 LTS; default-deny.
//
// Purpose: gate posting and profile actions where the principal's real identity
//          IS bound to every action — the LinkedIn-class surface. No blinded
//          credentials; the user's verified professional identity is the posting
//          identity. Privacy invariants here are about data-minimisation and
//          cross-tenant isolation, not identity concealment.
//
// Posting mode: identity-anchored
// Product surface: linkedin-style (verified professional identity + endorsements
//                  + connection graph + job board + posts + articles + recruiting)
//
// Owner:   axis-community + council-privacy
// Audit:   every permit/deny emits oya_community_identity_anchored_policy_eval_total
// References:
//   - ADR-0243 (Cedar universal gate)
//   - ADR-0244 (tenant scoping primitive)
//   - ADR-0300 (whistleblower + press-freedom doctrine — NOT applicable in this mode)
//   - community/capabilities/linkedin-mode.yaml
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
// PERMIT 1 — verified professional reads own tenant's content
// =============================================================================

permit (
  principal in VerifiedProfessional::?p,
  action in [
    Action::"read_post",
    Action::"read_article",
    Action::"read_comment",
    Action::"read_feed",
    Action::"read_connection_graph",
    Action::"read_endorsements",
    Action::"read_job_listing",
    Action::"read_profile"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has identity_verified &&
  principal.identity_verified == true
};

// =============================================================================
// PERMIT 2 — verified professional publishes identity-anchored content
// =============================================================================

permit (
  principal in VerifiedProfessional::?p,
  action in [
    Action::"publish_post",
    Action::"publish_article",
    Action::"publish_comment",
    Action::"send_connection_request",
    Action::"endorse_skill",
    Action::"apply_to_job",
    Action::"post_job_listing",
    Action::"update_profile"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has identity_verified &&
  principal.identity_verified == true &&
  principal has posting_mode &&
  principal.posting_mode == "identity-anchored"
};

// =============================================================================
// PERMIT 3 — recruiter reads candidate profiles (scoped to active job context)
// =============================================================================

permit (
  principal in Recruiter::?p,
  action in [
    Action::"read_profile",
    Action::"read_connection_graph",
    Action::"read_endorsements"
  ],
  resource in CandidateProfile::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has recruiter_entitlement &&
  principal.recruiter_entitlement == true &&
  resource has visibility &&
  resource.visibility in ["public", "connections", "open_to_work"]
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
// FORBID — identity-anchored mode CANNOT switch to blinded credentials
// =============================================================================
//
// Once a session is identity-anchored, blinded-credential issuance is refused.
// Switching posting mode requires a new session initiation with explicit
// consent and re-authentication.

forbid (
  principal in VerifiedProfessional::?p,
  action in [
    Action::"issue_blinded_credential",
    Action::"publish_post_anonymous",
    Action::"publish_post_pseudonymous",
    Action::"publish_post_persona_anchored"
  ],
  resource
);

// =============================================================================
// FORBID — third-party tracker egress (data-minimisation)
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
