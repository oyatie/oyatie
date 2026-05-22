// j156 — Carlos Reyes II cross-tenant maintenance Cedar policy
// Bundle id: cedar-bundle-cascade-meridianstack-cross-tenant-v1
// Deployed to: cascade-fm-services-llc-us + meridianstack-hosting-co-us + (sub-bundle) trane-technologies-emergency-vendor-na
// ADR-0311 dual-tenant identity boundary (vendor → host cross-tenant grant)
// ADR-0244 tenant scoping; ADR-0263 audit dual-seal; ADR-0250 build-ahead-of-certification

// ============================================================
// PERMIT — Vendor tech acting under active cross-tenant grant
// ============================================================

permit (
    principal == User::"carlos.reyes-ii@cascade-fm-services-llc-us",
    action in [
        Action::"incident.acknowledge",
        Action::"incident.resolve",
        Action::"tasks.execute",
        Action::"tasks.complete",
        Action::"workflow.permit_sign",
        Action::"workflow.loto_lock",
        Action::"workflow.loto_release",
        Action::"workflow.loto_transition",
        Action::"workflow.epa608_disclose_release",
        Action::"plant.cmms_close_workorder",
        Action::"facility.physical_room_entry"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "meridianstack-hosting-co-us" &&
    principal.cross_tenant_grant_active(
        host_tenant: "meridianstack-hosting-co-us",
        scope_facility: "dc-phx-3",
        scope_zone: "aisle-7b",
        valid_from: "2026-10-17T02:47:00-07:00",
        valid_until: "2026-10-17T09:00:00-07:00"
    ) &&
    principal.has_certification("EPA-608-Universal") &&
    principal.has_certification_unexpired("EPA-608-Universal") &&
    principal.has_certification("NFPA-70E-CAT-2") &&
    principal.has_certification_unexpired("NFPA-70E-CAT-2") &&
    principal.has_certification("OSHA-30-General-Industry") &&
    context.permit_id like "permit-dc-phx-3-2026-10-17-*" &&
    context.permit_status in ["co_signed_active", "post_repair_verification"]
};

// ============================================================
// PERMIT — Cascade manager co-signs permit
// ============================================================

permit (
    principal in Role::"cascade_manager",
    action == Action::"workflow.permit_sign",
    resource is Permit
) when {
    principal.tenant == "cascade-fm-services-llc-us" &&
    resource.permit_role_to_sign == "cascade_manager" &&
    context.attestation_text_length >= 16
};

// ============================================================
// PERMIT — MeridianStack NOC controller co-signs permit
// ============================================================

permit (
    principal in Role::"host_noc_controller",
    action == Action::"workflow.permit_sign",
    resource is Permit
) when {
    principal.tenant == "meridianstack-hosting-co-us" &&
    resource.permit_role_to_sign == "host_noc_controller" &&
    principal.on_shift_at(context.signed_at) == true
};

// ============================================================
// PERMIT — Cross-tenant grant issuance (MeridianStack-side)
// ============================================================

permit (
    principal in Role::"host_dispatch_or_facility_manager",
    action == Action::"identity.cross_tenant_grant_issue",
    resource is Tenant
) when {
    resource.tenant_id == "meridianstack-hosting-co-us" &&
    context.grantee_principal_tenant == "cascade-fm-services-llc-us" &&
    context.vendor_contract_active == true &&
    context.scope_facility == "dc-phx-3" &&
    context.scope_zone like "aisle-*" &&
    context.valid_until_minus_valid_from_hours <= 12
};

// ============================================================
// PERMIT — EPA-608 disclosure submission
// ============================================================

permit (
    principal == User::"carlos.reyes-ii@cascade-fm-services-llc-us",
    action == Action::"compliance.regulator_submit",
    resource is Workflow
) when {
    resource.workflow_template_id == "wkfl-epa608-release-disclosure-v2" &&
    principal.has_certification_unexpired("EPA-608-Universal") &&
    context.form_complete == true &&
    context.signatures_count == 3 &&
    context.signature_roles == ["technician_epa_608", "cascade_tenant_attestation", "meridianstack_site_attestation"] &&
    context.cylinder_of_origin_recorded == true &&
    context.release_estimate_lb > 0.0
};

// ============================================================
// PERMIT — Trane vendor read on disclosure-bundle resource only
// ============================================================

permit (
    principal in Role::"trane_emergency_vendor",
    action in [Action::"messenger.read", Action::"disclosure_bundle.read"],
    resource is DisclosureBundle
) when {
    resource.bundle_id like "disclosure-bundle-dc-phx-3-2026-10-17-*" &&
    resource.redaction_applied == true &&
    resource.contains_site_pii == false
};

// ============================================================
// FORBID-1 — Grant expired
// ============================================================

forbid (
    principal == User::"carlos.reyes-ii@cascade-fm-services-llc-us",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "meridianstack-hosting-co-us" &&
    context.now > context.grant_valid_until
}
advice {
    "forbid_rule_id": "forbid-grant-expired",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-CEDAR-DENY-EXPIRED-GRANT-009"
};

// ============================================================
// FORBID-2 — Scope outside aisle-7b
// ============================================================

forbid (
    principal == User::"carlos.reyes-ii@cascade-fm-services-llc-us",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "meridianstack-hosting-co-us" &&
    context.attempted_scope_zone != "aisle-7b"
}
advice {
    "forbid_rule_id": "forbid-scope-outside-aisle-7b",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-CEDAR-DENY-SCOPE-OUT-009a"
};

// ============================================================
// FORBID-3 — Missing required certification
// ============================================================

forbid (
    principal,
    action in [Action::"workflow.permit_sign", Action::"workflow.epa608_disclose_release", Action::"workflow.loto_lock"],
    resource is Permit
) when {
    !principal.has_certification_unexpired("EPA-608-Universal") ||
    !principal.has_certification_unexpired("NFPA-70E-CAT-2") ||
    !principal.has_certification_unexpired("OSHA-30-General-Industry")
}
advice {
    "forbid_rule_id": "forbid-cert-missing-or-expired",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-CEDAR-DENY-CERT-MISSING-009b",
    "regulator_anchor": "29-CFR-1910-General-Industry"
};

// ============================================================
// FORBID-4 — Invalid LOTO state transition
// ============================================================

forbid (
    principal,
    action == Action::"workflow.loto_transition",
    resource is Permit
) when {
    (context.from_state == "lockout_pending" && context.to_state == "energized_normal") ||
    (context.from_state == "disconnect_open" && context.to_state == "energized_normal") ||
    (context.from_state == "tested_voltage_absent" && context.to_state == "energized_normal") ||
    (context.from_state == "locked_isolated_verified" && context.to_state == "energized_normal")
}
advice {
    "forbid_rule_id": "forbid-loto-invalid-skip-transition",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-LOTO-INVALID-TRANSITION-009c",
    "regulator_anchor": "29-CFR-1910.147-LOTO"
};

// ============================================================
// FORBID-5 — EPA disclosure incomplete
// ============================================================

forbid (
    principal,
    action == Action::"compliance.regulator_submit",
    resource is Workflow
) when {
    resource.workflow_template_id == "wkfl-epa608-release-disclosure-v2" &&
    (context.form_complete == false ||
     context.signatures_count < 3 ||
     context.cylinder_of_origin_recorded == false)
}
advice {
    "forbid_rule_id": "forbid-epa-disclosure-incomplete",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-EPA-DISCLOSURE-INCOMPLETE-009d",
    "regulator_anchor": "40-CFR-82-F"
};

// ============================================================
// FORBID-6 — Trane vendor cannot read site PII
// ============================================================

forbid (
    principal in Role::"trane_emergency_vendor",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "meridianstack-hosting-co-us" &&
    !(resource is DisclosureBundle)
}
advice {
    "forbid_rule_id": "forbid-trane-site-pii",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-CEDAR-DENY-TRANE-SITE-PII-PROBE-018"
};

// ============================================================
// FORBID-7 — Payroll leak from Cascade to MeridianStack
// ============================================================

forbid (
    principal in Role::"host_facility_manager",
    action in [Action::"payroll.read", Action::"payroll.export"],
    resource is Tenant
) when {
    resource.tenant_id == "cascade-fm-services-llc-us" &&
    principal.tenant == "meridianstack-hosting-co-us"
}
advice {
    "forbid_rule_id": "forbid-host-reads-vendor-payroll",
    "audit_dual_seal": true,
    "audit_class": "EVT-J156-CEDAR-DENY-PAYROLL-CROSS-PROBE-019",
    "regulator_anchor": "US-DOL-FLSA-private-payroll"
};
