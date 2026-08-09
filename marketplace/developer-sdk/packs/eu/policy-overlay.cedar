// developer-sdk pack-eu policy overlay
// Cedar v4 fragment applied AFTER canonical microservices/developer-sdk/policy/tenant-scope.cedar

// pack-eu specific constraints
permit (
  principal in TenantOperator::?p,
  action in [Action::"install_plugin", Action::"download_sdk"],
  resource in TenantData::?r
)
when {
  principal has regulatory_pack &&
  principal.regulatory_pack == "eu" &&
  resource has pack_compatibility &&
  "eu" in resource.pack_compatibility
};
