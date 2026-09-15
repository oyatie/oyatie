use std::collections::BTreeMap;
use std::path::Path;

use foundry_caller_draft::{CallerVerifier, RosterVerifier};
use foundry_ontology_query_usecase::OntologyQueryExecutionUsecase;
use foundry_projection_draft::{ProjectionStore, ProjectionStoreError};
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use foundry_records_draft::{RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_spine::{ProjectionState, SyncStatus, catch_up, fold_from_scratch, store_sync_status};
use tokio::sync::Mutex;

use crate::authz::PolicyEnforcementPoint;
pub use crate::boot::BootError;
use crate::config::Config;
use crate::seed::registry_for;

pub struct TenantState {
    pub projection: ProjectionState,
    /// Query executions, held per tenant because the usecase recognises a
    /// replay by the keys it has already answered.
    pub queries: OntologyQueryExecutionUsecase,
    /// The policy decision each (principal, idempotency key) was FIRST
    /// authorized under.
    ///
    /// The usecase fingerprints an intent with its decision id, so that a
    /// replay may return a cached receipt without re-checking policy. A policy
    /// engine mints a fresh decision per call, so without this every honest
    /// retry would read as a different intent. Every attempt is still
    /// authorized on its own before reaching here; this only keeps the evidence
    /// the first attempt was granted, so the intent is the same intent.
    pub query_decisions: BTreeMap<(String, String), String>, // data_class: INTERNAL_ONLY
    /// Held behind the PORT so a test can install a log that fails on demand.
    pub action_log: Box<dyn RecordsLog + Send>,
    pub denial_log: Box<dyn RecordsLog + Send>,
    /// The durable mirror of `projection`; the sync status is read here.
    pub projection_store: Box<dyn ProjectionStore + Send>,
}

/// Why a tenant's sync status could not be read: either side may fail, and
/// an unreadable store is never lag zero.
#[derive(Debug)]
pub enum SyncError {
    Log(RecordsLogError),
    Store(ProjectionStoreError),
}

impl std::fmt::Debug for TenantState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TenantState")
            .field("applied_ordinal", &self.projection.applied_ordinal)
            .finish_non_exhaustive()
    }
}

impl TenantState {
    /// The four handles a write-through requires, borrowed together so the
    /// borrow checker enforces what the write path already assumes.
    pub fn write_handles(
        &mut self,
    ) -> (
        &mut dyn RecordsLog,
        &mut dyn RecordsLog,
        &mut ProjectionState,
        &mut dyn ProjectionStore,
    ) {
        (
            &mut *self.action_log,
            &mut *self.denial_log,
            &mut self.projection,
            &mut *self.projection_store,
        )
    }
}

impl TenantState {
    /// The two handles a query execution needs, borrowed together so the borrow
    /// checker enforces what the read path already assumes: the usecase is
    /// mutated (it remembers the keys it has answered) while the store it reads
    /// is not.
    pub fn query_handles(&mut self) -> (&mut OntologyQueryExecutionUsecase, &dyn ProjectionStore) {
        (&mut self.queries, &*self.projection_store)
    }
}

impl TenantState {
    /// Costs a full replay — O(tenant log), unbounded — and callers hold the
    /// tenant mutex across it, so the observation in `metrics` degrades while
    /// it runs.
    ///
    /// The `Applied` disposition `audit_view` reports is sound only while
    /// this process is the sole writer: an entry appended below our
    /// `applied_ordinal` by another writer would pass its filter and be
    /// labelled applied by a process that never folded it.
    pub fn entries_now(&self, tenant_id: &str) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        self.action_log.replay(tenant_id, 1)
    }

    /// The DURABLE store's position against the log head, never the
    /// in-memory fold's: what survives a restart is what an operator asks.
    /// The two positions agree while this process is the sole writer of its
    /// log and every mirror was taken, which is the same assumption
    /// `entries_now` states.
    pub fn sync_status(&self, tenant_id: &str) -> Result<SyncStatus, SyncError> {
        let head = self.action_log.head(tenant_id).map_err(SyncError::Log)?;
        store_sync_status(&*self.projection_store, tenant_id, head).map_err(SyncError::Store)
    }
}

/// Everything the surface serves from. One mutex per tenant is honest: a
/// submission needs the log and the projection together, and SQLite is a
/// single-writer store.
pub struct AppState {
    pub tenants: BTreeMap<String, Mutex<TenantState>>,
    pub pep: PolicyEnforcementPoint,
    pub metrics: crate::metrics::Metrics,
    /// Held behind the PORT so a test can install a verifier of its own.
    pub verifier: Box<dyn CallerVerifier>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("tenants", &self.tenants)
            .field("pep", &self.pep)
            .field("metrics", &self.metrics)
            .finish_non_exhaustive()
    }
}

impl AppState {
    pub fn tenant_count(&self) -> usize {
        self.tenants.len()
    }
}

pub fn compose(config: &Config) -> Result<AppState, BootError> {
    if config.tenants.is_empty() {
        return Err(BootError::NoTenantsConfigured);
    }
    if paths_alias(&config.action_log, &config.denial_log) {
        return Err(BootError::LogPathsAliased);
    }
    if paths_alias(&config.projection_store, &config.action_log)
        || paths_alias(&config.projection_store, &config.denial_log)
    {
        return Err(BootError::StorePathAliased);
    }
    let action_log = SqliteRecordsLog::open(&config.action_log).map_err(|error| {
        BootError::ActionLogUnopenable {
            detail: format!("{error:?}"),
        }
    })?;
    refuse_unless_denial_trail_opens(&config.denial_log)?;

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
        let mut store = SqliteProjectionStore::open(&config.projection_store).map_err(|error| {
            BootError::ProjectionStoreUnopenable {
                detail: format!("{error:?}"),
            }
        })?;
        catch_up(tenant_id, &registry, &mut store, &entries).map_err(|error| {
            BootError::CatchUpRefused {
                tenant_id: tenant_id.clone(),
                detail: format!("{error:?}"),
            }
        })?;
        // Folded a second time: catch-up folds to mirror and keeps nothing.
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
                queries: OntologyQueryExecutionUsecase::default(),
                query_decisions: BTreeMap::new(),
                action_log: Box::new(action_log),
                denial_log: Box::new(denial_log),
                projection_store: Box::new(store),
            }),
        );
    }
    Ok(AppState {
        tenants,
        pep,
        metrics: crate::metrics::Metrics::default(),
        verifier: Box::new(RosterVerifier::new(config.operators.clone())),
    })
}

const POLICY_VERSION: &str = "psv-000001";

fn refuse_unless_denial_trail_opens(path: &Path) -> Result<(), BootError> {
    SqliteRecordsLog::open(path)
        .map(drop)
        .map_err(|error| BootError::DenialLogUnopenable {
            detail: format!("{error:?}"),
        })
}

/// Two configured paths name one store. Canonicalization is best-effort —
/// the paths need not exist yet — so a literal match is the fallback.
fn paths_alias(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
