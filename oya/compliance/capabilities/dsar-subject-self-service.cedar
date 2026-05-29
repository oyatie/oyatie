// Cedar capability fragment per ADR-0183.
// Subject can open DSAR for their own pseudonymized identity within their tenant.

permit (
  principal is VerifiedSubject,
  action in [
    Action::"dsar-export",
    Action::"dsar-delete",
    Action::"dsar-rectify"
  ],
  resource is DsarRequest
) when {
  resource.tenant_id == principal.tenant_id &&
  resource.subject_id_pseudonym == principal.subject_id_pseudonym
};

// Subject cannot open DSAR for another subject — even in the same tenant.
forbid (
  principal is VerifiedSubject,
  action in [
    Action::"dsar-export",
    Action::"dsar-delete",
    Action::"dsar-rectify"
  ],
  resource is DsarRequest
) unless {
  resource.subject_id_pseudonym == principal.subject_id_pseudonym
};
