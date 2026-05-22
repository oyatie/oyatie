// messenger µservice — tenant_class policy fragment
// Authority: ADR-0329 tier retirement + ADR-0330 tenant_class model +
// ADR-0331 per-µservice adoption template.
//
// Purpose: bind messenger authorization to cloud-iam principal claims:
// principal.tenant_class, principal.billing_components, principal.cap_breached,
// and principal.demo_trial_expires_at. Messenger does not resolve tenant_class
// locally; it trusts the cloud-iam / identity session claim at the request edge.

// Eligible tenant classes for messenger are exactly demo_trial and paid.
forbid (
  principal,
  action,
  resource
)
unless {
  principal has tenant_class &&
  (principal.tenant_class == "demo_trial" || principal.tenant_class == "paid")
};

// demo_trial tenants receive the same capability surface, but compliance-pack
// activation and enterprise escrow/BYOK surfaces are structurally paid-only.
forbid (
  principal,
  action in [
    Action::"activate_compliance_pack",
    Action::"enable_hsm_key_escrow",
    Action::"enable_work_mode_mls_recovery_escrow",
    Action::"bind_tenant_byok",
    Action::"pair_slack_connect_workspace",
    Action::"set_retention_above_demo_cap"
  ],
  resource
)
when {
  principal.tenant_class == "demo_trial"
};

// Paid-only compliance-pack activation requires the paid class plus per-seat
// billing because packs bind to tenant seats, eDiscovery/legal-hold scope, and
// work-mode MLS recovery entitlements.
permit (
  principal,
  action in [
    Action::"activate_compliance_pack",
    Action::"enable_hsm_key_escrow",
    Action::"enable_work_mode_mls_recovery_escrow",
    Action::"bind_tenant_byok"
  ],
  resource
)
when {
  principal.tenant_class == "paid" &&
  principal has billing_components &&
  principal.billing_components.contains("per_seat")
};

// Paid tenants using metered messenger operations must carry per_usage. The
// corresponding meters are emitted to cloud-billing for message volume, active
// channels, huddle minutes, attachment bytes, MLS KeyPackage uploads, search,
// mention fanout, and workflow triggers.
forbid (
  principal,
  action in [
    Action::"post_message",
    Action::"create_channel",
    Action::"start_huddle",
    Action::"upload_attachment",
    Action::"publish_mls_key_package",
    Action::"search_messages",
    Action::"fanout_mention",
    Action::"trigger_workflow_from_message"
  ],
  resource
)
when {
  principal.tenant_class == "paid" &&
  (
    !(principal has billing_components) ||
    !(principal.billing_components.contains("per_usage"))
  )
};

// Demo trial hard caps are resolved into context by the messenger gateway. Read
// paths remain available; write-heavy paths fail closed when a cap is breached.
forbid (
  principal,
  action in [
    Action::"post_message",
    Action::"create_channel",
    Action::"start_huddle",
    Action::"upload_attachment",
    Action::"set_retention_policy"
  ],
  resource
)
when {
  principal.tenant_class == "demo_trial" &&
  (
    (context has monthly_message_count && context.monthly_message_count >= 50000) ||
    (context has active_channel_count && context.active_channel_count >= 10) ||
    (context has requested_retention_days && context.requested_retention_days > 30) ||
    (context has monthly_huddle_minutes && context.monthly_huddle_minutes >= 120) ||
    (context has attachment_storage_gb && context.attachment_storage_gb >= 2) ||
    (principal has cap_breached && principal.cap_breached == true)
  )
};

// Demo trial expiry blocks new write-heavy activity after the grace window.
forbid (
  principal,
  action in [
    Action::"post_message",
    Action::"create_channel",
    Action::"start_huddle",
    Action::"upload_attachment"
  ],
  resource
)
when {
  principal.tenant_class == "demo_trial" &&
  principal has demo_trial_expires_at &&
  context has request_time &&
  context.request_time > principal.demo_trial_expires_at &&
  (
    !(principal has demo_trial_grace_expires_at) ||
    context.request_time > principal.demo_trial_grace_expires_at
  )
};
