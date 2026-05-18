// Cedar policy: tenant-scope
// Microservice: calendar
// Purpose: enforce per-tenant scoping on every calendar operation; refuse cross-tenant
// read/write unless an explicit CrossTenantInviteGrant is on file.
// Owner: ops-security + axis-calendar
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/event-isolation.md, policy/data-residency.md
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: forbid cross-tenant reads / writes by default.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"event_read",
    Action::"event_write",
    Action::"availability_read",
    Action::"room_book",
    Action::"invitation_send",
    Action::"ics_import",
    Action::"ics_export",
    Action::"caldav_propfind",
    Action::"caldav_put",
    Action::"caldav_delete",
    Action::"caldav_report"
  ],
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// =====================================================================================
// PERMIT: same-tenant access with role-based authorization.
// =====================================================================================

permit (
  principal in CalendarRole::"tenant_operator",
  action in [
    Action::"event_read",
    Action::"event_write",
    Action::"availability_read",
    Action::"room_book",
    Action::"invitation_send",
    Action::"ics_import",
    Action::"ics_export"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.context_kind == resource.context_kind
};

// =====================================================================================
// CROSS-TENANT GRANT: permit only the free/busy projection over an explicit grant.
// =====================================================================================

permit (
  principal in CalendarRole::"cross_tenant_reader",
  action == Action::"availability_read",
  resource in EventField::"projection_freebusy"
)
when {
  principal.tenant_id != resource.tenant_id &&
  context.grant_exists == true &&
  context.grant_valid == true &&
  context.grant_scope == "freebusy" &&
  // The projection must be type-narrowed to FreeBusyProjection (Rust kernel guarantees this).
  resource.field_set == ["starts_at", "ends_at", "busy", "attendee_count_bucket"]
};

// =====================================================================================
// DUAL-CONTEXT ISOLATION: Personal vs Professional.
// =====================================================================================

// Refuse Professional reader on Personal-context event.
forbid (
  principal in CalendarRole::"professional_reader",
  action == Action::"event_read",
  resource in EventContext::"Personal"
);

// Refuse Personal reader on Professional-context event.
forbid (
  principal in CalendarRole::"personal_reader",
  action == Action::"event_read",
  resource in EventContext::"Professional"
);

// Permit context-bridge ONLY at field-level grant.
permit (
  principal in CalendarRole::"context_bridge_reader",
  action == Action::"event_read",
  resource
)
when {
  context.personal_to_professional_grant.field == resource.field_name &&
  context.personal_to_professional_grant.consent_at <= context.now &&
  context.personal_to_professional_grant.revoked_at == null
};

// =====================================================================================
// LEGAL HOLD: refuse hard-delete on held events.
// =====================================================================================

forbid (
  principal,
  action == Action::"event_purge",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == true
};

permit (
  principal in CalendarRole::"compliance_officer_with_2person_rule",
  action == Action::"event_purge",
  resource
)
when {
  // Hard-delete permitted only with 2-person rule + explicit lift of hold.
  resource has legal_hold_open && resource.legal_hold_open == false &&
  context.two_person_rule_approved == true &&
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// EVENT OWNERSHIP TRANSFER.
// =====================================================================================

permit (
  principal in CalendarRole::"event_organiser",
  action == Action::"event_transfer_ownership",
  resource
)
when {
  principal.user_id == resource.organiser_user_id &&
  // Transfer requires organiser explicit action; not inferred from RSVP.
  context.transfer_acknowledged_by_organiser == true
};

forbid (
  principal,
  action == Action::"event_transfer_ownership",
  resource
)
unless {
  principal.user_id == resource.organiser_user_id
};

// =====================================================================================
// CROSS-PACK DATA TRANSFER: forbid by default; permit with SCC.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"event_read",
    Action::"event_write"
  ],
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

permit (
  principal in CalendarRole::"cross_pack_reader",
  action == Action::"availability_read",
  resource in EventField::"projection_freebusy"
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag &&
  context.scc_on_file == true &&
  context.tenant_dpa_includes_scc_clause == true &&
  resource.field_set == ["starts_at", "ends_at", "busy", "attendee_count_bucket"]
};
