//! Cloud KMS operator composition root.
//!
//! This crate owns runtime wiring only: desired-state config, injected observed
//! state provider, injected domain actuator, and injected clock. Reconcile
//! decisions remain in the pure kernel; Kubernetes projection and domain
//! actuation remain in the transient adapter.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use secrets_kms_operator_k8s::{
    ExponentialBackoff, KmsOperatorActuator, ObservedStateProvider, ReconcileCycleFailure,
    ReconcileCycleReport, run_reconcile_cycle,
};
use secrets_kms_operator_kernel::{Clock, DesiredState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorConfig {
    pub desired: DesiredState,
    pub backoff: ExponentialBackoff,
}

pub fn default_operator_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        base_seconds: 5,
        max_seconds: 300,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperatorStartupConfigError {
    MissingStatePath,
}

impl std::fmt::Display for OperatorStartupConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingStatePath => write!(
                f,
                "OYATIE_KMS_OPERATOR_STATE_PATH must point to durable operator state"
            ),
        }
    }
}

impl std::error::Error for OperatorStartupConfigError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorStateStoreConfig {
    pub path: String,
}

impl OperatorStateStoreConfig {
    pub fn from_env() -> Result<Self, OperatorStartupConfigError> {
        Self::from_env_pairs(std::env::vars())
    }

    pub fn from_env_pairs<K, V, I>(pairs: I) -> Result<Self, OperatorStartupConfigError>
    where
        K: AsRef<str>,
        V: AsRef<str>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in pairs {
            if key.as_ref() == "OYATIE_KMS_OPERATOR_STATE_PATH" {
                let path = value.as_ref().trim();
                if !path.is_empty() {
                    return Ok(Self {
                        path: path.to_owned(),
                    });
                }
            }
        }
        Err(OperatorStartupConfigError::MissingStatePath)
    }
}

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

pub struct OperatorApp<P, A, C> {
    provider: P,
    actuator: A,
    clock: C,
    config: OperatorConfig,
}

impl<P, A, C> OperatorApp<P, A, C> {
    pub fn new(provider: P, actuator: A, clock: C, config: OperatorConfig) -> Self {
        Self {
            provider,
            actuator,
            clock,
            config,
        }
    }

    pub fn actuator(&self) -> &A {
        &self.actuator
    }

    pub fn config(&self) -> &OperatorConfig {
        &self.config
    }
}

impl<P, A, C> OperatorApp<P, A, C>
where
    P: ObservedStateProvider,
    A: KmsOperatorActuator,
    C: Clock,
{
    pub fn run_once(&mut self) -> Result<ReconcileCycleReport, ReconcileCycleFailure> {
        run_reconcile_cycle(
            &self.provider,
            &self.config.desired,
            &self.clock,
            &mut self.actuator,
        )
    }
}
