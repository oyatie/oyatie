//! In-memory adapters for every DR-pairing port: for tests, for a DR-drill
//! harness, and for single-process bring-up before a durable store exists.
//!
//! They are written to production hygiene — no `unwrap`/`expect`/`panic`,
//! and a poisoned lock surfaces as a typed port error rather than a crash —
//! but they are NOT a production store, and nothing here should be read as
//! one. Specifically:
//!
//! - State lives in process memory only. A restart loses every pair and
//!   every recorded event; there is no durability and no replication.
//! - [`RecordingEventSink`] retains events and dedupe keys in memory with
//!   no eviction policy of its own. A long-running harness must call
//!   [`RecordingEventSink::drain`] on a cadence, or it will grow without
//!   bound. It is a buffer, not an audit archive.
//! - The fault-injection switches ([`InMemoryDrPairRepository::set_fail_reads`],
//!   [`InMemoryDrPairRepository::set_fail_writes`],
//!   [`RecordingEventSink::set_failing`]) and the unchecked seed
//!   ([`InMemoryDrPairRepository::seed_unchecked`]) exist to reproduce
//!   outages and legacy rows in a drill. They take `&self` and are not
//!   feature-gated, so any holder of a shared reference can trip them:
//!   that is acceptable in a test or a drill and is one of the reasons
//!   these types do not belong in a deployment that matters.
//!
//! The durable Postgres/Valkey adapters are named as gaps in the crate docs.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use crate::kernel::{
    DrCellCandidate, DrCellCatalog, DrPair, DrPairAuditEvent, DrPairEventSink, DrPairRepository,
    DrPairingError, DrSloProbe, ResidencyPolicy,
};

/// A pair store backed by a `BTreeMap` under one mutex.
#[derive(Debug, Default)]
pub struct InMemoryDrPairRepository {
    state: Mutex<RepoState>,
}

/// Interior state of [`InMemoryDrPairRepository`].
#[derive(Debug, Default)]
struct RepoState {
    pairs: BTreeMap<String, DrPair>,
    fail_reads: bool,
    fail_writes: bool,
}

impl InMemoryDrPairRepository {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Install a pair WITHOUT the [`DrPair::validate`] and monotonicity
    /// checks that [`DrPairRepository::record`] enforces.
    ///
    /// A drill and test affordance, not a write path: it exists to
    /// reproduce rows an older writer could have left behind — a pair whose
    /// `home_cell` equals its `dr_cell`, a version that went backwards — so
    /// that the controller's defensive handling of them can be exercised.
    /// Every control this crate has lives above it, so a row installed here
    /// is a row nothing agreed to. See the module docs.
    ///
    /// # Errors
    /// [`DrPairingError::PersistenceUnavailable`] when the lock is poisoned.
    pub fn seed_unchecked(&self, pair: &DrPair) -> Result<(), DrPairingError> {
        let mut state = self.lock()?;
        state.pairs.insert(pair.tenant_id.clone(), pair.clone());
        Ok(())
    }

    /// Make every subsequent read fail with
    /// [`DrPairingError::PersistenceUnavailable`] until turned off.
    ///
    /// # Errors
    /// [`DrPairingError::PersistenceUnavailable`] when the lock is poisoned.
    pub fn set_fail_reads(&self, failing: bool) -> Result<(), DrPairingError> {
        self.lock()?.fail_reads = failing;
        Ok(())
    }

    /// Make every subsequent write fail with
    /// [`DrPairingError::PersistenceUnavailable`] until turned off.
    ///
    /// # Errors
    /// [`DrPairingError::PersistenceUnavailable`] when the lock is poisoned.
    pub fn set_fail_writes(&self, failing: bool) -> Result<(), DrPairingError> {
        self.lock()?.fail_writes = failing;
        Ok(())
    }

    /// How many tenants have a pair recorded.
    ///
    /// # Errors
    /// [`DrPairingError::PersistenceUnavailable`] when the lock is poisoned.
    pub fn len(&self) -> Result<usize, DrPairingError> {
        Ok(self.lock()?.pairs.len())
    }

    /// Whether the store holds no pairs.
    ///
    /// # Errors
    /// [`DrPairingError::PersistenceUnavailable`] when the lock is poisoned.
    pub fn is_empty(&self) -> Result<bool, DrPairingError> {
        Ok(self.lock()?.pairs.is_empty())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, RepoState>, DrPairingError> {
        self.state
            .lock()
            .map_err(|_poisoned| DrPairingError::PersistenceUnavailable)
    }
}

impl DrPairRepository for InMemoryDrPairRepository {
    fn current(&self, tenant_id: &str) -> Result<Option<DrPair>, DrPairingError> {
        let state = self.lock()?;
        if state.fail_reads {
            return Err(DrPairingError::PersistenceUnavailable);
        }
        Ok(state.pairs.get(tenant_id).cloned())
    }

    /// Validates the pair and enforces version monotonicity, per the port
    /// contract: a `record` that could move `pair_version` backwards would
    /// let a replayed command pass a compare-and-swap that should refuse
    /// it, which is the split-brain the version chain exists to prevent.
    fn record(&self, pair: &DrPair) -> Result<(), DrPairingError> {
        pair.validate()?;
        let mut state = self.lock()?;
        if state.fail_writes {
            return Err(DrPairingError::PersistenceUnavailable);
        }
        if let Some(stored) = state.pairs.get(&pair.tenant_id)
            && stored.pair_version >= pair.pair_version
        {
            return Err(DrPairingError::NonMonotonicPairVersion);
        }
        state.pairs.insert(pair.tenant_id.clone(), pair.clone());
        Ok(())
    }

    /// Compare and swap under a single lock, so the read and the write
    /// cannot be interleaved by another caller. The default trait
    /// implementation cannot promise that; this one can. Monotonicity is
    /// enforced here too: matching the expected version is not enough if
    /// the write would then park the chain where a replay could re-enter it.
    fn compare_and_swap(
        &self,
        expected_version: Option<u32>,
        pair: &DrPair,
    ) -> Result<(), DrPairingError> {
        pair.validate()?;
        let mut state = self.lock()?;
        if state.fail_reads || state.fail_writes {
            return Err(DrPairingError::PersistenceUnavailable);
        }
        let found = state
            .pairs
            .get(&pair.tenant_id)
            .map(|stored| stored.pair_version);
        if found != expected_version {
            return Err(DrPairingError::StalePairVersion);
        }
        if found.is_some_and(|stored_version| stored_version >= pair.pair_version) {
            return Err(DrPairingError::NonMonotonicPairVersion);
        }
        state.pairs.insert(pair.tenant_id.clone(), pair.clone());
        Ok(())
    }
}

/// An SLO probe answering from explicit sets of cells: healthy, unhealthy,
/// and probe-fails-outright.
///
/// The DR-side and HOME-side questions are answered from SEPARATE sets, so
/// this adapter cannot paper over a caller that asks one question about the
/// other side — a cell declared a healthy DR replica says nothing here
/// about whether it is a healthy home primary. A real adapter reads two
/// different signals; a fixture that read one would hide that.
///
/// A cell in none of the relevant sets is unknown to the probe, which is a
/// probe failure — an unknown cell is not a healthy cell.
#[derive(Debug, Default)]
pub struct StaticDrSloProbe {
    healthy: BTreeSet<String>,
    unhealthy: BTreeSet<String>,
    failing: BTreeSet<String>,
    home_healthy: BTreeSet<String>,
    home_unhealthy: BTreeSet<String>,
    home_failing: BTreeSet<String>,
}

impl StaticDrSloProbe {
    /// A probe that knows nothing; every cell reads as a probe failure.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare `cell` healthy.
    #[must_use]
    pub fn with_healthy(mut self, cell: &str) -> Self {
        self.healthy.insert(cell.to_owned());
        self
    }

    /// Declare `cell` unhealthy — the probe answers, and the answer is "no".
    #[must_use]
    pub fn with_unhealthy(mut self, cell: &str) -> Self {
        self.unhealthy.insert(cell.to_owned());
        self
    }

    /// Declare that probing `cell` for DR health fails — the probe cannot
    /// answer at all.
    #[must_use]
    pub fn with_probe_failure(mut self, cell: &str) -> Self {
        self.failing.insert(cell.to_owned());
        self
    }

    /// Declare `cell` healthy as a HOME primary. Independent of
    /// [`Self::with_healthy`], which speaks about the DR replica.
    #[must_use]
    pub fn with_home_healthy(mut self, cell: &str) -> Self {
        self.home_healthy.insert(cell.to_owned());
        self
    }

    /// Declare `cell` unhealthy as a HOME primary — the probe answers, and
    /// the answer is "no".
    #[must_use]
    pub fn with_home_unhealthy(mut self, cell: &str) -> Self {
        self.home_unhealthy.insert(cell.to_owned());
        self
    }

    /// Declare that probing `cell` for HOME health fails.
    #[must_use]
    pub fn with_home_probe_failure(mut self, cell: &str) -> Self {
        self.home_failing.insert(cell.to_owned());
        self
    }
}

impl DrSloProbe for StaticDrSloProbe {
    fn dr_replica_health(&self, cell: &str) -> Result<bool, DrPairingError> {
        answer(cell, &self.failing, &self.healthy, &self.unhealthy)
    }

    fn home_replica_health(&self, cell: &str) -> Result<bool, DrPairingError> {
        answer(
            cell,
            &self.home_failing,
            &self.home_healthy,
            &self.home_unhealthy,
        )
    }
}

/// Resolve one cell against a probe-failure / healthy / unhealthy triple.
///
/// A cell in none of them is unknown, and an unknown cell is a probe
/// failure rather than a health verdict.
fn answer(
    cell: &str,
    failing: &BTreeSet<String>,
    healthy: &BTreeSet<String>,
    unhealthy: &BTreeSet<String>,
) -> Result<bool, DrPairingError> {
    if failing.contains(cell) {
        return Err(DrPairingError::SloProbeFailed);
    }
    if healthy.contains(cell) {
        return Ok(true);
    }
    if unhealthy.contains(cell) {
        return Ok(false);
    }
    Err(DrPairingError::SloProbeFailed)
}

/// A cell catalog over an explicit candidate list.
#[derive(Debug, Default)]
pub struct InMemoryCellCatalog {
    cells: Vec<DrCellCandidate>,
    available: bool,
}

impl InMemoryCellCatalog {
    /// An available catalog holding no cells.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            available: true,
        }
    }

    /// Add a candidate cell.
    ///
    /// # Errors
    /// A validation error from [`DrCellCandidate::validate`] when the
    /// candidate is malformed; a catalog never serves a malformed cell.
    pub fn with_cell(mut self, cell: DrCellCandidate) -> Result<Self, DrPairingError> {
        cell.validate()?;
        self.cells.push(cell);
        Ok(self)
    }

    /// Make the catalog answer [`DrPairingError::CellCatalogUnavailable`].
    #[must_use]
    pub const fn unavailable(mut self) -> Self {
        self.available = false;
        self
    }
}

impl DrCellCatalog for InMemoryCellCatalog {
    fn jurisdiction_of(&self, cell: &str) -> Result<Option<String>, DrPairingError> {
        if !self.available {
            return Err(DrPairingError::CellCatalogUnavailable);
        }
        Ok(self
            .cells
            .iter()
            .find(|candidate| candidate.cell_id == cell)
            .map(|candidate| candidate.jurisdiction.clone()))
    }

    fn candidates(&self, jurisdiction: &str) -> Result<Vec<DrCellCandidate>, DrPairingError> {
        if !self.available {
            return Err(DrPairingError::CellCatalogUnavailable);
        }
        Ok(self
            .cells
            .iter()
            .filter(|candidate| candidate.jurisdiction == jurisdiction)
            .cloned()
            .collect())
    }
}

/// A residency policy that answers one fixed verdict.
///
/// The jurisdiction equality rule is enforced by the controller against the
/// cell catalog; this port models the additional, policy-authored refusals
/// that `tenancy/policy/data-residency.cedar` expresses.
#[derive(Debug, Clone, Copy)]
pub struct StaticResidencyPolicy {
    verdict: Result<bool, DrPairingError>,
}

impl Default for StaticResidencyPolicy {
    fn default() -> Self {
        Self::permitting()
    }
}

impl StaticResidencyPolicy {
    /// A policy that permits every same-jurisdiction pair.
    #[must_use]
    pub const fn permitting() -> Self {
        Self { verdict: Ok(true) }
    }

    /// A policy that refuses every pair.
    #[must_use]
    pub const fn refusing() -> Self {
        Self { verdict: Ok(false) }
    }

    /// A policy engine that cannot answer.
    #[must_use]
    pub const fn unavailable() -> Self {
        Self {
            verdict: Err(DrPairingError::ResidencyPolicyUnavailable),
        }
    }
}

impl ResidencyPolicy for StaticResidencyPolicy {
    fn permits_pair(
        &self,
        _tenant_id: &str,
        _jurisdiction: &str,
        _home_cell: &str,
        _dr_cell: &str,
    ) -> Result<bool, DrPairingError> {
        self.verdict
    }
}

/// An audit sink that keeps events in order and deduplicates on
/// `idempotency_key`.
///
/// The dedupe is what makes "committed but not narrated" recoverable: the
/// caller re-emits the identical event after a sink outage and the trail
/// gains one entry, not two.
#[derive(Debug, Default)]
pub struct RecordingEventSink {
    state: Mutex<SinkState>,
}

/// Interior state of [`RecordingEventSink`].
#[derive(Debug, Default)]
struct SinkState {
    events: Vec<DrPairAuditEvent>,
    seen: BTreeSet<String>,
    failing: bool,
}

impl RecordingEventSink {
    /// An empty, accepting sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Make every subsequent emit fail with
    /// [`DrPairingError::AuditEmitUnavailable`] until turned off.
    ///
    /// # Errors
    /// [`DrPairingError::AuditEmitUnavailable`] when the lock is poisoned.
    pub fn set_failing(&self, failing: bool) -> Result<(), DrPairingError> {
        self.lock()?.failing = failing;
        Ok(())
    }

    /// Every accepted event, in emit order. Clones; the buffer keeps its
    /// contents.
    ///
    /// # Errors
    /// [`DrPairingError::AuditEmitUnavailable`] when the lock is poisoned.
    pub fn events(&self) -> Result<Vec<DrPairAuditEvent>, DrPairingError> {
        Ok(self.lock()?.events.clone())
    }

    /// Take every buffered event, leaving the buffer empty.
    ///
    /// This is the reclamation path for a long-running harness: the buffer
    /// has no cap of its own, so a process that only ever reads through
    /// [`Self::events`] grows for as long as it runs. Draining hands the
    /// events to whoever is forwarding them and frees the memory.
    ///
    /// The dedupe key set is deliberately NOT cleared: forgetting the keys
    /// would turn a post-outage re-narration into a duplicate entry, which
    /// is the failure this sink exists to prevent. The set holds one short
    /// key per transition, and a caller that needs to reclaim it too drops
    /// the whole sink.
    ///
    /// # Errors
    /// [`DrPairingError::AuditEmitUnavailable`] when the lock is poisoned.
    pub fn drain(&self) -> Result<Vec<DrPairAuditEvent>, DrPairingError> {
        Ok(core::mem::take(&mut self.lock()?.events))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SinkState>, DrPairingError> {
        self.state
            .lock()
            .map_err(|_poisoned| DrPairingError::AuditEmitUnavailable)
    }
}

impl DrPairEventSink for RecordingEventSink {
    fn emit(&self, event: &DrPairAuditEvent) -> Result<(), DrPairingError> {
        let mut state = self.lock()?;
        if state.failing {
            return Err(DrPairingError::AuditEmitUnavailable);
        }
        if !state.seen.insert(event.idempotency_key.clone()) {
            return Ok(());
        }
        state.events.push(event.clone());
        Ok(())
    }
}
