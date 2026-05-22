// Cedar policy: Dose Compliance Officer can read aggregate dose data only.
//
// Authority: ADR-0243 + EURATOM 2013/59 + CMS QPP MIPS Measure 145.
//
// Principal: User with role "dose-compliance-officer".
// Action: imaging::Action::"ReadDoseAggregate" | "ExportDoseRegulatory"
// Resource: imaging::DoseAggregate | imaging::DoseRegulatoryExport

permit (
  principal in imaging::Role::"dose-compliance-officer",
  action in [
    imaging::Action::"ReadDoseAggregate",
    imaging::Action::"ExportDoseRegulatory",
    imaging::Action::"QueryDoseDeviations"
  ],
  resource is imaging::DoseAggregate
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.active &&
  !principal.suspended &&
  resource.aggregation_level in ["modality", "protocol", "technologist", "patient-cohort"]
};

permit (
  principal in imaging::Role::"dose-compliance-officer",
  action == imaging::Action::"ExportDoseRegulatory",
  resource is imaging::DoseRegulatoryExport
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.export_format in ["EURATOM-2013-59", "CMS-QPP-MIPS-145", "NEMA-XR-29-dose-check"]
};

// Forbid dose officer from accessing pixel-level PHI; they only need dose
// summaries.
forbid (
  principal in imaging::Role::"dose-compliance-officer",
  action in [
    imaging::Action::"ReadInstance",
    imaging::Action::"WADORetrieve",
    imaging::Action::"RenderFrame"
  ],
  resource
);

// Forbid dose officer from reading individual-patient identifiable dose data
// unless aggregation_level == "patient-cohort" with explicit ethics-board
// approval.
forbid (
  principal in imaging::Role::"dose-compliance-officer",
  action == imaging::Action::"ReadDoseAggregate",
  resource is imaging::DoseAggregate
) when {
  resource.aggregation_level == "patient" &&
  resource.ethics_board_approval == false
};
