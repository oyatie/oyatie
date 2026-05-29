// Cedar v4.2 LTS — Mail minor-protection (COPPA + KOSA + EU age-verification)
// Authority: ADR-0292 minor user doctrine

forbid (
  principal,
  action == Action::"MailAccountProvision",
  resource
) when {
  // COPPA <13: refuse provisioning of a standalone B2C account
  action has declared_age
  && action.declared_age < 13
  && resource has tier
  && resource.tier == "B2C_PERSONAL_STANDALONE"
};

forbid (
  principal,
  action == Action::"MailComposeAssist",
  resource
) when {
  // KOSA 14-17 tier: AI compose-assist disabled by default for minor mailboxes
  principal.minor_age_band == "KOSA_14_17"
  && !(principal has guardian_consent && principal.guardian_consent.scope.contains("AI_COMPOSE_ASSIST"))
};

forbid (
  principal,
  action == Action::"MailInboundDeliver",
  resource
) when {
  // Minor-mailboxes get strict inbound content filter
  resource has minor_age_band
  && resource.minor_age_band in ["COPPA_UNDER_13", "KOSA_14_17"]
  && action has content_classifier_verdict
  && action.content_classifier_verdict in ["ADULT", "GROOMING_HEURISTIC", "VIOLENCE_HIGH"]
};

permit (
  principal,
  action == Action::"MailGuardianDashboardRead",
  resource
) when {
  // Parental dashboard surface
  resource has minor_age_band
  && resource.minor_age_band in ["COPPA_UNDER_13", "KOSA_14_17"]
  && principal has guardian_link
  && principal.guardian_link.minor_id == resource.owner_id
  && principal.guardian_link.verified == true
};
