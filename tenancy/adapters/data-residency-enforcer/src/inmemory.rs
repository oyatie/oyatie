//! In-memory adapters: a region/jurisdiction catalog, a legal-transfer
//! register, and a denial audit sink.
//!
//! These are the process-local stand-ins for the three external systems IP-020
//! names — the tenant-registry table that resolves a cell to its pack, the
//! transfer register at `microservices/tenancy/legal/transfer-register.md`, and
//! the audit-chain that seals a denial. They implement the ports honestly: the
//! engine cannot tell them apart from a Postgres-backed catalog or an
//! audit-chain client.
//!
//! What they do NOT provide is durability, replication, retention, or a
//! cryptographic seal, and the sink is deliberately BOUNDED — see
//! [`InMemoryDenialAuditSink`] and "Gaps" in `lib.rs`. Wiring the in-memory
//! sink into a long-running dispatcher as the system of record is not what it
//! is for.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::domain::{ResidencyOutcome, ResidencyRule};
use crate::kernel::{
    CrossJurisdictionPermitEntry, RegionRecord, RegionRole, ResidencyRegionCatalog,
    ResidencyTransferRegister, SccRegisterEntry,
};
use crate::{ResidencyAdapterError, ResidencyContext, ResidencyDecision, ResidencyDenialAuditSink};

/// A region catalog holding every row it was given, including several cells in
/// one region.
///
/// Rows are NOT keyed by region id. A map keyed that way silently keeps the
/// last row for a duplicated key, which turns an operator config with a
/// duplicated entry into a quietly different policy — and hides the
/// contradiction from the engine's own duplicate check. Every supplied row is
/// reported from [`Self::regions`] so that check can see it.
#[derive(Clone, Debug, Default)]
pub struct InMemoryRegionCatalog {
    rows: Vec<RegionRecord>,
    unavailable: bool,
}

impl InMemoryRegionCatalog {
    /// Build a catalog from rows. EVERY row is kept: a repeated
    /// `(region_id, pack_id)` cell with different content reaches
    /// [`crate::usecase::ResidencyPolicyEngine::try_new`] as the malformed
    /// policy it is, and a region carrying two different packs reaches it as
    /// the documented co-tenancy it is.
    #[must_use]
    pub fn new(records: Vec<RegionRecord>) -> Self {
        let mut rows = records;
        rows.sort();
        Self {
            rows,
            unavailable: false,
        }
    }

    /// The documented oyatie pack roster.
    ///
    /// Transcribed from the pack table in `tenancy/policy/data-residency.md`
    /// §"Default: pack-pinning at creation time", cross-checked against
    /// `tenancy/multi-region.md` §"Pack Topology". Jurisdiction codes are the
    /// `jurisdiction_code` enumeration in `data-residency.md` §"Per-Pack
    /// Jurisdiction Tagging".
    ///
    /// `pack-us-healthcare` is registered on `us-ashburn-1` and `us-phoenix-1`
    /// — the SAME OCI regions as `pack-us`, which is what the source documents
    /// say, with the `US-HC` jurisdiction code that makes it a separate,
    /// isolated cluster. A route touching either region must therefore declare
    /// which pack's cell it means; see
    /// [`crate::ResidencyContext::with_pack`].
    #[must_use]
    pub fn oyatie_pack_roster() -> Self {
        Self::new(vec![
            RegionRecord::new("ap-seoul-1", "pack-kr", "KR", RegionRole::Primary),
            RegionRecord::new("eu-frankfurt-1", "pack-eu", "EU", RegionRole::Primary),
            RegionRecord::new("eu-amsterdam-1", "pack-eu", "EU", RegionRole::DrPair),
            RegionRecord::new("us-ashburn-1", "pack-us", "US", RegionRole::Primary),
            RegionRecord::new("us-phoenix-1", "pack-us", "US", RegionRole::DrPair),
            RegionRecord::new(
                "us-ashburn-1",
                "pack-us-healthcare",
                "US-HC",
                RegionRole::Primary,
            ),
            RegionRecord::new(
                "us-phoenix-1",
                "pack-us-healthcare",
                "US-HC",
                RegionRole::DrPair,
            ),
            RegionRecord::new("ap-tokyo-1", "pack-jp", "JP", RegionRole::Primary),
            RegionRecord::new("ap-singapore-1", "pack-sg", "SG", RegionRole::Primary),
            RegionRecord::new("ap-sydney-1", "pack-au", "AU", RegionRole::Primary),
            RegionRecord::new("ap-melbourne-1", "pack-au", "AU", RegionRole::DrPair),
            RegionRecord::new("ap-hyderabad-1", "pack-in", "IN", RegionRole::Primary),
            RegionRecord::new("ap-mumbai-1", "pack-in", "IN", RegionRole::DrPair),
            RegionRecord::new("sa-saopaulo-1", "pack-br", "BR", RegionRole::Primary),
            RegionRecord::new("sa-vinhedo-1", "pack-br", "BR", RegionRole::DrPair),
            RegionRecord::new("me-abudhabi-1", "pack-ae", "AE", RegionRole::Primary),
            RegionRecord::new("me-dubai-1", "pack-ae", "AE", RegionRole::DrPair),
            RegionRecord::new("me-jeddah-1", "pack-ksa", "KSA", RegionRole::Primary),
            RegionRecord::new("me-riyadh-1", "pack-ksa", "KSA", RegionRole::DrPair),
        ])
    }

    /// Simulate a catalog that cannot answer — the substrate outage a real
    /// tenant-registry read can hit. Present so the "the control could not run"
    /// path is exercisable rather than theoretical.
    pub fn set_unavailable(&mut self, unavailable: bool) {
        self.unavailable = unavailable;
    }

    /// How many rows this catalog holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether this catalog holds no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

impl ResidencyRegionCatalog for InMemoryRegionCatalog {
    fn lookup(&self, region_id: &str) -> Result<Option<RegionRecord>, ResidencyAdapterError> {
        let rows = self.rows_for(region_id)?;
        let Some(first) = rows.first() else {
            return Ok(None);
        };
        if rows.iter().any(|row| row != first) {
            // Several distinct cells share this region id. Picking one here is
            // how a KR row silently becomes a JP row; the caller must say which
            // cell it means, and `rows_for` reports them all so the engine can
            // refuse instead of guess.
            return Ok(None);
        }
        Ok(Some(first.clone()))
    }

    fn rows_for(&self, region_id: &str) -> Result<Vec<RegionRecord>, ResidencyAdapterError> {
        if self.unavailable {
            return Err(ResidencyAdapterError::EvaluationFailed);
        }
        Ok(self
            .rows
            .iter()
            .filter(|row| row.region_id == region_id)
            .cloned()
            .collect())
    }

    fn regions(&self) -> Result<Vec<RegionRecord>, ResidencyAdapterError> {
        if self.unavailable {
            return Err(ResidencyAdapterError::EvaluationFailed);
        }
        Ok(self.rows.clone())
    }
}

/// A legal-transfer register held in this process.
///
/// Stands in for `microservices/tenancy/legal/transfer-register.md` and the
/// control plane's issued cross-jurisdiction permits. Empty by default, which
/// authorises nothing.
#[derive(Clone, Debug, Default)]
pub struct InMemoryTransferRegister {
    sccs: Vec<SccRegisterEntry>,
    permits: Vec<CrossJurisdictionPermitEntry>,
    unavailable: bool,
}

impl InMemoryTransferRegister {
    /// A register holding these SCC rows and these permits.
    #[must_use]
    pub fn new(sccs: Vec<SccRegisterEntry>, permits: Vec<CrossJurisdictionPermitEntry>) -> Self {
        Self {
            sccs,
            permits,
            unavailable: false,
        }
    }

    /// Simulate a register that cannot answer.
    pub fn set_unavailable(&mut self, unavailable: bool) {
        self.unavailable = unavailable;
    }
}

impl ResidencyTransferRegister for InMemoryTransferRegister {
    fn scc_entry(
        &self,
        register_ref: &str,
    ) -> Result<Option<SccRegisterEntry>, ResidencyAdapterError> {
        if self.unavailable {
            return Err(ResidencyAdapterError::EvaluationFailed);
        }
        Ok(self
            .sccs
            .iter()
            .find(|entry| entry.register_ref == register_ref)
            .cloned())
    }

    fn cross_jurisdiction_permit(
        &self,
        permit_id: &str,
    ) -> Result<Option<CrossJurisdictionPermitEntry>, ResidencyAdapterError> {
        if self.unavailable {
            return Err(ResidencyAdapterError::EvaluationFailed);
        }
        Ok(self
            .permits
            .iter()
            .find(|entry| entry.permit_id == permit_id)
            .cloned())
    }
}

/// One recorded denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyDenialRecord {
    /// The route that was refused.
    pub context: ResidencyContext, // data_class: TENANT_SCOPED
    /// The decision recorded against it.
    pub decision: ResidencyDecision, // data_class: INTERNAL_ONLY
    /// WHICH rule refused it. A caller defect (`unknown-residency-overlay`) and
    /// a GDPR block (`eu-transfer-requires-scc`) are the same
    /// [`ResidencyDecision`] and must not be the same audit row.
    pub rule: ResidencyRule, // data_class: INTERNAL_ONLY
    /// Monotonic position in this sink, shared with the seal log so denials and
    /// authorised transfers can be interleaved in order. Not a clock: the
    /// domain reads no clock, and this adapter takes no clock port.
    pub sequence: u64, // data_class: INTERNAL_ONLY
}

/// One recorded transfer seal: an ALLOWED route that crossed a cell boundary.
///
/// `tenancy/policy/data-residency.md` §"Exception: tenant-executed SCCs"
/// requirement 4 wants every authorised transfer sealed, not only the refused
/// ones.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyTransferSealRecord {
    /// The route that was authorised.
    pub context: ResidencyContext, // data_class: TENANT_SCOPED
    /// The rule that authorised it.
    pub rule: ResidencyRule, // data_class: INTERNAL_ONLY
    /// Monotonic position in this sink; see
    /// [`ResidencyDenialRecord::sequence`].
    pub sequence: u64, // data_class: INTERNAL_ONLY
}

/// How many records a default [`InMemoryDenialAuditSink`] holds before it
/// refuses to accept more.
pub const DEFAULT_AUDIT_CAPACITY: usize = 1024;

#[derive(Debug, Default)]
struct AuditState {
    denials: Vec<ResidencyDenialRecord>,
    seals: Vec<ResidencyTransferSealRecord>,
    sequence: u64,
}

/// A denial audit sink that keeps its records in this process, BOUNDED.
///
/// Two properties this sink has that a naive `Vec` behind a `RefCell` does not:
///
/// - It is `Sync`. The guard sits on outbound event and RPC dispatch, which
///   runs on a multi-threaded runtime; a sink that cannot be shared across
///   threads cannot be wired there at all.
/// - It is bounded. An unbounded audit buffer is a memory-exhaustion primitive
///   handed to anyone who can drive denials — a caller retrying a blocked
///   cross-border replication in a backoff loop is enough. At capacity this
///   sink REFUSES with [`ResidencyAdapterError::AuditSinkUnavailable`] rather
///   than evicting: dropping the oldest record would lose exactly the evidence
///   a flood is trying to bury, and refusing keeps the crate's posture that an
///   unrecordable denial stops dispatch. Drain it with
///   [`Self::drain_denials`] / [`Self::drain_seals`].
///
/// It still has no retention window and no redaction: a full
/// [`ResidencyContext`] clone, tenant id included, lives until drained. That is
/// a real gap against the "Retention by Jurisdiction × Data Class" table — see
/// "Gaps" in `lib.rs`.
#[derive(Debug)]
pub struct InMemoryDenialAuditSink {
    state: Mutex<AuditState>,
    unavailable: AtomicBool,
    capacity: usize,
}

impl Default for InMemoryDenialAuditSink {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_AUDIT_CAPACITY)
    }
}

impl InMemoryDenialAuditSink {
    /// An empty sink holding at most [`DEFAULT_AUDIT_CAPACITY`] records of each
    /// kind.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty sink holding at most `capacity` denials and `capacity` seals.
    ///
    /// A capacity of zero records nothing and refuses everything, which is a
    /// legitimate way to assert that no denial may be dispatched unrecorded.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            state: Mutex::new(AuditState::default()),
            unavailable: AtomicBool::new(false),
            capacity,
        }
    }

    /// Recover the guard even if a writer panicked: the records already written
    /// are still evidence, and refusing to read them would be the one outcome
    /// worse than reading a truncated log.
    fn state(&self) -> MutexGuard<'_, AuditState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Simulate an audit path that is down.
    ///
    /// Not a test-only convenience: the posture this pins is that a denial with
    /// a broken audit path surfaces as an error, never as a quiet allow.
    pub fn set_unavailable(&self, unavailable: bool) {
        self.unavailable.store(unavailable, Ordering::SeqCst);
    }

    /// How many records of each kind this sink accepts.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Whether either log has reached capacity, i.e. the next record of that
    /// kind will be refused.
    #[must_use]
    pub fn is_full(&self) -> bool {
        let state = self.state();
        state.denials.len() >= self.capacity || state.seals.len() >= self.capacity
    }

    /// Every denial recorded so far, in order.
    #[must_use]
    pub fn denials(&self) -> Vec<ResidencyDenialRecord> {
        self.state().denials.clone()
    }

    /// How many denials are held.
    #[must_use]
    pub fn denial_count(&self) -> usize {
        self.state().denials.len()
    }

    /// Take every denial out, freeing the space. The pump an operator runs.
    pub fn drain_denials(&self) -> Vec<ResidencyDenialRecord> {
        std::mem::take(&mut self.state().denials)
    }

    /// Every transfer seal recorded so far, in order.
    #[must_use]
    pub fn seals(&self) -> Vec<ResidencyTransferSealRecord> {
        self.state().seals.clone()
    }

    /// How many transfer seals are held.
    #[must_use]
    pub fn seal_count(&self) -> usize {
        self.state().seals.len()
    }

    /// Take every seal out, freeing the space.
    pub fn drain_seals(&self) -> Vec<ResidencyTransferSealRecord> {
        std::mem::take(&mut self.state().seals)
    }
}

impl ResidencyDenialAuditSink for InMemoryDenialAuditSink {
    fn emit_denial(
        &self,
        ctx: &ResidencyContext,
        decision: ResidencyDecision,
    ) -> Result<(), ResidencyAdapterError> {
        self.emit_denial_detailed(
            ctx,
            ResidencyOutcome {
                decision,
                rule: ResidencyRule::RuleNotReported,
            },
        )
    }

    fn emit_denial_detailed(
        &self,
        ctx: &ResidencyContext,
        outcome: ResidencyOutcome,
    ) -> Result<(), ResidencyAdapterError> {
        if self.unavailable.load(Ordering::SeqCst) {
            return Err(ResidencyAdapterError::AuditSinkUnavailable);
        }
        let mut state = self.state();
        if state.denials.len() >= self.capacity {
            return Err(ResidencyAdapterError::AuditSinkUnavailable);
        }
        let sequence = state.sequence;
        state.sequence = state.sequence.saturating_add(1);
        state.denials.push(ResidencyDenialRecord {
            context: ctx.clone(),
            decision: outcome.decision,
            rule: outcome.rule,
            sequence,
        });
        Ok(())
    }

    fn emit_transfer_seal(&self, ctx: &ResidencyContext) -> Result<(), ResidencyAdapterError> {
        self.emit_transfer_seal_detailed(ctx, ResidencyRule::RuleNotReported)
    }

    fn emit_transfer_seal_detailed(
        &self,
        ctx: &ResidencyContext,
        rule: ResidencyRule,
    ) -> Result<(), ResidencyAdapterError> {
        if self.unavailable.load(Ordering::SeqCst) {
            return Err(ResidencyAdapterError::AuditSinkUnavailable);
        }
        let mut state = self.state();
        if state.seals.len() >= self.capacity {
            return Err(ResidencyAdapterError::AuditSinkUnavailable);
        }
        let sequence = state.sequence;
        state.sequence = state.sequence.saturating_add(1);
        state.seals.push(ResidencyTransferSealRecord {
            context: ctx.clone(),
            rule,
            sequence,
        });
        Ok(())
    }
}

/// The default engine: the documented pack roster behind the hand-written
/// policy mirror, with NO transfer register.
///
/// It decides every route the corpus decides without a legal basis. Because it
/// holds no register, an asserted SCC or migration permit resolves to nothing
/// and the routes that depend on one deny — use
/// [`default_engine_with_register`] to give the engine a register to check
/// assertions against.
///
/// # Errors
///
/// [`ResidencyAdapterError::PolicyMalformed`] if the roster above ever stops
/// validating — which would mean this file, not the caller, is broken.
pub fn default_engine()
-> Result<crate::usecase::ResidencyPolicyEngine<InMemoryRegionCatalog>, ResidencyAdapterError> {
    crate::usecase::ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::oyatie_pack_roster())
}

/// The documented pack roster behind the policy mirror, checking asserted
/// transfer bases against `register`.
///
/// # Errors
///
/// As [`default_engine`].
pub fn default_engine_with_register<R: ResidencyTransferRegister>(
    register: R,
) -> Result<crate::usecase::ResidencyPolicyEngine<InMemoryRegionCatalog, R>, ResidencyAdapterError>
{
    crate::usecase::ResidencyPolicyEngine::try_new_with_register(
        InMemoryRegionCatalog::oyatie_pack_roster(),
        register,
    )
}

/// The rule a route would be decided by, re-evaluated now.
///
/// This RE-RUNS the engine against the live catalog and register, so it can
/// disagree with what a record already holds if either moved in between. It is
/// a convenience for exploring the matrix, NOT the way to learn why a recorded
/// denial happened — [`ResidencyDenialRecord::rule`] is, because it was written
/// at the moment of the decision.
///
/// # Errors
///
/// Propagates whatever [`crate::usecase::ResidencyPolicyEngine::evaluate_detailed`]
/// reports.
pub fn explain<C: ResidencyRegionCatalog, R: ResidencyTransferRegister>(
    engine: &crate::usecase::ResidencyPolicyEngine<C, R>,
    ctx: &ResidencyContext,
) -> Result<ResidencyRule, ResidencyAdapterError> {
    Ok(engine.evaluate_detailed(ctx)?.rule)
}
