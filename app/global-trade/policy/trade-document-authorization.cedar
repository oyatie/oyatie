// Global Trade trade-document authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: GTS
// Benchmarks: SAP GTS Global Trade Services | Oracle Global Trade Management | Workday supplier-compliance workflow counterpart | NetSuite international tax and trade counterpart | Microsoft Dynamics 365 global trade and export-control counterpart

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"trade-document.read",
    Action::"trade-document.create",
    Action::"trade-document.amend",
    Action::"trade-document.approve",
    Action::"trade-document.reverse",
    Action::"trade-document.export"
  ],
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has purpose_code &&
  context has compliance_packs &&
  context has marketplace_settlement_ref &&
  context.marketplace_settlement_ref != "" &&
  context has policy_bundle_version &&
  context.policy_bundle_version >= "2026-05-21"
};

forbid (
  principal,
  action in [Action::"trade-document.approve", Action::"trade-document.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
