// Cedar policy: AI vendor model may read DE-IDENTIFIED pixel data only.
//
// Authority: ADR-0243 + HIPAA Safe Harbor 45 CFR 164.514(b)(2) + ISO/TS 25237.
// Critical compliance gate: PHI MUST NOT egress to AI vendor.
//
// Principal: imaging::AiVendorModel
// Action: imaging::Action::"InferOn"
// Resource: imaging::DeIdentifiedStudy

permit (
  principal is imaging::AiVendorModel,
  action == imaging::Action::"InferOn",
  resource is imaging::DeIdentifiedStudy
) when {
  resource.is_deidentified == true &&
  resource.deidentification_method in [
    "HIPAA-Safe-Harbor-2024",
    "ISO-TS-25237-pseudonymization"
  ] &&
  resource.deidentification_verified == true &&
  principal.fda_or_ce_cleared == true &&
  principal.cleared_indications.contains(resource.indication) &&
  principal.cleared_modalities.contains(resource.modality) &&
  principal.enabled_in_tenant(resource.tenant_id) &&
  resource.tenant_consent_for_ai_egress == true
};

// Forbid AI inference on PHI-identified studies.
forbid (
  principal is imaging::AiVendorModel,
  action == imaging::Action::"InferOn",
  resource is imaging::Study
) when {
  resource.is_deidentified == false
};

// Forbid off-label AI inference (indication/modality not on FDA/CE clearance).
forbid (
  principal is imaging::AiVendorModel,
  action == imaging::Action::"InferOn",
  resource is imaging::DeIdentifiedStudy
) when {
  !principal.cleared_indications.contains(resource.indication) ||
  !principal.cleared_modalities.contains(resource.modality)
};

// Forbid AI vendor inference when tenant has not consented to AI egress.
forbid (
  principal is imaging::AiVendorModel,
  action == imaging::Action::"InferOn",
  resource is imaging::DeIdentifiedStudy
) when {
  resource.tenant_consent_for_ai_egress == false
};
