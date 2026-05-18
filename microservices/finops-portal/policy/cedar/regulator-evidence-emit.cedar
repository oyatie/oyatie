// Cedar policy — regulator evidence emit
// Authored 2026-05-18.
// Authority: ADR-0183 + ADR-0162 + ADR-0174.
// Schema:  ./schema.cedarschema.json
//
// Regulators read quarterly RegulatorEvidence rows scoped to their
// pack + residency region. ops-finops can also read for audit-trail
// purposes. No principal class can write or modify a sealed envelope.

// Permit: regulator reads their pack's evidence in their region.
permit (
  principal is FinopsPortal::Regulator,
  action == FinopsPortal::Action::"ReadRegulatorEvidence",
  resource is FinopsPortal::RegulatorEvidence
) when {
  principal.pack == resource.regulatory_pack &&
  principal.residency_region == resource.residency_region
};

// Permit: ops-finops reads any regulator evidence for audit.
permit (
  principal is FinopsPortal::OpsFinops,
  action == FinopsPortal::Action::"ReadRegulatorEvidence",
  resource is FinopsPortal::RegulatorEvidence
) when {
  principal.tenant_scope == "fleet" &&
  principal.residency_region == resource.residency_region
};

// Forbid: regulator cannot read raw tenant invoices directly.
// They consume the aggregated quarterly evidence only.
forbid (
  principal is FinopsPortal::Regulator,
  action in [FinopsPortal::Action::"ReadInvoice", FinopsPortal::Action::"ReadInvoicePdf"],
  resource is FinopsPortal::Invoice
);

// Forbid: cross-pack read (EU regulator on KR data, etc.).
forbid (
  principal is FinopsPortal::Regulator,
  action == FinopsPortal::Action::"ReadRegulatorEvidence",
  resource is FinopsPortal::RegulatorEvidence
) when {
  principal.pack != resource.regulatory_pack
};

// Forbid: cross-region regardless of pack match (defensive double-guard).
forbid (
  principal is FinopsPortal::Regulator,
  action == FinopsPortal::Action::"ReadRegulatorEvidence",
  resource is FinopsPortal::RegulatorEvidence
) when {
  principal.residency_region != resource.residency_region
};
