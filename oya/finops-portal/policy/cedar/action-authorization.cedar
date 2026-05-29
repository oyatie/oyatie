// Cedar v4.2 LTS — finops-portal action-authorization (default-deny)
// Binding: ADR-0243 + ADR-0199
// x-fragment-soak-seconds: 60

permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"finops-admins",
    action in [
        Action::"view_invoice",
        Action::"download_invoice_pdf",
        Action::"download_focus_export",
        Action::"set_budget_alert",
        Action::"view_cost_drilldown",
        Action::"view_forecast",
        Action::"view_rightsizing_recommendations",
        Action::"apply_recommendation",
        Action::"manage_commitment"
    ],
    resource
) when {
    principal.tenant_id == resource.tenant_id
};

permit (
    principal in oyatie::FinopsPortal::CustomerSuccessRole::"on-call",
    action in [Action::"apply_credit", Action::"view_any_tenant_cost"],
    resource
) when {
    principal.engagement_active == true
};

// Substrate principals.
permit (
    principal in [
        oyatie::FinopsPortal::Principal::"tenant-billing-presenter",
        oyatie::FinopsPortal::Principal::"focus-exporter",
        oyatie::FinopsPortal::Principal::"forecaster",
        oyatie::FinopsPortal::Principal::"rightsizing-recommender",
        oyatie::FinopsPortal::Principal::"commitment-manager",
        oyatie::FinopsPortal::Principal::"budget-alert-notifier"
    ],
    action,
    resource
) when {
    principal.spiffe_attested == true
};

forbid (principal, action, resource) when {
    principal.tenant_id != resource.tenant_id
    && !(principal in oyatie::FinopsPortal::CustomerSuccessRole::"on-call")
    && !(principal.principal_kind == "workload")
};
