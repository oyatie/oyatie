// plugin-app-store pack-us-public-sector policy overlay
// Cedar v4 fragment applied AFTER canonical microservices/plugin-app-store/policy/tenant-scope.cedar

// pack-us-public-sector specific constraints
permit (
  principal in TenantOperator::?p,
  action in [Action::"install_plugin", Action::"download_sdk"],
  resource in TenantData::?r
)
when {
  principal has regulatory_pack &&
  principal.regulatory_pack == "us-public-sector" &&
  resource has pack_compatibility &&
  "us-public-sector" in resource.pack_compatibility
};
