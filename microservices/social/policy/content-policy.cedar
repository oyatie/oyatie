// Cedar v4.2 LTS — Social content-policy (community standards)
// Authority: ADR-SOC-0003 content-moderation-classifier-bounds
// EU DSA + KCC + US child-safety + Mastodon-style moderation reuse.

forbid (
  principal,
  action == Action::"SocialPost",
  resource
) when {
  // CSAM: hard block + NCMEC report (no opt-out, no soft-quarantine)
  action has csam_classifier_score
  && action.csam_classifier_score > 50
};

forbid (
  principal,
  action == Action::"SocialPost",
  resource
) when {
  // Imminent violence / terror content
  action has violence_classifier_score
  && action.violence_classifier_score > 90
};

forbid (
  principal,
  action == Action::"SocialPost",
  resource
) when {
  // Hate-speech (high-confidence verdict): block + appeal path
  action has hate_classifier_score
  && action.hate_classifier_score > 90
};

forbid (
  principal,
  action == Action::"SocialPost",
  resource
) when {
  // Targeted harassment of identified user
  action has harassment_score
  && action.harassment_score > 90
  && action has target_user_id
};

forbid (
  principal,
  action == Action::"SocialDmSend",
  resource
) when {
  // Sexual content to non-consenting recipient
  action has nsfw_score
  && action.nsfw_score > 85
  && !(action has recipient_consent_to_nsfw && action.recipient_consent_to_nsfw == true)
};

permit (
  principal,
  action == Action::"SocialAppealSubmit",
  resource
) when {
  // Every moderation action has an appeal path (DSA Art. 20)
  resource has moderation_action_id
  && principal has user_id
  && resource.target_user_id == principal.user_id
};
