// Cedar v4.2 LTS — finops-portal auditor scope
// Finance auditors get tenant-scoped read; engagement-window-bound.

permit (
    principal in oyatie::Compliance::AuditorGroup::"finance-engaged",
    action in [
        Action::"view_invoice",
        Action::"view_credit_history",
        Action::"download_focus_export",
        Action::"view_commitment_ledger"
    ],
    resource
) when {
    principal.engagement_active == true
    && principal.engagement_tenant_id == resource.tenant_id
};

forbid (
    principal in oyatie::Compliance::AuditorGroup::"finance-engaged",
    action,
    resource
) when {
    principal.engagement_ends_at < context.request_time
};
