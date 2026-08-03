//! Managed-Kubernetes SLA observability API / port layer.
//!
//! This crate is the transport-neutral seam for the SLA observability bounded
//! context. It owns only SLA DTOs and an object-safe observation port; the app
//! layer performs any adaptation from the sibling control-plane-host API seam.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;
use core::pin::Pin;

use serde::{Deserialize, Serialize};

pub use oya_managed_k8s_sla_observability_kernel::{
    AlertSeverity, AlertVerdict, AvailabilityState, BurnRatePolicy, BurnRateWindowPolicy,
    ErrorBudgetSummary, FleetSlaSummary, ObservedControlPlaneStatus, ProvisioningLatencyState,
    ProvisioningLatencySummary, SlaKernelError, SlaObservation, SlaPolicy, SlaSummary,
    summarize_burn_rate_alert, summarize_fleet_sla, summarize_sla,
};

/// A short alias for the boxed, `Send` future every async port method returns.
pub type BoxFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + Send + 'a>>;

/// Current OpenSLO path for managed-cluster availability evidence.
pub const MANAGED_CLUSTER_AVAILABILITY_OPENSLO_PATH: &str =
    "cloud/managed-k8s-sla-observability/slos/managed-cluster-availability.openslo.yaml";

/// Current OpenSLO path for provisioning-latency evidence.
pub const PROVISIONING_LATENCY_OPENSLO_PATH: &str =
    "cloud/managed-k8s-sla-observability/slos/provisioning-latency.openslo.yaml";

/// Stable, tenant-scoped reference to the managed Kubernetes cluster whose SLA
/// is being summarized.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ClusterRef {
    /// Tenant that owns the cluster.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Tenant-unique cluster name.
    pub cluster_name: String, // data_class: TENANT_SCOPED
}

impl ClusterRef {
    #[must_use]
    pub fn new(tenant_id: impl Into<String>, cluster_name: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            cluster_name: cluster_name.into(),
        }
    }

    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        !self.tenant_id.trim().is_empty() && !self.cluster_name.trim().is_empty()
    }
}

impl fmt::Display for ClusterRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.tenant_id, self.cluster_name)
    }
}

/// Observation-store errors. Unknown/missing clusters fail closed here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlaObservabilityError {
    /// Empty tenant or cluster name.
    InvalidClusterRef { cluster_ref: String },
    /// No observation is known for the requested cluster.
    UnknownCluster { cluster_ref: String },
    /// Caller scope does not match the requested tenant. Checked before loading summaries.
    TenantScopeDenied {
        requested_tenant_id: String,
        scope_tenant_id: String,
    },
    /// Live evidence is malformed before it can be normalized into a snapshot.
    InvalidEvidence { detail: String },
    /// The observation window is missing samples and must not synthesize green evidence.
    MissingEvidence {
        cluster_ref: String,
        source: SlaEvidenceSource,
    },
    /// The evidence freshness deadline has expired and must not remain visible as green.
    StaleEvidence {
        cluster_ref: String,
        observed_at_unix_millis: u64,
        freshness_deadline_unix_millis: u64,
        evaluation_time_unix_millis: u64,
    },
    /// Kernel rejected the observation/policy.
    Kernel(SlaKernelError),
    /// Downstream control-plane-host read failed; detail is normalized by the app
    /// layer so the API crate does not depend on a sibling API crate.
    ControlPlane { detail: String }, // data_class: INTERNAL_ONLY
    /// The observation store failed internally (for example, poisoned in-memory lock).
    Store { detail: String }, // data_class: INTERNAL_ONLY
}

impl fmt::Display for SlaObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidClusterRef { cluster_ref } => {
                write!(f, "invalid cluster reference: {cluster_ref}")
            }
            Self::UnknownCluster { cluster_ref } => {
                write!(f, "SLA observation not found for cluster {cluster_ref}")
            }
            Self::TenantScopeDenied {
                requested_tenant_id,
                scope_tenant_id,
            } => write!(
                f,
                "tenant scope {scope_tenant_id} cannot read SLA evidence for tenant {requested_tenant_id}"
            ),
            Self::InvalidEvidence { detail } => write!(f, "invalid SLA evidence: {detail}"),
            Self::MissingEvidence {
                cluster_ref,
                source,
            } => write!(
                f,
                "missing SLA evidence samples for cluster {cluster_ref} from {}",
                source.as_str()
            ),
            Self::StaleEvidence {
                cluster_ref,
                observed_at_unix_millis,
                freshness_deadline_unix_millis,
                evaluation_time_unix_millis,
            } => write!(
                f,
                "stale SLA evidence for cluster {cluster_ref}: observed_at={observed_at_unix_millis}, fresh_until={freshness_deadline_unix_millis}, evaluated_at={evaluation_time_unix_millis}"
            ),
            Self::Kernel(err) => write!(f, "SLA kernel error: {err}"),
            Self::ControlPlane { detail } => {
                write!(f, "control-plane status read failed: {detail}")
            }
            Self::Store { detail } => write!(f, "SLA observation store failed: {detail}"),
        }
    }
}

impl std::error::Error for SlaObservabilityError {}

impl From<SlaKernelError> for SlaObservabilityError {
    fn from(value: SlaKernelError) -> Self {
        Self::Kernel(value)
    }
}

/// Status-window DTO accepted by `ingest_status_snapshot`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatusWindow {
    pub total_status_samples: u64,   // data_class: INTERNAL_ONLY
    pub healthy_status_samples: u64, // data_class: INTERNAL_ONLY
}

impl StatusWindow {
    #[must_use]
    pub const fn new(total_status_samples: u64, healthy_status_samples: u64) -> Self {
        Self {
            total_status_samples,
            healthy_status_samples,
        }
    }

    /// Whether the window carries no samples and must be treated as missing evidence.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total_status_samples == 0
    }

    /// Whether the window cannot be summarized because healthy samples exceed total samples.
    #[must_use]
    pub const fn healthy_samples_exceed_total(&self) -> bool {
        self.healthy_status_samples > self.total_status_samples
    }
}

/// Snapshot DTO for ingestion from control-plane-host status reads or tests.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlPlaneSlaSnapshot {
    pub cluster_ref: ClusterRef, // data_class: TENANT_SCOPED
    pub control_plane_status: ObservedControlPlaneStatus, // data_class: TENANT_SCOPED
    pub status_window: StatusWindow, // data_class: INTERNAL_ONLY
    pub provisioning_latency_millis: Option<u64>, // data_class: INTERNAL_ONLY
}

impl ControlPlaneSlaSnapshot {
    /// Construct a snapshot directly.
    #[must_use]
    pub fn new(
        cluster_ref: ClusterRef,
        control_plane_status: ObservedControlPlaneStatus,
        status_window: StatusWindow,
        provisioning_latency_millis: Option<u64>,
    ) -> Self {
        Self {
            cluster_ref,
            control_plane_status,
            status_window,
            provisioning_latency_millis,
        }
    }

    /// Borrow-convert into the pure kernel observation shape.
    #[must_use]
    pub fn to_observation(&self) -> SlaObservation {
        SlaObservation::new(
            self.cluster_ref.tenant_id.clone(),
            self.cluster_ref.cluster_name.clone(),
            self.control_plane_status,
            self.status_window.total_status_samples,
            self.status_window.healthy_status_samples,
            self.provisioning_latency_millis,
        )
    }

    /// Convert into the pure kernel observation shape.
    #[must_use]
    pub fn into_observation(self) -> SlaObservation {
        SlaObservation::new(
            self.cluster_ref.tenant_id,
            self.cluster_ref.cluster_name,
            self.control_plane_status,
            self.status_window.total_status_samples,
            self.status_window.healthy_status_samples,
            self.provisioning_latency_millis,
        )
    }
}

/// Source class for normalized SLA evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaEvidenceSource {
    /// Evidence read through the sibling `ControlPlaneProvisioning::status` port.
    ControlPlaneStatusPort,
    /// Evidence normalized from Kubernetes watch state.
    KubernetesWatch,
    /// Evidence normalized from Prometheus query results.
    PrometheusQuery,
}

impl SlaEvidenceSource {
    /// Stable snake_case slug for evidence transcripts.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ControlPlaneStatusPort => "control_plane_status_port",
            Self::KubernetesWatch => "kubernetes_watch",
            Self::PrometheusQuery => "prometheus_query",
        }
    }
}

/// Wall-clock-free observation window carried by live evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservationWindow {
    pub start_unix_millis: u64, // data_class: INTERNAL_ONLY
    pub end_unix_millis: u64,   // data_class: INTERNAL_ONLY
}

impl ObservationWindow {
    /// Construct an evidence window with explicit start and end instants.
    #[must_use]
    pub const fn new(start_unix_millis: u64, end_unix_millis: u64) -> Self {
        Self {
            start_unix_millis,
            end_unix_millis,
        }
    }

    fn validate(&self) -> Result<(), SlaObservabilityError> {
        if self.end_unix_millis <= self.start_unix_millis {
            return Err(SlaObservabilityError::InvalidEvidence {
                detail: "observation window end must be after start".to_string(),
            });
        }
        Ok(())
    }
}

/// Claim ceiling attached to an evidence transcript.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaEvidenceClaimCeiling {
    /// Target/local evidence only; not measured production SLO evidence.
    TargetOnly,
    /// Measured development or test-environment evidence.
    MeasuredDevelopment,
    /// Measured production evidence after collector proof and review.
    MeasuredProduction,
}

/// Evidence collection and freshness timing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaEvidenceFreshness {
    pub observed_at_unix_millis: u64,        // data_class: INTERNAL_ONLY
    pub freshness_deadline_unix_millis: u64, // data_class: INTERNAL_ONLY
}

impl SlaEvidenceFreshness {
    /// Construct evidence freshness timing.
    #[must_use]
    pub const fn new(observed_at_unix_millis: u64, freshness_deadline_unix_millis: u64) -> Self {
        Self {
            observed_at_unix_millis,
            freshness_deadline_unix_millis,
        }
    }
}

/// Region/cell placement metadata for an evidence record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaEvidencePlacement {
    pub region: String,                     // data_class: TENANT_SCOPED
    pub cell: String,                       // data_class: TENANT_SCOPED
    pub control_plane_tier: Option<String>, // data_class: INTERNAL_ONLY
}

impl SlaEvidencePlacement {
    /// Construct evidence placement metadata.
    #[must_use]
    pub fn new(
        region: impl Into<String>,
        cell: impl Into<String>,
        control_plane_tier: Option<impl Into<String>>,
    ) -> Self {
        Self {
            region: region.into(),
            cell: cell.into(),
            control_plane_tier: control_plane_tier.map(Into::into),
        }
    }
}

/// Adapter-facing metadata that makes an SLA snapshot auditable.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaEvidenceContext {
    pub source: SlaEvidenceSource,           // data_class: INTERNAL_ONLY
    pub collector_id: String,                // data_class: INTERNAL_ONLY
    pub window: ObservationWindow,           // data_class: INTERNAL_ONLY
    pub observed_at_unix_millis: u64,        // data_class: INTERNAL_ONLY
    pub freshness_deadline_unix_millis: u64, // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: TENANT_SCOPED
    pub cell: String,                        // data_class: TENANT_SCOPED
    pub control_plane_tier: Option<String>,  // data_class: INTERNAL_ONLY
    pub availability_openslo_path: String,   // data_class: INTERNAL_ONLY
    pub provisioning_latency_openslo_path: String, // data_class: INTERNAL_ONLY
    pub evidence_handle: Option<String>,     // data_class: INTERNAL_ONLY
    pub claim_ceiling: SlaEvidenceClaimCeiling, // data_class: INTERNAL_ONLY
}

impl SlaEvidenceContext {
    /// Construct managed-k8s SLA evidence metadata with current OpenSLO authority paths.
    #[must_use]
    pub fn new(
        source: SlaEvidenceSource,
        collector_id: impl Into<String>,
        window: ObservationWindow,
        freshness: SlaEvidenceFreshness,
        placement: SlaEvidencePlacement,
    ) -> Self {
        Self {
            source,
            collector_id: collector_id.into(),
            window,
            observed_at_unix_millis: freshness.observed_at_unix_millis,
            freshness_deadline_unix_millis: freshness.freshness_deadline_unix_millis,
            region: placement.region,
            cell: placement.cell,
            control_plane_tier: placement.control_plane_tier,
            availability_openslo_path: MANAGED_CLUSTER_AVAILABILITY_OPENSLO_PATH.to_string(),
            provisioning_latency_openslo_path: PROVISIONING_LATENCY_OPENSLO_PATH.to_string(),
            evidence_handle: None,
            claim_ceiling: SlaEvidenceClaimCeiling::TargetOnly,
        }
    }

    /// Validate metadata and freshness against the caller-supplied evaluation instant.
    pub fn validate_at(
        &self,
        cluster_ref: &ClusterRef,
        evaluation_time_unix_millis: u64,
    ) -> Result<(), SlaObservabilityError> {
        self.window.validate()?;
        if self.collector_id.trim().is_empty() {
            return Err(SlaObservabilityError::InvalidEvidence {
                detail: "collector_id must be present".to_string(),
            });
        }
        if self.region.trim().is_empty() || self.cell.trim().is_empty() {
            return Err(SlaObservabilityError::InvalidEvidence {
                detail: "region and cell must be present".to_string(),
            });
        }
        if self.observed_at_unix_millis < self.window.start_unix_millis
            || self.observed_at_unix_millis > self.window.end_unix_millis
        {
            return Err(SlaObservabilityError::InvalidEvidence {
                detail: "observed_at must be inside the observation window".to_string(),
            });
        }
        if self.freshness_deadline_unix_millis < self.observed_at_unix_millis {
            return Err(SlaObservabilityError::InvalidEvidence {
                detail: "freshness deadline must not precede observed_at".to_string(),
            });
        }
        if self.availability_openslo_path != MANAGED_CLUSTER_AVAILABILITY_OPENSLO_PATH
            || self.provisioning_latency_openslo_path != PROVISIONING_LATENCY_OPENSLO_PATH
        {
            return Err(SlaObservabilityError::InvalidEvidence {
                detail: "OpenSLO evidence paths must use managed-k8s SLA authority paths"
                    .to_string(),
            });
        }
        if evaluation_time_unix_millis > self.freshness_deadline_unix_millis {
            return Err(SlaObservabilityError::StaleEvidence {
                cluster_ref: cluster_ref.to_string(),
                observed_at_unix_millis: self.observed_at_unix_millis,
                freshness_deadline_unix_millis: self.freshness_deadline_unix_millis,
                evaluation_time_unix_millis,
            });
        }
        Ok(())
    }
}

/// Fast and slow observation windows used to derive MWMB alert evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BurnRateEvidenceWindows {
    pub fast: StatusWindow, // data_class: INTERNAL_ONLY
    pub slow: StatusWindow, // data_class: INTERNAL_ONLY
}

impl BurnRateEvidenceWindows {
    /// Construct fast/slow MWMB evidence windows.
    #[must_use]
    pub const fn new(fast: StatusWindow, slow: StatusWindow) -> Self {
        Self { fast, slow }
    }

    /// Validate both windows before alert verdict generation.
    pub fn validate(
        &self,
        cluster_ref: &ClusterRef,
        source: SlaEvidenceSource,
    ) -> Result<(), SlaObservabilityError> {
        if self.fast.is_empty() || self.slow.is_empty() {
            return Err(SlaObservabilityError::MissingEvidence {
                cluster_ref: cluster_ref.to_string(),
                source,
            });
        }
        Ok(())
    }
}

/// Normalized live input before it is reduced to an effective SLA snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaLiveObservation {
    pub cluster_ref: ClusterRef, // data_class: TENANT_SCOPED
    pub control_plane_status: ObservedControlPlaneStatus, // data_class: TENANT_SCOPED
    pub live_status: Option<ObservedControlPlaneStatus>, // data_class: TENANT_SCOPED
    pub status_window: StatusWindow, // data_class: INTERNAL_ONLY
    pub provisioning_latency_millis: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence: SlaEvidenceContext, // data_class: INTERNAL_ONLY
}

impl SlaLiveObservation {
    /// Construct a normalized live observation input.
    #[must_use]
    pub fn new(
        cluster_ref: ClusterRef,
        control_plane_status: ObservedControlPlaneStatus,
        live_status: Option<ObservedControlPlaneStatus>,
        status_window: StatusWindow,
        provisioning_latency_millis: Option<u64>,
        evidence: SlaEvidenceContext,
    ) -> Self {
        Self {
            cluster_ref,
            control_plane_status,
            live_status,
            status_window,
            provisioning_latency_millis,
            evidence,
        }
    }

    /// Validate freshness and missing-evidence invariants before ingestion.
    pub fn validate_at(
        &self,
        evaluation_time_unix_millis: u64,
    ) -> Result<(), SlaObservabilityError> {
        validate_cluster_ref(&self.cluster_ref)?;
        self.evidence
            .validate_at(&self.cluster_ref, evaluation_time_unix_millis)?;
        if self.status_window.is_empty() {
            return Err(SlaObservabilityError::MissingEvidence {
                cluster_ref: self.cluster_ref.to_string(),
                source: self.evidence.source,
            });
        }
        Ok(())
    }

    /// The lower-claim/higher-risk status used for summary and alert evidence.
    #[must_use]
    pub fn effective_status(&self) -> ObservedControlPlaneStatus {
        match self.live_status {
            Some(live) if status_risk_rank(live) > status_risk_rank(self.control_plane_status) => {
                live
            }
            _ => self.control_plane_status,
        }
    }

    /// Whether the sibling CPH status and live observation disagree.
    #[must_use]
    pub fn has_status_disagreement(&self) -> bool {
        self.live_status
            .map(|live| live != self.control_plane_status)
            .unwrap_or(false)
    }

    /// Convert to the effective control-plane SLA snapshot.
    #[must_use]
    pub fn effective_snapshot(&self) -> ControlPlaneSlaSnapshot {
        ControlPlaneSlaSnapshot::new(
            self.cluster_ref.clone(),
            self.effective_status(),
            self.status_window.clone(),
            self.provisioning_latency_millis,
        )
    }
}

/// Tenant read scope for summary/evidence APIs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TenantSlaReadScope {
    pub tenant_id: String, // data_class: TENANT_SCOPED
}

impl TenantSlaReadScope {
    /// Construct a tenant read scope.
    #[must_use]
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
        }
    }

    /// Default-deny authorization check before loading summaries.
    pub fn authorize_cluster_read(
        &self,
        cluster_ref: &ClusterRef,
    ) -> Result<(), SlaObservabilityError> {
        validate_cluster_ref(cluster_ref)?;
        if self.tenant_id.trim().is_empty() || self.tenant_id != cluster_ref.tenant_id {
            return Err(SlaObservabilityError::TenantScopeDenied {
                requested_tenant_id: cluster_ref.tenant_id.clone(),
                scope_tenant_id: self.tenant_id.clone(),
            });
        }
        Ok(())
    }
}

/// Rollback/hold signals derived from evidence, not recomputed by a web layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackHoldReason {
    StatusDisagreement,
    MwmbPage,
    MwmbTicket,
    ProvisioningLatencyBreach,
    AvailabilityNotGreen,
}

/// Structured evidence emitted by a live ingestion pass.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaEvidenceEmission {
    pub snapshot: ControlPlaneSlaSnapshot, // data_class: TENANT_SCOPED
    pub summary: SlaSummary,               // data_class: TENANT_SCOPED
    pub alert_verdict: AlertVerdict,       // data_class: INTERNAL_ONLY
    pub evidence: SlaEvidenceContext,      // data_class: INTERNAL_ONLY
    pub rollback_holds: Vec<RollbackHoldReason>, // data_class: INTERNAL_ONLY
}

fn status_risk_rank(status: ObservedControlPlaneStatus) -> u8 {
    if status.is_available() {
        0
    } else if status.is_degraded() {
        1
    } else {
        2
    }
}

/// Object-safe SLA observability port.
pub trait SlaObservabilityPort: Send + Sync {
    /// Ingest or replace the latest SLA snapshot for a cluster.
    fn ingest_status_snapshot<'a>(
        &'a self,
        snapshot: ControlPlaneSlaSnapshot,
    ) -> BoxFuture<'a, Result<SlaSummary, SlaObservabilityError>>;

    /// Read a deterministic SLA summary for a cluster.
    fn summarize_cluster<'a>(
        &'a self,
        cluster_ref: &'a ClusterRef,
    ) -> BoxFuture<'a, Result<SlaSummary, SlaObservabilityError>>;
}

/// Fail-closed validation shared by adapters/apps.
pub fn validate_cluster_ref(cluster_ref: &ClusterRef) -> Result<(), SlaObservabilityError> {
    if cluster_ref.is_well_formed() {
        Ok(())
    } else {
        Err(SlaObservabilityError::InvalidClusterRef {
            cluster_ref: cluster_ref.to_string(),
        })
    }
}

/// Collision-free map key for cluster observations.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ClusterKey {
    tenant_id: String,    // data_class: TENANT_SCOPED
    cluster_name: String, // data_class: TENANT_SCOPED
}

impl ClusterKey {
    /// Build a key from a validated cluster reference.
    #[must_use]
    pub fn from_ref(cluster_ref: &ClusterRef) -> Self {
        Self {
            tenant_id: cluster_ref.tenant_id.clone(),
            cluster_name: cluster_ref.cluster_name.clone(),
        }
    }
}

impl fmt::Display for ClusterKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.tenant_id, self.cluster_name)
    }
}

/// Stable display key for diagnostics only. Storage must use [`ClusterKey`].
#[must_use]
pub fn cluster_key(cluster_ref: &ClusterRef) -> String {
    ClusterKey::from_ref(cluster_ref).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_serialization_is_deterministic() {
        let snapshot = ControlPlaneSlaSnapshot::new(
            ClusterRef::new("ten_acme", "prod-a"),
            ObservedControlPlaneStatus::Failed,
            StatusWindow::new(10, 8),
            None,
        );
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"control_plane_status\":\"failed\""));
        let back: ControlPlaneSlaSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, back);
    }

    #[test]
    fn invalid_cluster_ref_fails_closed() {
        let err = validate_cluster_ref(&ClusterRef::new("", "prod-a")).unwrap_err();
        assert!(matches!(
            err,
            SlaObservabilityError::InvalidClusterRef { .. }
        ));
    }
}
