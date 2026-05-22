// j154 — Trinity Cedar policy bundle
// File: cedar-bundle-trinity-glacpl-mfg-q1-v1
//
// Authority: docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch/handshake.md
// Binds ADR-0243 (Cedar universal gate), ADR-0244 (tenant scoping),
//       ADR-0311 (dual-tenant identity boundary), ADR-0263 (audit emission contract).
//
// Three tenants:
//   - partnerlift_nl                                      (PartnerLift home)
//   - glacier_erp_de                                       (Glacier home)
//   - glacier-partnerlift-q1-2027-mfg-de-nl-be             (shared joint-controller tenant)
//
// The bundle is sealed as part of the shared-tenant provisioning step and
// is loaded into the Cedar policy store on every µservice involved
// (tenancy, connect, comms-email, marketing-automation, crm, community,
// workflow-engine, payments, compliance, analytics).

// =============================================================
// §1 Home-tenant authority — Tomas (PartnerLift)
// =============================================================

@id("partnerlift-home-tomas-channel-partner-manager")
permit (
    principal == User::"tomas.pieter@partnerlift.nl",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "partnerlift_nl" &&
    principal.role_in_tenant("partnerlift_nl") == "channel_partner_manager" &&
    context.active_tenant_confirmed == true
};

// =============================================================
// §2 Home-tenant authority — Henrik (Glacier)
// =============================================================

@id("glacier-home-henrik-marketing-director")
permit (
    principal == User::"henrik.faulkner@glacier-erp.de",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "glacier_erp_de" &&
    principal.role_in_tenant("glacier_erp_de") == "marketing_director" &&
    context.active_tenant_confirmed == true
};

// =============================================================
// §3 Shared-tenant joint-controller scope (limited action set)
// =============================================================

@id("shared-tenant-joint-controller-action-scope")
permit (
    principal,
    action in [
        Action::"campaign.author",
        Action::"campaign.review",
        Action::"campaign.launch",
        Action::"crm.lead_route",
        Action::"crm.read_shared_pool",
        Action::"comms.send",
        Action::"community.post",
        Action::"community.read",
        Action::"analytics.read_joint",
        Action::"payments.read_escrow"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be" &&
    (
        principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") == "joint_controller_partnerlift" ||
        principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") == "joint_controller_glacier"
    ) &&
    context.gdpr_lawful_basis_set == true &&
    context.dpa_signed == true &&
    context.active_tenant_confirmed == true
};

// =============================================================
// §4 DPO observer — read-only
// =============================================================

@id("shared-tenant-dpo-observer-read-only")
permit (
    principal,
    action in [
        Action::"campaign.review",
        Action::"crm.read_shared_pool",
        Action::"community.read",
        Action::"analytics.read_joint",
        Action::"audit.read"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be" &&
    principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") == "dpo_observer"
};

@id("shared-tenant-dpo-observer-forbid-mutations")
forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be" &&
    principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") == "dpo_observer" &&
    action in [
        Action::"campaign.author",
        Action::"campaign.launch",
        Action::"crm.lead_route",
        Action::"comms.send",
        Action::"community.post",
        Action::"payments.release"
    ]
};

// =============================================================
// §5 Cross-partner internal CRM read — TRINITY FORBID
// =============================================================
// PartnerLift principals may NOT read Glacier's internal CRM,
// and vice versa. Only the shared lead pool is cross-visible.

@id("trinity-forbid-partnerlift-reads-glacier-internal-crm")
forbid (
    principal,
    action == Action::"crm.read",
    resource is Tenant
) when {
    resource.tenant_id == "glacier_erp_de" &&
    !(principal.role_in_tenant("glacier_erp_de") in ["marketing_director", "sales_director", "system_admin"])
};

@id("trinity-forbid-glacier-reads-partnerlift-internal-crm")
forbid (
    principal,
    action == Action::"crm.read",
    resource is Tenant
) when {
    resource.tenant_id == "partnerlift_nl" &&
    !(principal.role_in_tenant("partnerlift_nl") in ["channel_partner_manager", "sales_director", "system_admin"])
};

// =============================================================
// §6 Cross-partner marketing-asset preview — TRINITY FORBID
// =============================================================
// Internal (non-joint) marketing assets are not previewable by the other side.

@id("trinity-forbid-cross-partner-internal-asset-preview")
forbid (
    principal,
    action == Action::"marketing.asset_preview",
    resource is MarketingAsset
) when {
    resource.owner_tenant != principal.active_tenant &&
    resource.shared_visibility != "joint_controller_visible"
};

// =============================================================
// §7 Campaign launch — both CMOs required
// =============================================================

@id("campaign-launch-requires-both-cmo-approval")
forbid (
    principal,
    action == Action::"campaign.launch",
    resource is Campaign
) unless {
    resource.campaign_id == "camp-glacpl-mfg-q1-2027" &&
    context.approval_chain_present.contains("anneke.vandermeer@partnerlift.nl") &&
    context.approval_chain_present.contains("beate.hoffmann@glacier-erp.de") &&
    context.dpa_attestation_id == "att-dpa-glacpl-mfg-q1-2026-1230" &&
    context.deliverability_pre_check_passed == true
};

// =============================================================
// §8 Lead routing — consent required
// =============================================================

@id("crm-lead-route-requires-consent-where-natural-person")
forbid (
    principal,
    action == Action::"crm.lead_route",
    resource is Lead
) when {
    resource.country in ["NL", "BE-NL"] &&
    resource.consent.marketing_email_optin_method != "double_optin_confirmed"
};

@id("crm-lead-route-requires-consent-gdpr-art-6-1-f-or-1-a")
forbid (
    principal,
    action == Action::"crm.lead_route",
    resource is Lead
) when {
    resource.country in ["DE", "NL", "BE-NL", "BE-FR"] &&
    !(resource.consent_basis.contains("gdpr_art_6_1_f_legitimate_interest_b2b") ||
      resource.consent_basis.contains("gdpr_art_6_1_a_consent"))
};

// =============================================================
// §9 Escrow release — only after Q1 settlement timer
// =============================================================

@id("payments-escrow-release-requires-timer-fire")
forbid (
    principal,
    action == Action::"payments.release",
    resource is Escrow
) unless {
    resource.escrow_id == "esc-glacpl-mfg-q1-2027" &&
    context.workflow_timer_fired == "q1_attribution_settlement" &&
    context.attribution_finalized == true
};

// =============================================================
// §10 Shared-tenant writes — refused during wind-down
// =============================================================

@id("tenancy-shared-tenant-no-writes-during-wind-down")
forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be" &&
    resource.tenant_state in ["winding_down", "archived"] &&
    action in [
        Action::"campaign.author",
        Action::"campaign.launch",
        Action::"crm.lead_route",
        Action::"comms.send",
        Action::"community.post"
    ]
};

// =============================================================
// §11 Joint analytics — only joint-controllers + DPO observers
// =============================================================

@id("analytics-joint-readable-only-by-trinity-members")
forbid (
    principal,
    action == Action::"analytics.read_joint",
    resource is Tenant
) when {
    resource.tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be" &&
    !(principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") in
        ["joint_controller_partnerlift", "joint_controller_glacier", "dpo_observer"])
};

// =============================================================
// §12 Personal-tenant separation (ADR-0311)
// =============================================================
// Tomas-as-PartnerLift-employee cannot mutate Tomas's personal tenant
// from his work context, and vice versa.

@id("dual-tenant-personal-work-separation")
forbid (
    principal,
    action,
    resource is Tenant
) unless {
    resource.tenant_id == principal.active_tenant ||
    (
        context.explicit_cross_tenant_grant == true &&
        context.grant_scope_covers_resource == true &&
        context.audit_reason_code != ""
    )
};

// =============================================================
// §13 DSA transparency log — must be emitted for every impression
// =============================================================

@id("dsa-impression-requires-transparency-emission")
forbid (
    principal,
    action == Action::"ads.serve_impression",
    resource is Ad
) unless {
    context.dsa_transparency_log_destination != "" &&
    context.advertiser_legal_names.contains("PartnerLift B.V.") &&
    context.advertiser_legal_names.contains("Glacier ERP GmbH") &&
    context.targeting_criteria_plain != ""
};
