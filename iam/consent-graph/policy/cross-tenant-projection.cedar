// Cross-tenant projection enforcement policy
// Compiled per-agreement at acceptance; cached by enforcement-adapter-cedar.
// Authority: ADR-0214 §2.3, IP-005.

// 1. Base permit: grantee subscribes/reads under matching agreement
permit (
  principal in Tenant::"{{grantee_tenant_id}}",
  action in [Action::"project.subscribe", Action::"project.read"],
  resource is Resource
)
when {
  resource.entity_type == "{{entity_type}}" &&
  resource.grantor_tenant_id == "{{grantor_tenant_id}}" &&
  context.purpose_of_use == "{{purpose_of_use}}" &&
  context.request_time <= datetime("{{expiration_iso}}") &&
  context has "agreement_id" &&
  context.agreement_id == "{{agreement_id}}"
};

// 2. Field-level scope (allow-list path)
forbid (
  principal,
  action in [Action::"project.subscribe", Action::"project.read"],
  resource is Resource
)
when {
  resource has "field_set" &&
  !(resource.field_set in [{{allow_fields_list}}])
};

// 3. Sovereignty forbid: grantee not in permitted regions
forbid (
  principal in Tenant::"{{grantee_tenant_id}}",
  action,
  resource is Resource
)
when {
  context has "request_region" &&
  !(context.request_region in [{{permitted_grantee_regions_list}}])
};

// 4. Capability tier ceiling
forbid (
  principal in Tenant::"{{grantee_tenant_id}}",
  action,
  resource is Resource
)
when {
  context has "tenant_class" &&
  context.tenant_class > "{{max_tenant_class}}"
};

// 5. Stale revocation check — fail-closed if freshness budget breached
forbid (
  principal,
  action,
  resource
)
when {
  context has "prior_revocation_check_ms_ago" &&
  context.prior_revocation_check_ms_ago > 200
};

// 6. Rate-limit forbid (Cedar evaluates max_qps; actual rate-limit enforced at gateway,
//    but Cedar denies if the context says we've been over for >1min)
forbid (
  principal,
  action,
  resource
)
when {
  context has "tenant_max_qps_breach" &&
  context.tenant_max_qps_breach == true
};

// 7. Custom predicate (parsed by agreement-domain; rendered here)
// {{custom_predicate_clause}}

// 8. Partner-suspended guard
forbid (
  principal in Tenant::"{{grantee_tenant_id}}",
  action,
  resource
)
when {
  context has "partner_state" &&
  context.partner_state == "Suspended"
};
