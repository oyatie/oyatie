// Plant Maintenance auditor scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: PM
// Benchmarks: SAP PM Plant Maintenance | Oracle Fusion Maintenance | Workday Extend asset-maintenance workflow counterpart | NetSuite Fixed Assets and Field Service counterpart | Microsoft Dynamics 365 Asset Management

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
