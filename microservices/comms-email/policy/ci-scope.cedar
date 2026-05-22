// Cedar v4.2 LTS — comms-email CI scope per ADR-0295

permit (
    principal in oyatie::Foundry::CIPrincipal::"comms-email-validators",
    action in [
        Action::"send_synthetic_email",
        Action::"verify_dkim_signature",
        Action::"verify_dmarc_alignment",
        Action::"replay_synthetic_webhook"
    ],
    resource
) when {
    principal.spiffe_attested == true
    && resource.environment == "ci"
    && resource.synthetic == true
};

forbid (
    principal in oyatie::Foundry::CIPrincipal::"comms-email-validators",
    action,
    resource
) when {
    resource.environment != "ci"
    || resource.contains_production_data == true
};
