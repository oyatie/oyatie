// j159 — Saanvi Mehta dual-tenant MBA application Cedar policy
// Bundle id: cedar-bundle-dual-tenant-mba-v1
// Deployed to: saanvi.mehta.personal + stripe-india-pvt-ltd + stripe-corporate-us +
//              marico-india-pvt-ltd + wharton-r2-2027-prospective-applicants-community +
//              arjun.mehta.personal + wharton-mba-admissions-us
// Core doctrine: ADR-0311 personal-vs-work boundary. Work-tenant principals NEVER
// have ambient access to personal-tenant resources. Cross-tenant capabilities are
// granted action-by-action with explicit scope; never via generic share-links that
// decay into ambient access. Corporate cards are NEVER eligible for personal-tenant
// payment routing.

// ============================================================
// PERMIT — Saanvi acts in her own personal tenant
// ============================================================

permit (
    principal == User::"saanvi.mehta.personal:saanvi",
    action,
    resource is PersonalTenantResource
) when {
    principal.tenant == "saanvi.mehta.personal" &&
    resource.tenant == "saanvi.mehta.personal"
};

// ============================================================
// PERMIT — Saanvi acts in her own work tenant
// ============================================================

permit (
    principal == User::"saanvi.mehta@stripe-india-pvt-ltd",
    action,
    resource is WorkTenantResource
) when {
    principal.tenant == "stripe-india-pvt-ltd" &&
    resource.tenant == "stripe-india-pvt-ltd"
};

// ============================================================
// PERMIT — Cross-tenant recommender capability (write-once)
// ============================================================
// This is the keystone permit for the j159 doctrine. It allows a work-tenant
// principal to author a recommendation letter INTO a personal-tenant slot
// — but ONLY when the personal-tenant principal has explicitly granted
// a capability with scope "write_once_no_browse" for the specific named slot.

permit (
    principal is User,
    action == Action::"drive.write_recommendation_letter_to_slot",
    resource is RecommendationLetterSlot
) when {
    context.capability_grant.granted_by_personal_tenant_principal == true &&
    context.capability_grant.scope == "write_once_no_browse" &&
    context.capability_grant.capability_id like "cap-*-rec-*" &&
    resource.target_tenant != principal.tenant &&
    resource.target_tenant == context.capability_grant.granting_tenant &&
    context.capability_grant.revoked == false &&
    context.now <= context.capability_grant.auto_revoke_at
};

// ============================================================
// PERMIT — Cross-tenant recommender capability accept
// ============================================================

permit (
    principal,
    action == Action::"tenancy.cross_tenant_capability_accept",
    resource is CapabilityInvitation
) when {
    resource.capability_class == "recommendation_letter_write_once" &&
    resource.target_slot_id like "slot-*-rec-*" &&
    principal.tenant != resource.granting_tenant &&
    context.unicode_normalization == "NFC"
};

// ============================================================
// PERMIT — Spousal read-only folder capability
// ============================================================

permit (
    principal,
    action in [
        Action::"drive.read_file",
        Action::"drive.list_folder"
    ],
    resource is DriveFolder
) when {
    context.capability_class == "spousal_read_only_folder" &&
    context.relationship_basis like "joint_marriage_attestation_*" &&
    context.scope_actions_forbidden.contains("drive.download") == false &&
    context.now <= context.capability_grant.expires_at &&
    principal.tenant == context.capability_grant.grantee_tenant
};

// ============================================================
// PERMIT — Personal-tenant payment authorization (provider-credential BYOK card, ADR-0255 §D-4)
// ============================================================

permit (
    principal,
    action == Action::"payments.authorize",
    resource is PaymentAuthorization
) when {
    principal.tenant == "saanvi.mehta.personal" &&
    resource.credential_class == "personal_card_BYOK" &&
    resource.credential_tenant == principal.tenant &&
    resource.amount.currency in ["USD", "INR", "EUR", "GBP", "SGD"] &&
    context.cell == "ap-mumbai-primary"
};

// ============================================================
// PERMIT — Community participation from personal tenant
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
    resource.community_tenant == "wharton-r2-2027-prospective-applicants-community" &&
    principal.tenant_class == "personal_tenant" &&
    principal.membership_in_community == true &&
    context.mls_epoch_at_post >= 1
};

// ============================================================
// PERMIT — Cross-tenant admissions submission ack
// ============================================================

permit (
    principal is User,
    action == Action::"admissions.cross_tenant_ack",
    resource is ApplicationSubmission
) when {
    principal.tenant in [
        "wharton-mba-admissions-us",
        "stanford-gsb-admissions-us",
        "hbs-admissions-us",
        "chicago-booth-admissions-us",
        "insead-singapore-fontainebleau"
    ] &&
    resource.applicant_tenant == "saanvi.mehta.personal"
};

// ============================================================
// PERMIT — Saanvi withdraws / revokes recommender capability
// ============================================================

permit (
    principal == User::"saanvi.mehta.personal:saanvi",
    action == Action::"tenancy.capability_revoke",
    resource is Capability
) when {
    principal.tenant == "saanvi.mehta.personal" &&
    resource.granting_tenant == "saanvi.mehta.personal"
};

// ============================================================
// FORBID-1 — Work-tenant ambient access to personal-tenant resource (ADR-0311 keystone)
// ============================================================

forbid (
    principal,
    action,
    resource is PersonalTenantResource
) when {
    principal.tenant_class == "work_tenant" &&
    principal.tenant != "saanvi.mehta.personal" &&
    !context.cross_tenant_capability_grant_present &&
    !context.principal_is_same_human_dual_identity
}
advice {
    "forbid_rule_id": "forbid-work-tenant-ambient-access-to-personal-tenant",
    "doctrine_anchor": "ADR-0311",
    "audit_class": "EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-PERSONAL-014a",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-2 — Corporate card on personal-tenant payment
// ============================================================

forbid (
    principal,
    action == Action::"payments.authorize",
    resource is PaymentAuthorization
) when {
    principal.tenant_class == "personal_tenant" &&
    resource.credential_class == "corporate_card" &&
    resource.credential_tenant != principal.tenant
}
advice {
    "forbid_rule_id": "forbid-corporate-card-on-personal-tenant-payment",
    "doctrine_anchor": "ADR-0311 + ADR-0255 §D-4 provider-credential BYOK",
    "audit_class": "EVT-J159-CEDAR-DENY-CORPORATE-CARD-PERSONAL-PAYMENT-014b",
    "ui_message": "corporate_card_not_eligible_for_personal_tenant_payment"
};

// ============================================================
// FORBID-3 — Recommender browses beyond slot scope
// ============================================================

forbid (
    principal,
    action in [
        Action::"drive.list_folder",
        Action::"drive.read_file"
    ],
    resource is DriveFolder
) when {
    context.capability_present == true &&
    context.capability_scope == "write_once_no_browse" &&
    resource.path not_like context.capability_target_slot_path
}
advice {
    "forbid_rule_id": "forbid-recommender-browse-beyond-slot",
    "audit_class": "EVT-J159-CEDAR-DENY-RECOMMENDER-BROWSE-014c",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-4 — Recommender capability propagation (no forwarding)
// ============================================================

forbid (
    principal,
    action == Action::"tenancy.capability_propagate",
    resource is Capability
) when {
    resource.capability_class == "recommendation_letter_write_once"
}
advice {
    "forbid_rule_id": "forbid-recommender-capability-propagation",
    "audit_class": "EVT-J159-CEDAR-DENY-RECOMMENDER-FORWARD-014d",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-5 — Spousal capability write attempt
// ============================================================

forbid (
    principal,
    action in [
        Action::"drive.write_file",
        Action::"drive.delete_file",
        Action::"drive.download_file"
    ],
    resource is DriveFolder
) when {
    context.capability_class == "spousal_read_only_folder"
}
advice {
    "forbid_rule_id": "forbid-spousal-write-or-download",
    "audit_class": "EVT-J159-CEDAR-DENY-SPOUSAL-WRITE-014e"
};

// ============================================================
// FORBID-6 — Community membership from non-personal tenant
// ============================================================

forbid (
    principal,
    action in [
        Action::"community.post",
        Action::"community.reply",
        Action::"community.join"
    ],
    resource is CommunityGroup
) when {
    principal.tenant_class != "personal_tenant"
}
advice {
    "forbid_rule_id": "forbid-community-from-non-personal-tenant",
    "audit_class": "EVT-J159-CEDAR-DENY-COMMUNITY-WRONG-TENANT-014g"
};

// ============================================================
// FORBID-7 — Diacritic transliteration without explicit consent
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.legal_name_write",
        Action::"admissions.application_submit",
        Action::"recommender.letter_final_submit",
        Action::"payments.authorize"
    ],
    resource
) when {
    context.unicode_normalization != "NFC" ||
    context.transliterate_to_ascii == true &&
    context.user_explicit_consent_to_transliterate == false
}
advice {
    "forbid_rule_id": "forbid-diacritic-transliterate-without-consent",
    "audit_class": "EVT-J159-CEDAR-DENY-NAME-TRANSLITERATE-014h",
    "exception": "GMAT_score_send_uses_ICAO_9303_passport_ASCII_per_GMAC_rule",
    "audit_dual_seal": true
};

// ============================================================
// FORBID-8 — Personal-tenant data residency drift
// ============================================================

forbid (
    principal,
    action in [
        Action::"drive.write_file",
        Action::"notes.document_finalize",
        Action::"payments.authorize",
        Action::"community.post"
    ],
    resource is PersonalTenantResource
) when {
    resource.target_cell != "ap-mumbai-primary" &&
    !context.user_explicit_residency_override
}
advice {
    "forbid_rule_id": "forbid-personal-tenant-cell-drift",
    "audit_class": "EVT-J159-CEDAR-DENY-PERSONAL-CELL-DRIFT-014i",
    "doctrine_anchor": "IN-DPDP-2023 + ADR-0311"
};

// ============================================================
// FORBID-9 — Personal-tenant payment with non-personal credential
// ============================================================

forbid (
    principal,
    action == Action::"payments.authorize",
    resource is PaymentAuthorization
) when {
    principal.tenant_class == "personal_tenant" &&
    resource.credential_tenant != principal.tenant &&
    resource.credential_class != "personal_card_BYOK"
}
advice {
    "forbid_rule_id": "forbid-non-personal-credential-on-personal-payment",
    "audit_class": "EVT-J159-CEDAR-DENY-NON-PERSONAL-CREDENTIAL-014j"
};

// ============================================================
// FORBID-10 — Capability use after revocation
// ============================================================

forbid (
    principal,
    action,
    resource is RecommendationLetterSlot
) when {
    context.capability_grant.revoked == true ||
    context.now > context.capability_grant.auto_revoke_at
}
advice {
    "forbid_rule_id": "forbid-revoked-or-expired-capability",
    "audit_class": "EVT-J159-CEDAR-DENY-REVOKED-CAPABILITY-014k"
};

// ============================================================
// FORBID-11 — Recommender accept from same-tenant-as-applicant
// ============================================================
// A recommender writing AS the applicant is meaningless. This protects
// against accidental same-tenant self-recommendation.

forbid (
    principal,
    action == Action::"tenancy.cross_tenant_capability_accept",
    resource is CapabilityInvitation
) when {
    principal.tenant == resource.granting_tenant
}
advice {
    "forbid_rule_id": "forbid-self-recommendation",
    "audit_class": "EVT-J159-CEDAR-DENY-SELF-RECOMMENDATION-014l"
};

// ============================================================
// FORBID-12 — HR audit principal personal-tenant CSV export
// ============================================================

forbid (
    principal == User::"hr-systems@stripe-corporate-us",
    action in [
        Action::"audit.export_csv",
        Action::"audit.export_json",
        Action::"audit.export_pdf"
    ],
    resource is PersonalTenantResource
)
advice {
    "forbid_rule_id": "forbid-hr-export-personal-tenant",
    "audit_class": "EVT-J159-CEDAR-DENY-HR-EXPORT-014f",
    "doctrine_anchor": "ADR-0311 + ADR-0244"
};
