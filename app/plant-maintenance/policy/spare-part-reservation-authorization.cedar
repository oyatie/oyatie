// Plant Maintenance spare-part-reservation authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: PM
// Benchmarks: SAP PM Plant Maintenance | Oracle Fusion Maintenance | Workday Extend asset-maintenance workflow counterpart | NetSuite Fixed Assets and Field Service counterpart | Microsoft Dynamics 365 Asset Management

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"spare-part-reservation.read",
    Action::"spare-part-reservation.create",
    Action::"spare-part-reservation.amend",
    Action::"spare-part-reservation.approve",
    Action::"spare-part-reservation.reverse",
    Action::"spare-part-reservation.export"
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
  action in [Action::"spare-part-reservation.approve", Action::"spare-part-reservation.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
