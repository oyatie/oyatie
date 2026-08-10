// Cedar v4.2 LTS — Social minor-protection (COPPA + KOSA + EU age-verification)
// Authority: ADR-0292; documentation-rigor.md §3.2.3 (UX-floor)
// Hyperscaler precedent: Instagram Teen Accounts + TikTok Restricted Mode + Snapchat Family Center.

// ---- COPPA refusal ----

forbid (
  principal,
  action == Action::"SocialAccountProvision",
  resource
) when {
  action has declared_age
  && action.declared_age < 13
  && resource has tier
  && resource.tier == "B2C_PERSONAL_STANDALONE"
};

// ---- KOSA 14-17 strict defaults ----

forbid (
  principal,
  action == Action::"SocialDmReceive",
  resource
) when {
  // Minor recipient: DM from non-followers blocked
  resource has minor_age_band
  && resource.minor_age_band == "KOSA_14_17"
  && action has sender_is_follower_accepted
  && action.sender_is_follower_accepted == false
};

forbid (
  principal,
  action == Action::"SocialDmReceive",
  resource
) when {
  // Minor recipient: DM from adult-stranger pattern blocked
  resource has minor_age_band
  && resource.minor_age_band == "KOSA_14_17"
  && action has anti_grooming_score
  && action.anti_grooming_score > 80
};

forbid (
  principal,
  action == Action::"SocialFeedRankAlgorithmic",
  resource
) when {
  // KOSA minor: algorithmic ranking off by default
  principal.minor_age_band == "KOSA_14_17"
  && !(principal has guardian_consent && principal.guardian_consent.scope.contains("ALGORITHMIC_RANKING"))
};

forbid (
  principal,
  action == Action::"SocialAdsTargeted",
  resource
) when {
  // KOSA minor: targeted ads off
  principal.minor_age_band == "KOSA_14_17"
};

forbid (
  principal,
  action == Action::"SocialLiveStreamStart",
  resource
) when {
  // KOSA minor: live-streaming gated by guardian-acknowledgement
  principal.minor_age_band == "KOSA_14_17"
  && !(principal has guardian_consent && principal.guardian_consent.scope.contains("LIVE_STREAM"))
};

forbid (
  principal,
  action == Action::"SocialInAppPurchase",
  resource
) when {
  // KOSA + COPPA: in-app purchase requires guardian approval
  principal.minor_age_band in ["KOSA_14_17", "COPPA_UNDER_13"]
  && !(action has guardian_approval && action.guardian_approval == true)
};

forbid (
  principal,
  action == Action::"SocialProfileDiscoverable",
  resource
) when {
  // Minor profile non-discoverable to adult strangers
  resource has minor_age_band
  && resource.minor_age_band == "KOSA_14_17"
  && action has viewer_age_band
  && action.viewer_age_band == "ADULT_STRANGER"
};

// ---- Guardian dashboard ----

permit (
  principal,
  action == Action::"SocialGuardianDashboardRead",
  resource
) when {
  resource has minor_age_band
  && resource.minor_age_band in ["COPPA_UNDER_13", "KOSA_14_17"]
  && principal has guardian_link
  && principal.guardian_link.minor_id == resource.owner_id
  && principal.guardian_link.verified == true
};

// ---- Notifications muted overnight (per ADR-0292 design defaults) ----

forbid (
  principal,
  action == Action::"SocialPushNotificationDeliver",
  resource
) when {
  resource has minor_age_band
  && resource.minor_age_band == "KOSA_14_17"
  && action has local_hour
  && (action.local_hour >= 22 || action.local_hour < 7)
  && !(resource has guardian_override_overnight && resource.guardian_override_overnight == true)
};
