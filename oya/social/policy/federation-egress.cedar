// Cedar v4.2 LTS — Social federation egress (ActivityPub outbound)
// Authority: ADR-SOC-0004 federation-posture

forbid (
  principal,
  action == Action::"SocialFederationOutboundPost",
  resource
) when {
  // Federation egress requires both tenant-opt-in AND per-post toggle
  !(resource has tenant_federation_enabled && resource.tenant_federation_enabled == true)
};

forbid (
  principal,
  action == Action::"SocialFederationOutboundPost",
  resource
) when {
  !(action has per_post_federation_enabled && action.per_post_federation_enabled == true)
};

forbid (
  principal,
  action == Action::"SocialFederationOutboundPost",
  resource
) when {
  // Posts containing PHI / PCI / minor-tagged content cannot federate outbound
  action has compliance_tag_in
  && action.compliance_tag_in.containsAny(["PHI", "PCI", "MINOR"])
};

forbid (
  principal,
  action == Action::"SocialFederationOutboundPost",
  resource
) when {
  // Refuse outbound to known-malicious or de-federated instances
  action has target_instance_blocklist
  && action.target_instance_blocklist == true
};
