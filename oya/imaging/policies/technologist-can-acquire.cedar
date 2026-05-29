// Cedar policy: Technologist can acquire studies on assigned modalities only.
//
// Authority: ADR-0243 Cedar universal gate.
//
// Principal: User with role "technologist".
// Action: imaging::Action::"PullMWL" | "EmitMPPS" | "CStore" | "TechQC" |
//         "RouteToWorklist" | "EmitDose"
// Resource: imaging::Modality | imaging::WorklistWorkitem | imaging::Mpps |
//           imaging::Instance | imaging::DoseReport

permit (
  principal in imaging::Role::"technologist",
  action in [
    imaging::Action::"PullMWL",
    imaging::Action::"EmitMPPS",
    imaging::Action::"CStore",
    imaging::Action::"TechQC",
    imaging::Action::"RouteToWorklist",
    imaging::Action::"EmitDose",
    imaging::Action::"AbortAcquisition"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.assigned_modality_ae_titles.contains(resource.modality_ae_title) &&
  principal.active &&
  !principal.suspended
};

// Mammography acquisition requires MQSA-certified technologist.
permit (
  principal in imaging::Role::"technologist",
  action == imaging::Action::"CStore",
  resource is imaging::Instance
) when {
  resource.modality == "MG" &&
  principal.credentials.contains("MQSA-certified-technologist")
};

forbid (
  principal in imaging::Role::"technologist",
  action == imaging::Action::"CStore",
  resource is imaging::Instance
) when {
  resource.modality == "MG" &&
  !principal.credentials.contains("MQSA-certified-technologist")
};

// Fluoroscopy + IR require state-licensed-tech credential.
forbid (
  principal in imaging::Role::"technologist",
  action == imaging::Action::"CStore",
  resource is imaging::Instance
) when {
  resource.modality in ["RF", "XA", "IR"] &&
  !principal.credentials.contains("state-licensed-fluoroscopy")
};
