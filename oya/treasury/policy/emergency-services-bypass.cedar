// Treasury emergency services bypass
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: TRM
// Benchmarks: SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank

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
