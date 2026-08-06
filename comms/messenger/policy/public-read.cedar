// messenger µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access. The messenger surface
//          generally has NO anonymous reads — channels are members-only by
//          construction. This fragment only allows:
//            (a) public health / status probes
//            (b) public OAuth callback endpoints for personal-context onboarding
//            (c) public OpenAPI / AsyncAPI schema endpoints (no data)
// Owner:   ops-security + gtm-marketing
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — anonymous read of public health / status indicators
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_health",
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind in ["liveness", "readiness", "public-status"]
};

// =============================================================================
// PERMIT 2 — anonymous read of API schema artifacts
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_artifact",
  resource in PublicEndpoint::?e
)
when {
  resource has endpoint_kind &&
  resource.endpoint_kind in ["openapi-schema", "asyncapi-schema", "proto-descriptor"]
};

// =============================================================================
// PERMIT 3 — anonymous OAuth callback for personal-onboarding (handshake only)
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"oauth_callback",
  resource == PublicEndpoint::"oauth-personal-onboard"
);

// =============================================================================
// FORBID — anonymous cannot read any message / channel / thread / attachment
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_message",
    Action::"read_thread",
    Action::"read_channel",
    Action::"read_channel_metadata",
    Action::"list_channels",
    Action::"read_attachment",
    Action::"read_attachment_blob",
    Action::"read_presence",
    Action::"read_audit_chain",
    Action::"read_disclosure_history"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot write anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"post_message",
    Action::"create_channel",
    Action::"delete_channel",
    Action::"upload_attachment",
    Action::"add_channel_member",
    Action::"remove_channel_member",
    Action::"open_ediscovery_hold",
    Action::"disclose_message_body",
    Action::"update_own_presence"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot access non-public data classes
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class == "PUBLIC")
};
