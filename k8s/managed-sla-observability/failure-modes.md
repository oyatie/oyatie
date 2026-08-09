# Managed K8s Tenant Quota — Failure Modes

## Quota Store Unavailable
- **Impact**: `check_quota` returns `QuotaPortError::Persistence`; cluster-lifecycle
  must treat this as a hard deny (fail-closed).
- **Recovery**: retry with backoff; alert on persistent failure.

## Unknown Tenant
- **Impact**: `check_quota` returns `QuotaPortError::NotFound`; HTTP 404.
- **Recovery**: tenant-admin or platform-operator must set quota before provisioning.

## Cedar Policy Parse Failure
- **Impact**: `QuotaRbacAuthorizer::new_with_default_policies()` returns `Err`;
  service fails to start (fail-closed at boot).
- **Recovery**: fix policy text; redeploy.

## Tenant ID Mismatch
- **Impact**: `evaluate()` returns `Deny(TenantMismatch)`; provisioning blocked.
- **Recovery**: caller must pass consistent tenant IDs.
