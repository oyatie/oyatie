// Healthcare Integration ci-scope
// binding_adrs: ADR-0243, ADR-0244, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
// soak_window_seconds: 60
// default_deny: true
forbid(principal, action, resource) unless {
  context has tenant_id &&
  context has principal_id &&
  context has audience_type &&
  context has cell_tier &&
  context has purpose &&
  context has data_class
};

permit(principal, action == Action::"clinical.interoperability.ci_validate", resource) when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context.audience_type != "ANONYMOUS" &&
  context.policy_evaluation_mode == "caller_side_library_first" &&
  context.provider_credential_mode in ["none", "openbao_sidecar_ttl_60s"]
};

forbid(principal, action, resource) when {
  context has bot_score &&
  context.bot_score >= 95 &&
  context.audience_type != "EMERGENCY_SERVICES"
};

permit(principal, action, resource) when {
  context.audience_type == "EMERGENCY_SERVICES" &&
  context.emergency_attestation == "jurisdiction_registered" &&
  context.audit_event_class == "AbuseDefenceEmergencyServiceBypass"
};
