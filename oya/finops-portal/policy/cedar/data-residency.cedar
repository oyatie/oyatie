// Cedar v4.2 LTS — finops-portal data-residency
// Cost data == tenant-PII; honor home jurisdiction.

forbid (principal, action, resource) when {
    resource.processing_region != resource.tenant.home_jurisdiction
    && resource.tenant.residency_strict == true
};

// KR-CSAP: KR tenants → KR cells for invoice + FOCUS export.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("kr-csap")
    && resource.processing_region != "kr"
};

// EU-sovereign: EU tenants → EU-sovereign cells.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("eu-sovereign")
    && !resource.processing_region.startsWith("eu-sovereign-")
};

// HIPAA: BAA-eligible cells for HIPAA tenant invoices.
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("hipaa")
    && !resource.processing_region.in_baa_region == true
};
