// plugin-app-store µservice — tenant-scope policy
// Cedar v4 schema; conforms to microservices/plugin-app-store/policy/schema.cedarschema
// Purpose: enforce per-tenant scope on plugin browse/install/uninstall/configure surfaces.
// Owner: ops-security + axis-ecosystem
// Audit: every deny emits oya_plugin_app_store_unauthorized_attempt_total

forbid (principal, action, resource);

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"list_plugins",
    Action::"get_plugin",
    Action::"search_plugins",
    Action::"list_installations",
    Action::"get_installation",
    Action::"view_audit_trail",
    Action::"list_subscriptions"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"install_plugin",
    Action::"uninstall_plugin",
    Action::"grant_capability",
    Action::"revoke_capability",
    Action::"change_billing_components"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has tenant_class &&
  principal.tenant_class == "paid" &&
  principal has billing_components &&
  principal.billing_components.contains("per_seat") &&
  resource has trust_verdict &&
  resource.trust_verdict in [TrustVerdict::"reviewed", TrustVerdict::"verified", TrustVerdict::"attested", TrustVerdict::"pack_bound"]
};

permit (
  principal in TenantAdmin::?p,
  action in [
    Action::"set_install_policy",
    Action::"override_rate_limit",
    Action::"cap_plugin_spend",
    Action::"revoke_installation"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};
