// j161 — Soyeon Kim school cafeteria allergen recall Cedar policy
// Bundle id: cedar-bundle-school-meal-allergen-recall-v1
// Deployed to: seonhwa-cho-yuseong-daejeon-kr + kr-mfds-regulator-tenant +
//              kr-daejeon-deem-education-office-tenant + daewon-food-processing-ansan-kr +
//              cnuh-pediatric-er-tenant + kr-school-nutritionist-community-tenant +
//              805 parent personal tenants + KakaoTalk cross-tenant bridge
// Core doctrine: KR-FSA-Article-44 operator-authoritative food-service halt;
// KR-PIPA-2020-Article-15 lawful-safety-basis broadcast; per-family privacy preserved;
// KakaoTalk cross-tenant bridge is opt-in per family; Hangul UTF-8 NFC strict mode.

// ============================================================
// PERMIT — Soyeon as school nutritionist halts food service
// ============================================================

permit (
    principal == User::"soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
    action in [
        Action::"quality.food_service_halt",
        Action::"quality.batch_quarantine",
        Action::"quality.allergen_classify",
        Action::"workflow.recall_initiate",
        Action::"quality.ingredient_trace"
    ],
    resource is FoodServiceLine
) when {
    resource.tenant_id == "seonhwa-cho-yuseong-daejeon-kr" &&
    principal.has_certification_unexpired("KR-Nutritionist-License-2002") &&
    principal.has_certification_unexpired("KR-School-Nutritionist-2011") &&
    principal.has_certification_unexpired("ISO-22000-2018-Competency") &&
    principal.role_in_tenant("seonhwa-cho-yuseong-daejeon-kr") == "school_nutritionist_manager" &&
    context.service_window_active == true
};

// ============================================================
// PERMIT — Vice-principal endorses incident report + co-signs
// ============================================================

permit (
    principal,
    action in [
        Action::"quality.incident_report_co_sign",
        Action::"quality.capa_endorse"
    ],
    resource is IncidentReport
) when {
    resource.tenant_id == "seonhwa-cho-yuseong-daejeon-kr" &&
    principal.role_in_tenant("seonhwa-cho-yuseong-daejeon-kr") in ["vice_principal", "principal"]
};

// ============================================================
// PERMIT — Per-family privacy-preserving broadcast
// ============================================================

permit (
    principal,
    action == Action::"messenger.broadcast.per_family_privacy_preserving",
    resource is BroadcastEnvelope
) when {
    resource.broadcast_class in [
        "school_meal_allergen_recall_urgent",
        "school_meal_allergen_update",
        "school_safety_notice"
    ] &&
    principal.tenant == "seonhwa-cho-yuseong-daejeon-kr" &&
    context.kr_pipa_consent_basis in [
        "kr_pipa_2020_article_15_lawful_safety_basis",
        "kr_pipa_2020_article_15_explicit_consent_basis"
    ] &&
    context.cross_family_leakage_count == 0 &&
    context.unicode_normalization == "NFC" &&
    context.diacritics_preserved == true
};

// ============================================================
// PERMIT — KakaoTalk crossover for family-opt-in
// ============================================================

permit (
    principal,
    action == Action::"messenger.kakaotalk_cross_tenant_bridge_for_family",
    resource is KakaoTalkBridgeChannel
) when {
    resource.family_kakaotalk_crossover_opted_in == true &&
    context.broadcast_class in [
        "school_meal_allergen_recall_urgent",
        "school_safety_notice"
    ] &&
    context.kr_pipa_consent_basis == "lawful_safety_basis"
};

// ============================================================
// PERMIT — MFDS regulator notification
// ============================================================

permit (
    principal,
    action == Action::"compliance.kr_mfds_notification",
    resource is RegulatorNotification
) when {
    principal.tenant == "seonhwa-cho-yuseong-daejeon-kr" &&
    resource.regulator_tenant == "kr-mfds-regulator-tenant" &&
    resource.kr_fsa_article_ref.contains_all(["44", "86"]) &&
    context.sla_clock_hours <= 24 &&
    context.unicode_normalization == "NFC"
};

// ============================================================
// PERMIT — DEEM district notification
// ============================================================

permit (
    principal,
    action == Action::"compliance.kr_deem_notification",
    resource is RegulatorNotification
) when {
    principal.tenant == "seonhwa-cho-yuseong-daejeon-kr" &&
    resource.regulator_tenant == "kr-daejeon-deem-education-office-tenant" &&
    resource.kr_school_meals_act_ref.contains_all(["17", "19"]) &&
    context.sla_clock_hours <= 4
};

// ============================================================
// PERMIT — Supplier escalation cross-tenant thread
// ============================================================

permit (
    principal,
    action in [
        Action::"messenger.cross_tenant_thread_create",
        Action::"messenger.cross_tenant_thread_post"
    ],
    resource is MessengerGroup
) when {
    resource.thread_class == "supplier_escalation" &&
    resource.tenant_set.contains(principal.tenant)
};

// ============================================================
// PERMIT — Hospital cross-tenant share with parent consent
// ============================================================

permit (
    principal,
    action == Action::"tenancy.cross_tenant_status_share",
    resource is PatientStatusShareEnvelope
) when {
    resource.share_class == "pediatric_emergency_status_update" &&
    context.kr_pipa_consent_basis == "parents_consent" &&
    context.parent_consent_active == true &&
    resource.payload_scope_bounded == true
};

// ============================================================
// PERMIT — Community participation from school nutritionist
// ============================================================

permit (
    principal,
    action in [
        Action::"community.post",
        Action::"community.reply",
        Action::"community.read_thread"
    ],
    resource is CommunityGroup
) when {
    resource.community_tenant == "kr-school-nutritionist-community-tenant" &&
    principal.role_in_tenant(principal.tenant) in ["school_nutritionist_manager", "school_nutritionist_assistant"] &&
    principal.membership_in_community == true
};

// ============================================================
// FORBID-1 — Cert-missing halt attempt
// ============================================================

forbid (
    principal,
    action == Action::"quality.food_service_halt",
    resource is FoodServiceLine
) when {
    !principal.has_certification_unexpired("KR-Nutritionist-License-2002") ||
    !principal.has_certification_unexpired("KR-School-Nutritionist-2011") ||
    !principal.has_certification_unexpired("ISO-22000-2018-Competency")
}
advice {
    "forbid_rule_id": "forbid-cert-missing-or-expired",
    "audit_class": "EVT-J161-CEDAR-DENY-CERT-MISSING-014a",
    "escalation_path": "alert_school_nutritionist_plus_vp_via_priority_push"
};

// ============================================================
// FORBID-2 — Halt outside service window
// ============================================================

forbid (
    principal,
    action == Action::"quality.food_service_halt",
    resource is FoodServiceLine
) when {
    context.service_window_active == false
}
advice {
    "forbid_rule_id": "forbid-halt-outside-service-window",
    "audit_class": "EVT-J161-CEDAR-DENY-OFF-WINDOW-014b"
};

// ============================================================
// FORBID-3 — Parent broadcast without PIPA consent basis
// ============================================================

forbid (
    principal,
    action == Action::"messenger.broadcast.per_family_privacy_preserving",
    resource is BroadcastEnvelope
) when {
    context.kr_pipa_consent_basis not in [
        "kr_pipa_2020_article_15_lawful_safety_basis",
        "kr_pipa_2020_article_15_explicit_consent_basis"
    ]
}
advice {
    "forbid_rule_id": "forbid-broadcast-without-pipa-consent",
    "audit_class": "EVT-J161-CEDAR-DENY-PIPA-CONSENT-014c"
};

// ============================================================
// FORBID-4 — Cross-family broadcast leakage
// ============================================================

forbid (
    principal,
    action == Action::"messenger.broadcast.per_family_privacy_preserving",
    resource is BroadcastEnvelope
) when {
    context.cross_family_leakage_count > 0
}
advice {
    "forbid_rule_id": "forbid-cross-family-leakage",
    "audit_class": "EVT-J161-CEDAR-DENY-PER-FAMILY-LEAK-014d",
    "abort_broadcast_entirely": true
};

// ============================================================
// FORBID-5 — MFDS notification with invalid filing
// ============================================================

forbid (
    principal,
    action == Action::"compliance.kr_mfds_notification",
    resource is RegulatorNotification
) when {
    !resource.kr_fsa_article_ref.contains_all(["44", "86"]) ||
    resource.suspected_ingredient == "" ||
    resource.recall_action_taken == ""
}
advice {
    "forbid_rule_id": "forbid-mfds-invalid-filing",
    "audit_class": "EVT-J161-CEDAR-DENY-MFDS-INVALID-014e"
};

// ============================================================
// FORBID-6 — Supplier reads school payroll
// ============================================================

forbid (
    principal,
    action in [
        Action::"payroll.read_employee",
        Action::"payroll.list_employees"
    ],
    resource is PayrollRecord
) when {
    principal.tenant == "daewon-food-processing-ansan-kr" ||
    principal.tenant != resource.employer_tenant
}
advice {
    "forbid_rule_id": "forbid-cross-tenant-payroll-read",
    "audit_class": "EVT-J161-CEDAR-DENY-SUPPLIER-PAYROLL-014f",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-7 — Hangul Romanization in legal/regulator fields
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.legal_name_write",
        Action::"compliance.kr_mfds_notification",
        Action::"compliance.kr_deem_notification",
        Action::"quality.allergen_classify",
        Action::"workflow.recall_initiate"
    ],
    resource
) when {
    context.unicode_normalization != "NFC" ||
    context.diacritics_preserved == false ||
    (context.romanize_to_ascii == true && context.user_explicit_consent_to_romanize == false)
}
advice {
    "forbid_rule_id": "forbid-hangul-romanize-in-legal",
    "audit_class": "EVT-J161-CEDAR-DENY-NAME-ROMANIZE-014g",
    "exception": "EN_cross_reference_field_in_bilingual_context_only"
};

// ============================================================
// FORBID-8 — State machine invalid skip
// ============================================================

forbid (
    principal,
    action == Action::"workflow.recall_transition",
    resource is RecallWorkflow
) when {
    (context.from_state == "service_halted" && context.to_state in ["parent_notified", "regulator_notified", "root_cause_confirmed", "closure_post_mortem"]) ||
    (context.from_state == "quarantine_complete" && context.to_state in ["regulator_notified", "closure_post_mortem"]) ||
    (context.from_state == "ingredient_traced" && context.to_state == "closure_post_mortem") ||
    (context.from_state == "parent_notified" && context.to_state == "closure_post_mortem") ||
    (context.from_state == "regulator_notified" && context.to_state == "closure_post_mortem")
}
advice {
    "forbid_rule_id": "forbid-recall-invalid-skip",
    "audit_class": "EVT-J161-CEDAR-DENY-STATE-MACHINE-014h"
};

// ============================================================
// FORBID-9 — KakaoTalk crossover without family opt-in
// ============================================================

forbid (
    principal,
    action == Action::"messenger.kakaotalk_cross_tenant_bridge_for_family",
    resource is KakaoTalkBridgeChannel
) when {
    resource.family_kakaotalk_crossover_opted_in == false
}
advice {
    "forbid_rule_id": "forbid-kakaotalk-no-opt-in",
    "audit_class": "EVT-J161-CEDAR-DENY-KAKAO-NO-OPT-IN-014i",
    "alternate_channel": "oyatie_push_or_sms_fallback"
};

// ============================================================
// FORBID-10 — Hospital cross-tenant share without parent consent
// ============================================================

forbid (
    principal,
    action == Action::"tenancy.cross_tenant_status_share",
    resource is PatientStatusShareEnvelope
) when {
    context.parent_consent_active == false ||
    context.kr_pipa_consent_basis != "parents_consent"
}
advice {
    "forbid_rule_id": "forbid-hospital-share-without-consent",
    "audit_class": "EVT-J161-CEDAR-DENY-HOSPITAL-CONSENT-014j",
    "doctrine_anchor": "KR-PIPA-2020-Article-29-sensitive-info"
};

// ============================================================
// FORBID-11 — KR data residency drift
// ============================================================

forbid (
    principal,
    action,
    resource
) when {
    principal.tenant_country == "kr" &&
    resource.target_cell not in ["ap-seoul-primary", "ap-busan-secondary", "ap-osaka-tertiary"]
}
advice {
    "forbid_rule_id": "forbid-kr-cell-drift",
    "audit_class": "EVT-J161-CEDAR-DENY-KR-CELL-DRIFT-014k",
    "doctrine_anchor": "KR-PIPA-2020"
};

// ============================================================
// FORBID-12 — Service halt requires manager-gate (doctrine refusal)
// ============================================================
// IMPORTANT: This forbid rule is intentionally INFORMATIONAL. It documents in
// policy that no principal/VP-approval path exists for the operator food-service
// halt action. If a future agent attempts to add a manager-gate path, this rule
// blocks at policy-evaluation time and produces a doctrine-violation audit.

forbid (
    principal,
    action == Action::"quality.food_service_halt",
    resource is FoodServiceLine
) when {
    context.requires_principal_or_vp_approval == true
}
advice {
    "forbid_rule_id": "forbid-manager-gate-doctrine-violation",
    "doctrine_anchor": "KR-FSA-Article-44 + ADR-0263 audit doctrine",
    "audit_class": "EVT-J161-DOCTRINE-NO-MANAGER-GATE-014l"
};
