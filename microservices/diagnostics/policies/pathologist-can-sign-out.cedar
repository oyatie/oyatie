// pathologist-can-sign-out.cedar
// Pathologist sign-out gate for pathology cases.

permit (
  principal in Role::"pathologist",
  action in [
    Action::"pathology-case.read",
    Action::"pathology-case.sign-out",
    Action::"pathology-case.amend",
    Action::"DiagnosticReport.read"
  ],
  resource
)
when {
  resource.tenant_id == principal.tenant_id &&
  principal.id in resource.assigned_pathologist_ids &&
  context.purpose_of_use == "treatment" &&
  context.electronic_signature_verified == true
};

forbid (
  principal in Role::"pathologist",
  action in [Action::"pathology-case.sign-out"],
  resource
)
when {
  resource.tenant_id == principal.tenant_id &&
  context.electronic_signature_verified == false
};
