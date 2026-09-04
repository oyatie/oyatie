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

use foundry_records_draft::{RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_spine::{ProjectionState, SyncStatus, fold_from_scratch};
use tokio::sync::Mutex;

use crate::auth::OperatorCredential;
use crate::authz::PolicyEnforcementPoint;
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
    /// The policy seed did not compile or strict-validate. The process
    /// never serves a policy set it could not validate.
    PolicyRejected { detail: String },
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
            Self::PolicyRejected { detail } => {
                write!(formatter, "the policy seed was rejected: {detail}")
            }
        }
    }
}

/// One tenant's served state.
pub struct TenantState {
    pub projection: ProjectionState,
    /// The durable stores this tenant writes through. They live here, not
    /// in a shared pool, because a submission needs the action log, the
    /// denial trail and the projection together under one lock.
    /// Held behind the PORT so a test can install a log that fails on demand.
    /// A real SQLite handle CAN be driven to failure — a second connection
    /// dropping the table makes the open handle's next read `Err` — but doing
    /// that from here would put a `rusqlite` dev-dependency in the facade and
    /// raw SQL against the adapter's schema in its tests. The tests already
    /// open `SqliteRecordsLog` by path; it is the SQL, not the coupling, that
    /// would be new.
    pub action_log: Box<dyn RecordsLog + Send>,
    pub denial_log: Box<dyn RecordsLog + Send>,
}

impl std::fmt::Debug for TenantState {
    /// The durable handles carry no derivable Debug and nothing worth
    /// rendering; what an operator wants is where the fold stands, and
    /// `sync_status` needs a tenant id this type does not carry.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TenantState")
            .field("applied_ordinal", &self.projection.applied_ordinal)
            .finish_non_exhaustive()
    }
}

impl TenantState {
    /// The three handles `submit` requires, borrowed together so the
    /// borrow checker enforces what the write path already assumes.
    pub fn write_handles(
        &mut self,
    ) -> (
        &mut dyn RecordsLog,
        &mut dyn RecordsLog,
        &mut ProjectionState,
    ) {
        (
            &mut *self.action_log,
            &mut *self.denial_log,
            &mut self.projection,
        )
    }
}

impl TenantState {
    /// This tenant's log entries AS OF NOW, read from the durable log.
    ///
    /// History and audit read through here rather than holding a vector,
    /// because a retained copy is a second source that can disagree with the
    /// log — which it did: the boot snapshot this replaced could not show an
    /// entry the process had since accepted.
    ///
    /// Costs a full replay per request — O(tenant log), unbounded and growing
    /// — and neither caller is paged today, so this is the whole log to
    /// render one object's history. Callers hold the tenant mutex across it,
    /// so the observation in `metrics` degrades while it runs. `replay`
    /// already takes a `from_ordinal`, so bounding it later needs no port
    /// change.
    ///
    /// The `Applied` disposition `audit_view` reports is sound only while
    /// this process is the sole writer: an entry appended below our
    /// `applied_ordinal` by another writer would pass its filter and be
    /// labelled applied by a process that never folded it.
    pub fn entries_now(&self, tenant_id: &str) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        self.action_log.replay(tenant_id, 1)
    }

    /// Where this tenant's fold stands against its log.
    pub fn sync_status(&self, tenant_id: &str) -> Result<SyncStatus, RecordsLogError> {
        Ok(self
            .projection
            .sync_status(self.action_log.head(tenant_id)?))
    }
}

/// Everything the surface serves from. One mutex per tenant is honest: a
/// submission needs the log and the projection together, and SQLite is a
/// single-writer store.
#[derive(Debug)]
pub struct AppState {
    pub tenants: BTreeMap<String, Mutex<TenantState>>,
    /// The enforcement point every authorized surface consults.
    pub pep: PolicyEnforcementPoint,
    /// Request accounting the SLO indicators name.
    pub metrics: crate::metrics::Metrics,
    /// Who this process recognizes. Empty means deny-all serving: a
    /// process with no roster still answers its probes honestly rather
    /// than refusing to boot or, worse, serving openly.
    pub operators: Vec<OperatorCredential>,
}

impl AppState {
    /// How many tenants this process serves.
    pub fn tenant_count(&self) -> usize {
        self.tenants.len()
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

    let pep = PolicyEnforcementPoint::load(POLICY_VERSION).map_err(|error| {
        BootError::PolicyRejected {
            detail: error.to_string(),
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
        drop(entries);
        let action_log = SqliteRecordsLog::open(&config.action_log).map_err(|error| {
            BootError::ActionLogUnopenable {
                detail: format!("{error:?}"),
            }
        })?;
        let denial_log = SqliteRecordsLog::open(&config.denial_log).map_err(|error| {
            BootError::DenialLogUnopenable {
                detail: format!("{error:?}"),
            }
        })?;
        tenants.insert(
            tenant_id.clone(),
            Mutex::new(TenantState {
                projection,
                action_log: Box::new(action_log),
                denial_log: Box::new(denial_log),
            }),
        );
    }
    Ok(AppState {
        tenants,
        pep,
        metrics: crate::metrics::Metrics::default(),
        operators: config.operators.clone(),
    })
}

/// The bundle version this build serves. It moves when the seed does.
const POLICY_VERSION: &str = "psv-000001";

/// Two configured paths name one store. Canonicalization is best-effort —
/// the paths need not exist yet — so a literal match is the fallback.
fn paths_alias(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
