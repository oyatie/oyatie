// Cedar v4.2 LTS — Social DM scope gates
// Authority: ADR-0292; separates social-DM surface from primary messenger product

forbid (
  principal,
  action == Action::"SocialDmSend",
  resource
) when {
  // Anonymous DMs forbidden
  !(principal has user_id)
};

forbid (
  principal,
  action == Action::"SocialDmSend",
  resource
) when {
  // Brand → minor DM requires verified-creator badge + explicit opt-in
  resource has minor_age_band
  && resource.minor_age_band in ["KOSA_14_17"]
  && principal has account_class
  && principal.account_class == "B2B_BRAND"
  && !(resource has minor_opt_in_to_brand_dm && resource.minor_opt_in_to_brand_dm == true)
};

permit (
  principal,
  action == Action::"SocialDmSend",
  resource
) when {
  principal has user_id
  && resource has user_id
  && resource.follow_accepts_dms_from_followers == true
  && action has sender_is_follower_accepted
  && action.sender_is_follower_accepted == true
};
