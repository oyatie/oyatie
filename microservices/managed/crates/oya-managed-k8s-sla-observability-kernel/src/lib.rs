//! Managed-Kubernetes SLA observability kernel.
//!
//! Pure, deterministic golden-signal calculations for ADR-0376 managed
//! clusters. The kernel knows nothing about kube-rs, Prometheus, HTTP, or the
//! control-plane-host adapter. It accepts normalized observations and returns
//! stable DTO-shaped summaries for availability, provisioning latency, and
//! error-budget burn.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;

use serde::{Deserialize, Serialize};

/// Availability target and provisioning latency thresholds for one SLA window.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaPolicy {
    /// Availability objective in basis points (9990 = 99.90%).
    pub availability_target_basis_points: u16, // data_class: INTERNAL_ONLY
    /// Provisioning latency objective in milliseconds.
    pub provisioning_latency_target_millis: u64, // data_class: INTERNAL_ONLY
}

impl SlaPolicy {
    /// Default managed-K8s preview policy: 99.9% availability and 10-minute
    /// provisioning latency objective.
    #[must_use]
    pub const fn preview_default() -> Self {
        Self {
            availability_target_basis_points: 9_990,
            provisioning_latency_target_millis: 600_000,
        }
    }

    /// Validate policy bounds.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.availability_target_basis_points < 10_000
            && self.provisioning_latency_target_millis > 0
    }
}

impl Default for SlaPolicy {
    fn default() -> Self {
        Self::preview_default()
    }
}

/// Normalized lifecycle status from the control-plane-host seam.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedControlPlaneStatus {
    Requested,
    DatastoreBound,
    MediaFormed,
    Provisioning,
    EndpointReady,
    Active,
    Draining,
    Deleted,
    Failed,
}

impl ObservedControlPlaneStatus {
    /// Stable slug matching the control-plane-host status seam.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::DatastoreBound => "datastore_bound",
            Self::MediaFormed => "media_formed",
            Self::Provisioning => "provisioning",
            Self::EndpointReady => "endpoint_ready",
            Self::Active => "active",
            Self::Draining => "draining",
            Self::Deleted => "deleted",
            Self::Failed => "failed",
        }
    }

    /// Parse a stable slug; unknown values fail closed.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "datastore_bound" => Some(Self::DatastoreBound),
            "media_formed" => Some(Self::MediaFormed),
            "provisioning" => Some(Self::Provisioning),
            "endpoint_ready" => Some(Self::EndpointReady),
            "active" => Some(Self::Active),
            "draining" => Some(Self::Draining),
            "deleted" => Some(Self::Deleted),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    /// Whether the current point-in-time status is fully serving.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether the current point-in-time status is known but degraded rather
    /// than hard down.
    #[must_use]
    pub const fn is_degraded(&self) -> bool {
        matches!(
            self,
            Self::Requested
                | Self::DatastoreBound
                | Self::MediaFormed
                | Self::Provisioning
                | Self::EndpointReady
                | Self::Draining
        )
    }
}

impl fmt::Display for ObservedControlPlaneStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Current availability state for the cluster.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityState {
    Available,
    Degraded,
    Unavailable,
}

impl AvailabilityState {
    /// Stable slug for deterministic DTOs.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Provisioning latency SLO state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningLatencyState {
    Met,
    Breached,
    NotObserved,
}

impl ProvisioningLatencyState {
    /// Stable slug for deterministic DTOs.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::Breached => "breached",
            Self::NotObserved => "not_observed",
        }
    }
}

/// Normalized observation for one tenant cluster and one SLA window.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaObservation {
    /// Tenant that owns the cluster.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Tenant-unique cluster name.
    pub cluster_name: String, // data_class: TENANT_SCOPED
    /// Current control-plane lifecycle status.
    pub control_plane_status: ObservedControlPlaneStatus, // data_class: TENANT_SCOPED
    /// Total status samples in the window.
    pub total_status_samples: u64, // data_class: INTERNAL_ONLY
    /// Healthy/available samples in the window.
    pub healthy_status_samples: u64, // data_class: INTERNAL_ONLY
    /// Observed provisioning latency, if the cluster has completed provisioning.
    pub provisioning_latency_millis: Option<u64>, // data_class: INTERNAL_ONLY
}

impl SlaObservation {
    /// Construct an observation.
    #[must_use]
    pub fn new(
        tenant_id: impl Into<String>,
        cluster_name: impl Into<String>,
        control_plane_status: ObservedControlPlaneStatus,
        total_status_samples: u64,
        healthy_status_samples: u64,
        provisioning_latency_millis: Option<u64>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            cluster_name: cluster_name.into(),
            control_plane_status,
            total_status_samples,
            healthy_status_samples,
            provisioning_latency_millis,
        }
    }
}

/// Deterministic availability summary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AvailabilitySummary {
    pub state: AvailabilityState,    // data_class: INTERNAL_ONLY
    pub observed_basis_points: u16,  // data_class: INTERNAL_ONLY
    pub target_basis_points: u16,    // data_class: INTERNAL_ONLY
    pub healthy_status_samples: u64, // data_class: INTERNAL_ONLY
    pub total_status_samples: u64,   // data_class: INTERNAL_ONLY
}

/// Deterministic provisioning-latency summary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvisioningLatencySummary {
    pub state: ProvisioningLatencyState, // data_class: INTERNAL_ONLY
    pub observed_millis: Option<u64>,    // data_class: INTERNAL_ONLY
    pub target_millis: u64,              // data_class: INTERNAL_ONLY
}

/// Deterministic error-budget summary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorBudgetSummary {
    pub allowed_bad_basis_points: u16,   // data_class: INTERNAL_ONLY
    pub observed_bad_basis_points: u16,  // data_class: INTERNAL_ONLY
    pub remaining_bad_basis_points: i32, // data_class: INTERNAL_ONLY
    /// 10_000 = 1.0x allowed budget, 20_000 = 2.0x burn.
    pub burn_rate_basis_points: u32, // data_class: INTERNAL_ONLY
    pub exhausted: bool,                 // data_class: INTERNAL_ONLY
}

/// Stable golden-signal DTO returned by the bounded context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlaSummary {
    pub tenant_id: String,    // data_class: TENANT_SCOPED
    pub cluster_name: String, // data_class: TENANT_SCOPED
    pub cluster_slug: String, // data_class: TENANT_SCOPED
    pub control_plane_status: ObservedControlPlaneStatus, // data_class: TENANT_SCOPED
    pub availability: AvailabilitySummary, // data_class: INTERNAL_ONLY
    pub provisioning_latency: ProvisioningLatencySummary, // data_class: INTERNAL_ONLY
    pub error_budget: ErrorBudgetSummary, // data_class: INTERNAL_ONLY
}

/// Typed kernel errors. Unknown/malformed inputs fail closed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlaKernelError {
    InvalidPolicy,
    InvalidClusterIdentity,
    EmptyObservationWindow,
    HealthySamplesExceedTotal,
}

impl fmt::Display for SlaKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy => f.write_str("invalid SLA policy"),
            Self::InvalidClusterIdentity => f.write_str("invalid cluster identity"),
            Self::EmptyObservationWindow => f.write_str("empty SLA observation window"),
            Self::HealthySamplesExceedTotal => {
                f.write_str("healthy status samples exceed total status samples")
            }
        }
    }
}

impl std::error::Error for SlaKernelError {}

/// Compute a deterministic golden-signal SLA summary.
///
/// # Errors
/// Returns a typed error for malformed policy, cluster identity, or sample
/// counts. The function never panics on caller-supplied values.
pub fn summarize_sla(
    observation: &SlaObservation,
    policy: SlaPolicy,
) -> Result<SlaSummary, SlaKernelError> {
    if !policy.is_valid() {
        return Err(SlaKernelError::InvalidPolicy);
    }
    if observation.tenant_id.trim().is_empty() || observation.cluster_name.trim().is_empty() {
        return Err(SlaKernelError::InvalidClusterIdentity);
    }
    if observation.total_status_samples == 0 {
        return Err(SlaKernelError::EmptyObservationWindow);
    }
    if observation.healthy_status_samples > observation.total_status_samples {
        return Err(SlaKernelError::HealthySamplesExceedTotal);
    }

    let observed_basis_points = basis_points(
        observation.healthy_status_samples,
        observation.total_status_samples,
    );
    let observed_bad_basis_points = 10_000u16.saturating_sub(observed_basis_points);
    let allowed_bad_basis_points = 10_000u16 - policy.availability_target_basis_points;
    let remaining_bad_basis_points =
        i32::from(allowed_bad_basis_points) - i32::from(observed_bad_basis_points);
    let burn_rate_basis_points = if allowed_bad_basis_points == 0 {
        0
    } else {
        (u32::from(observed_bad_basis_points) * 10_000) / u32::from(allowed_bad_basis_points)
    };

    Ok(SlaSummary {
        tenant_id: observation.tenant_id.clone(),
        cluster_name: observation.cluster_name.clone(),
        cluster_slug: cluster_slug(&observation.tenant_id, &observation.cluster_name),
        control_plane_status: observation.control_plane_status,
        availability: AvailabilitySummary {
            state: availability_state(observation.control_plane_status),
            observed_basis_points,
            target_basis_points: policy.availability_target_basis_points,
            healthy_status_samples: observation.healthy_status_samples,
            total_status_samples: observation.total_status_samples,
        },
        provisioning_latency: provisioning_latency_summary(
            observation.provisioning_latency_millis,
            policy.provisioning_latency_target_millis,
        ),
        error_budget: ErrorBudgetSummary {
            allowed_bad_basis_points,
            observed_bad_basis_points,
            remaining_bad_basis_points,
            burn_rate_basis_points,
            exhausted: remaining_bad_basis_points < 0,
        },
    })
}

fn basis_points(numerator: u64, denominator: u64) -> u16 {
    // Rounded down deliberately: deterministic, conservative availability.
    let bps = (u128::from(numerator) * 10_000) / u128::from(denominator);
    u16::try_from(bps).unwrap_or(10_000)
}

fn availability_state(status: ObservedControlPlaneStatus) -> AvailabilityState {
    if status.is_available() {
        AvailabilityState::Available
    } else if status.is_degraded() {
        AvailabilityState::Degraded
    } else {
        AvailabilityState::Unavailable
    }
}

fn provisioning_latency_summary(
    observed_millis: Option<u64>,
    target_millis: u64,
) -> ProvisioningLatencySummary {
    let state = match observed_millis {
        Some(observed) if observed <= target_millis => ProvisioningLatencyState::Met,
        Some(_) => ProvisioningLatencyState::Breached,
        None => ProvisioningLatencyState::NotObserved,
    };
    ProvisioningLatencySummary {
        state,
        observed_millis,
        target_millis,
    }
}

/// Stable tenant/cluster slug for URLs, metrics labels, and evidence.
#[must_use]
pub fn cluster_slug(tenant_id: &str, cluster_name: &str) -> String {
    format!("{}--{}", slug_part(tenant_id), slug_part(cluster_name))
}

fn slug_part(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.trim().chars().flat_map(char::to_lowercase) {
        let keep = ch.is_ascii_alphanumeric();
        if keep {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_cluster_is_available_with_budget_remaining() {
        let obs = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            Some(120_000),
        );
        let summary = summarize_sla(&obs, SlaPolicy::default()).unwrap();
        assert_eq!(summary.availability.state, AvailabilityState::Available);
        assert_eq!(summary.availability.observed_basis_points, 10_000);
        assert_eq!(
            summary.provisioning_latency.state,
            ProvisioningLatencyState::Met
        );
        assert!(!summary.error_budget.exhausted);
    }

    #[test]
    fn degraded_cluster_burns_error_budget() {
        let obs = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Failed,
            100,
            80,
            Some(900_000),
        );
        let summary = summarize_sla(&obs, SlaPolicy::default()).unwrap();
        assert_eq!(summary.availability.state, AvailabilityState::Unavailable);
        assert_eq!(summary.availability.observed_basis_points, 8_000);
        assert_eq!(
            summary.provisioning_latency.state,
            ProvisioningLatencyState::Breached
        );
        assert!(summary.error_budget.exhausted);
        assert_eq!(summary.error_budget.burn_rate_basis_points, 2_000_000);
    }

    #[test]
    fn large_sample_windows_do_not_saturate_availability() {
        let obs = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            u64::MAX,
            u64::MAX,
            Some(120_000),
        );
        let summary = summarize_sla(&obs, SlaPolicy::default()).unwrap();
        assert_eq!(summary.availability.observed_basis_points, 10_000);
        assert_eq!(summary.error_budget.observed_bad_basis_points, 0);
    }

    #[test]
    fn invalid_observation_fails_closed() {
        let obs = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            10,
            11,
            None,
        );
        assert_eq!(
            summarize_sla(&obs, SlaPolicy::default()).unwrap_err(),
            SlaKernelError::HealthySamplesExceedTotal
        );
    }

    #[test]
    fn slugs_and_serialization_are_deterministic() {
        let obs = SlaObservation::new(
            "TEN Acme!",
            "Prod_A.01",
            ObservedControlPlaneStatus::EndpointReady,
            4,
            3,
            None,
        );
        let summary = summarize_sla(&obs, SlaPolicy::default()).unwrap();
        assert_eq!(summary.cluster_slug, "ten-acme--prod-a-01");
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"control_plane_status\":\"endpoint_ready\""));
        assert!(json.contains("\"state\":\"degraded\""));
        let back: SlaSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
    }
}
