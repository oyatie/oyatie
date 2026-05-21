// cloud-billing — demo_trial gates
//
// Per ADR-0330 §B.3.6, §B.3.7, §B.3.8 demo_trial tenants are denied:
//   - compliance pack activation
//   - BYOK opt-in (LLM / payment / KMS providers)
//   - marketplace listing publishing
//
// Per §B.3.4 demo_trial tenants have usage caps and time caps.

// ---------- Compliance pack activation ----------

@id("cap.cloud.billing.demo_trial.deny_compliance_pack_activation")
forbid (
  principal,
  action == Action::"activate_compliance_pack",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

@id("cap.cloud.billing.paid.permit_compliance_pack_activation")
permit (
  principal,
  action == Action::"activate_compliance_pack",
  resource
) when {
  principal.tenant_class == "paid" &&
  principal.has_role("tenant-compliance-admin") &&
  context.pack_id in [
    "soc2-type-ii",
    "soc1",
    "iso-27001",
    "gdpr",
    "pci-dss-v4",
    "eu-ai-act",
    "csap-kr",
    "k-fsi",
    "mas-trm",
    "sox-404",
    "fedramp-high",
    "hipaa-2024"
  ]
};

// ---------- BYOK opt-in ----------

@id("cap.cloud.billing.demo_trial.deny_byok_opt_in")
forbid (
  principal,
  action == Action::"configure_byok_provider",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

@id("cap.cloud.billing.paid.permit_byok_opt_in")
permit (
  principal,
  action == Action::"configure_byok_provider",
  resource
) when {
  principal.tenant_class == "paid" &&
  principal.has_role("tenant-admin") &&
  context.provider_kind in ["llm", "payment", "kms", "identity"] &&
  context.byok_mode in ["byok", "byok_required_by_pack"]
};

// ---------- Marketplace listing ----------

@id("cap.cloud.billing.demo_trial.deny_marketplace_listing")
forbid (
  principal,
  action == Action::"publish_marketplace_listing",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

@id("cap.cloud.billing.demo_trial.deny_paid_listing_purchase")
forbid (
  principal,
  action == Action::"purchase_marketplace_listing",
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  resource.listing_class != "free"
};

@id("cap.cloud.billing.demo_trial.permit_free_listing_consumption")
permit (
  principal,
  action == Action::"purchase_marketplace_listing",
  resource
) when {
  resource.listing_class == "free"
};

// ---------- Usage caps + grace ----------

@id("cap.cloud.billing.demo_trial.permit_within_caps")
permit (
  principal,
  action == Action::"emit_usage_event",
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.cap_breached == false
};

@id("cap.cloud.billing.demo_trial.deny_write_during_grace")
forbid (
  principal,
  action in [
    Action::"create",
    Action::"update",
    Action::"delete",
    Action::"emit_usage_event"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.cap_breached == true
};

@id("cap.cloud.billing.demo_trial.permit_read_during_grace")
permit (
  principal,
  action in [
    Action::"read",
    Action::"read_invoice",
    Action::"read_tenant_class"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.tenant_id == resource.tenant_id
};

// ---------- Conversion path remains open during grace ----------

@id("cap.cloud.billing.demo_trial.permit_conversion_during_grace")
permit (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.tenant_id == resource.tenant_id &&
  context.target_tenant_class == "paid" &&
  context.contract_id != ""
};

// ---------- Suspension state ----------

@id("cap.cloud.billing.suspended.deny_all_writes")
forbid (
  principal,
  action,
  resource
) when {
  resource has tenant_state &&
  resource.tenant_state == "suspended"
};

@id("cap.cloud.billing.suspended.permit_read_during_retention")
permit (
  principal,
  action in [Action::"read", Action::"read_invoice", Action::"read_tenant_class"],
  resource
) when {
  resource has tenant_state &&
  resource.tenant_state == "suspended" &&
  context.within_retention_window == true
};
