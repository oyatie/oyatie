// j164 — Hiroshi Tanaka retiree tax-prep Cedar policy
// Bundle id: cedar-bundle-retiree-tax-prep-v1
// Deployed to: personal-hiroshi-tanaka-jp
// ADR-0244 tenant scoping; ADR-0263 audit; ADR-0251 compliance-pack; ADR-0311 personal-tenant
// Pack overlays: JP-PIPA + JP-Income-Tax-Act + JP-National-Pension-Act + My-Number-Scoping + JIS-X-8341-3

// ============================================================
// PERMIT — Hiroshi: own personal tenant operations
// ============================================================

permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action in [
        Action::"workflow_studio.tax_prep_open",
        Action::"workflow_studio.year_over_year_compare",
        Action::"workflow_studio.step_advance",
        Action::"workflow_studio.step_pause",
        Action::"workflow_studio.step_resume",
        Action::"workflow_studio.voice_command",
        Action::"payments.pension_reconcile_read",
        Action::"payments.tax_payment_history_read",
        Action::"payments.bank_cross_check",
        Action::"drive.receipt_room_read_write",
        Action::"drive.write_worm_locked",
        Action::"notes.diary_write",
        Action::"notes.voice_dictate",
        Action::"compliance.tax_form_draft",
        Action::"compliance.tax_form_review",
        Action::"compliance.tax_form_submit_etax",
        Action::"intelligence.ocr_receipt",
        Action::"identity.passkey_assert"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "personal-hiroshi-tanaka-jp" &&
    principal.role_in_tenant("personal-hiroshi-tanaka-jp") == "owner" &&
    context.accessibility_context_valid == true &&
    context.accessibility_substrate in ["jis-x-8341-3-certified"]
};

// ============================================================
// PERMIT — My-Number per-purpose scoping (PIPA + My-Number Act Article 19/32)
// ============================================================

permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action == Action::"identity.my_number_read",
    resource is MyNumberIdentifier
) when {
    resource.subject_principal == "hiroshi.tanaka@personal-hiroshi-tanaka-jp" &&
    context.purpose_scope in [
        "tax_filing_kakutei_shinkoku",
        "pension_reconciliation",
        "etax_submission",
        "pension_authority_read"
    ] &&
    context.purpose_declared_in_audit == true &&
    context.access_per_purpose_count_today <= 4 &&
    context.my_number_card_nfc_assertion_present == true
};

// ============================================================
// FORBID — My-Number access without declared purpose
// ============================================================

forbid (
    principal,
    action == Action::"identity.my_number_read",
    resource is MyNumberIdentifier
) when {
    !context.purpose_declared_in_audit ||
    context.purpose_scope == "" ||
    context.purpose_scope == "general"
};

// ============================================================
// FORBID — My-Number bleed across purposes
// ============================================================

forbid (
    principal,
    action == Action::"identity.my_number_use",
    resource is MyNumberIdentifier
) when {
    resource.last_purpose_scope != context.current_purpose_scope &&
    !context.fresh_purpose_authentication_present
};

// ============================================================
// PERMIT — e-Tax submission: requires My-Number Card NFC + passkey + form review
// ============================================================

permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action == Action::"compliance.tax_form_submit_etax",
    resource is TaxForm
) when {
    resource.tenant_id == "personal-hiroshi-tanaka-jp" &&
    resource.form_class == "kakutei_shinkoku_b" &&
    context.my_number_card_nfc_assertion_present == true &&
    context.passkey_assertion_present == true &&
    context.form_review_complete == true &&
    context.year_over_year_comparison_reviewed == true &&
    context.signature_algorithm == "JPKI-RSA-2048-with-passkey-counter-sign"
};

// ============================================================
// FORBID — e-Tax submission without form review
// ============================================================

forbid (
    principal,
    action == Action::"compliance.tax_form_submit_etax",
    resource is TaxForm
) when {
    !context.form_review_complete
};

// ============================================================
// PERMIT — Drive WORM receipt write (Japan 7-year retention)
// ============================================================

permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action in [
        Action::"drive.write_worm_locked",
        Action::"drive.engage_worm_lock"
    ],
    resource is DriveRoom
) when {
    resource.room_path == "personal-hiroshi-tanaka-jp/tax/fy2026/receipts" ||
    resource.room_path == "personal-hiroshi-tanaka-jp/tax/fy2026/submission" &&
    context.retention_years >= 7 &&
    context.retention_authority == "JP-Income-Tax-Act-148"
};

// ============================================================
// FORBID — Cross-tenant access to Hiroshi's tax data (daughter / son)
// ============================================================

forbid (
    principal,
    action == Action::"drive.read_room",
    resource is DriveRoom
) when {
    resource.room_path startsWith "personal-hiroshi-tanaka-jp/tax/" &&
    !context.succession_activation_present
};

// ============================================================
// PERMIT — Notes succession-ready archival (late-life record keeping)
// ============================================================

permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action in [
        Action::"notes.diary_write",
        Action::"notes.voice_dictate",
        Action::"notes.tag_succession_ready",
        Action::"notes.series_continuation_tag"
    ],
    resource is Notebook
) when {
    resource.notebook_id == "late-life-record-keeping" &&
    resource.tenant_id == "personal-hiroshi-tanaka-jp"
};

// ============================================================
// PERMIT — Accessibility substrate operations
// ============================================================

permit (
    principal in Role::"oya.system.accessibility-substrate",
    action in [
        Action::"accessibility.talkback_announce",
        Action::"accessibility.voice_command_resolve",
        Action::"accessibility.high_contrast_theme_apply",
        Action::"accessibility.haptic_feedback_emit"
    ],
    resource is Tenant
) when {
    context.accessibility_substrate_certified == "jis-x-8341-3" &&
    context.coverage_percent >= 100
};

// ============================================================
// PERMIT — Pension authority read (JPS feed)
// ============================================================

permit (
    principal in Role::"oya.system.payments-pension-bridge",
    action == Action::"payments.pension_authority_query",
    resource is Tenant
) when {
    context.my_number_short_lived_token_present == true &&
    context.token_purpose_scope == "pension_record_read_one_fiscal_year" &&
    context.token_expiry_within_1_hour == true
};

// ============================================================
// PERMIT — Intelligence OCR on receipts
// ============================================================

permit (
    principal in Role::"oya.system.intelligence-ocr",
    action == Action::"intelligence.ocr_receipt",
    resource is Tenant
) when {
    context.ocr_class == "japanese_medical_receipt" ||
    context.ocr_class == "japanese_property_tax_receipt" ||
    context.ocr_class == "japanese_honorarium_receipt"
};

// ============================================================
// PERMIT — Workflow studio in-context help
// ============================================================

permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action == Action::"workflow_studio.in_context_help_invoke",
    resource is WorkflowStep
) when {
    resource.tenant_id == "personal-hiroshi-tanaka-jp"
};

// ============================================================
// FORBID — My-Number export across tenants
// ============================================================

forbid (
    principal,
    action == Action::"identity.my_number_export_cross_tenant",
    resource is MyNumberIdentifier
);
