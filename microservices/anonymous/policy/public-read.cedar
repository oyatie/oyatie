// anonymous µservice — public-read policy fragment
// Cedar v4.2 LTS; default-deny.
//
// Purpose: define what is publicly readable WITHOUT authentication. The
//          answer for anonymous is "almost nothing" — by design.
//          Posts are tenant + affinity-scoped; even unauthenticated reads are
//          refused.
//
//          The only public surface is:
//          - The quarterly transparency report (aggregate only)
//          - The competitor-parity-matrix.md and threat-model.md sections
//            tagged as PUBLIC_TENANT_DISCLOSURE
//          - The age-gate refusal page (a Section 230 / COPPA disclosure)
// Owner:   axis-anonymous + gtm + council-privacy

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — anonymous unauth read of transparency-report (public good)
// =============================================================================

permit (
  principal == AnonymousUnauthenticated::"public",
  action == Action::"read_transparency_report",
  resource in TransparencyReport::?r
)
when {
  resource has publication_status &&
  resource.publication_status == "published" &&
  resource has aggregate_only &&
  resource.aggregate_only == true
};

// =============================================================================
// PERMIT — anonymous unauth read of public legal disclosures (e.g., DSA Art. 14
//          mechanism documentation)
// =============================================================================

permit (
  principal == AnonymousUnauthenticated::"public",
  action == Action::"read_public_disclosure",
  resource in PublicDisclosure::?r
)
when {
  resource has disclosure_tag &&
  resource.disclosure_tag in [
    "DSA_Art_14_complaint_mechanism",
    "DSA_Art_17_statement_of_reasons_template",
    "section_230_provider_status",
    "coppa_age_gate_refusal",
    "competitor_parity_matrix_public_summary",
    "ai_act_art_50_transparency_disclosure"
  ]
};

// =============================================================================
// FORBID — every other resource (defence-in-depth)
// =============================================================================
//
// Posts, comments, votes, hashtags, trending, notifications, attestation
// bindings, blinded credentials — ALL of these refuse unauthenticated reads.

forbid (
  principal == AnonymousUnauthenticated::"public",
  action in [
    Action::"read_post",
    Action::"read_comment",
    Action::"read_dm_ciphertext",
    Action::"read_vote",
    Action::"read_hashtag",
    Action::"read_trending",
    Action::"read_notification",
    Action::"read_attestation_binding",
    Action::"read_blinded_credential"
  ],
  resource
);
