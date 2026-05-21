// Quality Management auditor scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: QM
// Benchmarks: SAP QM Quality Management | Oracle Fusion Quality Management | Workday Extend quality-workflow counterpart | NetSuite Quality Management | Microsoft Dynamics 365 Supply Chain Quality Management

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
