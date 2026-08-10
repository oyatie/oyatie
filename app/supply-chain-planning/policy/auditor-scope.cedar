// Supply Chain Planning auditor scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: SCM/APO
// Benchmarks: SAP SCM/APO and SAP IBP | Oracle Supply Chain Planning | Workday Adaptive Planning supply-chain scenario counterpart | NetSuite Demand Planning | Microsoft Dynamics 365 Master Planning

forbid (principal, action, resource);

permit (principal, action in [Action::"artifact.read", Action::"audit.read", Action::"dashboard.read", Action::"slo.read"], resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has valid_from &&
  context has valid_to &&
  context.valid_from <= context.now &&
  context.now <= context.valid_to
};
