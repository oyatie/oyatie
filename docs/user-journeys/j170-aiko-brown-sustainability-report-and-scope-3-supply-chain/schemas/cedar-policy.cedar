// j170 — Cedar policy bundle for FY2026 annual emissions report
// Implements ADR-0251 (compliance pack), ADR-0244 (tenant scoping), ADR-0263 (audit dual-seal),
// ADR-0252 (TrueTime), per-supplier NDA-scoped Cedar permits.

// =====================================================================
// EMISSIONS REPORT INIT — sustainability_officer or cso role
// =====================================================================
permit (
    principal,
    action == Action::"compliance.emissions_report_init",
    resource is EmissionsReport
) when {
    principal.role in ["sustainability_officer", "cso", "ceo"] &&
    principal.tenant == "marlboro-forge-industries-inc-cleveland-oh-us"
};

// =====================================================================
// UTILITY BILL STRUCTURED EXTRACT — sustainability team
// =====================================================================
permit (
    principal,
    action == Action::"cloud-data.structured_extract",
    resource is UtilityBillPDF
) when {
    principal in Group::"mfi-sustainability-team" &&
    resource.extract_schema in ["utility-bill-v3", "natural-gas-bill-v2", "diesel-fuel-receipt-v1"]
};

// =====================================================================
// CROSS-TENANT REQUEST INITIATE — sustainability_officer + NDA on file
// =====================================================================
permit (
    principal,
    action == Action::"connect.cross_tenant_data_request_initiate",
    resource is CrossTenantRequest
) when {
    principal.role in ["sustainability_officer", "cso"] &&
    resource.nda_reference != null &&
    resource.nda_active == true &&
    resource.scope_category like "scope-3-category-*" &&
    context.business_hours_local == true
};

// =====================================================================
// SUPPLIER DATA SUBMIT — supplier-side; NDA + scope-category + MLS active + TrueTime
// =====================================================================
permit (
    principal,
    action == Action::"connect.supplier_data_submit",
    resource is SupplierEmissionsData
) when {
    principal.tenant_role == "supplier-data-submitter" &&
    resource.recipient_tenant == "marlboro-forge-industries-inc-cleveland-oh-us" &&
    resource.nda_signed == true &&
    resource.nda_active == true &&
    resource.scope_category in ["scope-3-category-1", "scope-3-category-4", "scope-3-category-5", "scope-3-category-11", "scope-3-category-12"] &&
    context.mls_encryption_active == true &&
    context.business_hours_local == true &&
    context.truetime_uncertainty_ms <= 10 &&
    principal.passkey_attested == true
};

// =====================================================================
// SCOPE-CATEGORY ENFORCEMENT — restrict to agreed category
// =====================================================================
forbid (
    principal,
    action == Action::"connect.supplier_data_submit",
    resource is SupplierEmissionsData
) when {
    resource.submitted_scope_category != resource.agreed_scope_category
};

// =====================================================================
// ONTOLOGY MAPPING — system-triggered or sustainability_officer
// =====================================================================
permit (
    principal,
    action == Action::"ontology.map_supplier_to_supply_chain_partner",
    resource is OntologyMapping
) when {
    (principal == User::"system-principal.ontology-orchestrator" ||
     principal.role in ["sustainability_officer", "cso"]) &&
    resource.ontology_node_type == "Oyatie::SupplyChainPartner"
};

// =====================================================================
// ASSURANCE REPLAY READ — assurance partner during engagement window
// =====================================================================
permit (
    principal,
    action == Action::"audit-chain.assurance_replay_read",
    resource is MerkleReplay
) when {
    principal.tenant == "ernst-and-young-cleveland-assurance-tenant" &&
    principal.role == "assurance_partner" &&
    resource.engagement_id == "ey-mfi-fy2026-assurance" &&
    context.engagement_window_active == true
};

// =====================================================================
// MULTI-FRAMEWORK COMPOSE — sustainability_officer + assurance passed
// =====================================================================
permit (
    principal,
    action == Action::"compliance.multi_framework_compose",
    resource is EmissionsReport
) when {
    principal.role in ["sustainability_officer", "cso", "external_assurance"] &&
    resource.merkle_root_computed == true &&
    resource.scope_3_ingest_complete == true &&
    resource.assurance_review_passed == true
};

// =====================================================================
// FILING PERMIT — 4-of-4 quorum (CSO + CFO + GC + Audit-Committee Chair)
// =====================================================================
permit (
    principal,
    action == Action::"governance.emissions_report_filing_permit_vote",
    resource is EmissionsReportFilingPermit
) when {
    resource.quorum_count >= 4 &&
    resource.assurance_opinion_passed == true &&
    resource.all_framework_variants_composed == true &&
    context.business_hours_edt == true &&
    context.truetime_uncertainty_ms <= 10 &&
    principal in Group::"mfi-emissions-report-quorum-members"
};

// Forbid filing vote outside business hours
forbid (
    principal,
    action == Action::"governance.emissions_report_filing_permit_vote",
    resource is EmissionsReportFilingPermit
) when {
    context.business_hours_edt == false &&
    context.emergency_override_signed_by_ceo == false
};

// =====================================================================
// AUTHORITY SUBMISSION — filing permit signed + signer QES attested
// =====================================================================
permit (
    principal,
    action == Action::"compliance.authority_filing_submit",
    resource is AuthoritySubmission
) when {
    resource.filing_permit_seal like "EVT-J170-FILING-PERMIT-*" &&
    resource.signer != null &&
    resource.signer_qes_provider != null &&
    resource.framework in ["SEC-Climate-Disclosure", "CDP-2026", "EU-CSRD-ESRS-E1", "ISSB-IFRS-S2"]
};
