//! Boot: open the durable stores, seed the registries, replay each tenant's
//! log into its projection. Every step is fail-closed and ordered so that a
//! refusal happens before anything is served.
//!
//! Two invariants this module exists to hold. First, a configured durable
//! path that cannot be opened is a BOOT REFUSAL — never an in-memory
//! fallback, which would serve confident answers from state that does not
//! survive a restart. Second, the action log and the denial trail are two
//! distinct stores, so a refusal can never land in the log it was refused
//! from.

use std::collections::BTreeMap;
use std::path::Path;

use foundry_records_draft::{RecordsLog, SealedEnvelope};
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_spine::{ProjectionState, SyncStatus, fold_from_scratch};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::seed::registry_for;

/// Why the process refused to boot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootError {
    /// The action log could not be opened at the configured path.
    ActionLogUnopenable { detail: String },
    /// The denial trail could not be opened at the configured path.
    DenialLogUnopenable { detail: String },
    /// Both logs name one path. A shared store would let a refusal land in
    /// the log it was refused from.
    LogPathsAliased,
    /// The roster is empty, so the process would serve nothing while
    /// reporting itself healthy.
    NoTenantsConfigured,
    /// A tenant's registry could not be seeded.
    SeedRefused { tenant_id: String, detail: String },
    /// A tenant's log could not be replayed.
    ReplayFailed { tenant_id: String, detail: String },
}

impl std::fmt::Display for BootError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActionLogUnopenable { detail } => {
                write!(formatter, "the action log could not be opened: {detail}")
            }
            Self::DenialLogUnopenable { detail } => {
                write!(formatter, "the denial trail could not be opened: {detail}")
            }
            Self::LogPathsAliased => write!(
                formatter,
                "the action log and the denial trail must be distinct stores"
            ),
            Self::NoTenantsConfigured => {
                write!(
                    formatter,
                    "no tenants configured; the roster is the served set"
                )
            }
            Self::SeedRefused { tenant_id, detail } => {
                write!(
                    formatter,
                    "tenant {tenant_id} could not be seeded: {detail}"
                )
            }
            Self::ReplayFailed { tenant_id, detail } => {
                write!(
                    formatter,
                    "tenant {tenant_id} could not be replayed: {detail}"
                )
            }
        }
    }
}

/// One tenant's served state. The entries mirror is required, not an
/// optimization: the history and audit views read the log entries alongside
/// the projection, so a process that held only the projection could not
/// answer them.
#[derive(Debug)]
pub struct TenantState {
    pub projection: ProjectionState,
    pub entries: Vec<SealedEnvelope>,
}

impl TenantState {
    /// Where this tenant's fold stands against its log.
    pub fn sync_status(&self) -> SyncStatus {
        let head = self
            .entries
            .last()
            .map_or(0, |sealed| sealed.receipt.ordinal);
        self.projection.sync_status(head)
    }
}

/// Everything the surface serves from. One mutex per tenant is honest: a
/// submission needs the log and the projection together, and SQLite is a
/// single-writer store.
#[derive(Debug)]
pub struct AppState {
    pub tenants: BTreeMap<String, Mutex<TenantState>>,
}

impl AppState {
    /// How many tenants this process serves.
    pub fn tenant_count(&self) -> usize {
        self.tenants.len()
    }

    /// Ready means every tenant's fold has consumed its whole log. Poison
    /// does NOT enter this predicate: a poisoned entry advances the fold and
    /// touches nothing else, so counting it as un-ready would red the
    /// instrument exactly when the system is making progress.
    pub fn is_ready(&self) -> bool {
        self.tenants.values().all(|tenant| {
            tenant
                .try_lock()
                .is_ok_and(|state| state.sync_status().lag == 0)
        })
    }

    /// Poisoned entries across every tenant — surfaced, never hidden.
    pub fn poisoned_count(&self) -> u64 {
        self.tenants
            .values()
            .map(|tenant| {
                tenant
                    .try_lock()
                    .map_or(0, |state| state.sync_status().poisoned_count)
            })
            .sum()
    }
}

/// Boot the process from resolved configuration.
pub fn compose(config: &Config) -> Result<AppState, BootError> {
    if config.tenants.is_empty() {
        return Err(BootError::NoTenantsConfigured);
    }
    if paths_alias(&config.action_log, &config.denial_log) {
        return Err(BootError::LogPathsAliased);
    }
    let action_log = SqliteRecordsLog::open(&config.action_log).map_err(|error| {
        BootError::ActionLogUnopenable {
            detail: format!("{error:?}"),
        }
    })?;
    // The denial trail is opened at boot for the same reason the action log
    // is: discovering at refusal time that the trail is unwritable would
    // mean losing the record of a denial.
    let _denial_log = SqliteRecordsLog::open(&config.denial_log).map_err(|error| {
        BootError::DenialLogUnopenable {
            detail: format!("{error:?}"),
        }
    })?;

    let mut tenants = BTreeMap::new();
    for tenant_id in &config.tenants {
        let registry = registry_for(tenant_id).map_err(|error| BootError::SeedRefused {
            tenant_id: tenant_id.clone(),
            detail: error.to_string(),
        })?;
        let entries = action_log
            .replay(tenant_id, 1)
            .map_err(|error| BootError::ReplayFailed {
                tenant_id: tenant_id.clone(),
                detail: format!("{error:?}"),
            })?;
        let projection = fold_from_scratch(tenant_id, &registry, entries.iter());
        tenants.insert(
            tenant_id.clone(),
            Mutex::new(TenantState {
                projection,
                entries,
            }),
        );
    }
    Ok(AppState { tenants })
}

/// Two configured paths name one store. Canonicalization is best-effort —
/// the paths need not exist yet — so a literal match is the fallback.
fn paths_alias(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
