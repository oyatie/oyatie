// Cedar policy fragment — journey j153
// ADR anchors:
//   ADR-0243 (Cedar as universal gate)
//   ADR-0311 (dual-tenant identity strict separation)
//   ADR-0244 (tenant scoping invariant)

// ---------------------------------------------------------------------------
// Sole-proprietor admin (Devon) — full power inside the LLC tenant
// ---------------------------------------------------------------------------

permit (
    principal == User::"devon.williams@oyamail.network",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "devon_williams_hvac_llc" &&
    principal.role_in_tenant("devon_williams_hvac_llc") == "sole_proprietor_admin" &&
    principal.passkey_step_up_within_seconds(300)
};

// ---------------------------------------------------------------------------
// Year-end reconcile — narrow permit (also captured by the general permit above)
// ---------------------------------------------------------------------------

permit (
    principal == User::"devon.williams@oyamail.network",
    action == Action::"finops.year_end_reconcile",
    resource is Tenant
) when {
    resource.tenant_id == "devon_williams_hvac_llc" &&
    context.tax_year >= 2020 &&
    context.tax_year <= 2099
};

// ---------------------------------------------------------------------------
// Tax-preparer share — must specify the preparer ID + scope
// ---------------------------------------------------------------------------

permit (
    principal == User::"devon.williams@oyamail.network",
    action == Action::"connect.tax_preparer_share",
    resource is Tenant
) when {
    resource.tenant_id == "devon_williams_hvac_llc" &&
    context.preparer_id == "cookson-tax-accounting-hayward-ca-prep-id-7741" &&
    context.share_scope_fields containsAll ["schedule_c_lines", "1099_k_summary", "mileage_total"] &&
    context.share_scope_fields notContains "bayshore_w2" &&
    context.share_scope_fields notContains "customer_pii_full"
};

// ---------------------------------------------------------------------------
// ADR-0311 strict separation — Bayshore W-2 cannot enter the side-business
// ---------------------------------------------------------------------------

forbid (
    principal,
    action == Action::"finops.import_transactions",
    resource is Tenant
) when {
    context.source_tenant_id == "bayshore_climate_systems" &&
    resource.tenant_id == "devon_williams_hvac_llc" &&
    context.source_kind in ["w2_payroll", "w2_wage", "401k_employer", "benefits_pretax"]
};

// ---------------------------------------------------------------------------
// Forbid customer_pii on a tax-preparer share
// ---------------------------------------------------------------------------

forbid (
    principal,
    action == Action::"connect.tax_preparer_share",
    resource is Tenant
) when {
    context.share_scope_fields contains "customer_pii_full" ||
    context.share_scope_fields contains "customer_addresses" ||
    context.share_scope_fields contains "customer_phone_numbers"
};

// ---------------------------------------------------------------------------
// Forbid the Bayshore-role from touching the LLC's finops
// ---------------------------------------------------------------------------

forbid (
    principal,
    action == Action::"finops.year_end_reconcile",
    resource is Tenant
) when {
    resource.tenant_id == "devon_williams_hvac_llc" &&
    principal.active_role == "employee@bayshore_climate_systems"
};

// ---------------------------------------------------------------------------
// Stale step-up — refuse high-value operations
// ---------------------------------------------------------------------------

forbid (
    principal,
    action in [
        Action::"finops.year_end_reconcile",
        Action::"connect.tax_preparer_share",
        Action::"compliance.cdtfa.file_filing"
    ],
    resource
) when {
    !principal.passkey_step_up_within_seconds(300)
};
