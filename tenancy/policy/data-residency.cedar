// Cedar v4.2 LTS — tenancy µservice data-residency
// Binding: ADR-0244 + ADR-0248 + ADR-0251
// Companion: policy/data-residency.md (prose)

// Forbid tenant data processing outside its home cell jurisdiction.
forbid (principal, action, resource) when {
    resource.processing_region != resource.tenant.home_jurisdiction
    && resource.tenant.residency_strict == true
};

// KR-CSAP: KR tenants MUST process in KR cells.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("kr-csap")
    && resource.processing_region != "kr"
};

// EU-sovereign: EU tenants MUST process in EU-sovereign cells.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("eu-sovereign")
    && !resource.processing_region.startsWith("eu-sovereign-")
};

// CN-PIPL: CN tenants MUST process in CN-onshore cells.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("cn-pipl")
    && resource.processing_region != "cn-onshore"
};

// DR pairing constraint: DR cell MUST be in the same jurisdiction as home cell.
forbid (principal, action == Action::"assign_dr_cell", resource) when {
    resource.tenant.residency_strict == true
    && resource.dr_cell.jurisdiction != resource.tenant.home_jurisdiction
};
