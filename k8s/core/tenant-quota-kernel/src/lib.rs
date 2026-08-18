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
        if requested_nodes_per_cluster == 0 {
            return Err(QuotaModelError::ZeroCeiling("requested_nodes_per_cluster"));
        }
        if requested_vcpu_per_cluster == 0 {
            return Err(QuotaModelError::ZeroCeiling("requested_vcpu_per_cluster"));
        }
        if requested_ram_gib_per_cluster == 0 {
            return Err(QuotaModelError::ZeroCeiling(
                "requested_ram_gib_per_cluster",
            ));
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
/// `QuotaDecisionPort` trait in `k8s-tenant-quota-api` surfaces.
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
// Usage aggregation and headroom projection (ADR-0376 lifecycle pre-check)
// ============================================================

/// Per-cluster resource usage snapshot.
///
/// Mirrors K8s `ResourceQuota.status.used` fields for one cluster.
/// Used as input to [`aggregate_usage`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterResourceUsage {
    /// Cluster identifier (opaque string; must be non-empty).
    pub cluster_id: String, // data_class: INTERNAL_ONLY
    /// Number of worker nodes currently in this cluster.
    pub node_count: u32, // data_class: INTERNAL_ONLY
    /// Total vCPU across all nodes in this cluster.
    pub vcpu_total: u32, // data_class: INTERNAL_ONLY
    /// Total RAM in GiB across all nodes in this cluster.
    pub ram_gib_total: u32, // data_class: INTERNAL_ONLY
}

/// Aggregated resource summary across all clusters for one tenant.
///
/// Produced by [`aggregate_usage`]; consumed by [`project_headroom`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TenantResourceSummary {
    /// Owning tenant id.
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// Number of clusters counted in this summary.
    pub total_clusters: u32, // data_class: INTERNAL_ONLY
    /// Sum of `node_count` across all clusters.
    pub total_nodes: u32, // data_class: INTERNAL_ONLY
    /// Sum of `vcpu_total` across all clusters.
    pub total_vcpu: u32, // data_class: INTERNAL_ONLY
    /// Sum of `ram_gib_total` across all clusters.
    pub total_ram_gib: u32, // data_class: INTERNAL_ONLY
    /// Maximum `node_count` observed in any single cluster.
    pub max_nodes_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Maximum `vcpu_total` observed in any single cluster.
    pub max_vcpu_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Maximum `ram_gib_total` observed in any single cluster.
    pub max_ram_gib_per_cluster: u32, // data_class: INTERNAL_ONLY
}

/// Remaining quota headroom and percent-utilization for a tenant.
///
/// Produced by [`project_headroom`]. Used as a lifecycle pre-check signal
/// and as an input to future billing/alerting pipelines.
///
/// All `remaining_*` fields use saturating subtraction (never underflow).
/// All `*_utilized_pct` fields are clamped to `[0, 100]`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuotaHeadroom {
    /// Owning tenant id.
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// Clusters that can still be provisioned.
    pub remaining_clusters: u32, // data_class: INTERNAL_ONLY
    /// Additional nodes per new cluster that are still within quota.
    pub remaining_nodes_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Additional vCPU per new cluster that are still within quota.
    pub remaining_vcpu_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Additional RAM (GiB) per new cluster that are still within quota.
    pub remaining_ram_gib_per_cluster: u32, // data_class: INTERNAL_ONLY
    /// Cluster-count utilization: `total_clusters * 100 / max_clusters`, clamped to 100.
    pub clusters_utilized_pct: u8, // data_class: INTERNAL_ONLY
    /// Node utilization vs `max_nodes_per_cluster` ceiling, clamped to 100.
    pub nodes_utilized_pct: u8, // data_class: INTERNAL_ONLY
    /// vCPU utilization vs `max_vcpu_per_cluster` ceiling, clamped to 100.
    pub vcpu_utilized_pct: u8, // data_class: INTERNAL_ONLY
    /// RAM utilization vs `max_ram_gib_per_cluster` ceiling, clamped to 100.
    pub ram_utilized_pct: u8, // data_class: INTERNAL_ONLY
}

/// Errors produced by [`project_headroom`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeadroomError {
    /// The quota and summary belong to different tenants.
    TenantMismatch {
        /// Tenant from the quota record.
        quota_tenant: String,
        /// Tenant from the summary record.
        summary_tenant: String,
    },
}

impl std::fmt::Display for HeadroomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TenantMismatch {
                quota_tenant,
                summary_tenant,
            } => write!(
                f,
                "headroom tenant mismatch: quota belongs to {quota_tenant} but summary is from {summary_tenant}"
            ),
        }
    }
}

impl std::error::Error for HeadroomError {}

/// Fold a slice of per-cluster usage records into a single [`TenantResourceSummary`].
///
/// - Returns a zero-valued summary when `usages` is empty (valid; tenant has no clusters).
/// - All arithmetic is saturating; no overflow, no panic.
/// - O(n) in `usages.len()`.
#[must_use]
pub fn aggregate_usage(
    tenant_id: TenantId,
    usages: &[ClusterResourceUsage],
) -> TenantResourceSummary {
    let mut total_clusters: u32 = 0;
    let mut total_nodes: u32 = 0;
    let mut total_vcpu: u32 = 0;
    let mut total_ram_gib: u32 = 0;
    let mut max_nodes_per_cluster: u32 = 0;
    let mut max_vcpu_per_cluster: u32 = 0;
    let mut max_ram_gib_per_cluster: u32 = 0;

    for u in usages {
        total_clusters = total_clusters.saturating_add(1);
        total_nodes = total_nodes.saturating_add(u.node_count);
        total_vcpu = total_vcpu.saturating_add(u.vcpu_total);
        total_ram_gib = total_ram_gib.saturating_add(u.ram_gib_total);
        if u.node_count > max_nodes_per_cluster {
            max_nodes_per_cluster = u.node_count;
        }
        if u.vcpu_total > max_vcpu_per_cluster {
            max_vcpu_per_cluster = u.vcpu_total;
        }
        if u.ram_gib_total > max_ram_gib_per_cluster {
            max_ram_gib_per_cluster = u.ram_gib_total;
        }
    }

    TenantResourceSummary {
        tenant_id,
        total_clusters,
        total_nodes,
        total_vcpu,
        total_ram_gib,
        max_nodes_per_cluster,
        max_vcpu_per_cluster,
        max_ram_gib_per_cluster,
    }
}

/// Compute [`QuotaHeadroom`] for a tenant given their quota and aggregated usage summary.
///
/// Returns `Err(HeadroomError::TenantMismatch)` when `quota.tenant_id != summary.tenant_id`.
///
/// All `remaining_*` use saturating subtraction; all `*_utilized_pct` are clamped to `[0, 100]`.
/// No floating-point arithmetic; no allocations; O(1).
///
/// # Errors
/// Returns [`HeadroomError::TenantMismatch`] on cross-tenant input.
pub fn project_headroom(
    quota: &TenantQuota,
    summary: &TenantResourceSummary,
) -> Result<QuotaHeadroom, HeadroomError> {
    if quota.tenant_id != summary.tenant_id {
        return Err(HeadroomError::TenantMismatch {
            quota_tenant: quota.tenant_id.as_str().to_string(),
            summary_tenant: summary.tenant_id.as_str().to_string(),
        });
    }

    let remaining_clusters = quota.max_clusters.saturating_sub(summary.total_clusters);
    let remaining_nodes_per_cluster = quota
        .max_nodes_per_cluster
        .saturating_sub(summary.max_nodes_per_cluster);
    let remaining_vcpu_per_cluster = quota
        .max_vcpu_per_cluster
        .saturating_sub(summary.max_vcpu_per_cluster);
    let remaining_ram_gib_per_cluster = quota
        .max_ram_gib_per_cluster
        .saturating_sub(summary.max_ram_gib_per_cluster);

    let clusters_utilized_pct = utilized_pct(summary.total_clusters, quota.max_clusters);
    let nodes_utilized_pct =
        utilized_pct(summary.max_nodes_per_cluster, quota.max_nodes_per_cluster);
    let vcpu_utilized_pct = utilized_pct(summary.max_vcpu_per_cluster, quota.max_vcpu_per_cluster);
    let ram_utilized_pct = utilized_pct(
        summary.max_ram_gib_per_cluster,
        quota.max_ram_gib_per_cluster,
    );

    Ok(QuotaHeadroom {
        tenant_id: quota.tenant_id.clone(),
        remaining_clusters,
        remaining_nodes_per_cluster,
        remaining_vcpu_per_cluster,
        remaining_ram_gib_per_cluster,
        clusters_utilized_pct,
        nodes_utilized_pct,
        vcpu_utilized_pct,
        ram_utilized_pct,
    })
}

/// Compute integer percent utilization, clamped to `[0, 100]`.
///
/// Uses integer arithmetic only (no float). When `ceiling` is zero this
/// function returns 100 (fully utilized by convention), but in practice
/// `TenantQuota::new()` already rejects zero ceilings.
#[inline]
fn utilized_pct(used: u32, ceiling: u32) -> u8 {
    if ceiling == 0 {
        return 100;
    }
    // used * 100 / ceiling, saturating at u8::MAX then clamping to 100.
    let pct = (used as u64).saturating_mul(100) / (ceiling as u64);
    pct.min(100) as u8
}

// ============================================================
// Pressure band classifier (ADR-0130 SLO/alert inputs)
// ============================================================

/// The quota dimension that is most constrained (highest utilization pct).
///
/// Returned by [`QuotaHeadroom::most_constrained_dimension`].
///
/// ## Tie-break rule
/// When two or more dimensions share the same maximum utilization percentage,
/// the canonical priority order is `Clusters → Nodes → Vcpu → Ram` (first
/// highest wins). This order is stable across platforms and Rust versions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConstrainedDimension {
    /// Cluster-count dimension (`clusters_utilized_pct`).
    Clusters,
    /// Node-count dimension (`nodes_utilized_pct`).
    Nodes,
    /// vCPU dimension (`vcpu_utilized_pct`).
    Vcpu,
    /// RAM dimension (`ram_utilized_pct`).
    Ram,
}

impl ConstrainedDimension {
    /// Return a lowercase ASCII string representation.
    ///
    /// Inverse of [`ConstrainedDimension::parse`].
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clusters => "clusters",
            Self::Nodes => "nodes",
            Self::Vcpu => "vcpu",
            Self::Ram => "ram",
        }
    }

    /// Parse from a lowercase ASCII string. Returns `None` for unknown input (fail-closed).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "clusters" => Some(Self::Clusters),
            "nodes" => Some(Self::Nodes),
            "vcpu" => Some(Self::Vcpu),
            "ram" => Some(Self::Ram),
            _ => None,
        }
    }
}

/// Quota pressure band derived from the maximum utilization pct across all dimensions.
///
/// ## Thresholds
///
/// | max utilization pct | Band        |
/// |---------------------|-------------|
/// | `< 70`              | `Healthy`   |
/// | `70 ..< 90`         | `Warning`   |
/// | `90 ..< 100`        | `Critical`  |
/// | `== 100`            | `Exhausted` |
///
/// Produced by [`classify_pressure`]; consumed by ADR-0130 SLO/alert pipelines.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum QuotaPressure {
    /// Max utilization pct < 70. All dimensions have comfortable headroom.
    Healthy,
    /// Max utilization pct in `[70, 90)`. At least one dimension is approaching saturation.
    Warning,
    /// Max utilization pct in `[90, 100)`. At least one dimension is near exhaustion.
    Critical,
    /// Max utilization pct == 100. At least one dimension is fully exhausted.
    Exhausted,
}

impl QuotaPressure {
    /// Return a lowercase ASCII string representation.
    ///
    /// Inverse of [`QuotaPressure::parse`].
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Warning => "warning",
            Self::Critical => "critical",
            Self::Exhausted => "exhausted",
        }
    }

    /// Parse from a lowercase ASCII string. Returns `None` for unknown input (fail-closed).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "healthy" => Some(Self::Healthy),
            "warning" => Some(Self::Warning),
            "critical" => Some(Self::Critical),
            "exhausted" => Some(Self::Exhausted),
            _ => None,
        }
    }
}

impl QuotaHeadroom {
    /// Return the most constrained quota dimension (highest `*_utilized_pct`).
    ///
    /// When two or more dimensions share the same maximum utilization percentage,
    /// the tie-break priority order is `Clusters → Nodes → Vcpu → Ram` (first
    /// highest wins). This is deterministic and stable across platforms.
    #[must_use]
    pub fn most_constrained_dimension(&self) -> ConstrainedDimension {
        let max_pct = self
            .clusters_utilized_pct
            .max(self.nodes_utilized_pct)
            .max(self.vcpu_utilized_pct)
            .max(self.ram_utilized_pct);

        // Canonical tie-break order: Clusters → Nodes → Vcpu → Ram.
        if self.clusters_utilized_pct == max_pct {
            ConstrainedDimension::Clusters
        } else if self.nodes_utilized_pct == max_pct {
            ConstrainedDimension::Nodes
        } else if self.vcpu_utilized_pct == max_pct {
            ConstrainedDimension::Vcpu
        } else {
            ConstrainedDimension::Ram
        }
    }
}

/// Classify the quota pressure band from a [`QuotaHeadroom`] snapshot.
///
/// Computes `max_pct = max(clusters_utilized_pct, nodes_utilized_pct,
/// vcpu_utilized_pct, ram_utilized_pct)` and maps to a [`QuotaPressure`] band:
///
/// | max utilization pct | Band        |
/// |---------------------|-------------|
/// | `< 70`              | `Healthy`   |
/// | `70 ..< 90`         | `Warning`   |
/// | `90 ..< 100`        | `Critical`  |
/// | `== 100`            | `Exhausted` |
///
/// This function is:
/// - **Total**: never panics, always returns a value.
/// - **Deterministic**: same inputs always produce the same output.
/// - **Zero-alloc on the hot path**: no heap allocation.
/// - **O(1)**: four integer comparisons.
#[must_use]
pub fn classify_pressure(headroom: &QuotaHeadroom) -> QuotaPressure {
    let max_pct = headroom
        .clusters_utilized_pct
        .max(headroom.nodes_utilized_pct)
        .max(headroom.vcpu_utilized_pct)
        .max(headroom.ram_utilized_pct);

    if max_pct == 100 {
        QuotaPressure::Exhausted
    } else if max_pct >= 90 {
        QuotaPressure::Critical
    } else if max_pct >= 70 {
        QuotaPressure::Warning
    } else {
        QuotaPressure::Healthy
    }
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
    fn zero_requested_resources_rejected() {
        assert_eq!(
            ProvisionRequest::new("ten_acme", 0, 1, 1, 1),
            Err(QuotaModelError::ZeroCeiling("requested_clusters"))
        );
        assert_eq!(
            ProvisionRequest::new("ten_acme", 1, 0, 1, 1),
            Err(QuotaModelError::ZeroCeiling("requested_nodes_per_cluster"))
        );
        assert_eq!(
            ProvisionRequest::new("ten_acme", 1, 1, 0, 1),
            Err(QuotaModelError::ZeroCeiling("requested_vcpu_per_cluster"))
        );
        assert_eq!(
            ProvisionRequest::new("ten_acme", 1, 1, 1, 0),
            Err(QuotaModelError::ZeroCeiling(
                "requested_ram_gib_per_cluster"
            ))
        );
    }

    #[test]
    fn rbac_binding_round_trip() {
        let b = RbacBinding::new("ten_acme", "wl_admin_01", RbacRole::TenantAdmin).unwrap();
        assert_eq!(b.tenant_id.as_str(), "ten_acme");
        assert!(b.role.can_write_quota());
    }

    // ---- usage_projection tests (AC-1 through AC-8) ----

    fn cluster(id: &str, nodes: u32, vcpu: u32, ram: u32) -> ClusterResourceUsage {
        ClusterResourceUsage {
            cluster_id: id.to_string(),
            node_count: nodes,
            vcpu_total: vcpu,
            ram_gib_total: ram,
        }
    }

    fn tid(s: &str) -> TenantId {
        TenantId::new(s).unwrap()
    }

    /// AC-1: empty cluster list produces zero-valued summary.
    #[test]
    fn aggregate_empty_is_zero() {
        let summary = aggregate_usage(tid("ten_acme"), &[]);
        assert_eq!(summary.total_clusters, 0);
        assert_eq!(summary.total_nodes, 0);
        assert_eq!(summary.total_vcpu, 0);
        assert_eq!(summary.total_ram_gib, 0);
        assert_eq!(summary.max_nodes_per_cluster, 0);
        assert_eq!(summary.max_vcpu_per_cluster, 0);
        assert_eq!(summary.max_ram_gib_per_cluster, 0);
    }

    /// AC-2: multiple clusters are aggregated correctly.
    #[test]
    fn aggregate_multiple_clusters() {
        let usages = vec![cluster("cl-1", 3, 12, 48), cluster("cl-2", 5, 20, 80)];
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        assert_eq!(summary.total_clusters, 2);
        assert_eq!(summary.total_nodes, 8);
        assert_eq!(summary.total_vcpu, 32);
        assert_eq!(summary.total_ram_gib, 128);
        assert_eq!(summary.max_nodes_per_cluster, 5);
        assert_eq!(summary.max_vcpu_per_cluster, 20);
        assert_eq!(summary.max_ram_gib_per_cluster, 80);
    }

    /// AC-3: headroom within quota returns correct remaining and utilization.
    #[test]
    fn headroom_within_quota() {
        // quota: 5 clusters, 10 nodes/cl, 32 vcpu/cl, 128 ram/cl
        let q = quota("ten_acme");
        // summary: 2 clusters, max 5 nodes/cl, max 16 vcpu/cl, max 64 ram/cl
        let usages = vec![cluster("cl-1", 5, 16, 64), cluster("cl-2", 3, 8, 32)];
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();

        assert_eq!(h.remaining_clusters, 3); // 5 - 2
        assert_eq!(h.remaining_nodes_per_cluster, 5); // 10 - 5
        assert_eq!(h.remaining_vcpu_per_cluster, 16); // 32 - 16
        assert_eq!(h.remaining_ram_gib_per_cluster, 64); // 128 - 64
        assert_eq!(h.clusters_utilized_pct, 40); // 2*100/5
        assert_eq!(h.nodes_utilized_pct, 50); // 5*100/10
        assert_eq!(h.vcpu_utilized_pct, 50); // 16*100/32
        assert_eq!(h.ram_utilized_pct, 50); // 64*100/128
    }

    /// AC-4: headroom at the exact quota limit returns remaining=0, utilized=100.
    #[test]
    fn headroom_at_limit() {
        let q = quota("ten_acme"); // 5 cl, 10 nodes, 32 vcpu, 128 ram
        let usages = vec![
            cluster("cl-1", 10, 32, 128),
            cluster("cl-2", 10, 32, 128),
            cluster("cl-3", 10, 32, 128),
            cluster("cl-4", 10, 32, 128),
            cluster("cl-5", 10, 32, 128),
        ];
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();

        assert_eq!(h.remaining_clusters, 0);
        assert_eq!(h.remaining_nodes_per_cluster, 0);
        assert_eq!(h.remaining_vcpu_per_cluster, 0);
        assert_eq!(h.remaining_ram_gib_per_cluster, 0);
        assert_eq!(h.clusters_utilized_pct, 100);
        assert_eq!(h.nodes_utilized_pct, 100);
        assert_eq!(h.vcpu_utilized_pct, 100);
        assert_eq!(h.ram_utilized_pct, 100);
    }

    /// AC-5: over-limit summary (legacy over-provisioned) saturates to 0 remaining, 100%.
    #[test]
    fn headroom_over_limit_saturates() {
        let q = quota("ten_acme"); // 5 cl, 10 nodes, 32 vcpu, 128 ram
        // Simulate 7 clusters each over every per-cluster ceiling.
        let usages: Vec<_> = (0..7)
            .map(|i| cluster(&format!("cl-{i}"), 15, 50, 200))
            .collect();
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();

        assert_eq!(h.remaining_clusters, 0); // saturating_sub
        assert_eq!(h.remaining_nodes_per_cluster, 0);
        assert_eq!(h.remaining_vcpu_per_cluster, 0);
        assert_eq!(h.remaining_ram_gib_per_cluster, 0);
        assert_eq!(h.clusters_utilized_pct, 100);
        assert_eq!(h.nodes_utilized_pct, 100);
        assert_eq!(h.vcpu_utilized_pct, 100);
        assert_eq!(h.ram_utilized_pct, 100);
    }

    /// AC-6: tenant mismatch returns HeadroomError.
    #[test]
    fn headroom_tenant_mismatch() {
        let q = quota("ten_acme");
        let summary = aggregate_usage(tid("ten_globex"), &[]);
        let err = project_headroom(&q, &summary).unwrap_err();
        assert!(matches!(err, HeadroomError::TenantMismatch { .. }));
        let msg = err.to_string();
        assert!(msg.contains("ten_acme"));
        assert!(msg.contains("ten_globex"));
    }

    /// AC-7: all new types round-trip through serde_json.
    #[test]
    fn headroom_types_serde_roundtrip() {
        let q = quota("ten_acme");
        let usages = vec![cluster("cl-1", 4, 12, 48)];
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();

        let json = serde_json::to_string(&h).unwrap();
        let h2: QuotaHeadroom = serde_json::from_str(&json).unwrap();
        assert_eq!(h, h2);

        let json_s = serde_json::to_string(&summary).unwrap();
        let s2: TenantResourceSummary = serde_json::from_str(&json_s).unwrap();
        assert_eq!(summary, s2);

        let c = cluster("cl-x", 1, 4, 16);
        let json_c = serde_json::to_string(&c).unwrap();
        let c2: ClusterResourceUsage = serde_json::from_str(&json_c).unwrap();
        assert_eq!(c, c2);
    }

    /// AC-8: QuotaHeadroom composes with evaluate() as a lifecycle pre-check.
    ///
    /// If headroom shows remaining_clusters == 0 then evaluate() must Deny.
    #[test]
    fn headroom_compose_with_evaluate() {
        let q = quota("ten_acme"); // max 5 clusters
        // Fill all 5 cluster slots.
        let usages: Vec<_> = (0..5)
            .map(|i| cluster(&format!("cl-{i}"), 1, 1, 1))
            .collect();
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();
        assert_eq!(h.remaining_clusters, 0, "headroom must show 0 remaining");

        // evaluate() using existing TenantUsage: current_clusters matches total.
        let u = TenantUsage::new("ten_acme", summary.total_clusters, 0, 0, 0).unwrap();
        let r = ProvisionRequest::new("ten_acme", 1, 1, 1, 1).unwrap();
        let decision = evaluate(&q, &u, &r);
        assert!(
            decision.is_deny(),
            "evaluate must deny when headroom is zero"
        );
    }

    // ---- pressure band classifier tests ----

    /// Helper: build a QuotaHeadroom with explicit pct values. `remaining_*` set to
    /// saturating_sub(100 - pct) as placeholders (value not tested here).
    fn headroom_with_pcts(
        clusters_pct: u8,
        nodes_pct: u8,
        vcpu_pct: u8,
        ram_pct: u8,
    ) -> QuotaHeadroom {
        QuotaHeadroom {
            tenant_id: tid("ten_test"),
            remaining_clusters: 100u32.saturating_sub(clusters_pct as u32),
            remaining_nodes_per_cluster: 100u32.saturating_sub(nodes_pct as u32),
            remaining_vcpu_per_cluster: 100u32.saturating_sub(vcpu_pct as u32),
            remaining_ram_gib_per_cluster: 100u32.saturating_sub(ram_pct as u32),
            clusters_utilized_pct: clusters_pct,
            nodes_utilized_pct: nodes_pct,
            vcpu_utilized_pct: vcpu_pct,
            ram_utilized_pct: ram_pct,
        }
    }

    // --- Band edge tests ---

    /// pct=0 -> Healthy
    #[test]
    fn pressure_zero_pct_is_healthy() {
        let h = headroom_with_pcts(0, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Healthy);
    }

    /// pct=69 -> Healthy (upper edge of Healthy band)
    #[test]
    fn pressure_69_pct_is_healthy() {
        let h = headroom_with_pcts(69, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Healthy);
    }

    /// pct=70 -> Warning (lower edge of Warning band)
    #[test]
    fn pressure_70_pct_is_warning() {
        let h = headroom_with_pcts(70, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Warning);
    }

    /// pct=89 -> Warning (upper edge of Warning band)
    #[test]
    fn pressure_89_pct_is_warning() {
        let h = headroom_with_pcts(89, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Warning);
    }

    /// pct=90 -> Critical (lower edge of Critical band)
    #[test]
    fn pressure_90_pct_is_critical() {
        let h = headroom_with_pcts(90, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Critical);
    }

    /// pct=99 -> Critical (upper edge of Critical band)
    #[test]
    fn pressure_99_pct_is_critical() {
        let h = headroom_with_pcts(99, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Critical);
    }

    /// pct=100 -> Exhausted
    #[test]
    fn pressure_100_pct_is_exhausted() {
        let h = headroom_with_pcts(100, 0, 0, 0);
        assert_eq!(classify_pressure(&h), QuotaPressure::Exhausted);
    }

    // --- Each dimension being the constrained one ---

    #[test]
    fn pressure_driven_by_clusters_dimension() {
        let h = headroom_with_pcts(95, 10, 10, 10);
        assert_eq!(classify_pressure(&h), QuotaPressure::Critical);
        assert_eq!(
            h.most_constrained_dimension(),
            ConstrainedDimension::Clusters
        );
    }

    #[test]
    fn pressure_driven_by_nodes_dimension() {
        let h = headroom_with_pcts(10, 95, 10, 10);
        assert_eq!(classify_pressure(&h), QuotaPressure::Critical);
        assert_eq!(h.most_constrained_dimension(), ConstrainedDimension::Nodes);
    }

    #[test]
    fn pressure_driven_by_vcpu_dimension() {
        let h = headroom_with_pcts(10, 10, 95, 10);
        assert_eq!(classify_pressure(&h), QuotaPressure::Critical);
        assert_eq!(h.most_constrained_dimension(), ConstrainedDimension::Vcpu);
    }

    #[test]
    fn pressure_driven_by_ram_dimension() {
        let h = headroom_with_pcts(10, 10, 10, 95);
        assert_eq!(classify_pressure(&h), QuotaPressure::Critical);
        assert_eq!(h.most_constrained_dimension(), ConstrainedDimension::Ram);
    }

    // --- Tie-break determinism ---

    /// All equal -> Clusters wins (first in canonical order).
    #[test]
    fn most_constrained_tie_all_equal_returns_clusters() {
        let h = headroom_with_pcts(75, 75, 75, 75);
        assert_eq!(
            h.most_constrained_dimension(),
            ConstrainedDimension::Clusters
        );
    }

    /// Nodes == Vcpu == Ram tied at max; Clusters lower -> Nodes wins.
    #[test]
    fn most_constrained_tie_nodes_vcpu_ram_equal_nodes_wins() {
        let h = headroom_with_pcts(50, 80, 80, 80);
        assert_eq!(h.most_constrained_dimension(), ConstrainedDimension::Nodes);
    }

    /// Vcpu == Ram tied at max; Clusters and Nodes lower -> Vcpu wins.
    #[test]
    fn most_constrained_tie_vcpu_ram_equal_vcpu_wins() {
        let h = headroom_with_pcts(50, 50, 80, 80);
        assert_eq!(h.most_constrained_dimension(), ConstrainedDimension::Vcpu);
    }

    // --- Serde round-trip ---

    #[test]
    fn quota_pressure_serde_roundtrip() {
        for pressure in [
            QuotaPressure::Healthy,
            QuotaPressure::Warning,
            QuotaPressure::Critical,
            QuotaPressure::Exhausted,
        ] {
            let json = serde_json::to_string(&pressure).unwrap();
            let p2: QuotaPressure = serde_json::from_str(&json).unwrap();
            assert_eq!(pressure, p2);
        }
    }

    #[test]
    fn constrained_dimension_serde_roundtrip() {
        for dim in [
            ConstrainedDimension::Clusters,
            ConstrainedDimension::Nodes,
            ConstrainedDimension::Vcpu,
            ConstrainedDimension::Ram,
        ] {
            let json = serde_json::to_string(&dim).unwrap();
            let d2: ConstrainedDimension = serde_json::from_str(&json).unwrap();
            assert_eq!(dim, d2);
        }
    }

    // --- as_str / parse round-trip ---

    #[test]
    fn quota_pressure_as_str_parse_roundtrip() {
        for pressure in [
            QuotaPressure::Healthy,
            QuotaPressure::Warning,
            QuotaPressure::Critical,
            QuotaPressure::Exhausted,
        ] {
            let s = pressure.as_str();
            let p2 = QuotaPressure::parse(s).expect("parse must succeed for valid as_str output");
            assert_eq!(pressure, p2);
        }
    }

    #[test]
    fn quota_pressure_parse_unknown_returns_none() {
        assert!(QuotaPressure::parse("unknown").is_none());
        assert!(QuotaPressure::parse("").is_none());
        assert!(QuotaPressure::parse("Healthy").is_none()); // case-sensitive
    }

    #[test]
    fn constrained_dimension_as_str_parse_roundtrip() {
        for dim in [
            ConstrainedDimension::Clusters,
            ConstrainedDimension::Nodes,
            ConstrainedDimension::Vcpu,
            ConstrainedDimension::Ram,
        ] {
            let s = dim.as_str();
            let d2 =
                ConstrainedDimension::parse(s).expect("parse must succeed for valid as_str output");
            assert_eq!(dim, d2);
        }
    }

    #[test]
    fn constrained_dimension_parse_unknown_returns_none() {
        assert!(ConstrainedDimension::parse("unknown").is_none());
        assert!(ConstrainedDimension::parse("").is_none());
        assert!(ConstrainedDimension::parse("Clusters").is_none()); // case-sensitive
    }

    // --- classify_pressure on real QuotaHeadroom from project_headroom ---

    /// AC: classify_pressure on a real project_headroom result at ~50% -> Healthy.
    #[test]
    fn classify_pressure_on_real_headroom_healthy() {
        // quota: 5 cl, 10 nodes, 32 vcpu, 128 ram
        let q = quota("ten_acme");
        // 2 clusters, max 5 nodes/cl (50%), max 16 vcpu/cl (50%), max 64 ram/cl (50%)
        let usages = vec![cluster("cl-1", 5, 16, 64), cluster("cl-2", 3, 8, 32)];
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();
        // clusters: 2/5 = 40%, nodes: 5/10 = 50%, vcpu: 16/32 = 50%, ram: 64/128 = 50%
        assert_eq!(classify_pressure(&h), QuotaPressure::Healthy);
    }

    /// AC: classify_pressure at 100% -> Exhausted.
    #[test]
    fn classify_pressure_on_real_headroom_exhausted() {
        let q = quota("ten_acme"); // 5 cl, 10 nodes, 32 vcpu, 128 ram
        let usages: Vec<_> = (0..5)
            .map(|i| cluster(&format!("cl-{i}"), 10, 32, 128))
            .collect();
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();
        assert_eq!(classify_pressure(&h), QuotaPressure::Exhausted);
    }

    /// AC: classify_pressure at ~75% -> Warning.
    #[test]
    fn classify_pressure_on_real_headroom_warning() {
        // quota: 5 cl, 10 nodes, 32 vcpu, 128 ram
        // Use 4 clusters (80%), 7 nodes (70%), 16 vcpu (50%), 64 ram (50%)
        // max = 80% -> Warning
        let q = quota("ten_acme");
        let usages = vec![
            cluster("cl-1", 7, 16, 64),
            cluster("cl-2", 1, 1, 1),
            cluster("cl-3", 1, 1, 1),
            cluster("cl-4", 1, 1, 1),
        ];
        let summary = aggregate_usage(tid("ten_acme"), &usages);
        let h = project_headroom(&q, &summary).unwrap();
        // clusters: 4/5 = 80%, nodes: 7/10 = 70% -> max is 80% -> Warning
        assert_eq!(classify_pressure(&h), QuotaPressure::Warning);
    }
}
