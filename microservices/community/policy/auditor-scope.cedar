// community µservice — auditor-scope policy fragment
// Scope: per-tenant auditor (SOC 2 / ISO / GDPR DPA / KR PIPC).
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)
// Read-only; bound to a single tenant_id; cannot mutate.

forbid (
  principal,
  action,
  resource
);

// Auditor reads audit log + moderation log + KB articles + announcements for own tenant
permit (
  principal in Auditor::?p,
  action in [
    Action::"read_audit_log",
    Action::"read_moderation_action",
    Action::"read_post",
    Action::"read_kb_article",
    Action::"read_revision_history",
    Action::"read_dsr_completion_attestation",
    Action::"read_retention_policy"
  ],
  resource in CommunityResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has auditor_scope_expires_at &&
  context has now &&
  context.now < principal.auditor_scope_expires_at
};

// Auditor may read aggregated metrics but not raw IPs / user-agents
permit (
  principal in Auditor::?p,
  action == Action::"read_aggregated_metric",
  resource in CommunityResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// Auditor never sees raw IP or user-agent
forbid (
  principal in Auditor::?p,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "ABUSE_DETECTION_RAW"
};

// Auditor never mutates
forbid (
  principal in Auditor::?p,
  action in [
    Action::"create_post",
    Action::"edit_own_post",
    Action::"delete_post",
    Action::"hide_post",
    Action::"cast_vote",
    Action::"raise_flag",
    Action::"resolve_flag"
  ],
  resource
);
