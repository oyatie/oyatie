// Quality Management inspection-lot authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: QM
// Benchmarks: SAP QM Quality Management | Oracle Fusion Quality Management | Workday Extend quality-workflow counterpart | NetSuite Quality Management | Microsoft Dynamics 365 Supply Chain Quality Management

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"inspection-lot.read",
    Action::"inspection-lot.create",
    Action::"inspection-lot.amend",
    Action::"inspection-lot.approve",
    Action::"inspection-lot.reverse",
    Action::"inspection-lot.export"
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
  action in [Action::"inspection-lot.approve", Action::"inspection-lot.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
