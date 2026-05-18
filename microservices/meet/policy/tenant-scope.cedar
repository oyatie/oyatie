// meet µservice — tenant-scope policy fragment
// Cedar v4.2
//
// Purpose: enforce per-tenant read/write scope for meet entities.
// Owner:   ops-security + axis-meet
// Audit:   every deny emits oya_meet_tenant_unauthorized_attempt_total
// References: data-residency.md; messenger tenant-scope.cedar pattern
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
// PERMIT 1 — tenant operator reads own tenant's meeting data
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_meeting_room",
    Action::"read_meeting_room_metadata",
    Action::"list_meeting_rooms",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"read_audit_chain"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — end-user reads/writes meetings they participate in
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_meeting_room",
    Action::"start_meeting_instance",
    Action::"join_meeting",
    Action::"publish_audio",
    Action::"publish_video",
    Action::"start_screen_share",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"react",
    Action::"send_chat_in_meeting",
    Action::"submit_question",
    Action::"answer_poll"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has meeting_room_id &&
  principal has member_of_meeting_rooms &&
  resource.meeting_room_id in principal.member_of_meeting_rooms
};

// =============================================================================
// PERMIT 3 — meeting-room owner manages room ACL
// =============================================================================

permit (
  principal in MeetingRoomOwner::?a,
  action in [
    Action::"add_meeting_room_owner",
    Action::"remove_meeting_room_owner",
    Action::"set_meeting_room_retention",
    Action::"set_meeting_room_metadata",
    Action::"archive_meeting_room"
  ],
  resource in MeetingRoom::?c
)
when {
  principal has owner_of_meeting_rooms &&
  resource has meeting_room_id &&
  resource.meeting_room_id in principal.owner_of_meeting_rooms
};

// =============================================================================
// PERMIT 4 — compliance-officer issues recording legal hold
// =============================================================================

permit (
  principal in ComplianceOfficer::?o,
  action in [
    Action::"open_ediscovery_hold",
    Action::"close_ediscovery_hold",
    Action::"list_held_resources"
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
// PERMIT 5 — four-eyes recording disclosure (paired action; mirrors messenger)
// =============================================================================

permit (
  principal in DisclosureApprover::?d,
  action == Action::"disclose_recording_body",
  resource in Recording::?m
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
// PERMIT 6 — supervisor reviews recorded supervised-comms (pack-us-financial)
// =============================================================================
//
// FINRA 4511 + SEC 17a-4 supervisory review path; entitlement Cedar-gated.

permit (
  principal in Supervisor::?s,
  action == Action::"read_supervised_recording",
  resource in Recording::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "finra_supervisor" in principal.entitlements &&
  resource has pack &&
  resource.pack == "us-financial"
};

// =============================================================================
// FORBID — cross-tenant access (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_meeting_room",
    Action::"read_recording",
    Action::"read_transcript",
    Action::"list_meeting_rooms",
    Action::"read_audit_chain"
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
