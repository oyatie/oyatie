// Cedar v4.2 LTS — Mail PHI DLP (HIPAA-eligible mailboxes)
// Authority: ADR-0251 compliance pack primitive (HIPAA); ADR-0244 audience_type

forbid (
  principal,
  action == Action::"MailOutboundSend",
  resource
) when {
  // PHI outbound: receiving domain must be on HIPAA-attested partner list
  principal.tenant.audience_type == "B2B_HIPAA_PHI"
  && action has phi_detected
  && action.phi_detected == true
  && action has recipient_domain
  && !(resource has hipaa_attested_partner_domains
       && resource.hipaa_attested_partner_domains.contains(action.recipient_domain))
};

forbid (
  principal,
  action == Action::"MailIntelligenceCall",
  resource
) when {
  // Intelligence calls on PHI mailboxes must route through HIPAA-conformant variant
  principal.tenant.audience_type == "B2B_HIPAA_PHI"
  && !(action has intelligence_variant && action.intelligence_variant == "HIPAA_CONFORMANT")
};

forbid (
  principal,
  action == Action::"MailExportRequest",
  resource
) when {
  // PHI export requires both tenant-admin and PHI-officer roles
  resource.tenant.audience_type == "B2B_HIPAA_PHI"
  && !(principal has tenant_role && principal.tenant_role == "TENANT_ADMIN")
  && !(principal has phi_officer_role && principal.phi_officer_role == true)
};
