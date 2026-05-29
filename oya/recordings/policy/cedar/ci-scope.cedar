// recordings µservice — ci-scope policy fragment
// Cedar v4.2
//
// Purpose: CI principals authoring evidence / test fixtures.
//          Default-deny + targeted PERMITs for the CI service identity.

forbid (
  principal,
  action,
  resource
);

// CI may read recordings + transcripts in CI-test tenants only.
permit (
  principal in CiPrincipal::?c,
  action in [
    Action::"list_recordings",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"start_playback_session",
    Action::"export_mp4",
    Action::"export_transcript_vtt"
  ],
  resource
)
when {
  principal has spiffe_id &&
  principal.spiffe_id == "spiffe://oyatie/ci/recordings-conformance" &&
  resource has tenant_id &&
  // CI-test tenants only
  resource.tenant_id like "tenant-ci-recordings-*"
};

// CI may write test recordings (in CI-test tenants only).
permit (
  principal in CiPrincipal::?c,
  action in [
    Action::"ingest_recording",
    Action::"create_manual_upload_presign",
    Action::"finalize_manual_upload",
    Action::"add_redaction_overlay",
    Action::"engage_legal_hold",
    Action::"release_legal_hold"
  ],
  resource
)
when {
  principal has spiffe_id &&
  principal.spiffe_id == "spiffe://oyatie/ci/recordings-conformance" &&
  resource has tenant_id &&
  resource.tenant_id like "tenant-ci-recordings-*"
};

// CI may NEVER touch production tenants.
forbid (
  principal in CiPrincipal::?c,
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id like "tenant-ci-recordings-*")
};
