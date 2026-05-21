// Cedar v4.2 LTS — Notes HIPAA clinical-notes overlay
// Authority: ADR-0251 compliance pack primitive (HIPAA)

forbid (
  principal,
  action == Action::"NotesShareLinkIssue",
  resource
) when {
  // Clinical notes cannot be shared via public share-link
  resource.tenant.audience_type == "B2B_HIPAA_CLINICAL"
  && action has share_audience
  && action.share_audience == "PUBLIC_SHARE_VIEW"
};

forbid (
  principal,
  action == Action::"NotesIntelligenceCall",
  resource
) when {
  // Clinical notes intelligence routes through HIPAA-conformant variants only
  resource.tenant.audience_type == "B2B_HIPAA_CLINICAL"
  && !(action has intelligence_variant && action.intelligence_variant == "HIPAA_CONFORMANT")
};

forbid (
  principal,
  action == Action::"NotesExportRequest",
  resource
) when {
  resource.tenant.audience_type == "B2B_HIPAA_CLINICAL"
  && !(principal has tenant_role && principal.tenant_role == "TENANT_ADMIN")
  && !(principal has phi_officer_role && principal.phi_officer_role == true)
};

permit (
  principal,
  action == Action::"NotesPhiClassifyNote",
  resource
) when {
  // PHI classifier runs on every clinical-tier note on save
  resource.tenant.audience_type == "B2B_HIPAA_CLINICAL"
};
