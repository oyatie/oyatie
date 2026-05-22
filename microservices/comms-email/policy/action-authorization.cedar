// Cedar v4.2 LTS — comms-email µservice action-authorization (default-deny)
// Binding ADR: ADR-0243 + ADR-0273
// x-fragment-soak-seconds: 60

permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"comms-email-admins",
    action in [
        Action::"send_transactional",
        Action::"send_marketing",
        Action::"onboard_from_domain",
        Action::"rotate_dkim",
        Action::"view_deliverability",
        Action::"manage_suppression"
    ],
    resource
) when {
    principal.tenant_id == resource.tenant_id
};

permit (
    principal in [
        oyatie::CommsEmail::Principal::"outbound-sender",
        oyatie::CommsEmail::Principal::"inbound-receiver",
        oyatie::CommsEmail::Principal::"deliverability-tracker",
        oyatie::CommsEmail::Principal::"reputation-monitor"
    ],
    action,
    resource
) when {
    principal.spiffe_attested == true
};

// Recipient self-service: unsubscribe + view-preferences via signed token.
permit (
    principal,
    action in [Action::"unsubscribe", Action::"view_preferences"],
    resource
) when {
    principal.audience_type == "B2C_CONSUMER_RECIPIENT"
    && principal.unsubscribe_token_valid == true
};

// Cross-tenant defence-in-depth.
forbid (principal, action, resource) when {
    principal.tenant_id != resource.tenant_id
    && principal.audience_type != "B2C_CONSUMER_RECIPIENT"
};
