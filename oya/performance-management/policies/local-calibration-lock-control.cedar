// Local Performance Management policy: calibration-lock-control
// Domain: review, calibration, rating, feedback, goal, cycle
// ADR bindings: ADR-0130, ADR-0131, ADR-0139
// Default outcome: deny unless tenant, purpose, data class, and domain guard match.
forbid(principal, action, resource) unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has tenant_id &&
  context.tenant_id == principal.tenant_id &&
  context has principal_id &&
  context has purpose &&
  context has data_class &&
  context.data_class in ["review_form", "calibration_rating", "feedback_entry", "goal_alignment", "hr_export", "manager_note"] &&
  context has service &&
  context.service == "performance-management" &&
  context has audit_event_class
};

permit(principal, action == Action::"feedback.submit", resource) when {
  principal has role &&
  principal.role in ["owner", "operator", "service-account"] &&
  resource has domain &&
  resource.domain == "PerformanceReviewCycle" &&
  context.data_class == "calibration_rating" &&
  context.purpose in ["operate", "support", "security-review", "regulated-audit"] &&
  context.policy_evaluation_mode == "caller_side_library_first"
};

forbid(principal, action, resource) when {
  context has requested_data_class &&
  context.requested_data_class != context.data_class
};

forbid(principal, action, resource) when {
  context has regional_pack &&
  context.regional_pack == "hipaa" &&
  !(context.purpose in ["support", "security-review", "regulated-audit"])
};

permit(principal, action, resource) when {
  action == Action::"feedback.submit.breakglass" &&
  context has breakglass_ticket_id &&
  context has approval_chain &&
  context.approval_chain == "security-plus-service-owner" &&
  context.audit_event_class == "PerformanceManagementCalibrationLockControlBreakglass"
};
