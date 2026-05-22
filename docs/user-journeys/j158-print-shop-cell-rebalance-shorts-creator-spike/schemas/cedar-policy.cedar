// j158 — Hae-Won Kim dual-tenant disclosure + cell-rebalance Cedar policy
// Bundle id: cedar-bundle-dual-tenant-disclosure-v1
// Deployed to: personal-haewon-kim-kr + sungkyul-sangsa-print-co-kr
// ADR-0311 dual-tenant identity boundary (with creator-employer disclosure permit class)
// ADR-0244 tenant scoping; ADR-0263 audit dual-seal; ADR-0248 cellular architecture

// ============================================================
// PERMIT — Personal-tenant autoscale (internal)
// ============================================================

permit (
    principal == User::"haewon.kim@personal-haewon-kim-kr",
    action in [
        Action::"cell.autoscale_engage",
        Action::"cell.autoscale_observe",
        Action::"shorts.read_own_metrics",
        Action::"observability.read_personal"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "personal-haewon-kim-kr" &&
    principal.role_in_tenant("personal-haewon-kim-kr") == "owner"
};

// ============================================================
// PERMIT — Creator-employer disclosure signal (one-way, info-only)
// This is the ADR-0311 §dual-tenant-boundary-with-creator-employer-disclosure permit.
// ============================================================

permit (
    principal == User::"haewon.kim@personal-haewon-kim-kr",
    action == Action::"messenger.creator_employer_disclosure_signal",
    resource is Tenant
) when {
    resource.tenant_id == "sungkyul-sangsa-print-co-kr" &&
    principal.has_active_disclosure_record(
        disclosure_id: "disclosure-haewon-kim-sungkyul-sangsa-2024-08-12"
    ) &&
    context.payload_class in [
        "creator_spike_info_only",
        "creator_pause_info_only",
        "creator_milestone_info_only"
    ] &&
    context.payload_no_audience_pii == true &&
    context.payload_no_revenue_figures == true &&
    context.payload_no_audience_demographics == true &&
    context.spike_coarse_grain_only == true &&
    context.self_initiated_by_subject == true &&
    context.payload_size_bytes <= 1024
};

// ============================================================
// PERMIT — Employer-side logistics coordinator role
// ============================================================

permit (
    principal == User::"haewon.kim@sungkyul-sangsa-print-co-kr",
    action in [
        Action::"workflow.cell_rebalance_create",
        Action::"workflow.cell_rebalance_propose",
        Action::"workflow.cell_rebalance_transition",
        Action::"workflow.cell_traffic_shift",
        Action::"workflow.cell_rebalance_post_validate",
        Action::"tasks.materialize",
        Action::"tasks.complete",
        Action::"crm.inquiry_route"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "sungkyul-sangsa-print-co-kr" &&
    principal.role_in_tenant("sungkyul-sangsa-print-co-kr") == "logistics_coordinator" &&
    context.shift_active == true
};

// ============================================================
// PERMIT — Employer-side owner role (co-sign rebalance)
// ============================================================

permit (
    principal == User::"lee.minjun@sungkyul-sangsa-print-co-kr",
    action in [
        Action::"workflow.cell_rebalance_co_sign",
        Action::"workflow.cell_rebalance_decline",
        Action::"workflow.cell_rebalance_post_validate_endorse"
    ],
    resource is RebalanceWorkflow
) when {
    principal.role_in_tenant("sungkyul-sangsa-print-co-kr") == "owner" &&
    resource.owner_tenant == "sungkyul-sangsa-print-co-kr" &&
    context.passkey_assertion_present == true &&
    context.face_id_assertion_present == true
};

// ============================================================
// PERMIT — KR-LSA compliance evaluator
// ============================================================

permit (
    principal in Role::"oya.system.kr-lsa-evaluator",
    action == Action::"compliance.kr_lsa_evaluate",
    resource is Tenant
) when {
    resource.tenant_id == "sungkyul-sangsa-print-co-kr"
};

// ============================================================
// FORBID-1 — Employer cannot query personal tenant (immutable)
// This is the most important rule of the dual-tenant boundary.
// ============================================================

forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "personal-haewon-kim-kr" &&
    principal.acting_tenant == "sungkyul-sangsa-print-co-kr"
}
advice {
    "forbid_rule_id": "forbid-employer-to-personal",
    "audit_dual_seal": true,
    "audit_class": "EVT-J158-CEDAR-DENY-EMPLOYER-TO-PERSONAL-008"
};

// ============================================================
// FORBID-2 — Personal cannot query employer without disclosure permit
// ============================================================

forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "sungkyul-sangsa-print-co-kr" &&
    principal.acting_tenant == "personal-haewon-kim-kr" &&
    action != Action::"messenger.creator_employer_disclosure_signal"
}
advice {
    "forbid_rule_id": "forbid-personal-to-employer-no-permit",
    "audit_dual_seal": true,
    "audit_class": "EVT-J158-CEDAR-DENY-PERSONAL-TO-EMPLOYER-NO-PERMIT-008a"
};

// ============================================================
// FORBID-3 — Disclosure signal with PII (DLP-flagged)
// ============================================================

forbid (
    principal,
    action == Action::"messenger.creator_employer_disclosure_signal",
    resource is Tenant
) when {
    context.payload_no_audience_pii == false ||
    context.payload_no_revenue_figures == false ||
    context.payload_no_audience_demographics == false
}
advice {
    "forbid_rule_id": "forbid-disclosure-pii-leak",
    "audit_dual_seal": true,
    "audit_class": "EVT-J158-DISCLOSURE-PII-LEAK-DENY-008b"
};

// ============================================================
// FORBID-4 — Disclosure signal size overflow
// ============================================================

forbid (
    principal,
    action == Action::"messenger.creator_employer_disclosure_signal",
    resource is Tenant
) when {
    context.payload_size_bytes > 1024
}
advice {
    "forbid_rule_id": "forbid-disclosure-size-overflow",
    "audit_dual_seal": true,
    "audit_class": "EVT-J158-DISCLOSURE-SIZE-OVERFLOW-DENY-008c"
};

// ============================================================
// FORBID-5 — Disclosure signal without active disclosure record
// ============================================================

forbid (
    principal,
    action == Action::"messenger.creator_employer_disclosure_signal",
    resource is Tenant
) when {
    !principal.has_active_disclosure_record(
        disclosure_id: "disclosure-haewon-kim-sungkyul-sangsa-2024-08-12"
    )
}
advice {
    "forbid_rule_id": "forbid-disclosure-inactive",
    "audit_dual_seal": true,
    "audit_class": "EVT-J158-DISCLOSURE-INACTIVE-DENY-002a"
};

// ============================================================
// FORBID-6 — Rebalance state-machine invalid skip
// ============================================================

forbid (
    principal,
    action == Action::"workflow.cell_rebalance_transition",
    resource is RebalanceWorkflow
) when {
    (context.from_state == "capacity_signal_detected" && context.to_state in ["cross_cell_grant_negotiated", "traffic_shift", "post_rebalance_validation"]) ||
    (context.from_state == "rebalance_proposed" && context.to_state in ["traffic_shift", "post_rebalance_validation"]) ||
    (context.from_state == "cross_cell_grant_negotiated" && context.to_state == "post_rebalance_validation")
}
advice {
    "forbid_rule_id": "forbid-rebalance-state-machine-skip",
    "audit_class": "EVT-J158-WORKFLOW-INVALID-TRANSITION-008d"
};

// ============================================================
// FORBID-7 — KR-LSA weekly cap breach blocks shift assignment
// ============================================================

forbid (
    principal,
    action in [
        Action::"workflow.shift_assign",
        Action::"production-planning.assign_extra_shift"
    ],
    resource is StaffMember
) when {
    context.projected_weekly_hours > 52.0
}
advice {
    "forbid_rule_id": "forbid-kr-lsa-overcap",
    "regulator_anchor": "KR-Labour-Standards-Act-§50",
    "audit_class": "EVT-J158-KR-LSA-OVERCAP-DENY-008e"
};

// ============================================================
// FORBID-8 — Hangul normalization in identity/legal fields
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.legal_name_write",
        Action::"identity.preferred_name_write",
        Action::"contract.signatory_write"
    ],
    resource
) when {
    context.unicode_normalization != "NFC" ||
    context.hangul_preserved != true
}
advice {
    "forbid_rule_id": "forbid-hangul-normalize",
    "audit_dual_seal": true,
    "audit_class": "EVT-J158-HANGUL-NORMALIZE-DENY-008f"
};

// ============================================================
// FORBID-9 — Owner co-sign requires two-factor (passkey + face_id)
// ============================================================

forbid (
    principal,
    action == Action::"workflow.cell_rebalance_co_sign",
    resource is RebalanceWorkflow
) when {
    context.passkey_assertion_present == false ||
    context.face_id_assertion_present == false
}
advice {
    "forbid_rule_id": "forbid-owner-cosign-without-two-factor",
    "audit_class": "EVT-J158-OWNER-COSIGN-TWO-FACTOR-DENY-008g"
};
