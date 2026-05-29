// Cedar policy boundary for FD-001 Ops Dashboard / Control Center operator actions.
// Runtime policies will be generated from this design anchor and tested with per-action fixtures.

permit(
  principal,
  action in [
    OpsAction::"incident.declare",
    OpsAction::"incident.remediation.approve",
    OpsAction::"deployment.approve",
    OpsAction::"rollback.execute",
    OpsAction::"evidence.export"
  ],
  resource
)
when {
  principal.hasMfa == true &&
  principal.tenant == resource.tenant &&
  principal.role in ["sre-oncall", "release-manager", "compliance-operator"] &&
  context.idempotency_key_present == true &&
  context.audit_chain_seal_required == true
};

forbid(
  principal,
  action,
  resource
)
when {
  context.break_glass == true &&
  context.break_glass_ticket == ""
};

forbid(
  principal,
  action,
  resource
)
when {
  principal.tenant != resource.tenant
};
