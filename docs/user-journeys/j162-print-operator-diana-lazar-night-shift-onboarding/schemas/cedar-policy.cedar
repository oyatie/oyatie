// j162 — Diana Lazăr night-shift solo onboarding Cedar policy
// Bundle id: cedar-bundle-night-shift-solo-onboarding-v1
// Deployed to: tipografia-lazar-petrescu-ro (shared with j157) + antibiotice-sa-ro (cross-link) +
//              cz-securitas-alarm-cooperative-tenant-ro + adriana-stanciu-consulting-ro +
//              diana.lazar-petrescu.personal + mihai.lazar-petrescu.personal +
//              scoala-internationala-cluj-ro + ro-anaf-tenant
// Core doctrine: Competency is a Cedar context (not a static flag); night-shift authority is
// gated separately from day-shift authority even for the same human; lone-worker dead-man
// protocol's personal-tenant escalation contact is consented-not-defaulted; cross-journey
// persona continuity from j157 is verified through Cedar context predicates not data dump.

// ============================================================
// PERMIT — Diana operates solo night-shift on the press
// ============================================================

permit (
    principal == User::"diana.lazăr@tipografia-lazar-petrescu-ro",
    action in [
        Action::"press.operate_solo_night_shift",
        Action::"press.cold_startup_solo",
        Action::"press.ink_density_correct_solo",
        Action::"press.emergency_response_solo",
        Action::"press.paper_jam_clear_solo",
        Action::"press.substrate_change_solo",
        Action::"quality.delta_e_correct_solo"
    ],
    resource is PressLine
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") &&
    principal.has_certification_unexpired("ISO-12647-2-Trained") &&
    principal.has_competency_unexpired("night-shift-solo-authorization-2027") &&
    principal.workplace_integration_provisioned == "night-shift" &&
    principal.dead_man_protocol_enrolled == true &&
    context.shift_active_night == true &&
    context.lone_worker_low_light_mode == true &&
    context.unicode_normalization == "NFC"
};

// ============================================================
// PERMIT — Competency unlock after assessment
// ============================================================

permit (
    principal,
    action == Action::"learning_management.competency_unlock_night_shift_solo",
    resource is Competency
) when {
    resource.competency_id == "night-shift-solo-authorization-2027" &&
    context.assessment_score_min_85_per_category == true &&
    context.proctor_signoff == true &&
    context.hse_consultant_signoff == true &&
    context.prerequisite_certs_unexpired == true &&
    context.supervised_shifts_logged >= 8
};

// ============================================================
// PERMIT — Cross-journey continuity prerequisite read
// ============================================================

permit (
    principal,
    action == Action::"learning_management.competency_link_read",
    resource is CertChain
) when {
    resource.persona == principal.persona_uuid &&
    context.same_human_dual_journey == true &&
    context.read_purpose == "prerequisite_verification"
};

// ============================================================
// PERMIT — Workplace-integration provisioning bundle
// ============================================================

permit (
    principal,
    action in [
        Action::"workplace_integration.shift_schedule_create",
        Action::"workplace_integration.geofence_enable",
        Action::"workplace_integration.badge_role_update",
        Action::"workplace_integration.payroll_differential_enable"
    ],
    resource is WorkplaceIntegrationResource
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.role_in_tenant("tipografia-lazar-petrescu-ro") in [
        "managing_director",
        "hr_administrator",
        "bookkeeper"
    ] &&
    context.operator_principal_competency_unexpired == true
};

// ============================================================
// PERMIT — Securitas alarm-cooperative cross-tenant scope add
// ============================================================

permit (
    principal,
    action == Action::"securitas_alarm_cooperative.roster_add_night_dearmer",
    resource is AlarmCooperativeRoster
) when {
    resource.client_tenant == "tipografia-lazar-petrescu-ro" &&
    principal.tenant in ["tipografia-lazar-petrescu-ro", "cz-securitas-alarm-cooperative-tenant-ro"] &&
    context.auto_revoke_on_competency_expiry == true
};

// ============================================================
// PERMIT — Alarm de-arm by operator (cross-tenant)
// ============================================================

permit (
    principal,
    action == Action::"securitas.alarm_cooperative_dearm",
    resource is AlarmZone
) when {
    principal.workplace_integration_provisioned == "night-shift" &&
    principal.has_competency_unexpired("night-shift-solo-authorization-2027") &&
    context.shift_scheduled_for_now == true &&
    context.biometric_low_light_match == true
};

// ============================================================
// PERMIT — Shift clock-in via geofence + biometric
// ============================================================

permit (
    principal,
    action == Action::"workplace_integration.shift_clock_in",
    resource is ShiftEnvelope
) when {
    context.geofence_match == true &&
    context.biometric_match_score >= 0.95 &&
    context.shift_scheduled_for_principal == true
};

// ============================================================
// PERMIT — Lone-worker dead-man enrollment
// ============================================================

permit (
    principal,
    action == Action::"identity.lone_worker.dead_man_enroll",
    resource is LoneWorkerEnvelope
) when {
    principal.has_competency_unexpired("night-shift-solo-authorization-2027") &&
    context.personal_tenant_escalation_consent_active == true &&
    context.biometric_low_light_validated == true &&
    context.checkin_interval_hours <= 4 &&
    context.checkin_response_window_seconds == 60
};

// ============================================================
// PERMIT — Dead-man check-in within 60s window
// ============================================================

permit (
    principal,
    action == Action::"identity.lone_worker.dead_man_checkin",
    resource is DeadManCheckin
) when {
    context.checkin_within_60s_window == true &&
    context.biometric_match_score >= 0.95
};

// ============================================================
// PERMIT — Personal-tenant cross-tenant escalation consent
// ============================================================

permit (
    principal,
    action == Action::"tenancy.personal_tenant_cross_tenant_consent",
    resource is PersonalTenantConsent
) when {
    principal.tenant_class == "personal_tenant" &&
    context.consent_scope == "lone_worker_dead_man_miss_escalation_voice_plus_messenger" &&
    context.gdpr_consent_basis == "explicit_personal_tenant_principal_consent"
};

// ============================================================
// PERMIT — Night-shift premium payroll application
// ============================================================

permit (
    principal,
    action == Action::"payments.night_shift_differential_apply",
    resource is PayrollPeriod
) when {
    principal.role_in_tenant(resource.employer_tenant) in ["bookkeeper", "hr_administrator", "managing_director"] &&
    context.ro_codul_muncii_126_compliant == true &&
    context.differential_pct == 25
};

// ============================================================
// PERMIT — Shift handoff
// ============================================================

permit (
    principal,
    action == Action::"workflow.shift_handoff",
    resource is ShiftEnvelope
) when {
    context.outgoing_principal == principal.subject_id ||
    context.incoming_principal == principal.subject_id
};

// ============================================================
// FORBID-1 — Solo operation without competency
// ============================================================

forbid (
    principal,
    action in [
        Action::"press.operate_solo_night_shift",
        Action::"press.cold_startup_solo",
        Action::"press.ink_density_correct_solo",
        Action::"press.emergency_response_solo"
    ],
    resource is PressLine
) when {
    !principal.has_competency_unexpired("night-shift-solo-authorization-2027")
}
advice {
    "forbid_rule_id": "forbid-competency-missing",
    "audit_class": "EVT-J162-CEDAR-DENY-COMPETENCY-MISSING-014a",
    "fallback_path": "schedule_re_assessment_via_learning_management"
};

// ============================================================
// FORBID-2 — Alarm de-arm without competency
// ============================================================

forbid (
    principal,
    action == Action::"securitas.alarm_cooperative_dearm",
    resource is AlarmZone
) when {
    !principal.has_competency_unexpired("night-shift-solo-authorization-2027") ||
    principal.workplace_integration_provisioned != "night-shift"
}
advice {
    "forbid_rule_id": "forbid-alarm-competency-missing",
    "audit_class": "EVT-J162-CEDAR-DENY-ALARM-COMPETENCY-014b",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-3 — Dead-man check-in missed (auto-escalation)
// ============================================================

forbid (
    principal,
    action == Action::"identity.lone_worker.dead_man_checkin",
    resource is DeadManCheckin
) when {
    context.checkin_within_60s_window == false
}
advice {
    "forbid_rule_id": "forbid-dead-man-miss",
    "audit_class": "EVT-J162-CEDAR-DENY-DEAD-MAN-MISS-014c",
    "escalation_triggered": true,
    "escalation_chain_walk_within_seconds": 90,
    "audit_dual_seal": true
};

// ============================================================
// FORBID-4 — Personal-tenant escalation contact without consent
// ============================================================

forbid (
    principal,
    action == Action::"identity.lone_worker.dead_man_enroll",
    resource is LoneWorkerEnvelope
) when {
    context.personal_tenant_escalation_consent_active == false
}
advice {
    "forbid_rule_id": "forbid-personal-consent-missing",
    "audit_class": "EVT-J162-CEDAR-DENY-PERSONAL-CONSENT-014d",
    "doctrine_anchor": "EU-GDPR-Article-7 + ADR-0311"
};

// ============================================================
// FORBID-5 — Diacritic Romanization in legal/payroll fields
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.legal_name_write",
        Action::"payments.payroll_record_write",
        Action::"workflow.shift_handoff",
        Action::"ro_anaf.payroll_report_write"
    ],
    resource
) when {
    context.unicode_normalization != "NFC" ||
    context.diacritics_preserved == false ||
    (context.romanize_to_ascii == true && context.user_explicit_consent_to_romanize == false)
}
advice {
    "forbid_rule_id": "forbid-diacritic-romanize-in-legal",
    "audit_class": "EVT-J162-CEDAR-DENY-NAME-ROMANIZE-014e"
};

// ============================================================
// FORBID-6 — Securitas reads Tipografia payroll
// ============================================================

forbid (
    principal,
    action in [
        Action::"payroll.read_employee",
        Action::"payroll.list_employees"
    ],
    resource is PayrollRecord
) when {
    principal.tenant == "cz-securitas-alarm-cooperative-tenant-ro" ||
    principal.tenant != resource.employer_tenant
}
advice {
    "forbid_rule_id": "forbid-cross-tenant-payroll-read",
    "audit_class": "EVT-J162-CEDAR-DENY-PAYROLL-CROSS-TENANT-014f",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-7 — Shift clock-in outside geofence
// ============================================================

forbid (
    principal,
    action == Action::"workplace_integration.shift_clock_in",
    resource is ShiftEnvelope
) when {
    context.geofence_match == false
}
advice {
    "forbid_rule_id": "forbid-geofence-breach",
    "audit_class": "EVT-J162-CEDAR-DENY-GEOFENCE-014g"
};

// ============================================================
// FORBID-8 — Cross-journey identity mismatch
// ============================================================

forbid (
    principal,
    action == Action::"learning_management.competency_link_read",
    resource is CertChain
) when {
    resource.persona != principal.persona_uuid
}
advice {
    "forbid_rule_id": "forbid-identity-mismatch",
    "audit_class": "EVT-J162-CEDAR-DENY-IDENTITY-MISMATCH-014h",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-9 — Handoff to operator not scheduled
// ============================================================

forbid (
    principal,
    action == Action::"workflow.shift_handoff",
    resource is ShiftEnvelope
) when {
    context.incoming_principal_scheduled_for_shift == false
}
advice {
    "forbid_rule_id": "forbid-handoff-schedule-mismatch",
    "audit_class": "EVT-J162-CEDAR-DENY-HANDOFF-SCHEDULE-014i"
};

// ============================================================
// FORBID-10 — Night-shift premium on day-shift hours
// ============================================================

forbid (
    principal,
    action == Action::"payments.night_shift_differential_apply",
    resource is PayrollPeriod
) when {
    resource.hours_class != "night_shift"
}
advice {
    "forbid_rule_id": "forbid-differential-class-mismatch",
    "audit_class": "EVT-J162-CEDAR-DENY-DIFFERENTIAL-014j"
};

// ============================================================
// FORBID-11 — Cell residency drift (Romanian tenant data outside EU-Bucharest)
// ============================================================

forbid (
    principal,
    action,
    resource
) when {
    principal.tenant_country == "ro" &&
    resource.target_cell not in ["eu-bucharest-primary", "eu-frankfurt-secondary", "eu-amsterdam-readonly-replica"]
}
advice {
    "forbid_rule_id": "forbid-ro-cell-drift",
    "audit_class": "EVT-J162-CEDAR-DENY-RO-CELL-DRIFT-014k"
};

// ============================================================
// FORBID-12 — TrueTime fence breach on competency unlock
// ============================================================

forbid (
    principal,
    action == Action::"learning_management.competency_unlock_night_shift_solo",
    resource is Competency
) when {
    context.truetime_uncertainty_ms > 50
}
advice {
    "forbid_rule_id": "forbid-truetime-breach",
    "audit_class": "EVT-J162-CEDAR-DENY-TRUETIME-014l",
    "doctrine_anchor": "ADR-0252"
};
