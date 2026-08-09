// Cedar policy: Patient can view their own imaging studies via patient portal.
//
// Authority: ADR-0243 + consent-graph + HIPAA + GDPR Article 15 (right of access).
//
// Principal: imaging::PortalPatient
// Action: imaging::Action::"PortalViewStudy" | "PortalDownloadDICOM" |
//         "PortalReadLaypersonSummary" | "PortalShareWithExternal"
// Resource: imaging::Study | imaging::PortalToken | imaging::LaypersonSummary

permit (
  principal is imaging::PortalPatient,
  action in [
    imaging::Action::"PortalViewStudy",
    imaging::Action::"PortalReadLaypersonSummary"
  ],
  resource is imaging::Study
) when {
  principal.patient_id == resource.patient_id &&
  principal.consent_graph.affirmative_consent_for_portal == true &&
  principal.consent_graph.minor_or_proxy_status_resolved == true &&
  resource.tenant_class == imaging::TenantClass::"PAID"
};

permit (
  principal is imaging::PortalPatient,
  action == imaging::Action::"PortalDownloadDICOM",
  resource is imaging::Study
) when {
  principal.patient_id == resource.patient_id &&
  principal.consent_graph.affirmative_consent_for_dicom_download == true &&
  resource.tenant_class == imaging::TenantClass::"PAID"
};

permit (
  principal is imaging::PortalPatient,
  action == imaging::Action::"PortalShareWithExternal",
  resource is imaging::Study
) when {
  principal.patient_id == resource.patient_id &&
  principal.consent_graph.affirmative_consent_for_external_share == true &&
  principal.share_target_identified == true &&
  resource.tenant_class == imaging::TenantClass::"PAID"
};

forbid (
  principal is imaging::PortalPatient,
  action,
  resource is imaging::Study
) when {
  principal.patient_id != resource.patient_id
};

// Forbid patient access for demo_trial tenants (synthetic only; no real
// patient portal experience).
forbid (
  principal is imaging::PortalPatient,
  action,
  resource is imaging::Study
) when {
  resource.tenant_class == imaging::TenantClass::"DEMO_TRIAL"
};

// Token TTL must be bounded; never permit infinite tokens.
forbid (
  principal is imaging::PortalPatient,
  action,
  resource is imaging::PortalToken
) when {
  resource.ttl_seconds > 604800 // 7 days max
};
