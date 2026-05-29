// Cedar v4.2 LTS — comms-email data-residency
// Binding: ADR-0244 + ADR-0251

// KR-CSAP: KR-tenant email routed via Postal-onshore only.
forbid (principal, action == Action::"send_transactional", resource) when {
    resource.tenant.compliance_packs.contains("kr-csap")
    && resource.provider != "postal-kr"
};

// EU-sovereign: EU tenant via EU Postal (no SES, no Mailgun).
forbid (principal, action, resource) when {
    resource.tenant.compliance_packs.contains("eu-sovereign")
    && !resource.provider.startsWith("postal-eu")
};

// HIPAA: PHI emails MUST route via BAA-eligible provider (SES with BAA / Postal self-hosted).
forbid (principal, action == Action::"send_transactional", resource) when {
    resource.tenant.compliance_packs.contains("hipaa")
    && resource.contains_phi == true
    && !resource.provider.baa_eligible == true
};

// Suppression list MUST be co-located in tenant home jurisdiction.
forbid (principal, action == Action::"add_suppression", resource) when {
    resource.suppression_region != resource.tenant.home_jurisdiction
};
