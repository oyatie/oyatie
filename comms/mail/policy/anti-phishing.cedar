// Cedar v4.2 LTS — Mail anti-phishing
// Authority: ADR-0297; ADR-0273

forbid (
  principal,
  action == Action::"MailInboundDeliver",
  resource
) when {
  // High-confidence phishing classifier verdict + impersonates known internal sender
  action has phishing_score
  && action.phishing_score > 95
  && action has impersonates_internal_sender
  && action.impersonates_internal_sender == true
};

forbid (
  principal,
  action == Action::"MailInboundDeliver",
  resource
) when {
  // URL reputation: any URL with score > 90 in URL-reputation provider blocks delivery to inbox
  // (still saved to quarantine for forensics)
  action has max_url_reputation_score
  && action.max_url_reputation_score > 90
};

forbid (
  principal,
  action == Action::"MailInboundDeliver",
  resource
) when {
  // Display-name spoof: claims to be "<internal-name>" but envelope is external
  action has display_name_internal_claim
  && action.display_name_internal_claim == true
  && action has envelope_external
  && action.envelope_external == true
};

permit (
  principal,
  action == Action::"MailQuarantineRelease",
  resource
) when {
  // User can release from quarantine only when they own the recipient mailbox
  principal has mailbox_id
  && resource has mailbox_id
  && principal.mailbox_id == resource.mailbox_id
  && principal has auth_class
  && principal.auth_class in ["WEBAUTHN_PASSKEY", "WEBAUTHN_PLATFORM", "MFA_TOTP"]
};
