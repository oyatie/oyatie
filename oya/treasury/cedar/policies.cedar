// Cedar policy fragments for treasury preview metadata.
// Authority: specs/microservices/treasury.json (PRD-MICROSERVICE-TREASURY), status=preview.
// This file is an honest-claims metadata guardrail only: no runtime policy
// enforcement, no payment execution, no live bank connectivity, no durable
// persistence, no Workflow execution, no runtime audit-chain emission, no cloud
// deployment, no GA readiness, and no measured SLO/DR readiness.

forbid(
  principal,
  action in [
    Action::"ComputeAutoshardingPlacement",
    Action::"ExecuteAutoRebalance",
    Action::"ExecuteDynamicSharding",
    Action::"MigrateTenantCrossJurisdiction",
    Action::"RunSelfHostedCi",
    Action::"SyncApplication"
  ],
  resource
) when {
  resource has microservice &&
  resource.microservice == "treasury"
};

forbid(
  principal,
  action in [
    Action::"ConnectLiveBankNetwork",
    Action::"DeployTreasuryRuntime",
    Action::"EmitRuntimeAuditChain",
    Action::"ExecutePayment",
    Action::"PersistTreasuryRecords"
  ],
  resource
) when {
  resource has microservice &&
  resource.microservice == "treasury"
};
