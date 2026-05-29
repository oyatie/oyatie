// Cedar v4.2 LTS — finops-portal CI scope per ADR-0295

permit (
    principal in oyatie::Foundry::CIPrincipal::"finops-portal-validators",
    action in [
        Action::"emit_synthetic_invoice",
        Action::"verify_focus_export_schema",
        Action::"verify_cost_allocation_policy_eval",
        Action::"render_synthetic_invoice_pdf"
    ],
    resource
) when {
    principal.spiffe_attested == true
    && resource.environment == "ci"
    && resource.synthetic == true
};

forbid (
    principal in oyatie::Foundry::CIPrincipal::"finops-portal-validators",
    action,
    resource
) when {
    resource.environment != "ci"
    || resource.contains_production_data == true
};
