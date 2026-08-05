// Current authority: specs/microservices/global-trade.json metadata-only PRD; inventory/provenance/planned-only.
// This Cedar fragment is planned/provenance only and is not runtime policy-engine authority.
// Non-claims: no live authorization runtime, no Workflow execution, no runtime audit-chain emission, no cloud deployment.
// Global Trade auditor scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: GTS
// Benchmarks: SAP GTS Global Trade Services | Oracle Global Trade Management | Workday supplier-compliance workflow counterpart | NetSuite international tax and trade counterpart | Microsoft Dynamics 365 global trade and export-control counterpart

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
