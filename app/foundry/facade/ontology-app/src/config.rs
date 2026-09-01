//! The environment contract. Every field is explicit: this process invents
//! no paths, no tenants and no listen address beyond a loopback default, so
//! a misconfiguration is a refusal at boot rather than a surprise in
//! production.

use std::path::PathBuf;

/// Everything the process needs to serve, resolved before anything opens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// The address the listener binds. Loopback by default — exposure is a
    /// deployment decision, never a default.
    pub listen_addr: String, // data_class: INTERNAL_ONLY
    /// The durable action log.
    pub action_log: PathBuf, // data_class: INTERNAL_ONLY
    /// The durable denial trail — a SEPARATE store, so a refusal can never
    /// land in the log it was refused from.
    pub denial_log: PathBuf, // data_class: INTERNAL_ONLY
    /// The tenants this process serves. `RecordsLog` cannot enumerate
    /// tenants, so the roster IS the served set.
    pub tenants: Vec<String>, // data_class: INTERNAL_ONLY
}

/// Why the environment was rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// A required variable is absent or empty.
    Missing { variable: &'static str },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { variable } => {
                write!(formatter, "required configuration {variable} is absent")
            }
        }
    }
}

const LISTEN_ADDR: &str = "OYATIE_FOUNDRY_ONTOLOGY_LISTEN_ADDR";
const ACTION_LOG: &str = "OYATIE_FOUNDRY_ONTOLOGY_ACTION_LOG";
const DENIAL_LOG: &str = "OYATIE_FOUNDRY_ONTOLOGY_DENIAL_LOG";
const TENANTS: &str = "OYATIE_FOUNDRY_ONTOLOGY_TENANTS";

impl Config {
    /// Read the environment. Absent required values refuse; they are never
    /// defaulted into a path this process chose for itself.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            listen_addr: non_empty(LISTEN_ADDR).unwrap_or_else(|| "127.0.0.1:8090".into()),
            action_log: PathBuf::from(non_empty(ACTION_LOG).ok_or(ConfigError::Missing {
                variable: ACTION_LOG,
            })?),
            denial_log: PathBuf::from(non_empty(DENIAL_LOG).ok_or(ConfigError::Missing {
                variable: DENIAL_LOG,
            })?),
            tenants: non_empty(TENANTS)
                .ok_or(ConfigError::Missing { variable: TENANTS })?
                .split(',')
                .map(str::trim)
                .filter(|tenant| !tenant.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        })
    }
}

fn non_empty(variable: &str) -> Option<String> {
    std::env::var(variable)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
