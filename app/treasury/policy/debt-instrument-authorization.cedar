// Treasury debt-instrument authorization
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: TRM
// Benchmarks: SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank

forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"debt-instrument.read",
    Action::"debt-instrument.create",
    Action::"debt-instrument.amend",
    Action::"debt-instrument.approve",
    Action::"debt-instrument.reverse",
    Action::"debt-instrument.export"
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
  action in [Action::"debt-instrument.approve", Action::"debt-instrument.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
