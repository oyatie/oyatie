// Local Performance Management policy: engagement-pulse-anonymity
// Closes audit Finding 1.7.A (P0): IP-029 names the gate but no enforceable Cedar policy
// existed prior to this remediation wave.
// Domain: engagement-pulse, eNPS, sentiment, anonymity
// ADR bindings: ADR-0130, ADR-0131, ADR-0139, ADR-0243, ADR-0263, ADR-0321, ADR-0328, ADR-0331
// Default outcome: deny unless tenant + cohort + minimum-floor + tenant-class all match.
// Anonymity floor (k): default 8 respondents per slice; pack `eu-worker-council` raises to 12;
//                       pack `kr-pipa` raises to 10; tenant_class=demo_trial enforces synthetic-only.

forbid(principal, action, resource) unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has tenant_id &&
  context.tenant_id == principal.tenant_id &&
  context has principal_id &&
  context has purpose &&
  context has data_class &&
  context.data_class in ["engagement_pulse", "engagement_signal", "engagement_aggregate", "engagement_release"] &&
  context has service &&
  context.service == "performance-management" &&
  context has audit_event_class &&
  context has tenant_class
};

// Default permit: HRBP releases aggregate only when cohort >= floor and no dimension is below floor.
permit(principal, action == Action::"engagementPulse.releaseSummary", resource) when {
  principal has role &&
  principal.role in ["hrbp", "owner", "operator"] &&
  resource has cohort_size &&
  resource has minimum_cohort_size &&
  resource.cohort_size >= resource.minimum_cohort_size &&
  context has aggregate_only &&
  context.aggregate_only == true &&
  context.purpose in ["operate", "support", "regulated-audit"] &&
  context.tenant_class == "paid"
};

// Permit for demo_trial only with synthetic-only data path.
permit(principal, action == Action::"engagementPulse.releaseSummary", resource) when {
  principal has role &&
  principal.role in ["hrbp", "owner", "operator"] &&
  context.tenant_class == "demo_trial" &&
  context has data_origin &&
  context.data_origin == "synthetic" &&
  context.purpose in ["operate", "demo"]
};

// Floor breach: if cohort_size < minimum_cohort_size, block and emit hold event.
forbid(principal, action == Action::"engagementPulse.releaseSummary", resource) when {
  resource has cohort_size &&
  resource has minimum_cohort_size &&
  resource.cohort_size < resource.minimum_cohort_size
};

// Pack overlays raise the floor.
forbid(principal, action == Action::"engagementPulse.releaseSummary", resource) when {
  context has regional_pack &&
  context.regional_pack == "eu-worker-council" &&
  resource has cohort_size &&
  resource.cohort_size < 12
};

forbid(principal, action == Action::"engagementPulse.releaseSummary", resource) when {
  context has regional_pack &&
  context.regional_pack == "kr-pipa" &&
  resource has cohort_size &&
  resource.cohort_size < 10
};

// Dimension-level floor: any requested_dimensions slice with count < floor must be suppressed
// (not released). The application layer enforces suppression; this policy blocks attempts to
// override the suppression flag.
forbid(principal, action, resource) when {
  context has bypass_dimension_floor &&
  context.bypass_dimension_floor == true
};

// Row-level read of individual responses is always denied (even for HRBP).
forbid(principal, action == Action::"engagementPulse.readResponseRow", resource);

// Auditor read is permitted only on aggregate-only evidence and only with ticket_id.
permit(principal, action == Action::"engagementPulse.readEvidence", resource) when {
  principal has role &&
  principal.role == "auditor" &&
  context has aggregate_only &&
  context.aggregate_only == true &&
  context has ticket_id &&
  context.purpose == "regulated-audit"
};

// Cross-tenant access is always forbidden (defense in depth on top of base tenant scoping).
forbid(principal, action, resource) when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// Demo-trial export of any engagement evidence is denied (synthetic data may stay local only).
forbid(principal, action, resource) when {
  context has tenant_class &&
  context.tenant_class == "demo_trial" &&
  action in [
    Action::"engagementPulse.export",
    Action::"engagementPulse.publishToAnalytics",
    Action::"engagementPulse.shareExternal"
  ]
};

// HIPAA pack: workforce-health engagement data carries tighter floor of 15.
forbid(principal, action == Action::"engagementPulse.releaseSummary", resource) when {
  context has regional_pack &&
  context.regional_pack == "hipaa" &&
  resource has cohort_size &&
  resource.cohort_size < 15
};
