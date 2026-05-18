// recordings µservice — public-read policy fragment
// Cedar v4.2
//
// Purpose: anonymous / unauthenticated playback ONLY when share-link grants it.

forbid (
  principal,
  action,
  resource
);

// Anonymous playback when share-link is present, valid, non-expired, and within view-count cap.
permit (
  principal in AnonymousPrincipal::?a,
  action in [
    Action::"start_playback_session",
    Action::"read_recording",
    Action::"read_transcript"
  ],
  resource
)
when {
  context has share_link &&
  context.share_link has share_link_id &&
  context.share_link has tenant_id &&
  resource has tenant_id &&
  context.share_link.tenant_id == resource.tenant_id &&
  context.share_link has expires_at &&
  context has now &&
  context.now < context.share_link.expires_at &&
  context.share_link has view_count &&
  context.share_link has view_count_cap &&
  context.share_link.view_count < context.share_link.view_count_cap &&
  context.share_link has hmac_verified &&
  context.share_link.hmac_verified == true &&
  // Password-protected share-links require additional context.share_link.password_verified == true
  (
    !(context.share_link has password_required) ||
    context.share_link.password_required == false ||
    (
      context.share_link.password_required == true &&
      context.share_link has password_verified &&
      context.share_link.password_verified == true
    )
  )
};

// Anonymous principals may never write anything.
forbid (
  principal in AnonymousPrincipal::?a,
  action in [
    Action::"ingest_recording",
    Action::"update_recording_metadata",
    Action::"add_redaction_overlay",
    Action::"engage_legal_hold",
    Action::"release_legal_hold",
    Action::"create_share_link",
    Action::"export_for_ediscovery",
    Action::"delete_recording"
  ],
  resource
);
