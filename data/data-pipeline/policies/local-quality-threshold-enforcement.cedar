// Local Data Pipeline policy: quality-threshold-enforcement
// Domain: ingest, transform, quality, freshness, null-rate, lineage
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
  context.data_class in ["ingest_batch", "transform_run", "quality_check", "null_rate_metric", "deadletter_event", "lineage_edge"] &&
  context has service &&
  context.service == "data-pipeline" &&
  context has audit_event_class
};

permit(principal, action == Action::"quality.check.evaluate", resource) when {
  principal has role &&
  principal.role in ["owner", "operator", "service-account"] &&
  resource has domain &&
  resource.domain == "PipelineRun" &&
  context.data_class == "quality_check" &&
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
  action == Action::"quality.check.evaluate.breakglass" &&
  context has breakglass_ticket_id &&
  context has approval_chain &&
  context.approval_chain == "security-plus-service-owner" &&
  context.audit_event_class == "DataPipelineQualityThresholdEnforcementBreakglass"
};
