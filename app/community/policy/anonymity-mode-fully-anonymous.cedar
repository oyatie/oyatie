// community µservice — anonymity-mode: fully-anonymous
//   (whistleblower / press-source / bug-bounty surfaces)
// Cedar v4.2 LTS; default-deny.
//
// Purpose: gate submission and read actions for fully-anonymous surfaces where
//          NO handle, NO employer affinity, and NO stable identity token is
//          attached to the submission. This covers three sub-surfaces per
//          ADR-0300 (whistleblower + press-freedom doctrine):
//
//   1. Whistleblower submission — employee/citizen submits evidence of wrongdoing
//      to the platform's trust-and-safety team or a designated regulatory body.
//   2. SecureDrop press-source — journalist-facing encrypted drop; ciphertext
//      routed to verified journalist accounts; platform sees ONLY encrypted blob.
//   3. Bug-bounty submission — security researcher submits a vulnerability report
//      without revealing identity to the engineering team (only to the bug-bounty
//      triage principal).
//
// Posting mode: fully-anonymous
// ADR-0300 binding: every permit in this fragment MUST cite the ADR-0300
//                   sub-invariant it satisfies.
//
// Owner:   axis-community + ops-security + council-privacy + general-counsel
// Audit:   every permit/deny emits
//          community_fully_anonymous_policy_eval_total{surface, verdict}
// References:
//   - ADR-0243 (Cedar universal gate)
//   - ADR-0244 (tenant scoping primitive)
//   - ADR-0300 (whistleblower + press-freedom doctrine — PRIMARY authority)
//   - community/capabilities/whistleblower-submission.yaml
//   - community/capabilities/securedrop-press-source.yaml
//   - community/capabilities/bug-bounty-submission.yaml
//   - community/ARCHITECTURE.md §community-as-4-mixture-product
//   - ECPA / SCA §§2510-2523 (US)
//   - EU Whistleblower Protection Directive 2019/1937
//   - UK Public Interest Disclosure Act 1998
//   - KR 공익신고자 보호법 (Public Interest Disclosure Protection Act)
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
// PERMIT 1 — whistleblower submits encrypted evidence bundle
//            ADR-0300 §W-1: submission integrity guarantee
// =============================================================================
//
// The platform records an encrypted blob + a content-addressed hash. It NEVER
// decrypts the blob; only the designated trust-and-safety principal holds the
// private key (per ADR-0300 §W-3 key custody).

permit (
  principal == FullyAnonymousSubmitter::"public",
  action == Action::"submit_whistleblower_bundle",
  resource in WhistleblowerDropbox::?r
)
when {
  resource has tenant_id &&
  resource has submission_encryption_key_present &&
  resource.submission_encryption_key_present == true &&
  resource has channel &&
  resource.channel == "whistleblower" &&
  resource has rate_limited &&
  resource.rate_limited == false
};

// =============================================================================
// PERMIT 2 — press-source submits encrypted document to journalist dropbox
//            ADR-0300 §P-1: press-freedom encryption guarantee
// =============================================================================
//
// SecureDrop model: the platform routes the ciphertext to the journalist's
// mailbox and NEVER holds the decryption key. Platform sees only: ciphertext
// blob + content-hash + journalist_mailbox_id.

permit (
  principal == FullyAnonymousSubmitter::"public",
  action == Action::"submit_press_source_bundle",
  resource in JournalistDropbox::?r
)
when {
  resource has journalist_mailbox_id &&
  resource has e2ee_ciphertext_present &&
  resource.e2ee_ciphertext_present == true &&
  resource has channel &&
  resource.channel == "press_source" &&
  resource has rate_limited &&
  resource.rate_limited == false
};

// =============================================================================
// PERMIT 3 — security researcher submits bug-bounty report
//            ADR-0300 §B-1: researcher pseudonym protection
// =============================================================================
//
// The researcher submits an encrypted vulnerability report. Only the bug-bounty
// triage principal (BugBountyTriager) can decrypt. The researcher may choose to
// disclose identity after triage, but the initial submission is fully anonymous.

permit (
  principal == FullyAnonymousSubmitter::"public",
  action == Action::"submit_bug_bounty_report",
  resource in BugBountyDropbox::?r
)
when {
  resource has triage_encryption_key_present &&
  resource.triage_encryption_key_present == true &&
  resource has channel &&
  resource.channel == "bug_bounty" &&
  resource has rate_limited &&
  resource.rate_limited == false
};

// =============================================================================
// PERMIT 4 — trust-and-safety principal reads whistleblower bundle (after
//            key-custody ceremony per ADR-0300 §W-3)
// =============================================================================

permit (
  principal in TrustAndSafety::?p,
  action in [
    Action::"read_whistleblower_bundle",
    Action::"escalate_whistleblower_case",
    Action::"close_whistleblower_case"
  ],
  resource in WhistleblowerCase::?r
)
when {
  principal has trust_safety_entitlement &&
  principal.trust_safety_entitlement == true &&
  resource has decryption_ceremony_completed &&
  resource.decryption_ceremony_completed == true &&
  resource has dual_control_approved &&
  resource.dual_control_approved == true
};

// =============================================================================
// PERMIT 5 — journalist reads their own dropbox (after auth with journalist
//            identity; ciphertext only — platform cannot decrypt)
// =============================================================================

permit (
  principal in VerifiedJournalist::?p,
  action == Action::"read_press_source_ciphertext",
  resource in JournalistDropbox::?r
)
when {
  principal has journalist_mailbox_id &&
  resource has journalist_mailbox_id &&
  principal.journalist_mailbox_id == resource.journalist_mailbox_id
};

// =============================================================================
// PERMIT 6 — bug-bounty triage reads encrypted report
// =============================================================================

permit (
  principal in BugBountyTriager::?p,
  action in [
    Action::"read_bug_bounty_report",
    Action::"triage_bug_bounty_report",
    Action::"close_bug_bounty_report"
  ],
  resource in BugBountyCase::?r
)
when {
  principal has bug_bounty_triage_entitlement &&
  principal.bug_bounty_triage_entitlement == true &&
  resource has triage_encryption_key_present &&
  resource.triage_encryption_key_present == true
};

// =============================================================================
// FORBID — platform NEVER decrypts submissions without dual-control ceremony
//          ADR-0300 §W-3 + §P-2 + §B-2
// =============================================================================

forbid (
  principal,
  action in [
    Action::"decrypt_whistleblower_bundle",
    Action::"decrypt_press_source_bundle",
    Action::"decrypt_bug_bounty_report"
  ],
  resource
)
unless {
  principal has dual_control_approved &&
  principal.dual_control_approved == true
};

// =============================================================================
// FORBID — platform NEVER links submission to any user identity
//          ADR-0300 §W-4 + §P-3 + §B-3
// =============================================================================

forbid (
  principal,
  action in [
    Action::"correlate_submission_to_user",
    Action::"reveal_submission_author",
    Action::"link_submission_to_ip_address",
    Action::"link_submission_to_device_fingerprint"
  ],
  resource
);

// =============================================================================
// FORBID — third-party tracker egress (critical for anonymous surfaces)
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

// =============================================================================
// FORBID — rate-limit bypass attempts (DDoS/spam submission protection)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"submit_whistleblower_bundle",
    Action::"submit_press_source_bundle",
    Action::"submit_bug_bounty_report"
  ],
  resource
)
when {
  resource has rate_limited &&
  resource.rate_limited == true
};
