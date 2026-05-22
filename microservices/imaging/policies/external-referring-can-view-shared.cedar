// Cedar policy: External referring clinician (cross-organization) may view
// shared studies only via XCA-I / cross-clinician share-link, time-boxed
// and consent-graph-gated.
//
// Authority: ADR-0243 + IHE XCA-I + consent-graph.
//
// Principal: imaging::ExternalReferringClinician
// Action: imaging::Action::"ViewSharedStudy" | "ViewSharedReport"
// Resource: imaging::Study | imaging::StructuredReport | imaging::ShareLink

permit (
  principal is imaging::ExternalReferringClinician,
  action in [
    imaging::Action::"ViewSharedStudy",
    imaging::Action::"ViewSharedReport"
  ],
  resource
) when {
  principal.share_link != nil &&
  principal.share_link.tenant_id == resource.tenant_id &&
  principal.share_link.study_instance_uid == resource.study_instance_uid &&
  principal.share_link.expires_at > context.now &&
  principal.share_link.revoked == false &&
  principal.share_link.target_identified == true &&
  principal.consent_graph.patient_consent_for_external_share == true
};

forbid (
  principal is imaging::ExternalReferringClinician,
  action,
  resource
) when {
  principal.share_link.expires_at <= context.now ||
  principal.share_link.revoked == true
};

// Forbid external clinician from downloading raw DICOM (only WADO-rendered).
forbid (
  principal is imaging::ExternalReferringClinician,
  action == imaging::Action::"WADORetrieve",
  resource
) when {
  context.media_type == "application/dicom"
};

// Forbid external clinician from any write action.
forbid (
  principal is imaging::ExternalReferringClinician,
  action in [
    imaging::Action::"AuthorStructuredReport",
    imaging::Action::"SignStructuredReport",
    imaging::Action::"CStore",
    imaging::Action::"DispatchAi"
  ],
  resource
);
