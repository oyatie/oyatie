// Production Planning capacity-calendar authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: PP
// Benchmarks: SAP PP Production Planning | Oracle Fusion Cloud Manufacturing | Workday Adaptive Planning production-capacity counterpart | NetSuite Manufacturing WIP and Routings | Microsoft Dynamics 365 Supply Chain Management

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"capacity-calendar.read",
    Action::"capacity-calendar.create",
    Action::"capacity-calendar.amend",
    Action::"capacity-calendar.approve",
    Action::"capacity-calendar.reverse",
    Action::"capacity-calendar.export"
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
  action in [Action::"capacity-calendar.approve", Action::"capacity-calendar.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
