// plugin-app-store pack-kr policy overlay
// Cedar v4 fragment applied AFTER canonical marketplace/plugin-app-store/policy/tenant-scope.cedar

// pack-kr specific constraints
permit (
  principal in TenantOperator::?p,
  action in [Action::"install_plugin", Action::"download_sdk"],
  resource in TenantData::?r
)
when {
  principal has regulatory_pack &&
  principal.regulatory_pack == "kr" &&
  resource has pack_compatibility &&
  "kr" in resource.pack_compatibility
};
