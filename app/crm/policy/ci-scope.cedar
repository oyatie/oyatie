// Customer Relationship Management ci scope
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: CRM
// Benchmarks: SAP CRM Customer Relationship Management | Oracle Fusion Sales and Service | Workday customer and service-workflow counterpart | NetSuite CRM | Microsoft Dynamics 365 Customer Engagement

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
