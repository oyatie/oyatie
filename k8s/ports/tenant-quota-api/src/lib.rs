//! QuotaDecision PORT + DTOs for managed-K8s tenant quota.
//!
//! This crate defines the **port** (trait) that cluster-lifecycle calls before
//! provisioning any cluster. It is the stable seam between the quota service
//! and its callers; the implementation lives in `adapter-cedar` / `adapter-inmemory`.
//!
//! ## Design (ADR-0376 / ADR-0155 / ADR-0007)
//!
//! - `QuotaDecisionPort` — the trait with `check_quota(request) -> QuotaDecision`.
//! - DTOs: `QuotaDto`, `UsageDto` — the JSON-serialisable shapes for the admin
//!   REST API (`PUT/GET /tenants/{id}/quota`, `GET /tenants/{id}/usage`).
//! - Cedar default-deny: the port contract requires that any implementation
//!   deny by default when no explicit quota record exists.

// ADR-0083 Tier-3: panic-free on the request path.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub use k8s_tenant_quota_kernel::{
    DenyReason, ProvisionRequest, QuotaDecision, QuotaModelError, RbacBinding, RbacRole, TenantId,
    TenantQuota, TenantUsage, evaluate,
};

// ============================================================
// QuotaDecision PORT
// ============================================================

/// The port cluster-lifecycle calls before provisioning a cluster.
///
/// Implementations MUST be:
/// - **Deny-by-default**: if no quota record exists for the tenant, deny.
/// - **Cross-tenant safe**: an implementation MUST NOT return quota data for a
///   tenant other than the one in the request.
/// - **Fail-closed**: any store error is surfaced as `QuotaPortError`, never as
///   a silent allow.
pub trait QuotaDecisionPort {
    /// Check whether the provisioning request is within the tenant's quota.
    ///
    /// # Errors
    /// Returns [`QuotaPortError`] on persistence failure or tenant-not-found.
    fn check_quota(&self, request: &ProvisionRequest) -> Result<QuotaDecision, QuotaPortError>;
}

/// The port for reading and writing quota records (admin plane).
pub trait QuotaAdminPort {
    /// Set or replace the quota for a tenant (within plan ceiling).
    ///
    /// # Errors
    /// Returns [`QuotaPortError`] on validation or persistence failure.
    fn set_quota(&self, quota: TenantQuota) -> Result<(), QuotaPortError>;

    /// Read the quota record for a tenant.
    ///
    /// # Errors
    /// Returns [`QuotaPortError::NotFound`] when no record exists,
    /// or [`QuotaPortError::Persistence`] on store failure.
    fn get_quota(&self, tenant_id: &TenantId) -> Result<TenantQuota, QuotaPortError>;

    /// Read current usage for a tenant.
    ///
    /// # Errors
    /// Returns [`QuotaPortError::NotFound`] when no record exists,
    /// or [`QuotaPortError::Persistence`] on store failure.
    fn get_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage, QuotaPortError>;

    /// Record updated usage for a tenant (called by provisioning pipeline).
    ///
    /// # Errors
    /// Returns [`QuotaPortError`] on persistence failure.
    fn set_usage(&self, usage: TenantUsage) -> Result<(), QuotaPortError>;
}

/// Errors returned by the quota port implementations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaPortError {
    /// No quota record found for the given tenant.
    NotFound(String),
    /// A persistence / store failure.
    Persistence(String),
    /// A validation error from the kernel model.
    Validation(String),
}

impl std::fmt::Display for QuotaPortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "quota record not found for tenant {id}"),
            Self::Persistence(detail) => write!(f, "quota store error: {detail}"),
            Self::Validation(detail) => write!(f, "quota validation error: {detail}"),
        }
    }
}

impl std::error::Error for QuotaPortError {}

impl From<QuotaModelError> for QuotaPortError {
    fn from(e: QuotaModelError) -> Self {
        Self::Validation(e.to_string())
    }
}

// ============================================================
// DTOs (admin REST API surface)
// ============================================================

/// DTO for setting or reading a tenant's quota via the REST API.
///
/// Maps 1:1 to [`TenantQuota`]; used for JSON serialisation at the HTTP layer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaDto {
    /// Tenant id.
    pub tenant_id: String,
    /// Maximum concurrent clusters.
    pub max_clusters: u32,
    /// Maximum nodes per cluster.
    pub max_nodes_per_cluster: u32,
    /// Maximum vCPU per cluster.
    pub max_vcpu_per_cluster: u32,
    /// Maximum RAM (GiB) per cluster.
    pub max_ram_gib_per_cluster: u32,
}

impl QuotaDto {
    /// Convert this DTO into a kernel [`TenantQuota`].
    ///
    /// # Errors
    /// Returns [`QuotaPortError::Validation`] if the values fail kernel validation.
    pub fn into_quota(self) -> Result<TenantQuota, QuotaPortError> {
        TenantQuota::new(
            self.tenant_id,
            self.max_clusters,
            self.max_nodes_per_cluster,
            self.max_vcpu_per_cluster,
            self.max_ram_gib_per_cluster,
        )
        .map_err(Into::into)
    }
}

impl From<TenantQuota> for QuotaDto {
    fn from(q: TenantQuota) -> Self {
        Self {
            tenant_id: q.tenant_id.as_str().to_string(),
            max_clusters: q.max_clusters,
            max_nodes_per_cluster: q.max_nodes_per_cluster,
            max_vcpu_per_cluster: q.max_vcpu_per_cluster,
            max_ram_gib_per_cluster: q.max_ram_gib_per_cluster,
        }
    }
}

/// DTO for reading a tenant's current cluster resource usage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageDto {
    /// Tenant id.
    pub tenant_id: String,
    /// Current number of provisioned clusters.
    pub current_clusters: u32,
    /// Maximum nodes observed in any single cluster.
    pub max_nodes_in_any_cluster: u32,
    /// Maximum vCPU observed in any single cluster.
    pub max_vcpu_in_any_cluster: u32,
    /// Maximum RAM (GiB) observed in any single cluster.
    pub max_ram_gib_in_any_cluster: u32,
}

impl From<TenantUsage> for UsageDto {
    fn from(u: TenantUsage) -> Self {
        Self {
            tenant_id: u.tenant_id.as_str().to_string(),
            current_clusters: u.current_clusters,
            max_nodes_in_any_cluster: u.max_nodes_in_any_cluster,
            max_vcpu_in_any_cluster: u.max_vcpu_in_any_cluster,
            max_ram_gib_in_any_cluster: u.max_ram_gib_in_any_cluster,
        }
    }
}

/// Response body for a quota check (used by cluster-lifecycle callers).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaCheckResponse {
    /// Whether provisioning is allowed.
    pub allowed: bool,
    /// Denial reason, if `allowed == false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_reason: Option<String>,
}

impl From<QuotaDecision> for QuotaCheckResponse {
    fn from(d: QuotaDecision) -> Self {
        match d {
            QuotaDecision::Allow => Self {
                allowed: true,
                deny_reason: None,
            },
            QuotaDecision::Deny(reason) => Self {
                allowed: false,
                deny_reason: Some(reason.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_dto_round_trip() {
        let dto = QuotaDto {
            tenant_id: "ten_acme".to_string(),
            max_clusters: 5,
            max_nodes_per_cluster: 10,
            max_vcpu_per_cluster: 32,
            max_ram_gib_per_cluster: 128,
        };
        let quota = dto.clone().into_quota().unwrap();
        let back: QuotaDto = quota.into();
        assert_eq!(back, dto);
    }

    #[test]
    fn quota_check_response_allow() {
        let resp: QuotaCheckResponse = QuotaDecision::Allow.into();
        assert!(resp.allowed);
        assert!(resp.deny_reason.is_none());
    }

    #[test]
    fn quota_check_response_deny() {
        let resp: QuotaCheckResponse = QuotaDecision::Deny(DenyReason::ClusterLimitExceeded {
            current: 4,
            requested: 2,
            limit: 5,
        })
        .into();
        assert!(!resp.allowed);
        assert!(resp.deny_reason.is_some());
    }

    #[test]
    fn port_error_display() {
        assert!(
            QuotaPortError::NotFound("ten_x".into())
                .to_string()
                .contains("ten_x")
        );
    }
}
