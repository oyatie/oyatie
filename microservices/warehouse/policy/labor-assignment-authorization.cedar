// Warehouse labor-assignment authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: EWM
// Benchmarks: SAP EWM Extended Warehouse Management | Oracle Fusion Warehouse Management | Workday inventory-operations counterpart | NetSuite WMS | Microsoft Dynamics 365 Warehouse Management

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"labor-assignment.read",
    Action::"labor-assignment.create",
    Action::"labor-assignment.amend",
    Action::"labor-assignment.approve",
    Action::"labor-assignment.reverse",
    Action::"labor-assignment.export"
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
  action in [Action::"labor-assignment.approve", Action::"labor-assignment.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
