use super::*;

/// Map an sqlx error to the kernel's write-store error vocabulary.
pub(super) fn store_unavailable(error: sqlx::Error) -> ScimStoreError {
    ScimStoreError::Unavailable {
        detail: error.to_string(),
    }
}

pub(super) fn user_write_error(error: sqlx::Error) -> ScimStoreError {
    if let sqlx::Error::Database(database) = &error
        && database.code().as_deref() == Some("23505")
    {
        return ScimStoreError::Conflict;
    }
    store_unavailable(error)
}

pub(super) fn corrupt(detail: impl Into<String>) -> ScimStoreError {
    ScimStoreError::Corrupt {
        detail: detail.into(),
    }
}

/// Reject an empty / whitespace-only tenant scope at the adapter boundary BEFORE
/// it is bound into the per-transaction tenant GUC. An empty GUC makes the
/// migration's restrictive `require_tenant_guc` policy deny every row (a silent
/// deny-all), so a blank tenant is surfaced as a contract violation rather than
/// binding `SET set_config('oyatie.tenant_id', '', true)`.
pub(super) fn validate_tenant(tenant: &TenantId) -> Result<(), ScimStoreError> {
    if tenant.0.trim().is_empty() {
        return Err(corrupt("tenant scope is empty"));
    }
    Ok(())
}
