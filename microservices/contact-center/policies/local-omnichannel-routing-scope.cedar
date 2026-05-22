// Local Contact Center policy: omnichannel-routing-scope
// Domain: omnichannel, routing, call, transfer, queue, recording
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
  context.data_class in ["conversation_record", "call_recording", "routing_state", "transfer_event", "agent_presence", "callback_request"] &&
  context has service &&
  context.service == "contact-center" &&
  context has audit_event_class
};

permit(principal, action == Action::"omnichannel.route.assign", resource) when {
  principal has role &&
  principal.role in ["owner", "operator", "service-account"] &&
  resource has domain &&
  resource.domain == "OmnichannelInteraction" &&
  context.data_class == "conversation_record" &&
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
  action == Action::"omnichannel.route.assign.breakglass" &&
  context has breakglass_ticket_id &&
  context has approval_chain &&
  context.approval_chain == "security-plus-service-owner" &&
  context.audit_event_class == "ContactCenterOmnichannelRoutingScopeBreakglass"
};
