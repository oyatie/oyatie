// medical-lab-technologist-can-result.cedar
// Diagnostics lab-result entry and QC evidence gate.

permit (
  principal in Role::"medical-lab-technologist",
  action in [
    Action::"lab-result.record",
    Action::"quality-control.record",
    Action::"specimen.accession"
  ],
  resource
)
when {
  resource.tenant_id == principal.tenant_id &&
  context.purpose_of_use in ["treatment", "operations"] &&
  context.break_glass == false
};

forbid (
  principal in Role::"medical-lab-technologist",
  action in [
    Action::"lab-result.release",
    Action::"pathology-case.sign-out"
  ],
  resource
);
