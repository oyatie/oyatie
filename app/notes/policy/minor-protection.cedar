// Cedar v4.2 LTS — Notes minor-protection
// Authority: ADR-0292 minor user doctrine

forbid (
  principal,
  action == Action::"NotesAccountProvision",
  resource
) when {
  // COPPA <13 — refuse standalone account
  action has declared_age
  && action.declared_age < 13
  && resource has tier
  && resource.tier == "B2C_PERSONAL_STANDALONE"
};

forbid (
  principal,
  action == Action::"NotesAiAssist",
  resource
) when {
  // KOSA 14-17: AI compose-assist disabled by default
  principal.minor_age_band == "KOSA_14_17"
  && !(principal has guardian_consent && principal.guardian_consent.scope.contains("AI_ASSIST"))
};

forbid (
  principal,
  action == Action::"NotesShareLinkIssue",
  resource
) when {
  // Minor user cannot create PUBLIC_SHARE_VIEW share-links (private-share only)
  principal.minor_age_band in ["COPPA_UNDER_13", "KOSA_14_17"]
  && action has share_audience
  && action.share_audience == "PUBLIC_SHARE_VIEW"
};

permit (
  principal,
  action == Action::"NotesGuardianDashboardRead",
  resource
) when {
  resource has minor_age_band
  && resource.minor_age_band in ["COPPA_UNDER_13", "KOSA_14_17"]
  && principal has guardian_link
  && principal.guardian_link.minor_id == resource.owner_id
  && principal.guardian_link.verified == true
};
