//! DR-pairing kernel: the stable vocabulary — pair aggregate, lifecycle
//! state, blocking-reason enumeration, ports, audit event, and errors.
//!
//! Nothing in this module reads a clock, draws randomness, or performs
//! I/O: every time-dependent value arrives as an explicit parameter so a
//! promotion decision is reproducible from its recorded inputs.

use core::fmt;

/// Where a tenant's traffic sits in the failover lifecycle.
///
/// The transitions are closed, and [`crate::domain::is_legal_transition`]
/// is the single table that says so — every write path in this crate is
/// routed through it, including assignment:
///
/// - `Planned -> HomeActive` — activation.
/// - `HomeActive -> DrActive` — promotion (failover).
/// - `DrActive -> HomeActive` — restoration (failback).
/// - `HomeActive -> Planned` — re-planning: the DR side is re-cabled to a
///   cell that has not been exercised, so the pair loses its failover
///   capability until it is activated again. This edge is reachable ONLY
///   through [`crate::DrPairingController::replan_pair_to`], which exists
///   to make that withdrawal explicit and auditable. It is never a side
///   effect of [`crate::DrPairingController::assign_pair_to`], which
///   refuses to touch an activated pair.
///
/// `DrActive -> Planned` is not an edge: a pair is not re-cabled while it
/// is serving from its DR cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairState {
    /// The pair is recorded but has not been accepted for failover use.
    /// Promotion from this state is refused: an un-exercised DR cell is a
    /// plan, not a capability.
    Planned,
    /// Steady state — the home cell serves the tenant, the DR cell replicates.
    HomeActive,
    /// Failed over — the DR cell serves the tenant, the home cell does not.
    DrActive,
}

impl PairState {
    /// Stable lowercase label for logs, events, and dashboards.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::HomeActive => "home_active",
            Self::DrActive => "dr_active",
        }
    }
}

/// Whether an identifier is usable as a store key: non-blank, and carrying
/// no leading or trailing whitespace.
///
/// Padding is refused rather than trimmed because the padded and unpadded
/// forms are distinct keys in every store and cache this crate writes
/// through; accepting both would fork one tenant into two version chains.
#[must_use]
pub fn is_tight_identifier(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}

/// A tenant's home/DR cell pair.
///
/// Two invariants hold for every pair reachable through this crate's API
/// (see [`DrPair::validate`]): the two cells are distinct, and both sit in
/// the jurisdiction named by `jurisdiction`. The second is a data-residency
/// control, not a placement preference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrPair {
    /// The tenant this pair belongs to.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Cell that serves the tenant in steady state.
    pub home_cell: String, // data_class: TENANT_SCOPED
    /// Cell that replicates the tenant and may be promoted. Never equal to
    /// `home_cell`: a DR cell that is the home cell is not a DR cell.
    pub dr_cell: String, // data_class: TENANT_SCOPED
    /// Jurisdiction both cells sit in. Promotion may never move a tenant
    /// out of it.
    pub jurisdiction: String, // data_class: TENANT_SCOPED
    /// Optimistic-concurrency version. Every accepted transition bumps it;
    /// a write carrying a stale version is refused rather than applied.
    pub pair_version: u32, // data_class: INTERNAL_ONLY
    /// Which side is serving right now.
    pub state: PairState, // data_class: INTERNAL_ONLY
    /// Recovery-point objective for this pair, in seconds. A declared
    /// target carried with the pair — not a measured result.
    pub rpo_seconds: u32, // data_class: INTERNAL_ONLY
    /// Recovery-time objective for this pair, in seconds. A declared
    /// target carried with the pair — not a measured result.
    pub rto_seconds: u32, // data_class: INTERNAL_ONLY
}

impl DrPair {
    /// Check the shape invariants of a pair.
    ///
    /// Identifiers must be blank-free AND padding-free. Trimming would be
    /// worse than refusing: `"ten_alpha "` and `"ten_alpha"` are different
    /// keys in every store this crate writes through, so a padded id would
    /// open a second, invisible version chain for the same tenant and the
    /// compare-and-swap guard would never see across the two. A padded id
    /// is therefore rejected at the boundary rather than silently rewritten.
    ///
    /// # Errors
    /// - [`DrPairingError::InvalidTenantId`] when the tenant id is blank or padded.
    /// - [`DrPairingError::InvalidCellId`] when either cell id is blank or padded.
    /// - [`DrPairingError::InvalidJurisdiction`] when the jurisdiction is blank or padded.
    /// - [`DrPairingError::InvalidPairVersion`] when `pair_version` is 0 — every
    ///   stored pair is the result of a transition, and the first one writes 1.
    /// - [`DrPairingError::HomeCellIsDrCell`] when the two cells are equal.
    pub fn validate(&self) -> Result<(), DrPairingError> {
        if !is_tight_identifier(&self.tenant_id) {
            return Err(DrPairingError::InvalidTenantId);
        }
        if !is_tight_identifier(&self.home_cell) || !is_tight_identifier(&self.dr_cell) {
            return Err(DrPairingError::InvalidCellId);
        }
        if !is_tight_identifier(&self.jurisdiction) {
            return Err(DrPairingError::InvalidJurisdiction);
        }
        if self.pair_version == 0 {
            return Err(DrPairingError::InvalidPairVersion);
        }
        if self.home_cell == self.dr_cell {
            return Err(DrPairingError::HomeCellIsDrCell);
        }
        Ok(())
    }

    /// The cell currently serving the tenant, per [`PairState`].
    #[must_use]
    pub fn serving_cell(&self) -> &str {
        match self.state {
            PairState::DrActive => &self.dr_cell,
            PairState::Planned | PairState::HomeActive => &self.home_cell,
        }
    }
}

/// The outcome of a promotion assessment.
///
/// `Blocked` always carries a code from the [`reason`] enumeration, and
/// that code is never [`reason::UNSPECIFIED`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionDecision {
    /// The pair exists, is activated, still sits inside one jurisdiction,
    /// and the DR replica reported healthy.
    Eligible,
    /// Promotion is refused; `reason_code` names why. Look the code up with
    /// [`reason::text`].
    Blocked {
        /// A stable code from the [`reason`] module — not a magic number.
        reason_code: u16,
    },
}

/// The closed, stable enumeration of promotion-blocking reasons.
///
/// These codes appear verbatim in audit events and operator pages. They are
/// append-only: a code's meaning never changes and a retired code is never
/// reused, so a page from a year ago still reads correctly today.
pub mod reason {
    /// Not a blocking reason. Reserved so that `0` can never be read as
    /// both "unknown" and "the value a stub happened to return" — no
    /// [`super::PromotionDecision::Blocked`] produced by this crate carries
    /// it, and [`super::PromotionDecision::is_well_formed`] rejects one that does.
    pub const UNSPECIFIED: u16 = 0;
    /// No pair is recorded for the tenant. Assign one before promoting.
    pub const PAIR_NOT_FOUND: u16 = 1;
    /// The pair is still `planned` — the DR cell has not been accepted for
    /// failover use, so promoting to it is promoting to an unverified cell.
    pub const PAIR_NOT_ACTIVATED: u16 = 2;
    /// The pair is already serving from the DR cell. Promoting again would
    /// be a no-op at best and a split brain at worst.
    pub const ALREADY_PROMOTED: u16 = 3;
    /// The SLO probe reported the DR replica unhealthy. Distinct from a
    /// probe that failed to answer, which is [`super::DrPairingError::SloProbeFailed`].
    pub const DR_REPLICA_UNHEALTHY: u16 = 4;
    /// The two cells no longer agree on jurisdiction, or one of them has
    /// left the jurisdiction the pair was recorded under. Promotion would
    /// move the tenant's data across a residency boundary.
    pub const JURISDICTION_DRIFT: u16 = 5;
    /// The stored pair violates its own shape invariants — most often
    /// `home_cell == dr_cell` from a writer that predates this controller.
    pub const DEGENERATE_PAIR: u16 = 6;
    /// A cell named by the pair is no longer in the cell catalog, so its
    /// jurisdiction cannot be confirmed. Fail closed rather than assume.
    pub const CELL_UNKNOWN: u16 = 7;
    /// The residency policy refused this pair at assessment time.
    pub const RESIDENCY_POLICY_DENIED: u16 = 8;
    /// Failback was refused because the SLO probe reported the HOME replica
    /// unhealthy. Distinct from [`DR_REPLICA_UNHEALTHY`] so a page names the
    /// side that is actually broken.
    pub const HOME_REPLICA_UNHEALTHY: u16 = 9;

    /// Every blocking code paired with its one-line operator text, in code
    /// order. This is the lookup table behind a 3am page.
    pub const CATALOG: &[(u16, &str)] = &[
        (PAIR_NOT_FOUND, "no DR pair is recorded for this tenant"),
        (
            PAIR_NOT_ACTIVATED,
            "the DR pair is still planned and has not been activated",
        ),
        (
            ALREADY_PROMOTED,
            "the tenant is already serving from its DR cell",
        ),
        (
            DR_REPLICA_UNHEALTHY,
            "the SLO probe reported the DR replica unhealthy",
        ),
        (
            JURISDICTION_DRIFT,
            "the pair's cells no longer share the recorded jurisdiction",
        ),
        (
            DEGENERATE_PAIR,
            "the stored pair violates its shape invariants",
        ),
        (
            CELL_UNKNOWN,
            "a cell named by the pair is absent from the cell catalog",
        ),
        (
            RESIDENCY_POLICY_DENIED,
            "the residency policy refused this pair",
        ),
        (
            HOME_REPLICA_UNHEALTHY,
            "the SLO probe reported the home replica unhealthy",
        ),
    ];

    /// Operator-facing text for a blocking code.
    ///
    /// Returns `None` for [`UNSPECIFIED`] and for any code this build does
    /// not know, so an unrecognised code is visibly unrecognised rather
    /// than silently described as something else.
    #[must_use]
    pub fn text(code: u16) -> Option<&'static str> {
        CATALOG
            .iter()
            .find(|(known, _)| *known == code)
            .map(|(_, text)| *text)
    }
}

impl PromotionDecision {
    /// Whether this decision carries a code this build can explain.
    ///
    /// `Eligible` is always well formed; `Blocked` is well formed only when
    /// its code is in [`reason::CATALOG`], which excludes
    /// [`reason::UNSPECIFIED`].
    #[must_use]
    pub fn is_well_formed(self) -> bool {
        match self {
            Self::Eligible => true,
            Self::Blocked { reason_code } => reason::text(reason_code).is_some(),
        }
    }

    /// The blocking code, if this decision is a block.
    #[must_use]
    pub const fn reason_code(self) -> Option<u16> {
        match self {
            Self::Eligible => None,
            Self::Blocked { reason_code } => Some(reason_code),
        }
    }
}

impl fmt::Display for PromotionDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eligible => f.write_str("eligible"),
            Self::Blocked { reason_code } => match reason::text(*reason_code) {
                Some(text) => write!(f, "blocked[{reason_code}]: {text}"),
                None => write!(f, "blocked[{reason_code}]: unrecognised reason code"),
            },
        }
    }
}

/// A candidate cell offered by the cell catalog for DR placement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrCellCandidate {
    /// Catalog identity of the cell.
    pub cell_id: String, // data_class: INTERNAL_ONLY
    /// The jurisdiction this cell sits in.
    pub jurisdiction: String, // data_class: INTERNAL_ONLY
    /// Fault domain (rack/zone/site) the cell occupies. A DR cell in the
    /// home fault domain scores lower: it shares the failure it exists for.
    pub fault_domain: String, // data_class: INTERNAL_ONLY
    /// Whether the catalog currently reports the cell healthy.
    pub healthy: bool, // data_class: INTERNAL_ONLY
    /// Utilisation percentage, 0..=100. Values above 100 are rejected by
    /// [`DrCellCandidate::validate`].
    pub load_percent: u8, // data_class: INTERNAL_ONLY
}

impl DrCellCandidate {
    /// Check the shape invariants of a catalog candidate.
    ///
    /// # Errors
    /// - [`DrPairingError::InvalidCellId`] when the cell id is blank or padded.
    /// - [`DrPairingError::InvalidJurisdiction`] when the jurisdiction is blank or padded.
    /// - [`DrPairingError::InvalidLoadPercent`] when `load_percent > 100`.
    pub fn validate(&self) -> Result<(), DrPairingError> {
        if !is_tight_identifier(&self.cell_id) {
            return Err(DrPairingError::InvalidCellId);
        }
        if !is_tight_identifier(&self.jurisdiction) {
            return Err(DrPairingError::InvalidJurisdiction);
        }
        if self.load_percent > 100 {
            return Err(DrPairingError::InvalidLoadPercent);
        }
        Ok(())
    }
}

/// What kind of transition an audit event records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrPairEventKind {
    /// A pair was assigned for a tenant, or re-assigned while still
    /// `Planned`. The pair is `Planned` both before and after.
    PairAssigned,
    /// An activated pair was re-cabled to a different DR cell and returned
    /// to `Planned`. A distinct kind from [`Self::PairAssigned`] because it
    /// withdraws a failover capability the tenant had, and because it is
    /// the only event whose `from_state` is `HomeActive` and whose
    /// `to_state` is `Planned`.
    PairReplanned,
    /// A planned pair was accepted for failover use.
    PairActivated,
    /// The tenant was failed over to its DR cell.
    Promoted,
    /// The tenant was failed back to its home cell.
    Restored,
}

impl DrPairEventKind {
    /// Stable event-type label, matching the IP-019 topic names where one
    /// exists.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PairAssigned => "oya.tenancy.dr-pairing-assigned",
            Self::PairReplanned => "oya.tenancy.dr-pairing-replanned",
            Self::PairActivated => "oya.tenancy.dr-pairing-activated",
            Self::Promoted => "oya.tenancy.dr-pairing-promoted",
            Self::Restored => "oya.tenancy.dr-pairing-restored",
        }
    }

    /// The one `(from_state, to_state)` edge this kind records.
    ///
    /// The mapping is injective on `to_state` for every kind except the two
    /// that end in `Planned`, which is why re-planning has its own kind: it
    /// lets [`crate::DrPairingController::renarrate`] rebuild a lost event
    /// from the stored pair alone, with no ambiguity about where it came
    /// from. [`Self::PairAssigned`] reports `Planned -> Planned`; a first
    /// assignment has no predecessor state and is recorded from `Planned`.
    #[must_use]
    pub const fn edge(self) -> (PairState, PairState) {
        match self {
            Self::PairAssigned => (PairState::Planned, PairState::Planned),
            Self::PairReplanned => (PairState::HomeActive, PairState::Planned),
            Self::PairActivated => (PairState::Planned, PairState::HomeActive),
            Self::Promoted => (PairState::HomeActive, PairState::DrActive),
            Self::Restored => (PairState::DrActive, PairState::HomeActive),
        }
    }
}

/// One auditable DR-pairing transition.
///
/// Carries everything an auditor needs to re-derive the decision without
/// consulting another system: both cells, the jurisdiction, the version the
/// transition moved from and to, the decision that justified it, and the
/// instant the caller supplied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrPairAuditEvent {
    /// Which transition this is.
    pub kind: DrPairEventKind, // data_class: INTERNAL_ONLY
    /// Tenant the transition applies to.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Home cell at the time of the transition.
    pub home_cell: String, // data_class: TENANT_SCOPED
    /// DR cell at the time of the transition.
    pub dr_cell: String, // data_class: TENANT_SCOPED
    /// Jurisdiction both cells sat in.
    pub jurisdiction: String, // data_class: TENANT_SCOPED
    /// Lifecycle state before the transition.
    pub from_state: PairState, // data_class: INTERNAL_ONLY
    /// Lifecycle state after the transition.
    pub to_state: PairState, // data_class: INTERNAL_ONLY
    /// Pair version before the transition.
    pub from_version: u32, // data_class: INTERNAL_ONLY
    /// Pair version after the transition.
    pub to_version: u32, // data_class: INTERNAL_ONLY
    /// The promotion posture the controller had actually established when
    /// it committed this transition — see
    /// [`crate::domain::decision_for_event`], which derives it from `kind`
    /// alone so a lost event can be rebuilt byte-identically.
    ///
    /// Only [`DrPairEventKind::Promoted`] carries `Eligible`, because only
    /// a promotion runs the assessment that establishes it. Assignment,
    /// re-planning, and activation record `Blocked { PAIR_NOT_ACTIVATED }`;
    /// restoration records `Blocked { ALREADY_PROMOTED }`, the posture of
    /// the pair it failed back. No event asserts a health signal that the
    /// transition did not probe.
    pub decision: PromotionDecision, // data_class: INTERNAL_ONLY
    /// Caller-supplied instant, in milliseconds since the Unix epoch. The
    /// controller never reads a clock itself.
    pub at_millis: u64, // data_class: INTERNAL_ONLY
    /// Deterministic key over (tenant, kind, resulting version). Re-emitting
    /// after a sink failure produces the same key, so the sink can dedupe.
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    /// Caller-supplied audit-chain correlation id.
    pub correlation_id: String, // data_class: INTERNAL_ONLY
}

/// Persistence port for the pair aggregate.
pub trait DrPairRepository {
    /// The tenant's current pair, or `None` when none is recorded.
    ///
    /// # Errors
    /// [`DrPairingError::PersistenceUnavailable`] when the store cannot answer.
    fn current(&self, tenant_id: &str) -> Result<Option<DrPair>, DrPairingError>;

    /// Write a pair unconditionally.
    ///
    /// This is a PORT operation, not the application's write path. It is
    /// how [`Self::compare_and_swap`] lands its bytes and how a migration
    /// backfills a store; it performs no residency check, because a store
    /// cannot see the cell catalog, and no promotion check. Application
    /// code assigns and moves pairs through
    /// [`crate::DrPairingController`], which owns those controls. A caller
    /// that reaches past the controller to `record` is writing rows the
    /// controller never agreed to.
    ///
    /// Implementations MUST reject a pair that fails [`DrPair::validate`],
    /// and MUST refuse a write that would move `pair_version` backwards or
    /// leave it unchanged: the split-brain guard in
    /// [`Self::compare_and_swap`] is only as good as the monotonicity of
    /// the version chain underneath it.
    ///
    /// # Errors
    /// - [`DrPairingError::PersistenceUnavailable`] when the store cannot
    ///   accept the write.
    /// - [`DrPairingError::NonMonotonicPairVersion`] when the write would
    ///   not advance the stored version.
    /// - A validation error from [`DrPair::validate`].
    fn record(&self, pair: &DrPair) -> Result<(), DrPairingError>;

    /// Write a pair only if the stored version still matches
    /// `expected_version` (`None` meaning "no pair is stored yet").
    ///
    /// This is the split-brain guard: two controllers that both read
    /// version 4 and both try to promote produce one winner and one
    /// [`DrPairingError::StalePairVersion`], instead of two promotions.
    ///
    /// The default implementation is a read-then-write and is therefore
    /// only as atomic as the caller's own serialisation; a real store
    /// overrides it with a conditional write. The in-memory adapter in
    /// [`crate::inmemory`] overrides it under one lock.
    ///
    /// # Errors
    /// - [`DrPairingError::StalePairVersion`] when the stored version moved.
    /// - [`DrPairingError::PersistenceUnavailable`] when the store cannot answer.
    fn compare_and_swap(
        &self,
        expected_version: Option<u32>,
        pair: &DrPair,
    ) -> Result<(), DrPairingError> {
        let found = self.current(&pair.tenant_id)?.map(|p| p.pair_version);
        if found != expected_version {
            return Err(DrPairingError::StalePairVersion);
        }
        self.record(pair)
    }
}

/// Health signals for the two sides of a pair.
///
/// The two questions are genuinely different and an implementation may
/// answer them from different signals: a DR replica is judged on
/// replication lag and readiness to be promoted, a home primary on whether
/// it is serving. Neither method has a default implementation, because a
/// default would let an adapter answer one question with the other — which
/// is exactly the confusion that would strand a tenant in its DR cell.
pub trait DrSloProbe {
    /// Whether the DR replica in `cell` is healthy enough to serve.
    ///
    /// # Errors
    /// [`DrPairingError::SloProbeFailed`] when the probe cannot answer. A
    /// probe that cannot answer is NOT a healthy probe and NOT a plain
    /// block: the caller fails closed on a distinguishable error.
    fn dr_replica_health(&self, cell: &str) -> Result<bool, DrPairingError>;

    /// Whether the HOME side in `cell` is healthy enough to take traffic
    /// back. Asked only at failback, where the tenant is serving from its
    /// DR cell and the home cell hosts no DR replica to ask about.
    ///
    /// # Errors
    /// [`DrPairingError::SloProbeFailed`] when the probe cannot answer.
    fn home_replica_health(&self, cell: &str) -> Result<bool, DrPairingError>;
}

/// Read model of the cell fleet.
///
/// Locally declared rather than imported: IP-019 sources this from the
/// sibling `cell-assignment` bounded context, which this crate may not
/// depend on (see the `Gaps` paragraph in the crate docs).
pub trait DrCellCatalog {
    /// The jurisdiction a cell sits in, or `None` when the cell is unknown.
    ///
    /// # Errors
    /// [`DrPairingError::CellCatalogUnavailable`] when the catalog cannot answer.
    fn jurisdiction_of(&self, cell: &str) -> Result<Option<String>, DrPairingError>;

    /// Every candidate cell the catalog offers inside `jurisdiction`.
    ///
    /// # Errors
    /// [`DrPairingError::CellCatalogUnavailable`] when the catalog cannot answer.
    fn candidates(&self, jurisdiction: &str) -> Result<Vec<DrCellCandidate>, DrPairingError>;
}

/// Residency decision port.
///
/// Locally declared rather than imported: IP-019 evaluates
/// `tenancy/policy/data-residency.cedar`, which this crate may not link
/// (see the `Gaps` paragraph in the crate docs).
pub trait ResidencyPolicy {
    /// Whether `tenant_id` may hold a pair of `home_cell`/`dr_cell` inside
    /// `jurisdiction`. A jurisdiction match is necessary but not
    /// sufficient — the policy may still refuse a specific placement.
    ///
    /// # Errors
    /// [`DrPairingError::ResidencyPolicyUnavailable`] when the policy engine
    /// cannot answer. Callers fail closed.
    fn permits_pair(
        &self,
        tenant_id: &str,
        jurisdiction: &str,
        home_cell: &str,
        dr_cell: &str,
    ) -> Result<bool, DrPairingError>;
}

/// Audit-trail port. Synchronous by construction: the controller commits
/// nothing it cannot describe.
pub trait DrPairEventSink {
    /// Record one transition.
    ///
    /// # Errors
    /// [`DrPairingError::AuditEmitUnavailable`] when the sink cannot accept
    /// the event.
    fn emit(&self, event: &DrPairAuditEvent) -> Result<(), DrPairingError>;
}

/// Every way a DR-pairing operation can fail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrPairingError {
    /// No pair is recorded for the tenant and the operation needs one.
    PairNotFound,
    /// The requested pair would place a tenant's DR cell outside the
    /// jurisdiction of its home cell. A residency control, not a preference.
    JurisdictionMismatch,
    /// The SLO probe could not answer. Distinct from "answered unhealthy".
    SloProbeFailed,
    /// The pair store could not answer or accept a write.
    PersistenceUnavailable,
    /// The write carried a version older than the stored one. The write was
    /// refused; re-read and retry.
    StalePairVersion,
    /// `home_cell == dr_cell`. A DR cell that is the home cell is not a DR cell.
    HomeCellIsDrCell,
    /// No catalog candidate satisfies the same-jurisdiction placement rules.
    NoEligibleDrCell,
    /// A cell named by the request or the stored pair is not in the catalog.
    CellNotInCatalog,
    /// The cell catalog could not answer.
    CellCatalogUnavailable,
    /// The residency policy engine could not answer.
    ResidencyPolicyUnavailable,
    /// The residency policy refused the placement.
    ResidencyPolicyDenied,
    /// The audit sink could not accept the event. Returned by the sink port
    /// itself and by [`crate::DrPairingController::renarrate`]; a
    /// controller transition that had already committed reports
    /// [`Self::NarrationPending`] instead, so "the sink is down" and "your
    /// write landed but is not yet narrated" never share an error.
    AuditEmitUnavailable,
    /// The transition committed durably at `committed_version`, and the
    /// audit sink then refused its event. The write DID land — this is not
    /// [`Self::StalePairVersion`] and not a race with another writer.
    ///
    /// Recover with [`crate::DrPairingController::renarrate`], passing the
    /// same event kind and a command carrying `committed_version` together
    /// with the original `at_millis` and `correlation_id`. The rebuilt
    /// event is byte-identical, so its idempotency key is unchanged and a
    /// deduping sink accepts it exactly once.
    NarrationPending {
        /// The version the committed transition produced.
        committed_version: u32,
    },
    /// An assignment named a different home cell than the stored pair
    /// carries. The home cell records where the tenant is actually served
    /// from; changing it here would move `serving_cell` without moving the
    /// tenant. Re-home the tenant first, then assign.
    HomeCellImmutable,
    /// A stored or proposed pair carries `pair_version: 0`. Every stored
    /// pair is the product of a transition and the first one writes 1.
    InvalidPairVersion,
    /// A write would have left the stored `pair_version` where it is or
    /// moved it backwards. Refused: a version chain that can go backwards
    /// lets a replayed command pass a compare-and-swap it should fail.
    NonMonotonicPairVersion,
    /// The requested move is not an edge of the failover state machine —
    /// activating an already-active pair, failing back a pair that never
    /// failed over, or re-assigning a pair mid-failover.
    IllegalTransition,
    /// Promotion was assessed and refused; `reason_code` is from [`reason`].
    PromotionBlocked {
        /// A stable code from the [`reason`] module.
        reason_code: u16,
    },
    /// The tenant id is blank.
    InvalidTenantId,
    /// A cell id is blank.
    InvalidCellId,
    /// A jurisdiction label is blank.
    InvalidJurisdiction,
    /// A candidate reported a load percentage above 100.
    InvalidLoadPercent,
    /// `pair_version` reached `u32::MAX` and cannot be bumped again.
    PairVersionExhausted,
}

impl fmt::Display for DrPairingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PairNotFound => f.write_str("no DR pair is recorded for this tenant"),
            Self::JurisdictionMismatch => {
                f.write_str("home and DR cells are in different jurisdictions")
            }
            Self::SloProbeFailed => f.write_str("the DR SLO probe could not answer"),
            Self::PersistenceUnavailable => f.write_str("the DR pair store is unavailable"),
            Self::StalePairVersion => {
                f.write_str("the write carried a stale pair version and was refused")
            }
            Self::HomeCellIsDrCell => f.write_str("the DR cell is the home cell"),
            Self::NoEligibleDrCell => {
                f.write_str("no same-jurisdiction candidate cell is eligible")
            }
            Self::CellNotInCatalog => f.write_str("a named cell is absent from the cell catalog"),
            Self::CellCatalogUnavailable => f.write_str("the cell catalog is unavailable"),
            Self::ResidencyPolicyUnavailable => f.write_str("the residency policy is unavailable"),
            Self::ResidencyPolicyDenied => {
                f.write_str("the residency policy refused this placement")
            }
            Self::AuditEmitUnavailable => f.write_str("the DR-pairing audit sink is unavailable"),
            Self::NarrationPending { committed_version } => write!(
                f,
                "the transition committed at version {committed_version} but its audit event was not accepted; re-narrate it"
            ),
            Self::HomeCellImmutable => {
                f.write_str("the stored pair's home cell cannot be changed by an assignment")
            }
            Self::InvalidPairVersion => f.write_str("a pair version of 0 is not a stored version"),
            Self::NonMonotonicPairVersion => {
                f.write_str("the write would not advance the stored pair version")
            }
            Self::IllegalTransition => {
                f.write_str("the requested move is not a legal failover transition")
            }
            Self::PromotionBlocked { reason_code } => match reason::text(*reason_code) {
                Some(text) => write!(f, "promotion blocked[{reason_code}]: {text}"),
                None => write!(f, "promotion blocked[{reason_code}]"),
            },
            Self::InvalidTenantId => f.write_str("the tenant id is blank"),
            Self::InvalidCellId => f.write_str("a cell id is blank"),
            Self::InvalidJurisdiction => f.write_str("a jurisdiction label is blank"),
            Self::InvalidLoadPercent => f.write_str("a candidate load percentage exceeds 100"),
            Self::PairVersionExhausted => f.write_str("the pair version space is exhausted"),
        }
    }
}

impl std::error::Error for DrPairingError {}
