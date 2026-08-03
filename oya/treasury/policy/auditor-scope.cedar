// Treasury auditor scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: TRM
// Benchmarks: SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank
// Preview metadata-only: audit/dashboard/SLO actions below are artifact-read labels only,
// not claims of runtime audit-chain emission, deployed dashboards, or measured SLO readiness.

forbid (principal, action, resource);

// The permitted read actions are artifact labels only, not runtime surfaces.
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
