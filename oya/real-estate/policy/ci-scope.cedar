// Real Estate ci scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: RE-FX
// Benchmarks: SAP RE-FX Flexible Real Estate Management | Oracle Fusion Lease Accounting | Workday Lease Accounting | NetSuite Fixed Assets and lease-accounting counterpart | Microsoft Dynamics 365 Finance Lease Accounting

forbid (principal, action, resource);

permit (principal, action in [Action::"contract.read", Action::"policy.read", Action::"scorecard.read", Action::"evidence.read"], resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has valid_from &&
  context has valid_to &&
  context.valid_from <= context.now &&
  context.now <= context.valid_to
};
