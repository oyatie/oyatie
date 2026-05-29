// meet µservice — public-read policy fragment
// Cedar v4.2
//
// Purpose: govern anonymous / unauthenticated access. meet has NO anonymous
//          reads on meetings/recordings/transcripts. This fragment only allows:
//            (a) public health / status probes
//            (b) public OpenAPI / AsyncAPI schema endpoints (no data)
//            (c) public lobby join-token validation (guest scenario)
//            (d) public webinar registration form (when tenant publishes one)
// Owner:   ops-security + gtm-marketing
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — anonymous read of public health / status indicators
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_health",
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind in ["liveness", "readiness", "public-status"]
};

// =============================================================================
// PERMIT 2 — anonymous read of API schema artifacts
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_artifact",
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind in ["openapi-schema", "asyncapi-schema", "proto-descriptor"]
};

// =============================================================================
// PERMIT 3 — anonymous lobby-token redemption (guest scenario)
// =============================================================================
//
// A guest with a valid lobby token can submit it to enter lobby. The token
// itself authenticates them; this Cedar permit just allows the endpoint to be
// hit anonymously. Subsequent meeting actions require the lobby-approved bit.

permit (
  principal == User::"anonymous",
  action == Action::"redeem_lobby_token",
  resource == PublicEndpoint::"lobby-redeem"
);

// =============================================================================
// PERMIT 4 — anonymous webinar registration form (when tenant publishes one)
// =============================================================================
//
// When a tenant publishes a public webinar registration form, anonymous users
// can submit registration. The endpoint validates the form data + creates a
// registration record + emits a join token (which then carries authentication).

permit (
  principal == User::"anonymous",
  action == Action::"submit_webinar_registration",
  resource in WebinarRegistrationForm::?f
)
when {
  resource has public_form &&
  resource.public_form == true
};

// =============================================================================
// FORBID — anonymous cannot read recordings / transcripts / meeting bodies
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_recording",
    Action::"read_recording_body",
    Action::"download_recording_blob",
    Action::"read_transcript",
    Action::"read_transcript_body",
    Action::"read_meeting_room",
    Action::"read_meeting_room_metadata",
    Action::"list_meeting_rooms",
    Action::"read_audit_chain",
    Action::"read_disclosure_history"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot write meeting content
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"create_meeting_room",
    Action::"start_meeting_instance",
    Action::"start_recording",
    Action::"publish_audio",
    Action::"publish_video",
    Action::"start_screen_share",
    Action::"open_ediscovery_hold",
    Action::"disclose_recording_body",
    Action::"start_live_stream_egress"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot access non-public data classes
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class == "PUBLIC")
};
