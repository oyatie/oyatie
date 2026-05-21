// j155 — Stefan Kovács dual-role Cedar policy
// Bundle id: cedar-bundle-stefan-dual-role-v1
// Deployed to: personal-stefan-kovacs-hu + oszk-security-services_hu + bme-student-bodv75_hu + bme-research-cohort-2026-sleep-grade-fall
// ADR-0311 dual-tenant identity; ADR-0317 role-projection; ADR-0244 tenant scoping; ADR-0263 audit dual-seal on deny

// ===========================================
// PERMIT — Stefan as OSZK night-shift guard
// ===========================================

permit (
    principal == User::"stefan.kovacs@personal-id.oya",
    action in [
        Action::"calendar.confirm_shift",
        Action::"calendar.offer_swap",
        Action::"calendar.accept_swap",
        Action::"calendar.clock_out",
        Action::"messenger.read_work",
        Action::"messenger.send_work",
        Action::"messenger.reply_work",
        Action::"incident.report",
        Action::"workplace.clock_in",
        Action::"workplace.clock_out"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "oszk-security-services_hu" &&
    principal.role_in_tenant("oszk-security-services_hu") == "night_shift_guard" &&
    context.active_tenant == "oszk-security-services_hu" &&
    context.wtd_weekly_running_avg <= 48.0 &&
    context.rest_minimum_observed == true
};

// ===========================================
// PERMIT — Stefan as BME student
// ===========================================

permit (
    principal == User::"stefan.kovacs@personal-id.oya",
    action in [
        Action::"lms.read_notes",
        Action::"lms.read_past_exams",
        Action::"lms.submit_assignment",
        Action::"community.post_student",
        Action::"community.read_student",
        Action::"community.reply_student",
        Action::"payments.tuition_pay",
        Action::"payments.tuition_read_history"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "bme-student-bodv75_hu" &&
    principal.role_in_tenant("bme-student-bodv75_hu") == "active_student" &&
    context.active_tenant == "bme-student-bodv75_hu" &&
    context.enrolment_status == "enrolled_paid_through_q3"
};

// ===========================================
// PERMIT — Stefan as personal-tenant owner
// ===========================================

permit (
    principal == User::"stefan.kovacs@personal-id.oya",
    action in [
        Action::"workplace.bridge_consent",
        Action::"workplace.bridge_revoke",
        Action::"identity.tenant_switch",
        Action::"observability.sleep_grade_opt_in",
        Action::"observability.sleep_grade_opt_out",
        Action::"dpo.art15_export_request",
        Action::"payments.personal_read",
        Action::"payments.personal_transfer"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "personal-stefan-kovacs-hu" &&
    principal.role_in_tenant("personal-stefan-kovacs-hu") == "owner" &&
    context.active_tenant == "personal-stefan-kovacs-hu"
};

// ===========================================
// PERMIT — Workplace-integration bridge (cross-tenant, three-way)
// The bridge requires consent from ALL THREE tenants AND a TrueTime fence
// ===========================================

permit (
    principal == User::"oya.system.workplace-integration",
    action == Action::"workplace.bridge_commit",
    resource is Bridge
) when {
    resource.bridge_id like "bridge-osz-bme-stefan-*" &&
    resource.osz_proposal_status == "proposed" &&
    resource.personal_consent_status == "granted" &&
    resource.bme_installment_status == "intent_recorded" &&
    context.truetime_uncertainty_ms <= 10
};

// ===========================================
// PERMIT — BME research cohort observability emit
// ===========================================

permit (
    principal == User::"oya.system.observability-pipeline",
    action == Action::"analytics.cohort_emit",
    resource is Tenant
) when {
    resource.tenant_id == "bme-research-cohort-2026-sleep-grade-fall" &&
    context.source_tenant == "personal-stefan-kovacs-hu" &&
    context.pseudonymized == true &&
    context.identifier_columns_stripped == true &&
    context.cohort_consent_active == true
};

// ===========================================
// FORBID-1 — OSZK cannot read BME tenant (dual-seal required)
// ===========================================

forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "bme-student-bodv75_hu" &&
    principal.acting_tenant == "oszk-security-services_hu"
}
advice {
    "forbid_rule_id": "forbid-osz-to-bme",
    "audit_dual_seal": true,
    "audit_class": "EVT-J155-CEDAR-DENY-CROSS-TENANT-LMS-PROBE-005"
};

// ===========================================
// FORBID-2 — BME cannot read OSZK tenant (dual-seal required)
// ===========================================

forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "oszk-security-services_hu" &&
    principal.acting_tenant == "bme-student-bodv75_hu"
}
advice {
    "forbid_rule_id": "forbid-bme-to-osz",
    "audit_dual_seal": true,
    "audit_class": "EVT-J155-CEDAR-DENY-CROSS-TENANT-EMPLOYMENT-PROBE-016"
};

// ===========================================
// FORBID-3 — OSZK cannot read cohort analytics
// ===========================================

forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "bme-research-cohort-2026-sleep-grade-fall" &&
    principal.acting_tenant == "oszk-security-services_hu"
}
advice {
    "forbid_rule_id": "forbid-osz-to-cohort",
    "audit_dual_seal": true,
    "audit_class": "EVT-J155-CEDAR-DENY-CROSS-TENANT-COHORT-PROBE-017"
};

// ===========================================
// FORBID-4 — BME cannot read personal payments
// ===========================================

forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "personal-stefan-kovacs-hu" &&
    principal.acting_tenant == "bme-student-bodv75_hu" &&
    action in [Action::"payments.personal_read", Action::"payments.personal_transfer"]
}
advice {
    "forbid_rule_id": "forbid-bme-to-personal-payments",
    "audit_dual_seal": true,
    "audit_class": "EVT-J155-CEDAR-DENY-PERSONAL-PAYMENTS-PROBE-018"
};

// ===========================================
// FORBID-5 — WTD weekly cap hard-block at 48 hr
// ===========================================

forbid (
    principal,
    action == Action::"calendar.confirm_shift",
    resource is Tenant
) when {
    resource.tenant_id == "oszk-security-services_hu" &&
    context.wtd_weekly_running_avg + context.this_shift_hours > 48.0
}
advice {
    "forbid_rule_id": "forbid-wtd-cap-breach",
    "regulator_anchor": "EU-Working-Time-Directive-2003-88-EC-Art-6",
    "audit_class": "EVT-J155-CALENDAR-WTD-BLOCK-012"
};

// ===========================================
// FORBID-6 — Daily rest minimum (11 hr) hard-block
// ===========================================

forbid (
    principal,
    action == Action::"calendar.confirm_shift",
    resource is Tenant
) when {
    resource.tenant_id == "oszk-security-services_hu" &&
    context.hours_since_last_clock_out < 11.0
}
advice {
    "forbid_rule_id": "forbid-daily-rest-min-breach",
    "regulator_anchor": "EU-Working-Time-Directive-2003-88-EC-Art-3",
    "audit_class": "EVT-J155-CALENDAR-REST-MIN-BLOCK-019"
};

// ===========================================
// FORBID-7 — Sleep-grade cohort egress without pseudonymization
// ===========================================

forbid (
    principal == User::"oya.system.observability-pipeline",
    action == Action::"analytics.cohort_emit",
    resource is Tenant
) when {
    resource.tenant_id == "bme-research-cohort-2026-sleep-grade-fall" &&
    (context.pseudonymized == false ||
     context.identifier_columns_stripped == false ||
     context.cohort_consent_active == false)
}
advice {
    "forbid_rule_id": "forbid-cohort-egress-non-pseudonymized",
    "regulator_anchor": "EU-GDPR-Art-89-research-safeguards",
    "audit_class": "EVT-J155-COHORT-EGRESS-DENY-020"
};

// ===========================================
// PERMIT — DPO read-only on own tenant
// ===========================================

permit (
    principal in Role::"dpo",
    action in [Action::"audit.read_seal", Action::"audit.merkle_proof"],
    resource is Tenant
) when {
    principal.dpo_of_tenant == resource.tenant_id
};
