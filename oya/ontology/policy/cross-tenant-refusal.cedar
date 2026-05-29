// Cedar v4.2 LTS — Ontology cross-tenant refusal
// Authority: docs/decisions/ADR-0257-ontology-as-substrate.md (amendment)
// Authority: ADR-0244 tenant-as-universal-scoping-primitive
// Default-deny baseline for cross-tenant projection — only ontology_share_token surface permits it.

forbid (
  principal,
  action,
  resource
) when {
  principal has tenant
  && resource has tenant
  && principal.tenant.tenant_id != resource.tenant.tenant_id
  && !(principal has share_token_grant
        && principal.share_token_grant.resource_tenant_id == resource.tenant.tenant_id
        && principal.share_token_grant.is_active == true
        && principal.share_token_grant.action_scope.contains(action))
};

// Share-token issuance requires both tenant-admin auth AND audit-emission.
forbid (
  principal,
  action == Action::"OntologyShareTokenIssue",
  resource
) when {
  !(principal has tenant_role && principal.tenant_role == "TENANT_ADMIN")
};

forbid (
  principal,
  action == Action::"OntologyShareTokenIssue",
  resource
) when {
  // Share-token TTL capped at 90 days; longer requires governance approval
  action has requested_ttl_seconds
  && action.requested_ttl_seconds > 7776000
};
