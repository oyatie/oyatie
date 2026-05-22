// j160 — Tomáš Horák cleaning-co cross-tenant bid + onboard Cedar policy
// Bundle id: cedar-bundle-cleaning-bid-onboard-v1
// Deployed to: uklid-horak-sro-plzen-cz + polycraft-bohemia-as-plzen-cz +
//              cz-ossz-state-tenant + cz-vzp-zp-tenant + cz-financni-urad-tenant +
//              cz-datova-schranka-tenant + cz-cleaning-industry-owner-operators-community
// Core doctrine: vendor-client one-way data isolation; QES dual-tenant signing under TrueTime;
// biometric-badge cross-tenant enrollment is Cedar-scoped not ambient; Czech state-systems
// integration is first-class via per-system permits; diacritic-strict mode mandatory for
// Czech + Vietnamese + Slovak + Ukrainian + Cyrillic names.

// ============================================================
// PERMIT — Tomáš submits bid as Úklid Horák
// ============================================================

permit (
    principal == User::"tomas.horak@uklid-horak-sro-plzen-cz",
    action == Action::"marketplace.bid_submit",
    resource is BidRequest
) when {
    resource.target_tenant != principal.tenant &&
    principal.has_certification_unexpired("ISSA-CIMS-2024") &&
    principal.has_certification_unexpired("CSN-EN-ISO-9001-2015") &&
    principal.business_register_verified_via_ARES == true &&
    principal.cz_vat_id_active == true &&
    context.bid_window_open == true &&
    context.unicode_normalization == "NFC" &&
    context.diacritics_preserved == true
};

// ============================================================
// PERMIT — Tomáš + Procházková exchange contract drafts
// ============================================================

permit (
    principal,
    action in [
        Action::"contract.draft_revision",
        Action::"contract.draft_read"
    ],
    resource is Contract
) when {
    principal.tenant in resource.party_tenants &&
    resource.is_draft == true
};

// ============================================================
// PERMIT — QES dual-tenant contract sign under TrueTime fence
// ============================================================

permit (
    principal,
    action == Action::"contract.qes_sign_dual_tenant",
    resource is Contract
) when {
    principal.tenant in resource.party_tenants &&
    principal.qes_certificate_unexpired == true &&
    principal.qes_eidas_qualified == true &&
    context.truetime_uncertainty_ms <= 10 &&
    context.diacritics_preserved == true
};

// ============================================================
// PERMIT — Datová schránka notification
// ============================================================

permit (
    principal,
    action == Action::"cz_datova_schranka.notify",
    resource is DatovaSchrankaMessage
) when {
    resource.owner_ic == principal.cz_business_ic ||
    resource.owner_ic in principal.represented_business_ics
};

// ============================================================
// PERMIT — Crew member added to contract scope
// ============================================================

permit (
    principal,
    action == Action::"workflow.crew_assign_to_contract",
    resource is CrewAssignment
) when {
    principal.role_in_tenant(resource.employer_tenant) in [
        "owner_operator",
        "managing_director",
        "foreman"
    ] &&
    principal.tenant == resource.employer_tenant
};

// ============================================================
// PERMIT — OSSZ employee registration (Czech state system)
// ============================================================

permit (
    principal,
    action == Action::"cz_ossz.employee_register",
    resource is OsszRegistration
) when {
    principal.role_in_tenant(resource.employer_tenant) in [
        "owner_operator",
        "managing_director",
        "hr_administrator"
    ] &&
    resource.within_8_day_window == true &&
    context.diacritics_preserved == true
};

// ============================================================
// PERMIT — ZP registration (Czech state system)
// ============================================================

permit (
    principal,
    action == Action::"cz_zdravotni_pojistovna.employee_register",
    resource is ZpRegistration
) when {
    principal.role_in_tenant(resource.employer_tenant) in [
        "owner_operator",
        "managing_director",
        "hr_administrator"
    ] &&
    resource.zp_class in ["vzp", "ozp", "zpmv", "vozp", "rbp", "zp-ma", "cpzp"]
};

// ============================================================
// PERMIT — FÚ tax registration
// ============================================================

permit (
    principal,
    action == Action::"cz_financni_urad.employee_tax_register",
    resource is FuTaxRegistration
) when {
    principal.role_in_tenant(resource.employer_tenant) in [
        "owner_operator",
        "managing_director",
        "hr_administrator"
    ]
};

// ============================================================
// PERMIT — Biometric badge cross-tenant enrollment (Cedar-scoped)
// ============================================================

permit (
    principal,
    action == Action::"identity.biometric_badge_enroll_to_client_access_system",
    resource is ClientAccessSystem
) when {
    resource.client_tenant == "polycraft-bohemia-as-plzen-cz" &&
    principal.crew_assigned_to_contract == "contract-uklid-horak-polycraft-2027-01-04" &&
    principal.has_completed_training("csn-262-2006-safety") &&
    principal.has_completed_training("gdpr-data-handling") &&
    principal.has_completed_training("polycraft-induction") &&
    context.auto_revoke_on_contract_end == true &&
    context.scope_limited_to_named_site == true
};

// ============================================================
// PERMIT — First shift gate scan
// ============================================================

permit (
    principal is User,
    action == Action::"identity.biometric_gate_scan",
    resource is ClientAccessSystem
) when {
    resource.client_tenant == "polycraft-bohemia-as-plzen-cz" &&
    principal.biometric_badge_active == true &&
    context.scheduled_shift == true
};

// ============================================================
// PERMIT — Community participation from cleaning-industry tenant
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
    resource.community_tenant == "cz-cleaning-industry-owner-operators-community" &&
    principal.tenant_class == "business_tenant" &&
    principal.membership_in_community == true
};

// ============================================================
// FORBID-1 — Bid submit without required certifications
// ============================================================

forbid (
    principal,
    action == Action::"marketplace.bid_submit",
    resource is BidRequest
) when {
    !principal.has_certification_unexpired("ISSA-CIMS-2024") ||
    !principal.has_certification_unexpired("CSN-EN-ISO-9001-2015")
}
advice {
    "forbid_rule_id": "forbid-cert-missing-or-expired",
    "audit_class": "EVT-J160-CEDAR-DENY-CERT-MISSING-014a"
};

// ============================================================
// FORBID-2 — Bid window closed
// ============================================================

forbid (
    principal,
    action == Action::"marketplace.bid_submit",
    resource is BidRequest
) when {
    context.bid_window_open == false ||
    context.now > resource.bid_deadline
}
advice {
    "forbid_rule_id": "forbid-bid-window-closed",
    "audit_class": "EVT-J160-CEDAR-DENY-BID-WINDOW-CLOSED-014b",
    "ui_message_cs": "Termín pro podání nabídek vypršel."
};

// ============================================================
// FORBID-3 — Diacritic ASCII transliteration in legal field
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.legal_name_write",
        Action::"cz_ossz.employee_register",
        Action::"cz_zdravotni_pojistovna.employee_register",
        Action::"cz_financni_urad.employee_tax_register",
        Action::"contract.party_legal_entity_write",
        Action::"marketplace.bid_submit"
    ],
    resource
) when {
    context.unicode_normalization != "NFC" ||
    context.diacritics_preserved == false ||
    (context.transliterate_to_ascii == true && context.user_explicit_consent_to_transliterate == false)
}
advice {
    "forbid_rule_id": "forbid-diacritic-transliterate",
    "audit_class": "EVT-J160-CEDAR-DENY-NAME-TRANSLITERATE-014c",
    "exception": "ICAO_9303_passport_ASCII_explicit_field_only"
};

// ============================================================
// FORBID-4 — Biometric badge enroll without training prereqs
// ============================================================

forbid (
    principal,
    action == Action::"identity.biometric_badge_enroll_to_client_access_system",
    resource is ClientAccessSystem
) when {
    !principal.has_completed_training("csn-262-2006-safety") ||
    !principal.has_completed_training("gdpr-data-handling") ||
    !principal.has_completed_training("polycraft-induction")
}
advice {
    "forbid_rule_id": "forbid-badge-training-prereq-missing",
    "audit_class": "EVT-J160-CEDAR-DENY-BADGE-TRAINING-MISSING-014d"
};

// ============================================================
// FORBID-5 — Cross-tenant payroll read (vendor's data to client)
// ============================================================

forbid (
    principal,
    action in [
        Action::"payroll.read_employee",
        Action::"payroll.list_employees",
        Action::"payroll.read_compensation"
    ],
    resource is PayrollRecord
) when {
    principal.tenant != resource.employer_tenant
}
advice {
    "forbid_rule_id": "forbid-cross-tenant-payroll-read",
    "audit_class": "EVT-J160-CEDAR-DENY-PAYROLL-CROSS-TENANT-014e",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-6 — Cross-tenant customer-list read (client's data to vendor)
// ============================================================

forbid (
    principal,
    action in [
        Action::"crm.list_customers",
        Action::"crm.read_customer_pipeline",
        Action::"crm.read_competitor_intel"
    ],
    resource is Crm
) when {
    principal.tenant != resource.owner_tenant
}
advice {
    "forbid_rule_id": "forbid-cross-tenant-customer-read",
    "audit_class": "EVT-J160-CEDAR-DENY-CUSTOMER-CROSS-TENANT-014f",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-7 — Contract sign with incomplete dual-tenant signature
// ============================================================

forbid (
    principal,
    action == Action::"contract.execute",
    resource is Contract
) when {
    resource.qes_signatures_count < 2 ||
    resource.party_tenants_signed_count < 2
}
advice {
    "forbid_rule_id": "forbid-contract-incomplete-signature",
    "audit_class": "EVT-J160-CEDAR-DENY-CONTRACT-INCOMPLETE-SIG-014g"
};

// ============================================================
// FORBID-8 — ARES business-register verification failure
// ============================================================

forbid (
    principal,
    action == Action::"marketplace.bid_submit",
    resource is BidRequest
) when {
    principal.business_register_verified_via_ARES == false ||
    principal.cz_vat_id_active == false
}
advice {
    "forbid_rule_id": "forbid-ares-verification-failed",
    "audit_class": "EVT-J160-CEDAR-DENY-ARES-014h"
};

// ============================================================
// FORBID-9 — Cell residency drift (Czech tenant data outside EU-Prague)
// ============================================================

forbid (
    principal,
    action,
    resource
) when {
    principal.tenant_country == "cz" &&
    resource.target_cell not in ["eu-prague-primary", "eu-frankfurt-secondary", "eu-vienna-tertiary"]
}
advice {
    "forbid_rule_id": "forbid-cz-cell-drift",
    "audit_class": "EVT-J160-CEDAR-DENY-CZ-CELL-DRIFT-014i",
    "doctrine_anchor": "CZ-110-2019-Sb + EU-GDPR"
};

// ============================================================
// FORBID-10 — TrueTime fence breach on QES sign
// ============================================================

forbid (
    principal,
    action == Action::"contract.qes_sign_dual_tenant",
    resource is Contract
) when {
    context.truetime_uncertainty_ms > 10
}
advice {
    "forbid_rule_id": "forbid-truetime-breach",
    "audit_class": "EVT-J160-CEDAR-DENY-TRUETIME-BREACH-014j",
    "doctrine_anchor": "ADR-0252"
};

// ============================================================
// FORBID-11 — Biometric badge scope drift (any site beyond contract)
// ============================================================

forbid (
    principal,
    action == Action::"identity.biometric_badge_enroll_to_client_access_system",
    resource is ClientAccessSystem
) when {
    resource.site_id not in principal.contract_scoped_site_ids
}
advice {
    "forbid_rule_id": "forbid-biometric-scope-drift",
    "audit_class": "EVT-J160-CEDAR-DENY-BADGE-SCOPE-DRIFT-014k"
};

// ============================================================
// FORBID-12 — OSSZ registration outside 8-day window
// ============================================================

forbid (
    principal,
    action == Action::"cz_ossz.employee_register",
    resource is OsszRegistration
) when {
    resource.within_8_day_window == false
}
advice {
    "forbid_rule_id": "forbid-ossz-window-breach",
    "audit_class": "EVT-J160-CEDAR-DENY-OSSZ-WINDOW-014l",
    "doctrine_anchor": "CZ-OZ-262-2006-§38"
};
