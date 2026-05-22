// Real Estate occupancy-allocation authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: RE-FX
// Benchmarks: SAP RE-FX Flexible Real Estate Management | Oracle Fusion Lease Accounting | Workday Lease Accounting | NetSuite Fixed Assets and lease-accounting counterpart | Microsoft Dynamics 365 Finance Lease Accounting

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"occupancy-allocation.read",
    Action::"occupancy-allocation.create",
    Action::"occupancy-allocation.amend",
    Action::"occupancy-allocation.approve",
    Action::"occupancy-allocation.reverse",
    Action::"occupancy-allocation.export"
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
  action in [Action::"occupancy-allocation.approve", Action::"occupancy-allocation.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
