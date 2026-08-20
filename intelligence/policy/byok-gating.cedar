// intelligence µservice — BYOK-gating policy fragment
// Cedar v4
//
// Purpose: enforce provider BYOK for B2B/regulated tenants; allow platform-default only when
//          tenant + audience explicitly opt in (B2C consumer).
// Owner:   ops-security + council-privacy + axis-intelligence
// References: ADR-0255 §D-4 (BYOK); ARCHITECTURE.md §4 (sidecar credential-handle).

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — tenant BYOK
// =============================================================================
//
// A tenant with `openbao_secret_path_for_provider` set in its tenant config may resolve
// provider credentials via SecretReference kind=openbao_path.

permit (
  principal in TenantPrincipal::?p,
  action == Action::"resolve_credential",
  resource in SecretReference::?s
)
when {
  resource has kind &&
  resource.kind == "openbao_path" &&
  resource has value &&
  resource.value like "${openbao:secret/*}" &&
  principal has tenant_id &&
  resource has bound_tenant &&
  resource.bound_tenant == principal.tenant_id
};

// =============================================================================
// PERMIT — platform-default for audience=consumer
// =============================================================================
//
// Consumer-tier callers using platform-default credentials are permitted; the consumer-daily-cap
// gate in dispatch-authorization.cedar enforces budget.

permit (
  principal in ConsumerEndUser::?u,
  action == Action::"resolve_credential",
  resource in SecretReference::?s
)
when {
  resource has kind &&
  resource.kind == "platform_default" &&
  principal has tenant_audience &&
  principal.tenant_audience == "consumer"
};

// =============================================================================
// PERMIT — platform-default for audience=internal-foundry
// =============================================================================

permit (
  principal in FoundryAgent::?a,
  action == Action::"resolve_credential",
  resource in SecretReference::?s
)
when {
  resource has kind &&
  resource.kind == "platform_default" &&
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie/foundry/*"
};

// =============================================================================
// FORBID — B2B + regulated tenant with platform-default
// =============================================================================
//
// B2B + regulated tenants (HIPAA / FedRAMP / KR-CSAP / sensitive-data tenants) MUST use BYOK.

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"resolve_credential",
  resource in SecretReference::?s
)
when {
  principal has tenant_scope_pack &&
  principal.tenant_scope_pack in ["pack-us-healthcare", "pack-us-federal", "pack-kr-csap"] &&
  resource has kind &&
  resource.kind == "platform_default"
};

// =============================================================================
// FORBID — SecretReference bound to a different tenant
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"resolve_credential",
  resource in SecretReference::?s
)
when {
  principal has tenant_id &&
  resource has bound_tenant &&
  resource.bound_tenant != principal.tenant_id
};

// =============================================================================
// FORBID — CredentialHandle expired
// =============================================================================

forbid (
  principal,
  action == Action::"use_credential_handle",
  resource in CredentialHandle::?h
)
when {
  resource has expires_at &&
  context has now &&
  resource.expires_at <= context.now
};
