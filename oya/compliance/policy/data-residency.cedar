// Cedar v4.2 LTS — compliance µservice data-residency gate
// Binding ADR: ADR-0244 (tenant scoping) + ADR-0251 (compliance packs)
// Companion: policy/data-residency.md

// Evidence MUST be sealed in the tenant's home-cell jurisdiction.
forbid (principal, action == Action::"seal_evidence", resource) when {
    resource.seal_jurisdiction != resource.tenant.home_jurisdiction
};

// KR-CSAP pack: KR-jurisdiction tenants' evidence stays in KR-region cells.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("kr-csap")
    && resource.processing_region != "kr"
};

// EU-sovereign pack: EU tenants' evidence stays in EU-sovereign-cell tiers.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("eu-sovereign")
    && !resource.processing_region.startsWith("eu-")
};

// CN-PIPL pack: CN-jurisdiction tenant evidence in CN-onshore cells only.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("cn-pipl")
    && resource.processing_region != "cn-onshore"
};

// US-Healthcare (HIPAA) BAA-enforced regions only.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("hipaa")
    && !resource.processing_region.in_baa_region == true
};
