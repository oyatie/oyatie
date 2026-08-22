//! SVID-delivery operator composition root (G002 slice-1b-iii-c; ADR-0561).
//!
//! This crate owns runtime wiring only: env-driven desired-state config with
//! fail-closed validation, the injected issuance backend, the injected clock, and
//! the reconcile loop. The reconcile DECISION stays in the pure kernel; the
//! kube-rs Secret projection + real X.509 issuance stay in the transient adapter.
//!
//! ## Config (env, fail-closed)
//!
//! The operator derives its single desired SVID-delivery spec from the
//! environment; an invalid/empty value is a HARD startup error (the binary exits
//! non-zero, mirroring the kms-operator-app precedent — never boots with a
//! degraded config):
//! - `OYATIE_SVID_OPERATOR_CELL_ID`        — the cell id (forms the SPIFFE authority).
//! - `OYATIE_SVID_OPERATOR_NAMESPACE`      — the Secret namespace (default `cloud-iam`).
//! - `OYATIE_SVID_OPERATOR_TTL_SECS`       — leaf lifetime seconds (default 3600).
//! - `OYATIE_SVID_OPERATOR_ROTATION_WINDOW_SECS` — pre-expiry rotation window (default 600).
//!
//! The Secret name is FIXED to `cloud-iam-pdp-svid` (the consumer contract).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use iam_identity_workload_svid_operator_k8s::ExponentialBackoff;
use iam_identity_workload_svid_operator_kernel::{Clock, DesiredState};

/// The fixed Secret name the consumer (`MtlsContext::from_path`) mounts.
pub const PDP_SVID_SECRET_NAME: &str = "cloud-iam-pdp-svid";
/// The default Secret namespace (the cloud-iam namespace).
pub const DEFAULT_NAMESPACE: &str = "cloud-iam";
/// The default leaf lifetime in seconds (1 hour).
pub const DEFAULT_TTL_SECS: u64 = 3_600;
/// The default pre-expiry rotation window in seconds (10 minutes).
pub const DEFAULT_ROTATION_WINDOW_SECS: u64 = 600;

/// Env var: the cell id (forms `spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`).
pub const ENV_CELL_ID: &str = "OYATIE_SVID_OPERATOR_CELL_ID";
/// Env var: the Secret namespace.
pub const ENV_NAMESPACE: &str = "OYATIE_SVID_OPERATOR_NAMESPACE";
/// Env var: the leaf lifetime in seconds.
pub const ENV_TTL_SECS: &str = "OYATIE_SVID_OPERATOR_TTL_SECS";
/// Env var: the pre-expiry rotation window in seconds.
pub const ENV_ROTATION_WINDOW_SECS: &str = "OYATIE_SVID_OPERATOR_ROTATION_WINDOW_SECS";

/// Why the operator refused to start. Every variant is fail-closed: an invalid
/// config exits the process non-zero rather than booting degraded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperatorStartupConfigError {
    /// `OYATIE_SVID_OPERATOR_CELL_ID` was missing or empty (no SPIFFE authority).
    MissingCellId,
    /// The cell id carried whitespace/control/`/` (cannot form a cell authority).
    MalformedCellId(String),
    /// A numeric env var was present but not a valid positive integer.
    InvalidNumber {
        /// The env var name.
        var: String,
        /// The offending raw value.
        value: String,
    },
    /// `OYATIE_SVID_OPERATOR_TTL_SECS` resolved to zero (a leaf must outlive issuance).
    ZeroTtl,
    /// The rotation window was >= the TTL (a leaf would be born already-rotating).
    RotationWindowNotBelowTtl {
        /// The configured TTL seconds.
        ttl_secs: u64,
        /// The configured rotation-window seconds.
        rotation_window_secs: u64,
    },
}

impl fmt::Display for OperatorStartupConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCellId => write!(f, "{ENV_CELL_ID} must be set to a non-empty cell id"),
            Self::MalformedCellId(value) => write!(
                f,
                "{ENV_CELL_ID} value {value:?} is malformed (no whitespace/control/'/' allowed)"
            ),
            Self::InvalidNumber { var, value } => {
                write!(f, "{var} value {value:?} is not a valid positive integer")
            }
            Self::ZeroTtl => write!(f, "{ENV_TTL_SECS} must be greater than zero"),
            Self::RotationWindowNotBelowTtl {
                ttl_secs,
                rotation_window_secs,
            } => write!(
                f,
                "{ENV_ROTATION_WINDOW_SECS} ({rotation_window_secs}) must be strictly below {ENV_TTL_SECS} ({ttl_secs})"
            ),
        }
    }
}

impl std::error::Error for OperatorStartupConfigError {}

/// The validated operator config: the desired SVID-delivery spec + requeue backoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorConfig {
    /// The single desired SVID-delivery spec the operator converges.
    pub desired: DesiredState,
    /// The reconcile requeue backoff.
    pub backoff: ExponentialBackoff,
}

impl OperatorConfig {
    /// Build the config from the process environment, fail-closed.
    ///
    /// # Errors
    /// [`OperatorStartupConfigError`] on any missing/malformed/invalid value.
    pub fn from_env() -> Result<Self, OperatorStartupConfigError> {
        Self::from_env_pairs(std::env::vars())
    }

    /// Build the config from explicit env pairs (the testable core of
    /// [`OperatorConfig::from_env`]).
    ///
    /// # Errors
    /// [`OperatorStartupConfigError`] on any missing/malformed/invalid value.
    pub fn from_env_pairs<K, V, I>(pairs: I) -> Result<Self, OperatorStartupConfigError>
    where
        K: AsRef<str>,
        V: AsRef<str>,
        I: IntoIterator<Item = (K, V)>,
    {
        let mut cell_id: Option<String> = None;
        let mut namespace: Option<String> = None;
        let mut ttl_secs: Option<u64> = None;
        let mut rotation_window_secs: Option<u64> = None;

        for (key, value) in pairs {
            let key = key.as_ref();
            let value = value.as_ref().trim();
            match key {
                ENV_CELL_ID if !value.is_empty() => cell_id = Some(value.to_owned()),
                ENV_NAMESPACE if !value.is_empty() => namespace = Some(value.to_owned()),
                ENV_TTL_SECS if !value.is_empty() => {
                    ttl_secs = Some(parse_u64(ENV_TTL_SECS, value)?);
                }
                ENV_ROTATION_WINDOW_SECS if !value.is_empty() => {
                    rotation_window_secs = Some(parse_u64(ENV_ROTATION_WINDOW_SECS, value)?);
                }
                _ => {}
            }
        }

        let cell_id = cell_id.ok_or(OperatorStartupConfigError::MissingCellId)?;
        if cell_id
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '/')
        {
            return Err(OperatorStartupConfigError::MalformedCellId(cell_id));
        }
        let namespace = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_owned());
        let ttl_secs = ttl_secs.unwrap_or(DEFAULT_TTL_SECS);
        let rotation_window_secs = rotation_window_secs.unwrap_or(DEFAULT_ROTATION_WINDOW_SECS);

        if ttl_secs == 0 {
            return Err(OperatorStartupConfigError::ZeroTtl);
        }
        if rotation_window_secs >= ttl_secs {
            return Err(OperatorStartupConfigError::RotationWindowNotBelowTtl {
                ttl_secs,
                rotation_window_secs,
            });
        }

        Ok(Self {
            desired: DesiredState {
                spiffe_id: format!("spiffe://oyatie.cell-{cell_id}/platform/cloud-iam-pdp"),
                ttl_secs,
                rotation_window_secs,
                secret_name: PDP_SVID_SECRET_NAME.to_owned(),
                secret_namespace: namespace,
            },
            backoff: default_operator_backoff(),
        })
    }
}

fn parse_u64(var: &str, value: &str) -> Result<u64, OperatorStartupConfigError> {
    value
        .parse::<u64>()
        .map_err(|_| OperatorStartupConfigError::InvalidNumber {
            var: var.to_owned(),
            value: value.to_owned(),
        })
}

/// The default reconcile requeue backoff (base 30s, capped at 600s).
#[must_use]
pub fn default_operator_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        base_seconds: 30,
        max_seconds: 600,
    }
}

/// The production wall-clock (epoch seconds).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_epoch_seconds(&self) -> u64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => 0,
        }
    }
}
