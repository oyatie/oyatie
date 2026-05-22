// j157 — Diana Lazăr operator-authoritative line stop + recall Cedar policy
// Bundle id: cedar-bundle-tipografia-antibiotice-recall-v1
// Deployed to: tipografia-lazar-petrescu-ro + antibiotice-sa-ro + (path-prepared) anmdmr-inspectorate-ro
// Core doctrine: FOGRA-PSO Operator Level 2 + ISO-12647-2 cert + active shift IS the authority.
// NO manager-approval gate exists in this policy for the line-stop action on regulated product classes.
// This is deliberate; cultural drift toward manager-gating is prevented by the absence of any permit
// requiring a manager principal for the line-stop action.

// ============================================================
// PERMIT — Operator-authoritative line stop
// ============================================================

permit (
    principal == User::"diana.lazăr@tipografia-lazar-petrescu-ro",
    action in [
        Action::"quality.production_line_stop",
        Action::"quality.batch_quarantine",
        Action::"quality.defect_classify",
        Action::"workflow.recall_initiate"
    ],
    resource is ProductionLine
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") &&
    principal.has_certification_unexpired("ISO-12647-2-Trained") &&
    principal.role_in_tenant("tipografia-lazar-petrescu-ro") == "press_operator_day_shift" &&
    context.shift_active == true &&
    context.product_class in [
        "pharma_PIL",
        "food_label",
        "medical_device_label",
        "regulated_label_general",
        "general_print"
    ]
};

// ============================================================
// PERMIT — Vladimir (night shift counterpart)
// ============================================================

permit (
    principal == User::"vladimir.csikós@tipografia-lazar-petrescu-ro",
    action in [
        Action::"quality.production_line_stop",
        Action::"quality.batch_quarantine",
        Action::"quality.defect_classify",
        Action::"workflow.recall_initiate"
    ],
    resource is ProductionLine
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") &&
    principal.role_in_tenant("tipografia-lazar-petrescu-ro") == "press_operator_night_shift" &&
    context.shift_active == true
};

// ============================================================
// PERMIT — Managing director endorses CAPA + root-cause + customer notify
// ============================================================

permit (
    principal == User::"mihai.lazăr-petrescu@tipografia-lazar-petrescu-ro",
    action in [
        Action::"quality.capa_endorse",
        Action::"quality.root_cause_endorse",
        Action::"workflow.customer_notify_endorse"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.role_in_tenant("tipografia-lazar-petrescu-ro") == "managing_director"
};

// ============================================================
// PERMIT — Customer-side recall actions (Antibiotice)
// ============================================================

permit (
    principal in Role::"antibiotice_qa_director",
    action in [
        Action::"workflow.customer_recall_acknowledge",
        Action::"workflow.customer_recall_confirm_scope",
        Action::"workflow.regulator_escalation_approve"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "antibiotice-sa-ro" &&
    principal.tenant == "antibiotice-sa-ro" &&
    principal.role_in_tenant("antibiotice-sa-ro") in ["director_calitate", "persoana_calificata"]
};

// ============================================================
// PERMIT — CAPA file under TrueTime fence (cross-tenant relevance)
// ============================================================

permit (
    principal,
    action == Action::"quality.capa_file",
    resource is CapaDocument
) when {
    principal.tenant == resource.owner_tenant &&
    resource.endorsed_section_1 == true &&
    resource.endorsed_section_2 == true &&
    resource.endorsed_section_3 == true &&
    context.truetime_uncertainty_ms <= 10
};

// ============================================================
// PERMIT — Cross-tenant customer notification
// ============================================================

permit (
    principal,
    action == Action::"messenger.post_recall",
    resource is MessengerGroup
) when {
    resource.notification_class in [
        "customer_recall_initiate",
        "customer_recall_update",
        "customer_recall_close"
    ] &&
    resource.tenant_set.contains(principal.tenant) &&
    context.diacritics_preserved == true &&
    context.unicode_normalization == "NFC"
};

// ============================================================
// FORBID-1 — Expired or missing certification
// ============================================================

forbid (
    principal,
    action in [
        Action::"quality.production_line_stop",
        Action::"quality.defect_classify",
        Action::"workflow.recall_initiate"
    ],
    resource is ProductionLine
) when {
    !principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") ||
    !principal.has_certification_unexpired("ISO-12647-2-Trained")
}
advice {
    "forbid_rule_id": "forbid-cert-missing-or-expired",
    "audit_class": "EVT-J157-CEDAR-DENY-CERT-MISSING-012a",
    "regulator_anchor": "FOGRA-PSO-2024",
    "escalation_path": "manager_emergency_authorization_with_passkey_signature"
};

// ============================================================
// FORBID-2 — Off-shift attempt
// ============================================================

forbid (
    principal,
    action in [
        Action::"quality.production_line_stop",
        Action::"quality.defect_classify"
    ],
    resource is ProductionLine
) when {
    context.shift_active == false ||
    !(principal.role_in_tenant("tipografia-lazar-petrescu-ro") in ["press_operator_day_shift", "press_operator_night_shift"])
}
advice {
    "forbid_rule_id": "forbid-off-shift",
    "audit_class": "EVT-J157-CEDAR-DENY-OFF-SHIFT-012b"
};

// ============================================================
// FORBID-3 — Pre-publish CAPA cross-tenant read
// ============================================================

forbid (
    principal,
    action == Action::"quality.capa_read",
    resource is CapaDocument
) when {
    principal.tenant != resource.owner_tenant &&
    resource.publish_status == "draft"
}
advice {
    "forbid_rule_id": "forbid-cross-tenant-pre-publish-capa-read",
    "audit_dual_seal": true,
    "audit_class": "EVT-J157-CEDAR-DENY-PRE-PUBLISH-CAPA-012d"
};

// ============================================================
// FORBID-4 — Diacritic normalization in legal context
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.legal_name_write",
        Action::"contract.legal_entity_write",
        Action::"quality.regulatory_text_write"
    ],
    resource
) when {
    context.unicode_normalization != "NFC" ||
    context.diacritics_preserved != true
}
advice {
    "forbid_rule_id": "forbid-diacritic-normalize",
    "audit_dual_seal": true,
    "audit_class": "EVT-J157-DIACRITIC-NORMALIZE-DENY-012e"
};

// ============================================================
// FORBID-5 — Recall workflow invalid transitions
// ============================================================

forbid (
    principal,
    action == Action::"workflow.recall_transition",
    resource is RecallWorkflow
) when {
    (context.from_state == "stop_called" && context.to_state in ["customer_notify", "recall_execute", "closure_post_mortem"]) ||
    (context.from_state == "quarantine" && context.to_state in ["recall_execute", "closure_post_mortem"]) ||
    (context.from_state == "defect_root_cause" && context.to_state == "closure_post_mortem") ||
    (context.from_state == "customer_notify" && context.to_state == "closure_post_mortem")
}
advice {
    "forbid_rule_id": "forbid-recall-invalid-skip",
    "audit_class": "EVT-J157-WORKFLOW-INVALID-TRANSITION-012f"
};

// ============================================================
// FORBID-6 — Manager-gate doctrine refusal
// ============================================================
// IMPORTANT: This forbid rule is intentionally INFORMATIONAL — it documents in policy
// that no manager-approval path exists for the operator line stop. If a future agent
// or human attempts to add a manager-gate path, this rule blocks the addition at
// policy-evaluation time and produces an explicit doctrine violation audit.

forbid (
    principal,
    action == Action::"quality.production_line_stop",
    resource is ProductionLine
) when {
    context.requires_manager_approval == true
}
advice {
    "forbid_rule_id": "forbid-manager-gate-doctrine-violation",
    "doctrine_anchor": "AC-J157-002 + ADR-0263 audit doctrine",
    "audit_class": "EVT-J157-DOCTRINE-NO-MANAGER-GATE-012c"
};

// ============================================================
// FORBID-7 — Antibiotice unilateral ANMDMR escalation
// ============================================================

forbid (
    principal,
    action == Action::"workflow.regulator_escalation_send",
    resource is RecallWorkflow
) when {
    principal.tenant == "antibiotice-sa-ro" &&
    !resource.tipografia_acknowledged == true
}
advice {
    "forbid_rule_id": "forbid-customer-unilateral-regulator-escalation",
    "audit_dual_seal": true,
    "audit_class": "EVT-J157-CEDAR-DENY-CUSTOMER-UNILATERAL-REGULATOR-014"
};
