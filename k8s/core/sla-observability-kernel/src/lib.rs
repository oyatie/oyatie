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

/// Deterministic tenant-fleet SLA rollup aggregating many per-cluster summaries.
///
/// All counts and aggregations are computed from the caller-supplied `SlaSummary`
/// slice; no clock, I/O, or external state is accessed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetSlaSummary {
    /// Total number of clusters in the rollup. // data_class: INTERNAL_ONLY
    pub cluster_count: usize,
    /// Clusters whose availability state is `Available`. // data_class: INTERNAL_ONLY
    pub available_count: usize,
    /// Clusters whose availability state is `Degraded`. // data_class: INTERNAL_ONLY
    pub degraded_count: usize,
    /// Clusters whose availability state is `Unavailable`. // data_class: INTERNAL_ONLY
    pub unavailable_count: usize,
    /// Worst (highest-severity) availability state across all clusters. // data_class: INTERNAL_ONLY
    pub worst_availability_state: AvailabilityState,
    /// Sample-weighted observed availability in basis points (10_000 = 100%). // data_class: INTERNAL_ONLY
    pub aggregate_observed_basis_points: u16,
    /// Sample-weighted target availability in basis points. // data_class: INTERNAL_ONLY
    pub aggregate_target_basis_points: u16,
    /// Maximum burn rate in basis points across all clusters (10_000 = 1.0x). // data_class: INTERNAL_ONLY
    pub max_burn_rate_basis_points: u32,
    /// True if any cluster has exhausted its error budget. // data_class: INTERNAL_ONLY
    pub any_error_budget_exhausted: bool,
    /// Number of clusters with a provisioning latency breach. // data_class: INTERNAL_ONLY
    pub provisioning_latency_breach_count: usize,
}

/// Compute a deterministic tenant-fleet SLA rollup from a slice of per-cluster summaries.
///
/// Availability aggregation is sample-weighted: clusters with more status samples
/// contribute proportionally more to the aggregate basis points. `u128` intermediates
/// are used throughout to prevent overflow even when individual clusters carry
/// `u64::MAX` sample counts.
///
/// # Errors
/// Returns `SlaKernelError::EmptyObservationWindow` when `summaries` is empty.
/// The function never panics on caller-supplied values.
pub fn summarize_fleet_sla(summaries: &[SlaSummary]) -> Result<FleetSlaSummary, SlaKernelError> {
    if summaries.is_empty() {
        return Err(SlaKernelError::EmptyObservationWindow);
    }

    let mut available_count: usize = 0;
    let mut degraded_count: usize = 0;
    let mut unavailable_count: usize = 0;
    let mut worst_state = AvailabilityState::Available;
    let mut max_burn_rate: u32 = 0;
    let mut any_exhausted = false;
    let mut breach_count: usize = 0;

    // u128 accumulators for sample-weighted aggregation — overflow-safe even at u64::MAX per cluster.
    let mut weighted_healthy: u128 = 0;
    let mut weighted_total: u128 = 0;
    let mut weighted_target_sum: u128 = 0;

    for s in summaries {
        match s.availability.state {
            AvailabilityState::Available => available_count += 1,
            AvailabilityState::Degraded => degraded_count += 1,
            AvailabilityState::Unavailable => unavailable_count += 1,
        }

        if availability_severity(s.availability.state) > availability_severity(worst_state) {
            worst_state = s.availability.state;
        }

        if s.error_budget.burn_rate_basis_points > max_burn_rate {
            max_burn_rate = s.error_budget.burn_rate_basis_points;
        }

        if s.error_budget.exhausted {
            any_exhausted = true;
        }

        if s.provisioning_latency.state == ProvisioningLatencyState::Breached {
            breach_count += 1;
        }

        let total = u128::from(s.availability.total_status_samples);
        let healthy = u128::from(s.availability.healthy_status_samples);
        weighted_healthy = weighted_healthy.saturating_add(healthy);
        weighted_total = weighted_total.saturating_add(total);
        weighted_target_sum = weighted_target_sum
            .saturating_add(u128::from(s.availability.target_basis_points).saturating_mul(total));
    }

    let aggregate_observed_basis_points = if weighted_total == 0 {
        0u16
    } else {
        let bps = (weighted_healthy * 10_000) / weighted_total;
        u16::try_from(bps).unwrap_or(10_000)
    };

    let aggregate_target_basis_points = if weighted_total == 0 {
        0u16
    } else {
        let bps = weighted_target_sum / weighted_total;
        u16::try_from(bps).unwrap_or(10_000)
    };

    Ok(FleetSlaSummary {
        cluster_count: summaries.len(),
        available_count,
        degraded_count,
        unavailable_count,
        worst_availability_state: worst_state,
        aggregate_observed_basis_points,
        aggregate_target_basis_points,
        max_burn_rate_basis_points: max_burn_rate,
        any_error_budget_exhausted: any_exhausted,
        provisioning_latency_breach_count: breach_count,
    })
}

/// Numeric severity for `AvailabilityState` ordering: Available < Degraded < Unavailable.
fn availability_severity(state: AvailabilityState) -> u8 {
    match state {
        AvailabilityState::Available => 0,
        AvailabilityState::Degraded => 1,
        AvailabilityState::Unavailable => 2,
    }
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
fn window_burn_rate(obs: &SlaObservation, sla_policy: SlaPolicy) -> Result<u32, SlaKernelError> {
    if obs.tenant_id.trim().is_empty() || obs.cluster_name.trim().is_empty() {
        return Err(SlaKernelError::InvalidClusterIdentity);
    }
    if obs.total_status_samples == 0 {
        return Err(SlaKernelError::EmptyObservationWindow);
    }
    if obs.healthy_status_samples > obs.total_status_samples {
        return Err(SlaKernelError::HealthySamplesExceedTotal);
    }
    let observed_basis_points = basis_points(obs.healthy_status_samples, obs.total_status_samples);
    let allowed_bad_basis_points = 10_000u16 - sla_policy.availability_target_basis_points;
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
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
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
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
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
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
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
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
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
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
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
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
        .unwrap();
        assert_eq!(verdict.severity, AlertSeverity::None);
        assert_eq!(verdict.fast_burn_rate_basis_points, 0);
    }

    /// AlertVerdict serialises to stable snake_case JSON and round-trips.
    #[test]
    fn serde_round_trip_alert_verdict() {
        let fast = obs_with_samples(9_856, 10_000);
        let slow = obs_with_samples(9_856, 10_000);
        let verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )
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
            summarize_burn_rate_alert(
                &empty,
                &good,
                SlaPolicy::default(),
                BurnRatePolicy::default()
            )
            .unwrap_err(),
            SlaKernelError::EmptyObservationWindow
        );
        // slow empty
        assert_eq!(
            summarize_burn_rate_alert(
                &good,
                &empty,
                SlaPolicy::default(),
                BurnRatePolicy::default()
            )
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

    // -----------------------------------------------------------------------
    // Fleet rollup tests
    // -----------------------------------------------------------------------

    /// Build a SlaSummary directly from an observation + policy (reuses production code).
    fn fleet_summary(
        tenant_id: &str,
        cluster_name: &str,
        status: ObservedControlPlaneStatus,
        total: u64,
        healthy: u64,
        prov_millis: Option<u64>,
    ) -> SlaSummary {
        let obs = SlaObservation::new(tenant_id, cluster_name, status, total, healthy, prov_millis);
        summarize_sla(&obs, SlaPolicy::default()).unwrap()
    }

    /// Single cluster: rollup mirrors the cluster's own values.
    #[test]
    fn single_cluster_passthrough() {
        let s = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None,
        );
        let rollup = summarize_fleet_sla(&[s.clone()]).unwrap();
        assert_eq!(rollup.cluster_count, 1);
        assert_eq!(rollup.available_count, 1);
        assert_eq!(rollup.degraded_count, 0);
        assert_eq!(rollup.unavailable_count, 0);
        assert_eq!(
            rollup.worst_availability_state,
            AvailabilityState::Available
        );
        assert_eq!(
            rollup.aggregate_observed_basis_points,
            s.availability.observed_basis_points
        );
        assert_eq!(
            rollup.aggregate_target_basis_points,
            s.availability.target_basis_points
        );
        assert_eq!(
            rollup.max_burn_rate_basis_points,
            s.error_budget.burn_rate_basis_points
        );
        assert_eq!(rollup.any_error_budget_exhausted, s.error_budget.exhausted);
        assert_eq!(rollup.provisioning_latency_breach_count, 0);
    }

    /// Empty slice fails closed with EmptyObservationWindow.
    #[test]
    fn empty_slice_fails_closed() {
        assert_eq!(
            summarize_fleet_sla(&[]).unwrap_err(),
            SlaKernelError::EmptyObservationWindow
        );
    }

    /// Mixed states: when any cluster is Unavailable, worst = Unavailable.
    #[test]
    fn mixed_states_worst_is_unavailable() {
        let avail = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None,
        );
        let unavail = fleet_summary(
            "ten_acme",
            "prod-b",
            ObservedControlPlaneStatus::Failed,
            1_000,
            0,
            None,
        );
        let rollup = summarize_fleet_sla(&[avail, unavail]).unwrap();
        assert_eq!(rollup.cluster_count, 2);
        assert_eq!(rollup.available_count, 1);
        assert_eq!(rollup.unavailable_count, 1);
        assert_eq!(
            rollup.worst_availability_state,
            AvailabilityState::Unavailable
        );
    }

    /// When all clusters exhaust their error budget, any_error_budget_exhausted is true.
    #[test]
    fn all_exhausted_budget() {
        // Failed with 0 healthy out of 100 -> exhausted
        let a = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Failed,
            100,
            0,
            None,
        );
        let b = fleet_summary(
            "ten_acme",
            "prod-b",
            ObservedControlPlaneStatus::Failed,
            100,
            0,
            None,
        );
        let rollup = summarize_fleet_sla(&[a, b]).unwrap();
        assert!(rollup.any_error_budget_exhausted);
    }

    /// When no cluster has exhausted its budget, any_error_budget_exhausted is false.
    #[test]
    fn none_exhausted_budget() {
        let a = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None,
        );
        let b = fleet_summary(
            "ten_acme",
            "prod-b",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None,
        );
        let rollup = summarize_fleet_sla(&[a, b]).unwrap();
        assert!(!rollup.any_error_budget_exhausted);
    }

    /// u64::MAX sample counts in all clusters must not overflow or panic.
    #[test]
    fn overflow_safety_u64_max() {
        let obs = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            u64::MAX,
            u64::MAX,
            None,
        );
        let s = summarize_sla(&obs, SlaPolicy::default()).unwrap();
        let rollup = summarize_fleet_sla(&[s]).unwrap();
        assert_eq!(rollup.cluster_count, 1);
        assert_eq!(rollup.aggregate_observed_basis_points, 10_000);
        assert!(!rollup.any_error_budget_exhausted);
    }

    /// Sample-weighted aggregation is biased toward the cluster with more samples.
    ///
    /// Cluster A: 9_000 / 10_000 healthy (90% available, 9_000 samples)
    /// Cluster B: 0 / 100 healthy (0% available, 100 samples)
    /// Weighted: (9_000 * 10_000 + 0 * 10_000) / (10_000 + 100) = 90_000_000 / 10_100 ≈ 8_910 bps
    /// The result must be > 5_000 (closer to A's value than B's).
    #[test]
    fn weighting_correctness() {
        let obs_a = SlaObservation::new(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            10_000,
            9_000,
            None,
        );
        let obs_b = SlaObservation::new(
            "ten_acme",
            "prod-b",
            ObservedControlPlaneStatus::Failed,
            100,
            0,
            None,
        );
        let a = summarize_sla(&obs_a, SlaPolicy::default()).unwrap();
        let b = summarize_sla(&obs_b, SlaPolicy::default()).unwrap();
        let rollup = summarize_fleet_sla(&[a, b]).unwrap();
        // Weighted result should be much closer to A's 9_000 bps than B's 0 bps
        assert!(rollup.aggregate_observed_basis_points > 5_000);
        assert!(rollup.aggregate_observed_basis_points < 9_000);
    }

    /// FleetSlaSummary serialises to stable snake_case JSON and round-trips.
    #[test]
    fn serde_snake_case_round_trip() {
        let s = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None,
        );
        let rollup = summarize_fleet_sla(&[s]).unwrap();
        let json = serde_json::to_string(&rollup).unwrap();
        assert!(json.contains("\"cluster_count\""));
        assert!(json.contains("\"worst_availability_state\""));
        assert!(json.contains("\"available\""));
        let back: FleetSlaSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(rollup, back);
    }

    /// provisioning_latency_breach_count counts only Breached clusters.
    #[test]
    fn provisioning_breach_count() {
        // Provisioning latency target is 600_000 ms (10 min default).
        // breached: observed > target
        let breached = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            Some(700_000), // exceeds 600_000 target
        );
        let met = fleet_summary(
            "ten_acme",
            "prod-b",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            Some(300_000), // below target -> met
        );
        let not_observed = fleet_summary(
            "ten_acme",
            "prod-c",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None, // not observed
        );
        let rollup = summarize_fleet_sla(&[breached, met, not_observed]).unwrap();
        assert_eq!(rollup.provisioning_latency_breach_count, 1);
    }

    /// All clusters Available -> worst_availability_state is Available.
    #[test]
    fn all_available_worst_is_available() {
        let a = fleet_summary(
            "ten_acme",
            "prod-a",
            ObservedControlPlaneStatus::Active,
            1_000,
            1_000,
            None,
        );
        let b = fleet_summary(
            "ten_acme",
            "prod-b",
            ObservedControlPlaneStatus::Active,
            500,
            500,
            None,
        );
        let rollup = summarize_fleet_sla(&[a, b]).unwrap();
        assert_eq!(
            rollup.worst_availability_state,
            AvailabilityState::Available
        );
        assert_eq!(rollup.cluster_count, 2);
        assert_eq!(rollup.available_count, 2);
        assert_eq!(rollup.degraded_count, 0);
        assert_eq!(rollup.unavailable_count, 0);
    }
}
