// Cedar v4.2 LTS — comms-email pack-overlay authorization per ADR-0251 + ADR-0064.

permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"comms-email-admins",
    action in [Action::"subscribe_pack_overlay", Action::"unsubscribe_pack_overlay"],
    resource
) when {
    principal.tenant_id == resource.tenant_id
    && resource.pack_status == "published"
};

// Sovereign-pack: forbid if provider != postal-self-hosted.
forbid (principal, action == Action::"subscribe_pack_overlay", resource) when {
    resource.pack_id in ["ksa", "uae", "cn-pipl"]
    && resource.tenant.default_provider != "postal"
};

// HIPAA: forbid if BAA not signed.
forbid (principal, action == Action::"subscribe_pack_overlay", resource) when {
    resource.pack_id == "hipaa"
    && resource.tenant.baa_signed == false
};
