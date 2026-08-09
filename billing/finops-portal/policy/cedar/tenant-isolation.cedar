// Cedar policy — tenant-isolation
// Authored 2026-05-18.
// Authority: ADR-0183 cedar-policy-discipline + ADR-0162.
// Schema:  ./schema.cedarschema.json
//
// Goal: tenants see only their own data. ops-finops sees fleet.
// Dual context (request.context + resource.attrs) per ADR-0183.
// Residency-aware: principal.residency_region must equal
// resource.residency_region for every cross-region call.

// Permit: tenant admin reads own tenant's invoices.
permit (
  principal is FinopsPortal::TenantAdmin,
  action in [FinopsPortal::Action::"ReadInvoice", FinopsPortal::Action::"ReadInvoicePdf"],
  resource is FinopsPortal::Invoice
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.residency_region == resource.residency_region &&
  context has regulatory_pack &&
  context.regulatory_pack == resource.regulatory_pack
};

// Permit: tenant admin reads own tenant's credit ledger.
permit (
  principal is FinopsPortal::TenantAdmin,
  action == FinopsPortal::Action::"ReadCreditLedger",
  resource is FinopsPortal::CreditApplication
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.residency_region == context.residency_region
};

// Permit: ops-finops reads any invoice in their region scope.
permit (
  principal is FinopsPortal::OpsFinops,
  action in [
    FinopsPortal::Action::"ReadInvoice",
    FinopsPortal::Action::"ReadInvoicePdf",
    FinopsPortal::Action::"FinalizeInvoice",
    FinopsPortal::Action::"ReadCreditLedger"
  ],
  resource is FinopsPortal::Invoice
) when {
  principal.tenant_scope == "fleet" &&
  principal.residency_region == resource.residency_region
};

// Forbid: a tenant admin without phi_authorized cannot read PHI-classified
// invoices in the US-healthcare pack.
forbid (
  principal is FinopsPortal::TenantAdmin,
  action in [FinopsPortal::Action::"ReadInvoice", FinopsPortal::Action::"ReadInvoicePdf"],
  resource is FinopsPortal::Invoice
) when {
  resource.regulatory_pack == "us-healthcare" &&
  resource has classification &&
  resource.classification == "phi" &&
  !(principal has phi_authorized && principal.phi_authorized == true)
};
