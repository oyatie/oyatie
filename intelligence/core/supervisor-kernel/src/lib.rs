//! Foundry supervisor kernel — pure value types + port traits.
//!
//! Per ADR-0056 (12-layer enum, port-in-kernel): all port traits that cross
//! crate boundaries live here. Adapter crates implement these traits; the app
//! composes them. No I/O. No per-provider serialization.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── Identity types (live in kernel; re-exported here for back-compat) ──────
pub use intelligence_account_domain::{
    AccountId, AccountState, ProviderAccount, ProviderFamily, SecretReference,
};
pub use intelligence_autonomy_ceiling_kernel::{AutonomyTier, CeilingVerdict};

// ── Scalar newtypes ───────────────────────────────────────────────────────────

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MessageId(pub String);

/// data_class: INTERNAL_ONLY (request idempotency key; opaque to tenant)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RequestId(pub String);

/// data_class: INTERNAL_ONLY
/// Identifies a usage-window instance for cross-struct referencing.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WindowId(pub String);

// ── Shared Value Types ────────────────────────────────────────────────────────

/// data_class: INTERNAL_ONLY
/// Supervisor-specific view of a provider account, including its secret reference.
/// (ADR-0100: avoids changing intelligence-account-domain).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupervisorAccount {
    // data_class: INTERNAL_ONLY
    pub id: AccountId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub provider_family: ProviderFamily, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state: AccountState, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub secret_ref: SecretReference, // data_class: SECRET
}

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug)]
pub struct UsageWindowSnapshot {
    // data_class: INTERNAL_ONLY
    pub started_at_epoch_secs: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub ends_at_epoch_secs: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub tokens_in: u64, // data_class: SECRET
    // data_class: INTERNAL_ONLY
    pub tokens_out: u64, // data_class: SECRET
    /// % of nominal limit consumed in current window (0..200).
    // data_class: INTERNAL_ONLY
    pub usage_limit_pct: u8, // data_class: INTERNAL_ONLY
    /// % of reserve buffer remaining (0..100).
    // data_class: INTERNAL_ONLY
    pub reserve_remaining_pct: u8, // data_class: INTERNAL_ONLY
}

/// data_class: INTERNAL_ONLY
/// Enforcement-only projection computed from live UsageWindow at tick_once step 7.5.
/// Used by step 13.5 cost-ceiling gate.
/// Applied F-PROJECTED-P95-COLDSTART-FAIL-CLOSED-1: when sample window is cold
/// (<10 samples), the projected cost is treated as cost_ceiling + 1
/// so the comparator fails closed, or consult `ColdStartPolicy` config.
#[derive(Clone, Debug)]
pub struct EnforcementProjection {
    // data_class: INTERNAL_ONLY
    pub projected_tokens_p95: u64, // data_class: SECRET
    // data_class: INTERNAL_ONLY
    pub window_id: WindowId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub computed_at_epoch_secs: u64, // data_class: INTERNAL_ONLY
}

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug)]
pub struct SessionTicket {
    // data_class: INTERNAL_ONLY
    pub account_id: AccountId, // data_class: INTERNAL_ONLY
    /// Re-exported from intelligence-account-kernel.
    // data_class: INTERNAL_ONLY
    pub provider_family: ProviderFamily, // data_class: INTERNAL_ONLY
    /// Re-exported from intelligence-autonomy-ceiling-kernel.
    // data_class: INTERNAL_ONLY
    pub autonomy_tier: AutonomyTier, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub usage_window_snapshot: UsageWindowSnapshot, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub message_id: MessageId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub request_id: RequestId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub cost_ceiling_tokens: u64, // data_class: SECRET
    // data_class: INTERNAL_ONLY
    pub model_hint: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub secret_ref: SecretReference, // data_class: SECRET
}

/// data_class: INTERNAL_ONLY (state machine; no tenant payload)
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InboxState {
    Queued,
    Locked {
        reservation_id: String,
        ttl_epoch_secs: u64,
    },
    InFlight {
        reservation_id: String,
    },
    DraftedResponse,
    Committed,
    DeadLettered {
        reason: String,
    },
    Released {
        reason: String,
    },
}

/// data_class: INTERNAL_ONLY
pub struct SupervisorConfig {
    // data_class: INTERNAL_ONLY
    pub max_in_flight: usize, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub blocking_pool_size: usize, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub default_cost_ceiling: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub watchdog_secs: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub heartbeat_interval_secs: u64, // data_class: INTERNAL_ONLY
    // v6 BLOCKER-1 + BLOCKER-6:
    // data_class: INTERNAL_ONLY
    pub settings_renderer_mode: RendererMode, // data_class: INTERNAL_ONLY
    /// TTL for per-(account_id, template_blake3) verify cache.
    /// 0 = cache disabled; default 60s.
    // data_class: INTERNAL_ONLY
    pub settings_verify_debounce_secs: u64, // data_class: INTERNAL_ONLY
    /// If eligible_count drops below this after drift exclusion, return
    /// TickOutcome::DriftExcluded instead of spawning. Default 1.
    // data_class: INTERNAL_ONLY
    pub minimum_eligible_accounts: usize, // data_class: INTERNAL_ONLY
}

/// data_class: INTERNAL_ONLY
/// Controls whether SettingsRenderer::verify/render is invoked each tick.
///
/// Default is `Disabled` — no per-tick settings I/O. Enable via config only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RendererMode {
    /// Default — verify never invoked; existing tick_once behavior preserved.
    Disabled,
    /// Invoke verify; log drift but do not reconcile on-disk files.
    VerifyOnly,
    /// Invoke verify; if drift detected, invoke render (atomic-tempfile sequence).
    Reconcile,
}

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug)]
pub enum TickOutcome {
    Spawned(MessageId),
    Saturated,
    Idle,
    Quarantined(MessageId),
    /// Returned when drift exclusion drops `eligible_count` below
    /// `SupervisorConfig::minimum_eligible_accounts`.
    DriftExcluded {
        excluded_accounts: Vec<AccountId>,
        eligible_count: usize,
    },
}

/// data_class: INTERNAL_ONLY
/// Audit event variants emitted by the supervisor at each decision point.
/// Consumers write these to the audit-chain per ADR-0003.
#[derive(Clone, Debug)]
pub enum SupervisorEvent {
    TickSpawned {
        account_id: AccountId,
        message_id: MessageId,
    },
    TickSaturated,
    TickIdle,
    TickQuarantined {
        message_id: MessageId,
    },
    TickDriftExcluded {
        excluded_count: usize,
        eligible_count: usize,
    },
    SessionKilled {
        message_id: MessageId,
    },
    WindowRotated {
        account_id: AccountId,
    },
    AccountDegraded {
        account_id: AccountId,
    },
    TierBlocked {
        account_id: AccountId,
        message_id: MessageId,
    },
    SettingsRendered {
        account_id: AccountId,
        provider_family: ProviderFamily,
    },
    SettingsDriftExcluded {
        account_id: AccountId,
        provider_family: ProviderFamily,
    },
}

impl SupervisorEvent {
    /// Returns the complete capability path for this event per BLOCKER-3.
    pub fn capability_path(&self) -> &'static str {
        match self {
            Self::TickSpawned { .. } => "foundry.supervisor.tick.spawn",
            Self::TickSaturated => "foundry.supervisor.tick.saturated",
            Self::TickIdle => "foundry.supervisor.tick.idle",
            Self::TickQuarantined { .. } => "foundry.supervisor.tick.quarantine",
            Self::TickDriftExcluded { .. } => "foundry.supervisor.tick.drift_exclude",
            Self::SessionKilled { .. } => "foundry.supervisor.session.kill",
            Self::WindowRotated { .. } => "foundry.supervisor.account.window_rotate",
            Self::AccountDegraded { .. } => "foundry.supervisor.account.degrade",
            Self::TierBlocked { .. } => "foundry.supervisor.account.tier_block",
            Self::SettingsRendered { .. } => "foundry.supervisor.settings.render",
            Self::SettingsDriftExcluded { .. } => "foundry.supervisor.settings.drift_exclude",
        }
    }
}

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorError {
    /// RoutePolicy returned an account_id absent from concurrent snapshot.
    /// Returned instead of panicking (eliminates `.unwrap()` per BLOCKER-5).
    NoEligibleAccount {
        chosen: AccountId,
        snapshot_ids: Vec<AccountId>,
    },
    UsageBlocked(String),
    /// ATOMIC Locked → DeadLettered contract (InboxStore::dead_letter).
    /// Returned by dead_letter() when message is not in Locked state.
    /// (v4 patch #6 atomicity contract; v6 BLOCKER-7 failing fixture test v4.50)
    InvalidTransition,
    DriverError(String),
    InboxError(String),
    OutboxError(String),
    Quarantined(String),
}

/// data_class: TENANT_SCOPED (bridges to tenant_id via spend_to_usage_record)
#[derive(Clone, Debug)]
pub struct SpendRecord {
    // data_class: INTERNAL_ONLY
    pub account_id: AccountId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub message_id: MessageId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub tokens_in: u64, // data_class: SECRET
    // data_class: INTERNAL_ONLY
    pub tokens_out: u64, // data_class: SECRET
    // data_class: INTERNAL_ONLY
    pub completed_at_epoch_secs: u64, // data_class: INTERNAL_ONLY
}

// ── Port-trait supporting types ───────────────────────────────────────────────

/// Handle to a live session returned by SessionDriver::spawn_for_message.
/// Value-only: no Arc/Box/& per kernel conventions.
#[derive(Clone, Debug)]
pub struct SpawnedSession {
    // data_class: INTERNAL_ONLY
    pub session_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub account_id: AccountId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub message_id: MessageId, // data_class: INTERNAL_ONLY
}

/// Health check verdict from a SessionDriver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriverHealth {
    Healthy,
    Degraded { reason: String },
    Unavailable { reason: String },
}

/// Wrapper for a locked inbox item + reservation metadata.
#[derive(Clone, Debug)]
pub struct Locked<T> {
    // data_class: INTERNAL_ONLY
    pub reservation_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub ttl_epoch_secs: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub item: T, // data_class: INTERNAL_ONLY
}

/// data_class: TENANT_SCOPED (opaque payload; id is INTERNAL_ONLY)
#[derive(Clone, Debug)]
pub struct InboxItem {
    // data_class: INTERNAL_ONLY
    pub message_id: MessageId, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub payload: Vec<u8>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub enqueued_at_epoch_secs: u64, // data_class: INTERNAL_ONLY
}

// ── Port traits ───────────────────────────────────────────────────────────────

pub trait SessionDriver: Send + Sync {
    fn provider_family(&self) -> ProviderFamily;
    fn spawn_for_message(&self, ticket: &SessionTicket) -> Result<SpawnedSession, SupervisorError>;
    fn inject_message(&self, session: &SpawnedSession, msg: &[u8]) -> Result<(), SupervisorError>;
    fn drain_response(&self, session: &SpawnedSession) -> Result<Vec<u8>, SupervisorError>;
    fn kill(&self, session: &SpawnedSession) -> Result<(), SupervisorError>;
    fn health_check(&self) -> DriverHealth;
}

pub trait InboxStore: Send + Sync {
    fn peek_lock(&self, ttl: u64) -> Result<Option<Locked<InboxItem>>, SupervisorError>;
    fn commit(&self, id: &MessageId) -> Result<(), SupervisorError>;
    fn release(&self, id: &MessageId, reason: &str) -> Result<(), SupervisorError>;
    /// ATOMIC Locked → DeadLettered transition. Consumes the peek-lock.
    /// Returns `SupervisorError::InvalidTransition` when message is not locked.
    /// (v4 patch #6 atomicity contract; v6 BLOCKER-7 fixture test v4.50)
    fn dead_letter(&self, id: &MessageId, reason: &str) -> Result<(), SupervisorError>;
}

pub trait OutboxSink: Send + Sync {
    fn push(&self, account_id: &AccountId, payload: Vec<u8>) -> Result<(), SupervisorError>;
}

pub trait AccountSnapshotProvider: Send + Sync {
    fn snapshot(&self) -> Vec<SupervisorAccount>;
}

pub trait HeartbeatPolicy: Send + Sync {
    fn should_emit(&self, last_epoch_secs: u64, now_epoch_secs: u64) -> bool;
    fn interval_secs(&self) -> u64;
}

pub trait AuditChainPort: Send + Sync {
    fn emit(&self, event: SupervisorEvent) -> Result<(), SupervisorError>;
}

pub trait AutonomyCeilingPort: Send + Sync {
    fn enforce(&self, account_id: &AccountId, tier: AutonomyTier) -> Result<(), SupervisorError>;
}

pub trait UsageWindowPort: Send + Sync {
    fn check_usage(
        &self,
        account_id: &AccountId,
        now_epoch_secs: u64,
    ) -> Result<UsageWindowSnapshot, SupervisorError>;
}

// ── Decision Logic ────────────────────────────────────────────────────────────

pub fn record_spend(ticket: &SessionTicket, tokens_in: u64, tokens_out: u64) -> SpendRecord {
    // SystemTime::now() before UNIX_EPOCH would only occur if the host clock is
    // mis-configured backward into the 1970-prior past; treat that as 0 epoch
    // seconds (Tier 1: no production `.unwrap()` on a fallible Result per
    // ADR-0083). The on-disk audit trail records the as-observed value; an
    // upstream clock-sanity gate (ADR-0103, time-skew lane) will catch the
    // anomaly out-of-band.
    let completed_at_epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    SpendRecord {
        account_id: ticket.account_id.clone(),
        message_id: ticket.message_id.clone(),
        tokens_in,
        tokens_out,
        completed_at_epoch_secs,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_wrapper_carries_reservation_id() {
        let locked = Locked {
            reservation_id: "res-001".to_owned(),
            ttl_epoch_secs: 100,
            item: "item",
        };
        assert_eq!(locked.reservation_id, "res-001");
    }

    #[test]
    fn message_id_clone_and_eq() {
        let m1 = MessageId("m1".to_owned());
        let m2 = m1.clone();
        assert_eq!(m1, m2);
    }

    #[test]
    fn inbox_state_queued_not_locked() {
        let s = InboxState::Queued;
        assert!(!matches!(s, InboxState::Locked { .. }));
    }

    #[test]
    fn inbox_state_locked_carries_ttl() {
        let s = InboxState::Locked {
            reservation_id: "res-001".to_owned(),
            ttl_epoch_secs: 200,
        };
        if let InboxState::Locked { ttl_epoch_secs, .. } = s {
            assert_eq!(ttl_epoch_secs, 200);
        } else {
            panic!("not locked");
        }
    }

    #[test]
    fn inbox_state_dead_lettered_carries_reason() {
        let s = InboxState::DeadLettered {
            reason: "malformed".to_owned(),
        };
        if let InboxState::DeadLettered { reason } = s {
            assert_eq!(reason, "malformed");
        } else {
            panic!("not dead lettered");
        }
    }

    #[test]
    fn enforcement_projection_fields_accessible() {
        let p = EnforcementProjection {
            projected_tokens_p95: 500,
            window_id: WindowId("w1".to_owned()),
            computed_at_epoch_secs: 1700000000,
        };
        assert_eq!(p.projected_tokens_p95, 500);
    }

    #[test]
    fn driver_health_variants_distinguishable() {
        let h = DriverHealth::Degraded {
            reason: "timeout".to_owned(),
        };
        assert!(!matches!(h, DriverHealth::Healthy));
        assert!(matches!(h, DriverHealth::Degraded { .. }));
    }
}
