// plugin-app-store pack-us-healthcare policy overlay
// Cedar v4 fragment applied AFTER canonical microservices/plugin-app-store/policy/tenant-scope.cedar

// pack-us-healthcare specific constraints
permit (
  principal in TenantOperator::?p,
  action in [Action::"install_plugin", Action::"download_sdk"],
  resource in TenantData::?r
)
when {
  principal has regulatory_pack &&
  principal.regulatory_pack == "us-healthcare" &&
  resource has pack_compatibility &&
  "us-healthcare" in resource.pack_compatibility
};
