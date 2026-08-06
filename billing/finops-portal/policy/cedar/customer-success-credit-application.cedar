// Cedar policy — customer-success credit application
// Authored 2026-05-18.
// Authority: ADR-0183 cedar-policy-discipline + ADR-0174.
// Schema:  ./schema.cedarschema.json
//
// Customer-success can apply negotiated credits to tenants in their
// managed list. Cannot edit cost-allocation policies. Read-only on
// invoices.

// Permit: customer-success applies a credit to a managed tenant.
permit (
  principal is FinopsPortal::CustomerSuccess,
  action == FinopsPortal::Action::"ApplyCredit",
  resource is FinopsPortal::CreditApplication
) when {
  resource.tenant_id in principal.managed_tenants &&
  principal.residency_region == context.residency_region &&
  resource.source == "Negotiated"
};

// Permit: customer-success reads credit ledger for managed tenants.
permit (
  principal is FinopsPortal::CustomerSuccess,
  action == FinopsPortal::Action::"ReadCreditLedger",
  resource is FinopsPortal::CreditApplication
) when {
  resource.tenant_id in principal.managed_tenants
};

// Permit: customer-success reads invoices (no finalize) for managed tenants.
permit (
  principal is FinopsPortal::CustomerSuccess,
  action == FinopsPortal::Action::"ReadInvoice",
  resource is FinopsPortal::Invoice
) when {
  resource.tenant_id in principal.managed_tenants
};

// Forbid: customer-success cannot edit cost-allocation policy at any scope.
forbid (
  principal is FinopsPortal::CustomerSuccess,
  action in [
    FinopsPortal::Action::"EditCostAllocationPolicy",
    FinopsPortal::Action::"PromoteCostAllocationPolicy"
  ],
  resource is FinopsPortal::CostAllocationPolicy
);

// Forbid: customer-success cannot apply non-negotiated credits
// (committed-use and SLA-refund must go through ops-finops).
forbid (
  principal is FinopsPortal::CustomerSuccess,
  action == FinopsPortal::Action::"ApplyCredit",
  resource is FinopsPortal::CreditApplication
) when {
  resource.source != "Negotiated"
};
