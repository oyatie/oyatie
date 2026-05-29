// connector service — auditor-scope policy fragment
// Cedar v4.2 LTS
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA) read
//          time-boxed, tenant-scoped subsets of connector data during engagement.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md cross-cutting; ADR-0263
// soak-window-seconds: 60
// publish-stage: active

forbid (principal, action, resource);

// =============================================================================
// PERMIT 1 — auditor reads tenant-scoped data within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_audit_chain",
    Action::"read_oauth_grant_metadata",
    Action::"read_webhook_metadata",
    Action::"read_dlq_metadata",
    Action::"read_dashboards",
    Action::"read_runbooks",
    Action::"read_cedar_fragments"
  ],
  resource
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// PERMIT 2 — auditor reads pack-level policy artifacts (no tenant scope)
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"read_artifact",
  resource in PolicyArtifact::?r
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has artifact_type &&
  resource.artifact_type in [
    "threat-model",
    "dpia",
    "compliance",
    "data-residency",
    "policy-cedar",
    "runbook",
    "openapi",
    "asyncapi",
    "proto"
  ]
};

// =============================================================================
// FORBID — auditor cannot write or rotate
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"oauth_grant_revoke",
    Action::"webhook_endpoint_register",
    Action::"connector_invoke",
    Action::"rotate_secret",
    Action::"dlq_replay"
  ],
  resource
);

// =============================================================================
// FORBID — auditor outside engagement window
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
unless {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// FORBID — auditor cannot read raw credentials
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
