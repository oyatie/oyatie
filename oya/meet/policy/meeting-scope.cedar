// meet µservice — meeting-scope policy fragment
// Cedar v4.2
//
// Purpose: enforce per-participant role-based access + lobby gate + recording
//          consent gate + E2E-mode recording-forbid + cross-tenant deny.
// Owner:   axis-meet + ops-security + council-privacy
// Audit:   every deny emits oya_meet_cedar_decision_total{decision="deny"}
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny (defence-in-depth FORBID)
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant member creates a meeting room
// =============================================================================

permit (
  principal,
  action == Action::"create_meeting_room",
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has can_create_meeting_room &&
  principal.can_create_meeting_room == true
};

// =============================================================================
// PERMIT 2 — host starts a meeting instance
// =============================================================================

permit (
  principal,
  action == Action::"start_meeting_instance",
  resource in MeetingRoom::?r
)
when {
  resource has owners &&
  principal has user_ref &&
  principal.user_ref in resource.owners
};

// =============================================================================
// PERMIT 3 — meeting participants join after lobby approval
// =============================================================================

permit (
  principal,
  action == Action::"join_meeting",
  resource in MeetingInstance::?i
)
when {
  resource has lobby_approved_users &&
  principal has user_ref &&
  principal.user_ref in resource.lobby_approved_users
};

// =============================================================================
// PERMIT 4 — host or co-host approves lobby members
// =============================================================================

permit (
  principal,
  action in [Action::"approve_lobby", Action::"deny_lobby", Action::"remove_participant"],
  resource in MeetingInstance::?i
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts
};

// =============================================================================
// PERMIT 5 — participants publish their own tracks
// =============================================================================

permit (
  principal,
  action in [
    Action::"publish_audio",
    Action::"publish_video",
    Action::"start_screen_share",
    Action::"react",
    Action::"send_chat_in_meeting",
    Action::"submit_question",
    Action::"answer_poll"
  ],
  resource in MeetingInstance::?i
)
when {
  resource has active_participants &&
  principal has user_ref &&
  principal.user_ref in resource.active_participants &&
  resource has lobby_approved_users &&
  principal.user_ref in resource.lobby_approved_users
};

// =============================================================================
// PERMIT 6 — host starts recording (consent prerequisites checked separately)
// =============================================================================

permit (
  principal,
  action == Action::"start_recording",
  resource in MeetingInstance::?i
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts
};

// =============================================================================
// PERMIT 7 — host starts transcription (live captions) or AI summary
// =============================================================================

permit (
  principal,
  action in [
    Action::"start_transcription",
    Action::"start_live_captions",
    Action::"start_ai_summary"
  ],
  resource in MeetingInstance::?i
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts
};

// =============================================================================
// PERMIT 8 — host starts live-stream egress (RTMP/WHIP) per ADR-MEET-0004
// =============================================================================

permit (
  principal,
  action == Action::"start_live_stream_egress",
  resource in MeetingInstance::?i
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts &&
  context has egress_destination &&
  context.egress_destination has host &&
  principal has tenant_allowed_egress_destinations &&
  context.egress_destination.host in principal.tenant_allowed_egress_destinations
};

// =============================================================================
// PERMIT 9 — host opens/closes breakout rooms
// =============================================================================

permit (
  principal,
  action in [
    Action::"create_breakout_rooms",
    Action::"merge_breakout_rooms",
    Action::"reassign_breakout_member"
  ],
  resource in MeetingInstance::?i
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts
};

// =============================================================================
// PERMIT 10 — host grants/revokes remote-control on screen-share
// =============================================================================

permit (
  principal,
  action in [Action::"grant_remote_control", Action::"revoke_remote_control"],
  resource in MeetingInstance::?i
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts
};

// =============================================================================
// PERMIT 11 — interpreter publishes on a language overlay channel
// =============================================================================

permit (
  principal,
  action == Action::"publish_interpretation_audio",
  resource in MeetingInstance::?i
)
when {
  resource has interpreter_users &&
  principal has user_ref &&
  principal.user_ref in resource.interpreter_users &&
  principal has entitlements &&
  "interpreter" in principal.entitlements
};

// =============================================================================
// PERMIT 12 — webinar host opens registration
// =============================================================================

permit (
  principal,
  action in [
    Action::"open_registration",
    Action::"close_registration",
    Action::"approve_registration"
  ],
  resource in Webinar::?w
)
when {
  resource has hosts &&
  principal has user_ref &&
  principal.user_ref in resource.hosts
};

// =============================================================================
// PERMIT 13 — compliance-officer issues recording legal hold
// =============================================================================

permit (
  principal in ComplianceOfficer::?o,
  action in [
    Action::"open_ediscovery_hold",
    Action::"close_ediscovery_hold",
    Action::"list_held_recordings"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "ediscovery_hold" in principal.entitlements
};

// =============================================================================
// PERMIT 14 — four-eyes recording disclosure (paired action)
// =============================================================================

permit (
  principal in DisclosureApprover::?d,
  action == Action::"disclose_recording_body",
  resource in Recording::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "disclosure_approver" in principal.entitlements &&
  context has paired_approver_id &&
  context.paired_approver_id != principal.principal_id
};

// =============================================================================
// FORBID — E2E mode disables recording/transcription/AI per ADR-MEET-0003
// =============================================================================

forbid (
  principal,
  action in [
    Action::"start_recording",
    Action::"start_transcription",
    Action::"start_live_captions",
    Action::"start_ai_summary"
  ],
  resource in MeetingInstance::?i
)
when {
  resource has e2e_mode &&
  resource.e2e_mode == true
};

// =============================================================================
// FORBID — E2E mode disables live-stream egress
// =============================================================================

forbid (
  principal,
  action == Action::"start_live_stream_egress",
  resource in MeetingInstance::?i
)
when {
  resource has e2e_mode &&
  resource.e2e_mode == true
};

// =============================================================================
// FORBID — recording-start when any active participant has not consented
// =============================================================================
//
// Per `policy/recording-consent.md` Mode A.

forbid (
  principal,
  action == Action::"start_recording",
  resource in MeetingInstance::?i
)
when {
  resource has unconsented_participant_count &&
  resource.unconsented_participant_count > 0 &&
  resource has pack_consent_mode &&
  resource.pack_consent_mode == "A"
};

// =============================================================================
// FORBID — non-lobby-approved guest publishes
// =============================================================================

forbid (
  principal,
  action in [
    Action::"publish_audio",
    Action::"publish_video",
    Action::"start_screen_share"
  ],
  resource in MeetingInstance::?i
)
when {
  resource has lobby_approved_users &&
  principal has user_ref &&
  !(principal.user_ref in resource.lobby_approved_users)
};

// =============================================================================
// FORBID — cross-tenant access (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"join_meeting",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"list_meetings"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — SECRET-class data never via tenant reads
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — destructive ops on ended meetings (replay-prevention)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"start_recording",
    Action::"publish_audio",
    Action::"publish_video"
  ],
  resource
)
when {
  resource has ended_at &&
  resource.ended_at != null
};

// =============================================================================
// FORBID — biometric inference from face/voice without explicit Art. 9(2)(a) consent
// =============================================================================
//
// Per ADR-MEET-0006: biometric inference DISABLED by default; opt-in requires
// explicit consent + EU AI Act high-risk conformity assessment.

forbid (
  principal,
  action in [
    Action::"infer_biometric_identity",
    Action::"infer_emotion_from_face",
    Action::"infer_emotion_from_voice"
  ],
  resource
)
unless {
  resource has biometric_consent &&
  resource.biometric_consent == true &&
  resource has high_risk_conformity_assessment_id
};
