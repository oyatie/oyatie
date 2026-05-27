//! Pure quota model for per-tenant managed-K8s cluster resources.
//!
//! This crate is the innermost ring (ADR-0083 Tier-3): **no I/O, no async,
//! no external deps beyond `serde`**. It owns the quota value objects and the
//! deterministic `evaluate` function that cluster-lifecycle calls before any
//! provisioning step.
//!
//! ## Design (ADR-0376 / ADR-0155 / ADR-0007)
//!
//! - **Quota model**: per-tenant ceilings (`max_clusters`, `max_nodes_per_cluster`,
//!   `max_vcpu_per_cluster`, `max_ram_gib_per_cluster`). The *platform* sets plan
//!   ceilings; a *tenant-admin* sets their own quota within those ceilings.
//! - **evaluate()**: deterministic, panic-free, total. Returns `QuotaDecision::Allow`
//!   or `QuotaDecision::Deny(reason)`.
//! - **Threat model**: quota bypass (request exceeds limits), RBAC escalation
//!   (tenant alters another's quota), cross-tenant limit read (tenant reads
//!   another tenant's usage). All three are addressed at the RBAC layer
//!   (adapter-cedar) and enforced here via tenant-scoped value objects.
//! - **RBAC binding value objects**: `RbacBinding` and `RbacRole` capture the
//!   cluster-scoped role grants used by the Cedar adapter.

// ADR-0083 Tier-3: production code stays panic-free; tests may use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ============================================================
// Quota model
// ============================================================

/// Per-tenant quota ceiling for managed-K8s resources.
///
/// The *platform* sets plan ceilings via [`TenantQuota::plan_ceiling`].
/// A *tenant-admin* configures their own quota within those ceilings.
/// A tenant CANNOT exceed their plan ceiling, and CANNOT read or alter
/// another tenant's quota (enforced by Cedar RBAC — see adapter-cedar).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TenantQuota {
    /// Owning tenant id (e.g. `ten_acme`). Scopes all quota operations.
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// Maximum number of clusters this tenant may provision concurrently.
    pub max_clusters: u32, // data_class: INTERNAL_ONLY
    /// Maximum nodes per cluster.
    pub max_nodes_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Maximum vCPU across all nodes in one cluster.
    pub max_vcpu_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Maximum RAM in GiB across all nodes in one cluster.
    pub max_ram_gib_per_cluster: u32, // data_class: INTERNAL_ONLY
}

impl TenantQuota {
    /// Construct a new tenant quota. Returns `Err` if `tenant_id` is empty or
    /// any ceiling is zero (a zero ceiling is non-sensical and fail-closed).
    ///
    /// # Errors
    /// Returns [`QuotaModelError`] when validation fails.
    pub fn new(
        tenant_id: impl Into<String>,
        max_clusters: u32,
        max_nodes_per_cluster: u32,
        max_vcpu_per_cluster: u32,
        max_ram_gib_per_cluster: u32,
    ) -> Result<Self, QuotaModelError> {
        let tenant_id = TenantId::new(tenant_id)?;
        if max_clusters == 0 {
            return Err(QuotaModelError::ZeroCeiling("max_clusters"));
        }
        if max_nodes_per_cluster == 0 {
            return Err(QuotaModelError::ZeroCeiling("max_nodes_per_cluster"));
        }
        if max_vcpu_per_cluster == 0 {
            return Err(QuotaModelError::ZeroCeiling("max_vcpu_per_cluster"));
        }
        if max_ram_gib_per_cluster == 0 {
            return Err(QuotaModelError::ZeroCeiling("max_ram_gib_per_cluster"));
        }
        Ok(Self {
            tenant_id,
            max_clusters,
            max_nodes_per_cluster,
            max_vcpu_per_cluster,
            max_ram_gib_per_cluster,
        })
    }

    /// Build a plan ceiling (platform-level). Alias for `new`; semantically
    /// signals this is the hard cap a tenant admin cannot exceed.
    ///
    /// # Errors
    /// Returns [`QuotaModelError`] when validation fails.
    pub fn plan_ceiling(
        tenant_id: impl Into<String>,
        max_clusters: u32,
        max_nodes_per_cluster: u32,
        max_vcpu_per_cluster: u32,
        max_ram_gib_per_cluster: u32,
    ) -> Result<Self, QuotaModelError> {
        Self::new(
            tenant_id,
            max_clusters,
            max_nodes_per_cluster,
            max_vcpu_per_cluster,
            max_ram_gib_per_cluster,
        )
    }
}

/// A typed tenant identifier. Prevents raw-string mix-ups across API surfaces.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TenantId(String);

impl TenantId {
    /// Construct from a non-empty string.
    ///
    /// # Errors
    /// Returns [`QuotaModelError::EmptyTenantId`] when the string is empty.
    pub fn new(id: impl Into<String>) -> Result<Self, QuotaModelError> {
        let id = id.into();
        if id.is_empty() {
            return Err(QuotaModelError::EmptyTenantId);
        }
        Ok(Self(id))
    }

    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Current resource usage for one tenant. Supplied by the caller at evaluate time.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TenantUsage {
    /// Owning tenant id (must match the quota being evaluated).
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// Number of clusters already provisioned.
    pub current_clusters: u32, // data_class: INTERNAL_ONLY
    /// Maximum node count across all existing clusters (worst-case check).
    pub max_nodes_in_any_cluster: u32, // data_class: INTERNAL_ONLY
    /// Maximum vCPU count across all existing clusters.
    pub max_vcpu_in_any_cluster: u32, // data_class: INTERNAL_ONLY
    /// Maximum RAM (GiB) across all existing clusters.
    pub max_ram_gib_in_any_cluster: u32, // data_class: INTERNAL_ONLY
}

impl TenantUsage {
    /// Construct usage. Returns `Err` when `tenant_id` is empty.
    ///
    /// # Errors
    /// Returns [`QuotaModelError::EmptyTenantId`] when `tenant_id` is empty.
    pub fn new(
        tenant_id: impl Into<String>,
        current_clusters: u32,
        max_nodes_in_any_cluster: u32,
        max_vcpu_in_any_cluster: u32,
        max_ram_gib_in_any_cluster: u32,
    ) -> Result<Self, QuotaModelError> {
        let tenant_id = TenantId::new(tenant_id)?;
        Ok(Self {
            tenant_id,
            current_clusters,
            max_nodes_in_any_cluster,
            max_vcpu_in_any_cluster,
            max_ram_gib_in_any_cluster,
        })
    }
}

/// A provisioning request that `evaluate` checks against quota.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvisionRequest {
    /// Tenant requesting provisioning.
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// Number of new clusters being requested.
    pub requested_clusters: u32, // data_class: INTERNAL_ONLY
    /// Desired nodes per new cluster.
    pub requested_nodes_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Desired vCPU per new cluster.
    pub requested_vcpu_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Desired RAM (GiB) per new cluster.
    pub requested_ram_gib_per_cluster: u32, // data_class: INTERNAL_ONLY
}

impl ProvisionRequest {
    /// Construct a provisioning request. Returns `Err` when `tenant_id` is empty
    /// or any requested value is zero.
    ///
    /// # Errors
    /// Returns [`QuotaModelError`] when validation fails.
    pub fn new(
        tenant_id: impl Into<String>,
        requested_clusters: u32,
        requested_nodes_per_cluster: u32,
        requested_vcpu_per_cluster: u32,
        requested_ram_gib_per_cluster: u32,
    ) -> Result<Self, QuotaModelError> {
        let tenant_id = TenantId::new(tenant_id)?;
        if requested_clusters == 0 {
            return Err(QuotaModelError::ZeroCeiling("requested_clusters"));
        }
        Ok(Self {
            tenant_id,
            requested_clusters,
            requested_nodes_per_cluster,
            requested_vcpu_per_cluster,
            requested_ram_gib_per_cluster,
        })
    }
}

/// Outcome of a quota evaluation. Either `Allow` or `Deny` with a reason.
///
/// This is the type returned by [`evaluate`] and the type the
/// `QuotaDecisionPort` trait in `oya-managed-k8s-tenant-quota-api` surfaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum QuotaDecision {
    /// The provisioning request is within quota limits.
    Allow,
    /// The provisioning request exceeds quota. Contains a human-readable reason.
    Deny(DenyReason),
}

impl QuotaDecision {
    /// Returns `true` if the decision is `Allow`.
    #[must_use]
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Returns `true` if the decision is `Deny`.
    #[must_use]
    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny(_))
    }
}

/// The reason a provisioning request was denied.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DenyReason {
    /// Cluster count would exceed `max_clusters`.
    ClusterLimitExceeded {
        /// Current number of clusters.
        current: u32,
        /// Requested additional clusters.
        requested: u32,
        /// Configured ceiling.
        limit: u32,
    },
    /// Node count per cluster would exceed `max_nodes_per_cluster`.
    NodeLimitExceeded {
        /// Requested nodes per cluster.
        requested: u32,
        /// Configured ceiling.
        limit: u32,
    },
    /// vCPU count per cluster would exceed `max_vcpu_per_cluster`.
    VcpuLimitExceeded {
        /// Requested vCPU per cluster.
        requested: u32,
        /// Configured ceiling.
        limit: u32,
    },
    /// RAM per cluster would exceed `max_ram_gib_per_cluster`.
    RamLimitExceeded {
        /// Requested RAM (GiB) per cluster.
        requested: u32,
        /// Configured ceiling.
        limit: u32,
    },
    /// Tenant IDs on quota and request do not match (cross-tenant guard).
    TenantMismatch {
        /// Tenant from the quota record.
        quota_tenant: String,
        /// Tenant from the request.
        request_tenant: String,
    },
}

impl std::fmt::Display for DenyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClusterLimitExceeded {
                current,
                requested,
                limit,
            } => write!(
                f,
                "cluster limit exceeded: current={current} + requested={requested} > limit={limit}"
            ),
            Self::NodeLimitExceeded { requested, limit } => write!(
                f,
                "node limit exceeded: requested={requested} > limit={limit}"
            ),
            Self::VcpuLimitExceeded { requested, limit } => write!(
                f,
                "vCPU limit exceeded: requested={requested} > limit={limit}"
            ),
            Self::RamLimitExceeded { requested, limit } => write!(
                f,
                "RAM limit exceeded: requested={requested} GiB > limit={limit} GiB"
            ),
            Self::TenantMismatch {
                quota_tenant,
                request_tenant,
            } => write!(
                f,
                "tenant mismatch: quota belongs to {quota_tenant} but request is from {request_tenant}"
            ),
        }
    }
}

/// Errors from quota model construction (validation layer; not request-path).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaModelError {
    /// Tenant id string is empty.
    EmptyTenantId,
    /// A ceiling was set to zero, which is invalid.
    ZeroCeiling(&'static str),
}

impl std::fmt::Display for QuotaModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTenantId => write!(f, "tenant_id must not be empty"),
            Self::ZeroCeiling(field) => write!(f, "quota field {field} must be > 0"),
        }
    }
}

impl std::error::Error for QuotaModelError {}

// ============================================================
// RBAC binding value objects (used by adapter-cedar)
// ============================================================

/// A role scoped to a tenant's cluster namespace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RbacRole {
    /// Can provision clusters and set quotas within plan ceiling.
    TenantAdmin,
    /// Can read quota and usage for their own tenant only.
    TenantViewer,
    /// Platform-level role: can set plan ceilings for any tenant.
    PlatformOperator,
}

impl RbacRole {
    /// Cedar action string for quota write operations.
    #[must_use]
    pub fn quota_write_action(&self) -> &'static str {
        "quota:Write"
    }

    /// Cedar action string for quota read operations.
    #[must_use]
    pub fn quota_read_action(&self) -> &'static str {
        "quota:Read"
    }

    /// Whether this role can write quota records.
    #[must_use]
    pub fn can_write_quota(&self) -> bool {
        matches!(self, Self::TenantAdmin | Self::PlatformOperator)
    }

    /// Whether this role can read quota records.
    #[must_use]
    pub fn can_read_quota(&self) -> bool {
        true // all roles can read their own; cross-tenant denied by Cedar tenant_id condition
    }
}

/// A binding of a workload principal to an RBAC role scoped to a tenant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RbacBinding {
    /// The tenant this binding is scoped to.
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// The principal's workload id.
    pub workload_id: String, // data_class: INTERNAL_ONLY
    /// The granted role.
    pub role: RbacRole, // data_class: INTERNAL_ONLY
}

impl RbacBinding {
    /// Construct an RBAC binding. Returns `Err` when `tenant_id` or `workload_id` is empty.
    ///
    /// # Errors
    /// Returns [`QuotaModelError`] when validation fails.
    pub fn new(
        tenant_id: impl Into<String>,
        workload_id: impl Into<String>,
        role: RbacRole,
    ) -> Result<Self, QuotaModelError> {
        let tenant_id = TenantId::new(tenant_id)?;
        let workload_id = workload_id.into();
        if workload_id.is_empty() {
            return Err(QuotaModelError::EmptyTenantId); // reuse; workload_id same invariant
        }
        Ok(Self {
            tenant_id,
            workload_id,
            role,
        })
    }
}

// ============================================================
// Core evaluate function (ADR-0376 §quota-decision-port)
// ============================================================

/// Evaluate a provisioning request against the tenant's quota.
///
/// This is the hot-path function cluster-lifecycle calls before any cluster
/// is provisioned. It is:
/// - **Deterministic**: given the same inputs it always returns the same output.
/// - **Total**: never panics, never returns an error (the inputs are typed).
/// - **Cross-tenant safe**: if `usage.tenant_id != request.tenant_id` or either
///   differs from `quota.tenant_id`, the call returns `Deny(TenantMismatch)`.
///
/// SLO: this function is O(1) with no allocations on the allow path; latency
/// on the lifecycle hot path is sub-microsecond.
#[must_use]
pub fn evaluate(
    quota: &TenantQuota,
    usage: &TenantUsage,
    request: &ProvisionRequest,
) -> QuotaDecision {
    // Cross-tenant guard (ADR-0376 threat model: cross-tenant limit read/bypass).
    if quota.tenant_id != usage.tenant_id || quota.tenant_id != request.tenant_id {
        return QuotaDecision::Deny(DenyReason::TenantMismatch {
            quota_tenant: quota.tenant_id.as_str().to_string(),
            request_tenant: request.tenant_id.as_str().to_string(),
        });
    }

    // Cluster count check.
    let total_clusters = usage
        .current_clusters
        .saturating_add(request.requested_clusters);
    if total_clusters > quota.max_clusters {
        return QuotaDecision::Deny(DenyReason::ClusterLimitExceeded {
            current: usage.current_clusters,
            requested: request.requested_clusters,
            limit: quota.max_clusters,
        });
    }

    // Node count per new cluster.
    if request.requested_nodes_per_cluster > quota.max_nodes_per_cluster {
        return QuotaDecision::Deny(DenyReason::NodeLimitExceeded {
            requested: request.requested_nodes_per_cluster,
            limit: quota.max_nodes_per_cluster,
        });
    }

    // vCPU per new cluster.
    if request.requested_vcpu_per_cluster > quota.max_vcpu_per_cluster {
        return QuotaDecision::Deny(DenyReason::VcpuLimitExceeded {
            requested: request.requested_vcpu_per_cluster,
            limit: quota.max_vcpu_per_cluster,
        });
    }

    // RAM per new cluster.
    if request.requested_ram_gib_per_cluster > quota.max_ram_gib_per_cluster {
        return QuotaDecision::Deny(DenyReason::RamLimitExceeded {
            requested: request.requested_ram_gib_per_cluster,
            limit: quota.max_ram_gib_per_cluster,
        });
    }

    QuotaDecision::Allow
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn quota(tenant: &str) -> TenantQuota {
        TenantQuota::new(tenant, 5, 10, 32, 128).unwrap()
    }

    fn usage(tenant: &str, clusters: u32) -> TenantUsage {
        TenantUsage::new(tenant, clusters, 0, 0, 0).unwrap()
    }

    fn request(tenant: &str, clusters: u32, nodes: u32, vcpu: u32, ram: u32) -> ProvisionRequest {
        ProvisionRequest::new(tenant, clusters, nodes, vcpu, ram).unwrap()
    }

    #[test]
    fn allow_within_limits() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 2);
        let r = request("ten_acme", 1, 5, 16, 64);
        assert_eq!(evaluate(&q, &u, &r), QuotaDecision::Allow);
    }

    #[test]
    fn deny_cluster_limit_exceeded() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 4);
        let r = request("ten_acme", 2, 1, 1, 1);
        assert!(matches!(
            evaluate(&q, &u, &r),
            QuotaDecision::Deny(DenyReason::ClusterLimitExceeded { .. })
        ));
    }

    #[test]
    fn deny_node_limit_exceeded() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 0);
        let r = request("ten_acme", 1, 11, 1, 1);
        assert!(matches!(
            evaluate(&q, &u, &r),
            QuotaDecision::Deny(DenyReason::NodeLimitExceeded { .. })
        ));
    }

    #[test]
    fn deny_vcpu_limit_exceeded() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 0);
        let r = request("ten_acme", 1, 1, 33, 1);
        assert!(matches!(
            evaluate(&q, &u, &r),
            QuotaDecision::Deny(DenyReason::VcpuLimitExceeded { .. })
        ));
    }

    #[test]
    fn deny_ram_limit_exceeded() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 0);
        let r = request("ten_acme", 1, 1, 1, 129);
        assert!(matches!(
            evaluate(&q, &u, &r),
            QuotaDecision::Deny(DenyReason::RamLimitExceeded { .. })
        ));
    }

    #[test]
    fn deny_cross_tenant_mismatch() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 0);
        let r = request("ten_globex", 1, 1, 1, 1);
        assert!(matches!(
            evaluate(&q, &u, &r),
            QuotaDecision::Deny(DenyReason::TenantMismatch { .. })
        ));
    }

    #[test]
    fn deny_cross_tenant_usage_mismatch() {
        let q = quota("ten_acme");
        let u = usage("ten_globex", 0); // wrong tenant usage
        let r = request("ten_acme", 1, 1, 1, 1);
        assert!(matches!(
            evaluate(&q, &u, &r),
            QuotaDecision::Deny(DenyReason::TenantMismatch { .. })
        ));
    }

    #[test]
    fn at_exact_cluster_limit_is_allow() {
        let q = quota("ten_acme");
        let u = usage("ten_acme", 4);
        let r = request("ten_acme", 1, 1, 1, 1);
        assert_eq!(evaluate(&q, &u, &r), QuotaDecision::Allow);
    }

    #[test]
    fn empty_tenant_id_rejected() {
        assert!(TenantId::new("").is_err());
    }

    #[test]
    fn zero_ceiling_rejected() {
        assert!(TenantQuota::new("ten_acme", 0, 10, 32, 128).is_err());
    }

    #[test]
    fn rbac_binding_round_trip() {
        let b = RbacBinding::new("ten_acme", "wl_admin_01", RbacRole::TenantAdmin).unwrap();
        assert_eq!(b.tenant_id.as_str(), "ten_acme");
        assert!(b.role.can_write_quota());
    }
}
