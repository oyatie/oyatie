// connector service — webhook-receiver-gating policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate webhook endpoint registration; enforce signature verification;
//          gate replay-window; gate per-tenant DNS routing.
// Owner:   axis-integration + ops-security
// References: ARCHITECTURE.md §abuse-defence; threat-model.md T-S-03/T-T-03
// soak-window-seconds: 60
// publish-stage: active

// =============================================================================
// Default deny
// =============================================================================

forbid (principal, action, resource);

// =============================================================================
// PERMIT 1 — TIE registers webhook endpoint for their tenant
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action == Action::"webhook_endpoint_register",
  resource in WebhookEndpoint::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  principal has role &&
  principal.role in ["tenant_integration_engineer", "tenant_admin"]
};

// =============================================================================
// PERMIT 2 — anonymous inbound vendor webhook (post-signature-verify)
// =============================================================================

permit (
  principal == Principal::"anonymous_vendor_webhook",
  action == Action::"webhook_payload_receive",
  resource in WebhookEndpoint::?e
)
when {
  context has signature_verified &&
  context.signature_verified == true &&
  context has timestamp_age_seconds &&
  context.timestamp_age_seconds <= 300 &&
  context has idempotency_key_unseen &&
  context.idempotency_key_unseen == true
};

// =============================================================================
// FORBID — receive without HMAC verify
// =============================================================================

forbid (
  principal,
  action == Action::"webhook_payload_receive",
  resource
)
when {
  context has signature_verified &&
  context.signature_verified != true
};

// =============================================================================
// FORBID — replay (timestamp > 5min OR idempotency-key seen)
// =============================================================================

forbid (
  principal,
  action == Action::"webhook_payload_receive",
  resource
)
when {
  context has timestamp_age_seconds &&
  context.timestamp_age_seconds > 300
};

forbid (
  principal,
  action == Action::"webhook_payload_receive",
  resource
)
when {
  context has idempotency_key_unseen &&
  context.idempotency_key_unseen != true
};
