// Supply Chain Planning planning-scenario authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: SCM/APO
// Benchmarks: SAP SCM/APO and SAP IBP | Oracle Supply Chain Planning | Workday Adaptive Planning supply-chain scenario counterpart | NetSuite Demand Planning | Microsoft Dynamics 365 Master Planning

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"planning-scenario.read",
    Action::"planning-scenario.create",
    Action::"planning-scenario.amend",
    Action::"planning-scenario.approve",
    Action::"planning-scenario.reverse",
    Action::"planning-scenario.export"
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
  action in [Action::"planning-scenario.approve", Action::"planning-scenario.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
