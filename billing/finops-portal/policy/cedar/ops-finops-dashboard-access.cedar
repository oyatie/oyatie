// Cedar policy — ops-finops dashboard access (+ tenant-admin scoped)
// Authored 2026-05-18.
// Authority: ADR-0183 + ADR-0186 + ADR-0199.
// Schema:  ./schema.cedarschema.json
//
// Governs EmbedDashboard action. Tenant admins embed their own
// drill-down; ops-finops embeds fleet rollup + anomaly investigation.

// Permit: tenant admin embeds their drill-down dashboard.
permit (
  principal is FinopsPortal::TenantAdmin,
  action == FinopsPortal::Action::"EmbedDashboard",
  resource is FinopsPortal::Dashboard
) when {
  resource.id == "tenant-cost-drilldown" &&
  resource.scope == "tenant"
};

// Permit: ops-finops embeds fleet + anomaly dashboards.
permit (
  principal is FinopsPortal::OpsFinops,
  action == FinopsPortal::Action::"EmbedDashboard",
  resource is FinopsPortal::Dashboard
) when {
  principal.tenant_scope == "fleet" &&
  resource.id in ["fleet-cost-rollup", "anomaly-investigation", "tenant-cost-drilldown"]
};

// Forbid: tenant-admin embeds fleet-rollup or anomaly dashboards.
forbid (
  principal is FinopsPortal::TenantAdmin,
  action == FinopsPortal::Action::"EmbedDashboard",
  resource is FinopsPortal::Dashboard
) when {
  resource.id in ["fleet-cost-rollup", "anomaly-investigation"]
};

// Forbid: customer-success embeds dashboards (read-only access via
// the API is enough; embedded dashboards are scoped to tenant or
// fleet roles).
forbid (
  principal is FinopsPortal::CustomerSuccess,
  action == FinopsPortal::Action::"EmbedDashboard",
  resource is FinopsPortal::Dashboard
);
