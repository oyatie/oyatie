// Cedar policy: HIPAA pack default-deny on PHI access.
//
// Authority: ADR-0243 + ADR-0251 HIPAA pack + HIPAA 45 CFR 164.312.
//
// Default posture: DENY on any PHI-resource action unless an explicit
// permit policy applies. Cedar's default-deny semantics combined with this
// explicit forbid serve as a belt-and-suspenders enforcement.

forbid (
  principal,
  action,
  resource is imaging::Study
) when {
  resource.contains_phi == true &&
  !context.cedar_explicit_permit_evaluated == true
};

forbid (
  principal,
  action,
  resource is imaging::Instance
) when {
  resource.contains_phi == true &&
  !context.cedar_explicit_permit_evaluated == true
};

forbid (
  principal,
  action,
  resource is imaging::StructuredReport
) when {
  resource.contains_phi == true &&
  !context.cedar_explicit_permit_evaluated == true
};

// HIPAA minimum-necessary: forbid full-study download when only metadata is
// required by the role (e.g., billing clerk does not need pixel data).
forbid (
  principal,
  action == imaging::Action::"WADORetrieve",
  resource is imaging::Study
) when {
  principal.role in [imaging::Role::"billing-clerk", imaging::Role::"scheduler", imaging::Role::"front-desk"] &&
  resource.contains_phi == true
};

// Forbid PHI access from non-tenant-bound principals (cross-tenant
// reinforcement).
forbid (
  principal,
  action,
  resource is imaging::Study
) when {
  resource.contains_phi == true &&
  principal.tenant_id != resource.tenant_id
};

// Forbid PHI access without active BAA (Business Associate Agreement) when
// the principal is a business associate.
forbid (
  principal is imaging::BusinessAssociate,
  action,
  resource is imaging::Study
) when {
  resource.contains_phi == true &&
  principal.baa_status != "active"
};
