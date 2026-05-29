// Customer Relationship Management emergency services bypass
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: CRM
// Benchmarks: SAP CRM Customer Relationship Management | Oracle Fusion Sales and Service | Workday customer and service-workflow counterpart | NetSuite CRM | Microsoft Dynamics 365 Customer Engagement

permit (principal, action, resource)
when {
  context has audience_type &&
  context.audience_type == "EMERGENCY_SERVICES" &&
  context has emergency_attestation_chain &&
  context.emergency_attestation_chain != "" &&
  context has audit_event_class &&
  context.audit_event_class == "AbuseDefenceEmergencyServiceBypass"
};

forbid (principal, action, resource)
when {
  context has audience_type &&
  context.audience_type == "EMERGENCY_SERVICES" &&
  !(context has emergency_attestation_chain)
};
