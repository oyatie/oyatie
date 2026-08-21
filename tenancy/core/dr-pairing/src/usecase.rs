//! The DR-pairing controller: assignment, promotion assessment, failover
//! and failback, each committed through an optimistic-concurrency write and
//! each narrated to the audit sink.
//!
//! Composition is by reference, so a caller owns its ports and the
//! controller is a thin, cheap value. Every method that changes state takes
//! the version it believes it is changing; a write whose version has moved
//! is refused, never applied. That refusal is the split-brain guard: two
//! controllers that both read version 4 produce one promotion and one
//! [`DrPairingError::StalePairVersion`].

use crate::domain::{
    decision_for_event, derive_idempotency_key, is_legal_transition, next_version,
    permits_in_place_reassignment, promotion_block_for_state, select_dr_cell,
};
use crate::kernel::{
    DrCellCatalog, DrPair, DrPairAuditEvent, DrPairEventKind, DrPairEventSink, DrPairRepository,
    DrPairingError, DrSloProbe, PairState, PromotionDecision, ResidencyPolicy, is_tight_identifier,
    reason,
};

/// A request to assign (or re-assign) a tenant's DR pair.
///
/// `expected_version` is the version the caller believes is stored:
/// `None` for a first assignment, `Some(v)` for a re-assignment. A mismatch
/// is refused with [`DrPairingError::StalePairVersion`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairAssignment {
    /// Tenant to assign a pair for.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// The tenant's home cell.
    pub home_cell: String, // data_class: TENANT_SCOPED
    /// Jurisdiction the pair must stay inside.
    pub jurisdiction: String, // data_class: TENANT_SCOPED
    /// Fault domain of the home cell, used to prefer a separated DR cell.
    pub home_fault_domain: String, // data_class: INTERNAL_ONLY
    /// Declared recovery-point objective, in seconds.
    pub rpo_seconds: u32, // data_class: INTERNAL_ONLY
    /// Declared recovery-time objective, in seconds.
    pub rto_seconds: u32, // data_class: INTERNAL_ONLY
    /// Version the caller believes is stored; `None` means "no pair yet".
    pub expected_version: Option<u32>, // data_class: INTERNAL_ONLY
    /// Caller-supplied instant, milliseconds since the Unix epoch.
    pub at_millis: u64, // data_class: INTERNAL_ONLY
    /// Audit-chain correlation id for the emitted event.
    pub correlation_id: String, // data_class: INTERNAL_ONLY
}

/// A request to move a pair across the failover state machine — activation,
/// promotion, or restoration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailoverCommand {
    /// Tenant whose pair is moving.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Version the caller believes is stored. A stale value is refused.
    pub expected_version: u32, // data_class: INTERNAL_ONLY
    /// Caller-supplied instant, milliseconds since the Unix epoch.
    pub at_millis: u64, // data_class: INTERNAL_ONLY
    /// Audit-chain correlation id for the emitted event.
    pub correlation_id: String, // data_class: INTERNAL_ONLY
}

/// One committed transition, ready to be narrated to the audit sink.
/// Private: it exists to keep the emit path a single argument rather than
/// seven positional ones.
struct TransitionRecord<'r> {
    kind: DrPairEventKind,
    pair: &'r DrPair,
    from_state: PairState,
    from_version: u32,
    at_millis: u64,
    correlation_id: &'r str,
}

/// The DR-pairing control plane over its five ports.
pub struct DrPairingController<'a, R, S, C, P, E> {
    repo: &'a R,
    probe: &'a S,
    catalog: &'a C,
    policy: &'a P,
    sink: &'a E,
}

impl<'a, R, S, C, P, E> DrPairingController<'a, R, S, C, P, E>
where
    R: DrPairRepository,
    S: DrSloProbe,
    C: DrCellCatalog,
    P: ResidencyPolicy,
    E: DrPairEventSink,
{
    /// Build a controller over borrowed ports.
    pub const fn new(
        repo: &'a R,
        probe: &'a S,
        catalog: &'a C,
        policy: &'a P,
        sink: &'a E,
    ) -> Self {
        Self {
            repo,
            probe,
            catalog,
            policy,
            sink,
        }
    }

    /// Confirm that a cell is in the catalog and sits in `jurisdiction`.
    ///
    /// # Errors
    /// - [`DrPairingError::CellNotInCatalog`] when the catalog does not know the cell.
    /// - [`DrPairingError::JurisdictionMismatch`] when it sits elsewhere.
    /// - [`DrPairingError::CellCatalogUnavailable`] when the catalog cannot answer.
    fn require_cell_in(&self, cell: &str, jurisdiction: &str) -> Result<(), DrPairingError> {
        match self.catalog.jurisdiction_of(cell)? {
            None => Err(DrPairingError::CellNotInCatalog),
            Some(found) if found == jurisdiction => Ok(()),
            Some(_) => Err(DrPairingError::JurisdictionMismatch),
        }
    }

    /// Assign a DR pair, choosing the DR cell from the catalog.
    ///
    /// The chosen cell is always in the same jurisdiction as the home cell
    /// and is never the home cell — [`select_dr_cell`] applies both as hard
    /// filters, and [`Self::assign_pair_to`] re-checks them against the
    /// catalog before the write.
    ///
    /// # Errors
    /// Every error of [`Self::assign_pair_to`], plus
    /// [`DrPairingError::NoEligibleDrCell`] when the catalog offers no
    /// same-jurisdiction, non-home, healthy candidate.
    pub fn assign_pair(&self, assignment: &PairAssignment) -> Result<DrPair, DrPairingError> {
        self.require_cell_in(&assignment.home_cell, &assignment.jurisdiction)?;
        let candidates = self.catalog.candidates(&assignment.jurisdiction)?;
        let chosen = select_dr_cell(
            &assignment.home_cell,
            &assignment.home_fault_domain,
            &assignment.jurisdiction,
            &candidates,
        )?
        .cell_id
        .clone();
        self.assign_pair_to(assignment, &chosen)
    }

    /// Assign a DR pair to a named DR cell.
    ///
    /// This is the application's only first-write path for a pair, and it
    /// is closed against cross-jurisdiction placement: the catalog must
    /// place BOTH cells in `assignment.jurisdiction`, and the residency
    /// policy must permit the specific placement. A pair that fails either
    /// check is refused, not downgraded to a warning.
    ///
    /// It writes [`PairState::Planned`], because a DR cell that has just
    /// been chosen has not been exercised. It therefore refuses to touch a
    /// pair that has been activated: overwriting a `HomeActive` pair with a
    /// plan would leave the tenant serving normally while its failover
    /// capability silently disappeared, and the state machine has no such
    /// edge. Re-cabling an activated pair is [`Self::replan_pair_to`],
    /// which performs the same withdrawal deliberately and audibly. A pair
    /// that is `DrActive` is not re-cabled at all.
    ///
    /// The home cell of a stored pair is immutable here: it records where
    /// the tenant is actually served from, so changing it would move
    /// [`DrPair::serving_cell`] without moving the tenant.
    ///
    /// # Errors
    /// - [`DrPairingError::InvalidTenantId`] / [`DrPairingError::InvalidCellId`] /
    ///   [`DrPairingError::InvalidJurisdiction`] on blank or padded input.
    /// - [`DrPairingError::HomeCellIsDrCell`] when the two cells are equal.
    /// - [`DrPairingError::CellNotInCatalog`] / [`DrPairingError::JurisdictionMismatch`]
    ///   when either cell is unknown or outside `assignment.jurisdiction`.
    /// - [`DrPairingError::ResidencyPolicyDenied`] when the policy refuses.
    /// - [`DrPairingError::IllegalTransition`] when the stored pair is not `Planned`.
    /// - [`DrPairingError::HomeCellImmutable`] when the stored pair names a
    ///   different home cell.
    /// - [`DrPairingError::StalePairVersion`] when `expected_version` has moved.
    /// - [`DrPairingError::NarrationPending`] when the pair committed but the
    ///   audit sink refused its event — recover with [`Self::renarrate`].
    /// - [`DrPairingError::PairVersionExhausted`], [`DrPairingError::PersistenceUnavailable`],
    ///   [`DrPairingError::CellCatalogUnavailable`],
    ///   [`DrPairingError::ResidencyPolicyUnavailable`] from the ports.
    pub fn assign_pair_to(
        &self,
        assignment: &PairAssignment,
        dr_cell: &str,
    ) -> Result<DrPair, DrPairingError> {
        self.write_plan(assignment, dr_cell, DrPairEventKind::PairAssigned)
    }

    /// Re-cable an ACTIVATED pair to a different DR cell: `HomeActive ->
    /// Planned`.
    ///
    /// This is the explicit form of the withdrawal that
    /// [`Self::assign_pair_to`] refuses to perform by accident. The new DR
    /// cell has not been exercised, so the pair returns to
    /// [`PairState::Planned`] and promotion is refused with
    /// [`reason::PAIR_NOT_ACTIVATED`] until [`Self::activate_pair`] runs
    /// again. The audit trail records it under its own event kind,
    /// [`DrPairEventKind::PairReplanned`], so a reader can see that a
    /// tenant's failover capability was deliberately stood down.
    ///
    /// Every residency and shape check that [`Self::assign_pair_to`] runs
    /// runs here too, and a `DrActive` pair is refused: a pair is not
    /// re-cabled while it is serving from its DR cell.
    ///
    /// # Errors
    /// Every error of [`Self::assign_pair_to`], plus
    /// [`DrPairingError::PairNotFound`] when no pair is stored — there is
    /// nothing to re-cable.
    pub fn replan_pair_to(
        &self,
        assignment: &PairAssignment,
        dr_cell: &str,
    ) -> Result<DrPair, DrPairingError> {
        self.write_plan(assignment, dr_cell, DrPairEventKind::PairReplanned)
    }

    /// The shared body of [`Self::assign_pair_to`] and
    /// [`Self::replan_pair_to`]: validate, re-check residency, refuse an
    /// illegal edge, commit under compare-and-swap, then narrate.
    ///
    /// `kind` decides which stored state is admissible, via
    /// [`DrPairEventKind::edge`] — the same table the state machine uses,
    /// so neither path can write an edge the machine denies.
    fn write_plan(
        &self,
        assignment: &PairAssignment,
        dr_cell: &str,
        kind: DrPairEventKind,
    ) -> Result<DrPair, DrPairingError> {
        if !is_tight_identifier(&assignment.tenant_id) {
            return Err(DrPairingError::InvalidTenantId);
        }
        if !is_tight_identifier(&assignment.home_cell) || !is_tight_identifier(dr_cell) {
            return Err(DrPairingError::InvalidCellId);
        }
        if !is_tight_identifier(&assignment.jurisdiction) {
            return Err(DrPairingError::InvalidJurisdiction);
        }
        if assignment.home_cell == dr_cell {
            return Err(DrPairingError::HomeCellIsDrCell);
        }
        self.require_cell_in(&assignment.home_cell, &assignment.jurisdiction)?;
        self.require_cell_in(dr_cell, &assignment.jurisdiction)?;
        if !self.policy.permits_pair(
            &assignment.tenant_id,
            &assignment.jurisdiction,
            &assignment.home_cell,
            dr_cell,
        )? {
            return Err(DrPairingError::ResidencyPolicyDenied);
        }

        let stored = self.repo.current(&assignment.tenant_id)?;
        let stored_version = stored.as_ref().map(|pair| pair.pair_version);
        if stored_version != assignment.expected_version {
            return Err(DrPairingError::StalePairVersion);
        }
        let from_state = match stored.as_ref() {
            None => {
                if kind == DrPairEventKind::PairReplanned {
                    return Err(DrPairingError::PairNotFound);
                }
                PairState::Planned
            }
            Some(current) => {
                if current.home_cell != assignment.home_cell {
                    return Err(DrPairingError::HomeCellImmutable);
                }
                let admissible = match kind {
                    // A plan may be rewritten in place; an activated pair
                    // may not be demoted by an assignment.
                    DrPairEventKind::PairAssigned => permits_in_place_reassignment(current.state),
                    // Re-planning walks a real edge of the state machine.
                    _ => is_legal_transition(current.state, PairState::Planned),
                };
                if !admissible {
                    return Err(DrPairingError::IllegalTransition);
                }
                current.state
            }
        };
        let to_version = match stored_version {
            None => 1,
            Some(current) => next_version(current)?,
        };

        let pair = DrPair {
            tenant_id: assignment.tenant_id.clone(),
            home_cell: assignment.home_cell.clone(),
            dr_cell: dr_cell.to_owned(),
            jurisdiction: assignment.jurisdiction.clone(),
            pair_version: to_version,
            state: PairState::Planned,
            rpo_seconds: assignment.rpo_seconds,
            rto_seconds: assignment.rto_seconds,
        };
        pair.validate()?;
        self.repo.compare_and_swap(stored_version, &pair)?;
        self.emit(&TransitionRecord {
            kind,
            pair: &pair,
            from_state,
            from_version: stored_version.unwrap_or(0),
            at_millis: assignment.at_millis,
            correlation_id: &assignment.correlation_id,
        })
        .map_err(|_unnarrated| DrPairingError::NarrationPending {
            committed_version: to_version,
        })?;
        Ok(pair)
    }

    /// Accept a planned pair for failover use: `Planned -> HomeActive`.
    ///
    /// Until a pair is activated, promotion is refused with
    /// [`reason::PAIR_NOT_ACTIVATED`] — an un-exercised DR cell is a plan,
    /// not a capability.
    ///
    /// # Errors
    /// - [`DrPairingError::PairNotFound`] when no pair is stored.
    /// - [`DrPairingError::StalePairVersion`] when `expected_version` has moved.
    /// - [`DrPairingError::IllegalTransition`] when the pair is not `Planned`.
    /// - [`DrPairingError::NarrationPending`] when the activation committed
    ///   but the audit sink refused its event — recover with [`Self::renarrate`].
    /// - Port errors from the store and the audit sink.
    pub fn activate_pair(&self, command: &FailoverCommand) -> Result<DrPair, DrPairingError> {
        let stored = self.load_at_version(command)?;
        self.transition(
            &stored,
            PairState::HomeActive,
            DrPairEventKind::PairActivated,
            command,
        )
    }

    /// Assess whether the tenant may be promoted to its DR cell.
    ///
    /// Precedence, highest first: pair exists, pair shape is sound,
    /// lifecycle state permits promotion, both cells are known to the
    /// catalog, the recorded jurisdiction still holds for both, the
    /// residency policy still permits the pair, the DR replica is healthy.
    /// The order is fixed so the same fault always reports the same code.
    ///
    /// A probe that cannot answer is NOT a block: it returns
    /// [`DrPairingError::SloProbeFailed`], so "unknown" can never be read
    /// as "healthy" and can never be mistaken for one of the reasoned
    /// refusals.
    ///
    /// # Errors
    /// - [`DrPairingError::SloProbeFailed`] when the probe cannot answer.
    /// - [`DrPairingError::PersistenceUnavailable`],
    ///   [`DrPairingError::CellCatalogUnavailable`],
    ///   [`DrPairingError::ResidencyPolicyUnavailable`] from the ports.
    pub fn assess_promotion(&self, tenant_id: &str) -> Result<PromotionDecision, DrPairingError> {
        let Some(pair) = self.repo.current(tenant_id)? else {
            return Ok(PromotionDecision::Blocked {
                reason_code: reason::PAIR_NOT_FOUND,
            });
        };
        self.assess_stored(&pair)
    }

    /// Assess a pair the caller has already read.
    ///
    /// [`Self::promote`] uses this rather than re-reading, so the pair it
    /// assesses is exactly the pair it commits against — a second read
    /// could see a different row and produce a decision about one pair
    /// while the compare-and-swap acts on another.
    ///
    /// # Errors
    /// The errors of [`Self::assess_promotion`], minus the store read.
    pub fn assess_stored(&self, pair: &DrPair) -> Result<PromotionDecision, DrPairingError> {
        if pair.validate().is_err() {
            return Ok(PromotionDecision::Blocked {
                reason_code: reason::DEGENERATE_PAIR,
            });
        }
        if let Some(reason_code) = promotion_block_for_state(pair.state) {
            return Ok(PromotionDecision::Blocked { reason_code });
        }
        if let Some(reason_code) = self.residency_block(pair)? {
            return Ok(PromotionDecision::Blocked { reason_code });
        }
        if self.probe.dr_replica_health(&pair.dr_cell)? {
            Ok(PromotionDecision::Eligible)
        } else {
            Ok(PromotionDecision::Blocked {
                reason_code: reason::DR_REPLICA_UNHEALTHY,
            })
        }
    }

    /// The residency-shaped blocking reason for a stored pair, if any.
    fn residency_block(&self, pair: &DrPair) -> Result<Option<u16>, DrPairingError> {
        let home = self.catalog.jurisdiction_of(&pair.home_cell)?;
        let dr = self.catalog.jurisdiction_of(&pair.dr_cell)?;
        let (Some(home), Some(dr)) = (home, dr) else {
            return Ok(Some(reason::CELL_UNKNOWN));
        };
        if home != pair.jurisdiction || dr != pair.jurisdiction {
            return Ok(Some(reason::JURISDICTION_DRIFT));
        }
        if !self.policy.permits_pair(
            &pair.tenant_id,
            &pair.jurisdiction,
            &pair.home_cell,
            &pair.dr_cell,
        )? {
            return Ok(Some(reason::RESIDENCY_POLICY_DENIED));
        }
        Ok(None)
    }

    /// Fail the tenant over to its DR cell: `HomeActive -> DrActive`.
    ///
    /// Refuses unless [`Self::assess_promotion`] says `Eligible`, and
    /// refuses a command carrying a version that has already moved. The
    /// version bump and the audit event are part of the promotion, not a
    /// follow-up.
    ///
    /// # Errors
    /// - [`DrPairingError::PairNotFound`] when no pair is stored.
    /// - [`DrPairingError::StalePairVersion`] when `expected_version` has moved.
    /// - [`DrPairingError::PromotionBlocked`] carrying the [`reason`] code.
    /// - [`DrPairingError::SloProbeFailed`] when the probe cannot answer.
    /// - [`DrPairingError::NarrationPending`] when the promotion committed
    ///   but the audit sink refused its event. The failover DID happen;
    ///   finish the record with [`Self::renarrate`] rather than retrying
    ///   the promotion, which would now be stale.
    /// - Port errors from the store, catalog, policy, and audit sink.
    pub fn promote(&self, command: &FailoverCommand) -> Result<DrPair, DrPairingError> {
        let stored = self.load_at_version(command)?;
        if let PromotionDecision::Blocked { reason_code } = self.assess_stored(&stored)? {
            return Err(DrPairingError::PromotionBlocked { reason_code });
        }
        self.transition(
            &stored,
            PairState::DrActive,
            DrPairEventKind::Promoted,
            command,
        )
    }

    /// Fail the tenant back to its home cell: `DrActive -> HomeActive`.
    ///
    /// Failback is gated on [`DrSloProbe::home_replica_health`] for the home
    /// cell. Asking [`DrSloProbe::dr_replica_health`] about the home cell
    /// would be a different question with a plausible-looking answer: a
    /// faithful adapter finds no DR replica in a cell that hosts a primary
    /// and reports "not healthy", stranding a recovered tenant in its DR
    /// cell forever. The same residency checks promotion runs also run
    /// here, because a jurisdiction that drifted while the tenant was
    /// failed over is still a residency incident.
    ///
    /// # Errors
    /// - [`DrPairingError::PairNotFound`] when no pair is stored.
    /// - [`DrPairingError::StalePairVersion`] when `expected_version` has moved.
    /// - [`DrPairingError::IllegalTransition`] when the pair is not `DrActive`.
    /// - [`DrPairingError::PromotionBlocked`] carrying
    ///   [`reason::HOME_REPLICA_UNHEALTHY`] or a residency code.
    /// - [`DrPairingError::SloProbeFailed`] when the home probe cannot answer.
    /// - [`DrPairingError::NarrationPending`] when the failback committed
    ///   but the audit sink refused its event — recover with [`Self::renarrate`].
    /// - Port errors from the store, catalog, policy, and audit sink.
    pub fn restore(&self, command: &FailoverCommand) -> Result<DrPair, DrPairingError> {
        let stored = self.load_at_version(command)?;
        if stored.state != PairState::DrActive {
            return Err(DrPairingError::IllegalTransition);
        }
        stored.validate()?;
        if let Some(reason_code) = self.residency_block(&stored)? {
            return Err(DrPairingError::PromotionBlocked { reason_code });
        }
        if !self.probe.home_replica_health(&stored.home_cell)? {
            return Err(DrPairingError::PromotionBlocked {
                reason_code: reason::HOME_REPLICA_UNHEALTHY,
            });
        }
        self.transition(
            &stored,
            PairState::HomeActive,
            DrPairEventKind::Restored,
            command,
        )
    }

    /// Load the stored pair and refuse a command whose version has moved.
    fn load_at_version(&self, command: &FailoverCommand) -> Result<DrPair, DrPairingError> {
        let stored = self
            .repo
            .current(&command.tenant_id)?
            .ok_or(DrPairingError::PairNotFound)?;
        if stored.pair_version != command.expected_version {
            return Err(DrPairingError::StalePairVersion);
        }
        Ok(stored)
    }

    /// Apply one legal state-machine edge: bump the version, commit under
    /// compare-and-swap, then narrate it.
    fn transition(
        &self,
        stored: &DrPair,
        to_state: PairState,
        kind: DrPairEventKind,
        command: &FailoverCommand,
    ) -> Result<DrPair, DrPairingError> {
        if !is_legal_transition(stored.state, to_state) {
            return Err(DrPairingError::IllegalTransition);
        }
        let to_version = next_version(stored.pair_version)?;
        let moved = DrPair {
            pair_version: to_version,
            state: to_state,
            ..stored.clone()
        };
        moved.validate()?;
        self.repo
            .compare_and_swap(Some(stored.pair_version), &moved)?;
        self.emit(&TransitionRecord {
            kind,
            pair: &moved,
            from_state: stored.state,
            from_version: stored.pair_version,
            at_millis: command.at_millis,
            correlation_id: &command.correlation_id,
        })
        .map_err(|_unnarrated| DrPairingError::NarrationPending {
            committed_version: to_version,
        })?;
        Ok(moved)
    }

    /// Re-emit the audit event for a transition that committed but was not
    /// narrated, reported as [`DrPairingError::NarrationPending`].
    ///
    /// This is what makes "committed but not narrated" a recoverable state
    /// rather than a permanent hole in the compliance trail. Every field of
    /// the event is a function of the stored pair, the event kind, and the
    /// caller's own `at_millis`/`correlation_id`: `to_version` is the
    /// stored version, `from_version` is one below it (every transition
    /// bumps by exactly one), the states come from
    /// [`DrPairEventKind::edge`], and the decision comes from
    /// [`decision_for_event`]. Nothing is drawn or re-probed, so the
    /// rebuilt event — and therefore its idempotency key — is identical to
    /// the one that was lost, and a deduping sink accepts it exactly once
    /// no matter how many times the caller retries.
    ///
    /// `command.expected_version` is the version the transition PRODUCED,
    /// which is the value [`DrPairingError::NarrationPending`] carries.
    ///
    /// # Errors
    /// - [`DrPairingError::PairNotFound`] when the tenant has no stored pair.
    /// - [`DrPairingError::StalePairVersion`] when the stored version is no
    ///   longer the one the transition produced — a later transition has
    ///   already moved the pair, and re-narrating from a state that no
    ///   longer exists would fabricate the missing detail rather than
    ///   recover it.
    /// - [`DrPairingError::IllegalTransition`] when the stored state is not
    ///   the state `kind` ends in.
    /// - [`DrPairingError::AuditEmitUnavailable`] when the sink is still down.
    /// - [`DrPairingError::PersistenceUnavailable`] from the store.
    pub fn renarrate(
        &self,
        kind: DrPairEventKind,
        command: &FailoverCommand,
    ) -> Result<DrPairAuditEvent, DrPairingError> {
        let stored = self
            .repo
            .current(&command.tenant_id)?
            .ok_or(DrPairingError::PairNotFound)?;
        if stored.pair_version != command.expected_version {
            return Err(DrPairingError::StalePairVersion);
        }
        let (from_state, to_state) = kind.edge();
        if stored.state != to_state {
            return Err(DrPairingError::IllegalTransition);
        }
        let from_version = stored
            .pair_version
            .checked_sub(1)
            .ok_or(DrPairingError::InvalidPairVersion)?;
        let event = build_event(&TransitionRecord {
            kind,
            pair: &stored,
            from_state,
            from_version,
            at_millis: command.at_millis,
            correlation_id: &command.correlation_id,
        });
        self.sink.emit(&event)?;
        Ok(event)
    }

    /// Build and emit the audit event for a committed transition.
    ///
    /// The state change is already durable when this runs, so a sink
    /// failure means "committed but not yet narrated". Callers translate it
    /// into [`DrPairingError::NarrationPending`], which names the committed
    /// version so [`Self::renarrate`] can finish the job.
    fn emit(&self, record: &TransitionRecord<'_>) -> Result<(), DrPairingError> {
        self.sink.emit(&build_event(record))
    }
}

/// Render one transition as its audit event.
///
/// A free function, and a total one: the same record always produces the
/// same event, which is what lets [`DrPairingController::renarrate`] rebuild
/// a lost event instead of inventing a replacement for it.
fn build_event(record: &TransitionRecord<'_>) -> DrPairAuditEvent {
    let pair = record.pair;
    DrPairAuditEvent {
        kind: record.kind,
        tenant_id: pair.tenant_id.clone(),
        home_cell: pair.home_cell.clone(),
        dr_cell: pair.dr_cell.clone(),
        jurisdiction: pair.jurisdiction.clone(),
        from_state: record.from_state,
        to_state: pair.state,
        from_version: record.from_version,
        to_version: pair.pair_version,
        decision: decision_for_event(record.kind),
        at_millis: record.at_millis,
        idempotency_key: derive_idempotency_key(
            &pair.tenant_id,
            record.kind.label(),
            pair.pair_version,
        ),
        correlation_id: record.correlation_id.to_owned(),
    }
}

/// Assess promotion from the pair store and the SLO probe alone.
///
/// This is the crate's original entry point and its signature is unchanged.
/// It answers the subset of the question those two ports can answer: does a
/// sound, activated pair exist, and is its DR replica healthy?
///
/// It CANNOT see jurisdiction drift, an unknown cell, or a residency-policy
/// change, because neither port can observe them — including the case where
/// a caller wrote a cross-jurisdiction row straight through
/// [`DrPairRepository::record`], past the controller that would have
/// refused it. Callers that must not promote across a residency boundary
/// use [`DrPairingController::assess_promotion`], which consults the cell
/// catalog and the residency policy as well.
///
/// # Errors
/// - [`DrPairingError::SloProbeFailed`] when the probe cannot answer — a
///   probe failure is never reported as healthy and never as a plain block.
/// - [`DrPairingError::PersistenceUnavailable`] when the store cannot answer.
pub fn evaluate_promotion<R: DrPairRepository, S: DrSloProbe>(
    repo: &R,
    probe: &S,
    tenant_id: &str,
) -> Result<PromotionDecision, DrPairingError> {
    let Some(pair) = repo.current(tenant_id)? else {
        return Ok(PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_FOUND,
        });
    };
    if pair.validate().is_err() {
        return Ok(PromotionDecision::Blocked {
            reason_code: reason::DEGENERATE_PAIR,
        });
    }
    if let Some(reason_code) = promotion_block_for_state(pair.state) {
        return Ok(PromotionDecision::Blocked { reason_code });
    }
    if probe.dr_replica_health(&pair.dr_cell)? {
        Ok(PromotionDecision::Eligible)
    } else {
        Ok(PromotionDecision::Blocked {
            reason_code: reason::DR_REPLICA_UNHEALTHY,
        })
    }
}
