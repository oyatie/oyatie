// hipaa-deny-default.cedar
// Diagnostics HIPAA minimum-necessary gate for lab and pathology PHI.

forbid (
  principal,
  action,
  resource
)
when {
  resource.tenant_id != principal.tenant_id
};

forbid (
  principal,
  action,
  resource
)
when {
  !(context.purpose_of_use in ["treatment", "payment", "operations", "quality", "patient-request"])
};

permit (
  principal,
  action in [
    Action::"lab-order.read",
    Action::"lab-result.read",
    Action::"pathology-case.read",
    Action::"critical-result.read",
    Action::"specimen.read",
    Action::"DiagnosticReport.read",
    Action::"Observation.read"
  ],
  resource
)
when {
  resource.tenant_id == principal.tenant_id &&
  context.minimum_necessary == true
};
