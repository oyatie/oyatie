// Cedar policy: Radiologist can read studies in their tenant + assigned-group
//
// Authority: ADR-0243 Cedar universal gate + ADR-0244 tenant scoping +
// ADR-0251 HIPAA pack overlay.
//
// Principal: User with role "radiologist".
// Action: imaging::Action::"ReadStudy" | "ReadInstance" | "RenderFrame" |
//         "QueryWorklist" | "OpenWorklistItem" | "QIDOQuery" | "WADORetrieve"
// Resource: imaging::Study | imaging::Series | imaging::Instance | imaging::WorklistItem
//
// Tenant boundary: principal.tenant_id == resource.tenant_id.
// Subspecialty boundary: principal.subspecialty must include resource.subspecialty
//   OR principal has "all-subspecialties" credential.
// Patient relationship: principal must be in the study's authorized reader set
//   OR be on call OR have explicit handoff assignment.

permit (
  principal in imaging::Role::"radiologist",
  action in [
    imaging::Action::"ReadStudy",
    imaging::Action::"ReadInstance",
    imaging::Action::"RenderFrame",
    imaging::Action::"QueryWorklist",
    imaging::Action::"OpenWorklistItem",
    imaging::Action::"QIDOQuery",
    imaging::Action::"WADORetrieve",
    imaging::Action::"AuthorStructuredReport",
    imaging::Action::"SignStructuredReport",
    imaging::Action::"ApplyHangingProtocol",
    imaging::Action::"MeasureROI",
    imaging::Action::"DispatchAi",
    imaging::Action::"ToggleAiOverlay",
    imaging::Action::"FetchPriors",
    imaging::Action::"AddendumReport"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  (
    principal.subspecialties.contains(resource.subspecialty) ||
    principal.credentials.contains("all-subspecialties")
  ) &&
  (
    resource.assigned_readers.contains(principal) ||
    principal.on_call_for_period(resource.acquisition_time) ||
    principal.handoff_assignments.contains(resource.study_instance_uid)
  ) &&
  principal.active &&
  !principal.suspended &&
  resource.tenant_class != imaging::TenantClass::"DEMO_TRIAL"
};

// Demo_trial tenant: radiologist may only read synthetic studies
permit (
  principal in imaging::Role::"radiologist",
  action in [
    imaging::Action::"ReadStudy",
    imaging::Action::"ReadInstance",
    imaging::Action::"RenderFrame",
    imaging::Action::"QueryWorklist",
    imaging::Action::"OpenWorklistItem"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == imaging::TenantClass::"DEMO_TRIAL" &&
  resource.is_synthetic == true
};

forbid (
  principal,
  action,
  resource
) when {
  principal.tenant_id != resource.tenant_id
};
