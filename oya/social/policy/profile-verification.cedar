// Cedar v4.2 LTS — Social profile verification
// Authority: ADR-0297 (anti-spoof)

forbid (
  principal,
  action == Action::"SocialProfileBadgeAssign",
  resource
) when {
  // Verified badge requires attested gov-ID IDV
  action has badge_class
  && action.badge_class == "VERIFIED_CREATOR"
  && !(resource has idv_verified && resource.idv_verified == true)
};

forbid (
  principal,
  action == Action::"SocialDisplayNameUpdate",
  resource
) when {
  // Verified-creator display-name change requires re-IDV
  resource has verified_creator_badge
  && resource.verified_creator_badge == true
  && !(action has re_idv_verified && action.re_idv_verified == true)
};

permit (
  principal,
  action == Action::"SocialProfileBadgeAssign",
  resource
) when {
  action has badge_class
  && action.badge_class == "VERIFIED_CREATOR"
  && resource has idv_verified
  && resource.idv_verified == true
};
