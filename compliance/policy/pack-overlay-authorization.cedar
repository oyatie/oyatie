// Cedar v4.2 LTS — compliance µservice pack-overlay authorization
// Binding ADR: ADR-0251 (compliance packs) + ADR-0064 (canonical-base + localization)

// Pack-registry admin may add/remove pack manifests for the platform owner only.
permit (
    principal in oyatie::Compliance::PackRegistryAdminGroup::"platform-owner",
    action in [
        Action::"add_pack_manifest",
        Action::"deprecate_pack_manifest",
        Action::"publish_pack_overlay"
    ],
    resource
) when {
    principal.tenant_id == oyatie::Tenancy::Tenant::"oyatie-platform-owner"
};

// Tenant admin may subscribe their tenant to a pack from the published registry.
permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"compliance-admins",
    action in [Action::"subscribe_pack", Action::"unsubscribe_pack"],
    resource
) when {
    principal.tenant_id == resource.tenant_id
    && resource.pack_status == "published"
    && resource.pack_available_to_audience.contains(principal.tenant.audience_type)
};

// Forbid pack-subscription if pack jurisdiction conflicts with tenant home jurisdiction.
forbid (principal, action == Action::"subscribe_pack", resource) when {
    resource.pack_required_jurisdiction != null
    && resource.pack_required_jurisdiction != principal.tenant.home_jurisdiction
};
