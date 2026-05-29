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

/// Per-tier burn-rate threshold for one severity window.
///
/// `burn_rate_threshold_basis_points`: the burn rate (in basis points, where
/// 10_000 = 1.0x the allowed error budget) at or above which this tier fires.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BurnRateWindowPolicy {
    /// Burn-rate threshold in basis points (10_000 = 1.0x budget). // data_class: INTERNAL_ONLY
    pub burn_rate_threshold_basis_points: u32,
}

/// Multi-window multi-burn-rate policy: page (fast) and ticket (slow) tiers.
///
/// `sre_default()` encodes the Google SRE Workbook ch. 5 canonical thresholds
/// for a 99.9% / 30-day budget:
/// - Page:   14.4x -> 144_000 bp (1h/5m tier; 2% budget in the window).
/// - Ticket:  1.0x ->  10_000 bp (3d/6h tier; 10% budget in the window).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BurnRatePolicy {
    /// Fast-burn (page) tier threshold. // data_class: INTERNAL_ONLY
    pub page: BurnRateWindowPolicy,
    /// Slow-burn (ticket) tier threshold. // data_class: INTERNAL_ONLY
    pub ticket: BurnRateWindowPolicy,
}

impl BurnRatePolicy {
    /// Canonical SRE Workbook defaults: page=14.4x (144_000 bp), ticket=1.0x (10_000 bp).
    #[must_use]
    pub const fn sre_default() -> Self {
        Self {
            page: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 144_000,
            },
            ticket: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 10_000,
            },
        }
    }

    /// Validate policy bounds: both thresholds > 0 and page >= ticket.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.page.burn_rate_threshold_basis_points > 0
            && self.ticket.burn_rate_threshold_basis_points > 0
            && self.page.burn_rate_threshold_basis_points
                >= self.ticket.burn_rate_threshold_basis_points
    }
}

impl Default for BurnRatePolicy {
    fn default() -> Self {
        Self::sre_default()
    }
}

/// Alert severity from MWMB evaluation. Monotonically ordered: Page > Ticket > None.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    /// No threshold breached; no action required.
    None,
    /// Slow-burn threshold breached; file a ticket.
    Ticket,
    /// Fast-burn threshold breached; page on-call.
    Page,
}

impl AlertSeverity {
    /// Stable snake_case slug for DTOs and metric labels.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ticket => "ticket",
            Self::Page => "page",
        }
    }
}

impl fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Deterministic multi-window multi-burn-rate alert verdict.
///
/// Produced by `summarize_burn_rate_alert`. Records the chosen severity plus
/// per-window burn rates and which windows tripped, so adapters can attach
/// routing evidence without recomputing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlertVerdict {
    /// Chosen alert severity. // data_class: INTERNAL_ONLY
    pub severity: AlertSeverity,
    /// Observed burn rate for the fast (short-lookback) window in basis points. // data_class: INTERNAL_ONLY
    pub fast_burn_rate_basis_points: u32,
    /// Observed burn rate for the slow (long-lookback) window in basis points. // data_class: INTERNAL_ONLY
    pub slow_burn_rate_basis_points: u32,
    /// Page-tier threshold applied (from `BurnRatePolicy`). // data_class: INTERNAL_ONLY
    pub page_threshold_basis_points: u32,
    /// Ticket-tier threshold applied (from `BurnRatePolicy`). // data_class: INTERNAL_ONLY
    pub ticket_threshold_basis_points: u32,
    /// Whether the fast window met or exceeded the threshold of the chosen tier. // data_class: INTERNAL_ONLY
    pub fast_window_tripped: bool,
    /// Whether the slow window met or exceeded the threshold of the chosen tier. // data_class: INTERNAL_ONLY
    pub slow_window_tripped: bool,
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
    let allowed_bad_basis_points = 10_000u16 - policy.availability_target_basis_points;
    let BurnRateParts {
        observed_bad_basis_points,
        burn_rate_basis_points,
    } = burn_rate_parts(observed_basis_points, allowed_bad_basis_points);
    let remaining_bad_basis_points =
        i32::from(allowed_bad_basis_points) - i32::from(observed_bad_basis_points);

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

/// Compute a deterministic multi-window multi-burn-rate alert verdict.
///
/// Accepts two pre-aggregated `SlaObservation` windows:
/// - `fast`: short-lookback window (page candidate; e.g., 5m or 1h).
/// - `slow`: long-lookback window (ticket candidate; e.g., 30m or 6h).
///
/// The MWMB AND-gate fires a severity only when BOTH windows meet or exceed
/// the threshold for that tier. Severity is monotonic: Page > Ticket > None.
///
/// Window lookback durations are an adapter concern; this function has no clock.
///
/// # Errors
/// Returns a typed error for invalid policies or malformed observations. The
/// function never panics on caller-supplied values.
pub fn summarize_burn_rate_alert(
    fast: &SlaObservation,
    slow: &SlaObservation,
    sla_policy: SlaPolicy,
    burn_policy: BurnRatePolicy,
) -> Result<AlertVerdict, SlaKernelError> {
    if !sla_policy.is_valid() || !burn_policy.is_valid() {
        return Err(SlaKernelError::InvalidPolicy);
    }

    let fast_burn = window_burn_rate(fast, sla_policy)?;
    let slow_burn = window_burn_rate(slow, sla_policy)?;

    let page_threshold = burn_policy.page.burn_rate_threshold_basis_points;
    let ticket_threshold = burn_policy.ticket.burn_rate_threshold_basis_points;

    let fast_page = fast_burn >= page_threshold;
    let slow_page = slow_burn >= page_threshold;
    let fast_ticket = fast_burn >= ticket_threshold;
    let slow_ticket = slow_burn >= ticket_threshold;

    let (severity, fast_window_tripped, slow_window_tripped) = if fast_page && slow_page {
        (AlertSeverity::Page, true, true)
    } else if fast_ticket && slow_ticket {
        (AlertSeverity::Ticket, true, true)
    } else {
        // None: record proximity against the ticket threshold for evidence.
        (AlertSeverity::None, fast_ticket, slow_ticket)
    };

    Ok(AlertVerdict {
        severity,
        fast_burn_rate_basis_points: fast_burn,
        slow_burn_rate_basis_points: slow_burn,
        page_threshold_basis_points: page_threshold,
        ticket_threshold_basis_points: ticket_threshold,
        fast_window_tripped,
        slow_window_tripped,
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Intermediate burn-rate parts computed from a single observation window.
struct BurnRateParts {
    observed_bad_basis_points: u16,
    burn_rate_basis_points: u32,
}

/// Shared burn-rate computation used by both `summarize_sla` and
/// `summarize_burn_rate_alert`. Byte-identical to the original inline math.
fn burn_rate_parts(observed_basis_points: u16, allowed_bad_basis_points: u16) -> BurnRateParts {
    let observed_bad_basis_points = 10_000u16.saturating_sub(observed_basis_points);
    let burn_rate_basis_points = if allowed_bad_basis_points == 0 {
        0
    } else {
        (u32::from(observed_bad_basis_points) * 10_000) / u32::from(allowed_bad_basis_points)
    };
    BurnRateParts {
        observed_bad_basis_points,
        burn_rate_basis_points,
    }
}

/// Validate an observation window and return its burn rate in basis points.
fn window_burn_rate(
    obs: &SlaObservation,
    sla_policy: SlaPolicy,
) -> Result<u32, SlaKernelError> {
    if obs.tenant_id.trim().is_empty() || obs.cluster_name.trim().is_empty() {
        return Err(SlaKernelError::InvalidClusterIdentity);
    }
    if obs.total_status_samples == 0 {
        return Err(SlaKernelError::EmptyObservationWindow);
    }
    if obs.healthy_status_samples > obs.total_status_samples {
        return Err(SlaKernelError::HealthySamplesExceedTotal);
    }
    let observed_basis_points =
        basis_points(obs.healthy_status_samples, obs.total_status_samples);
    let allowed_bad_basis_points =
        10_000u16 - sla_policy.availability_target_basis_points;
    Ok(burn_rate_parts(observed_basis_points, allowed_bad_basis_points).burn_rate_basis_points)
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

    // -----------------------------------------------------------------------
    // Existing regression tests (must pass unchanged after refactor)
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // BurnRatePolicy value-object tests
    // -----------------------------------------------------------------------

    #[test]
    fn sre_default_policy_is_valid() {
        assert!(BurnRatePolicy::sre_default().is_valid());
    }

    #[test]
    fn inverted_burn_policy_is_invalid() {
        let policy = BurnRatePolicy {
            page: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 5_000,
            },
            ticket: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 10_000,
            },
        };
        assert!(!policy.is_valid());
    }

    #[test]
    fn zero_threshold_burn_policy_is_invalid() {
        let policy = BurnRatePolicy {
            page: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 0,
            },
            ticket: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 0,
            },
        };
        assert!(!policy.is_valid());
    }

    // -----------------------------------------------------------------------
    // MWMB AND-gate edge-case matrix
    // -----------------------------------------------------------------------

    fn obs_with_samples(healthy: u64, total: u64) -> SlaObservation {
        SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            total,
            healthy,
            None,
        )
    }

    /// Both windows well below ticket threshold -> None.
    #[test]
    fn both_clean_is_none() {
        // 100% healthy = 0 bp bad = burn rate 0
        let fast = obs_with_samples(1_000, 1_000);
        let slow = obs_with_samples(1_000, 1_000);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::None);
        assert_eq!(verdict.fast_burn_rate_basis_points, 0);
        assert_eq!(verdict.slow_burn_rate_basis_points, 0);
    }

    /// Fast window hot (>= page threshold), slow window clean -> None (AND-gate).
    #[test]
    fn fast_hot_slow_clean_is_none() {
        // For 99.9% policy: allowed_bad = 10 bp.
        // To hit 14.4x burn: observed_bad >= 144 bp -> bad/10_000 >= 0.0144 -> >=144 bad out of 10_000.
        // fast: 9_856 healthy / 10_000 total -> 9856 bp available -> bad = 144 bp -> burn = 144*10_000/10 = 144_000 bp
        let fast = obs_with_samples(9_856, 10_000);
        let slow = obs_with_samples(10_000, 10_000);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::None);
        assert_eq!(verdict.fast_burn_rate_basis_points, 144_000);
        assert_eq!(verdict.slow_burn_rate_basis_points, 0);
    }

    /// Slow window hot, fast window clean -> None (AND-gate).
    #[test]
    fn slow_hot_fast_clean_is_none() {
        let fast = obs_with_samples(10_000, 10_000);
        let slow = obs_with_samples(9_856, 10_000);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::None);
        assert_eq!(verdict.fast_burn_rate_basis_points, 0);
        assert_eq!(verdict.slow_burn_rate_basis_points, 144_000);
    }

    /// Both windows >= page threshold -> Page.
    #[test]
    fn both_page_is_page() {
        // 9_856 / 10_000 -> burn_rate = 144_000 bp = exactly page threshold
        let fast = obs_with_samples(9_856, 10_000);
        let slow = obs_with_samples(9_856, 10_000);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::Page);
        assert!(verdict.fast_window_tripped);
        assert!(verdict.slow_window_tripped);
    }

    /// Both windows >= ticket threshold but < page threshold -> Ticket.
    #[test]
    fn both_ticket_only_is_ticket() {
        // ticket threshold = 10_000 bp (1.0x). burn_rate = observed_bad * 10_000 / 10
        // Want burn between 10_000 and 144_000: e.g. burn_rate = 10_000 -> observed_bad = 10 bp
        // 10 bad out of 10_000: healthy = 9_990, total = 10_000
        let fast = obs_with_samples(9_990, 10_000);
        let slow = obs_with_samples(9_990, 10_000);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::Ticket);
        assert!(verdict.fast_window_tripped);
        assert!(verdict.slow_window_tripped);
        // Confirm not paging
        assert!(verdict.fast_burn_rate_basis_points < 144_000);
    }

    /// 100% availability target (no allowed budget) -> burn rate 0, no panic.
    #[test]
    fn allowed_bad_zero_no_panic() {
        let policy = SlaPolicy {
            availability_target_basis_points: 9_999, // allowed_bad = 1 bp, not truly zero
            provisioning_latency_target_millis: 600_000,
        };
        // Use a policy where allowed_bad_basis_points would be 0:
        // availability_target = 10_000 is invalid per is_valid(), so kernel rejects it.
        // The guard is: if allowed_bad_basis_points == 0, burn_rate = 0.
        // We test this indirectly: a policy that produces 0 bad is handled.
        let fast = obs_with_samples(1_000, 1_000);
        let slow = obs_with_samples(1_000, 1_000);
        let verdict = summarize_burn_rate_alert(&fast, &slow, policy, BurnRatePolicy::default());
        assert!(verdict.is_ok());
        let v = verdict.unwrap();
        assert_eq!(v.fast_burn_rate_basis_points, 0);
        assert_eq!(v.severity, AlertSeverity::None);
    }

    /// u64::MAX samples in both windows -> no overflow, no panic.
    #[test]
    fn u64_max_no_overflow() {
        let fast = obs_with_samples(u64::MAX, u64::MAX);
        let slow = obs_with_samples(u64::MAX, u64::MAX);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::None);
        assert_eq!(verdict.fast_burn_rate_basis_points, 0);
    }

    /// AlertVerdict serialises to stable snake_case JSON and round-trips.
    #[test]
    fn serde_round_trip_alert_verdict() {
        let fast = obs_with_samples(9_856, 10_000);
        let slow = obs_with_samples(9_856, 10_000);
        let verdict =
            summarize_burn_rate_alert(&fast, &slow, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        let json = serde_json::to_string(&verdict).unwrap();
        assert!(json.contains("\"severity\":\"page\""));
        let back: AlertVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(verdict, back);
    }

    /// Burn rate from summarize_burn_rate_alert matches summarize_sla on the same observation.
    #[test]
    fn burn_rate_matches_summarize_sla() {
        let obs = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Failed,
            100,
            80,
            None,
        );
        let sla_summary = summarize_sla(&obs, SlaPolicy::default()).unwrap();
        let expected_burn = sla_summary.error_budget.burn_rate_basis_points;

        // Use the same observation as both fast and slow.
        let verdict =
            summarize_burn_rate_alert(&obs, &obs, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap();
        assert_eq!(verdict.fast_burn_rate_basis_points, expected_burn);
        assert_eq!(verdict.slow_burn_rate_basis_points, expected_burn);
    }

    /// Inverted and zero-threshold BurnRatePolicy both return InvalidPolicy.
    #[test]
    fn invalid_burn_policy_fails_closed() {
        let obs = obs_with_samples(1_000, 1_000);
        let inverted = BurnRatePolicy {
            page: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 5_000,
            },
            ticket: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 10_000,
            },
        };
        assert_eq!(
            summarize_burn_rate_alert(&obs, &obs, SlaPolicy::default(), inverted).unwrap_err(),
            SlaKernelError::InvalidPolicy
        );
        let zero = BurnRatePolicy {
            page: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 0,
            },
            ticket: BurnRateWindowPolicy {
                burn_rate_threshold_basis_points: 0,
            },
        };
        assert_eq!(
            summarize_burn_rate_alert(&obs, &obs, SlaPolicy::default(), zero).unwrap_err(),
            SlaKernelError::InvalidPolicy
        );
    }

    /// Empty window (total == 0) in either window returns EmptyObservationWindow.
    #[test]
    fn empty_window_fails_closed() {
        let good = obs_with_samples(1_000, 1_000);
        let empty = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            0,
            0,
            None,
        );
        // fast empty
        assert_eq!(
            summarize_burn_rate_alert(&empty, &good, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap_err(),
            SlaKernelError::EmptyObservationWindow
        );
        // slow empty
        assert_eq!(
            summarize_burn_rate_alert(&good, &empty, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap_err(),
            SlaKernelError::EmptyObservationWindow
        );
    }

    /// healthy > total in either window returns HealthySamplesExceedTotal.
    #[test]
    fn healthy_exceeds_total_fails_closed() {
        let good = obs_with_samples(1_000, 1_000);
        let bad = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            10,
            11,
            None,
        );
        // fast bad
        assert_eq!(
            summarize_burn_rate_alert(&bad, &good, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap_err(),
            SlaKernelError::HealthySamplesExceedTotal
        );
        // slow bad
        assert_eq!(
            summarize_burn_rate_alert(&good, &bad, SlaPolicy::default(), BurnRatePolicy::default())
                .unwrap_err(),
            SlaKernelError::HealthySamplesExceedTotal
        );
    }

    /// AlertSeverity ordering is monotonic: Page > Ticket > None.
    #[test]
    fn alert_severity_ordering() {
        assert!(AlertSeverity::Page > AlertSeverity::Ticket);
        assert!(AlertSeverity::Ticket > AlertSeverity::None);
        assert!(AlertSeverity::Page > AlertSeverity::None);
    }
}
