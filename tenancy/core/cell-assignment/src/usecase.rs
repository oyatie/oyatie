//! Cell-assignment usecase: the orchestrators IP-008 assigns to
//! `oya-tenancy-cell-assignment-usecase`, collapsed into a module here.
//!
//! This layer owns the sequencing — read the existing assignment, choose,
//! confirm the choice against the live health port, record it — and
//! nothing else. Every decision it makes is delegated to the pure
//! functions in [`crate::domain`], so the orchestration can be reasoned
//! about without re-reading the placement algebra.

use crate::domain::{
    Placement, RebalancePlan, cell_for_shard, derive_shard_key, plan_rebalance, select_least_loaded,
};
use crate::kernel::{
    CellAssignmentRepository, CellCandidate, CellHealthProbe, CellId, CellKernelError, ShardKey,
};

/// What an assignment call concluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssignmentOutcome {
    /// The tenant already had a recorded assignment; nothing was written.
    /// Assignment is idempotent — a retried call reports this, it does
    /// not re-select and it does not move a live tenant.
    AlreadyAssigned(CellId),
    /// A cell was selected, confirmed healthy, and recorded.
    Assigned(CellId),
}

impl AssignmentOutcome {
    /// The cell the tenant is on, whichever way the call concluded.
    #[must_use]
    pub fn cell(&self) -> &CellId {
        match self {
            Self::AlreadyAssigned(cell) | Self::Assigned(cell) => cell,
        }
    }

    /// Whether this call is the one that wrote the assignment.
    #[must_use]
    pub fn is_new(&self) -> bool {
        matches!(self, Self::Assigned(_))
    }
}

/// The cell-assignment control plane over its two ports.
///
/// Generic over the ports rather than boxed, so the in-memory adapter and
/// a future Citus adapter cost the same at the call site and neither is
/// privileged by the type.
#[derive(Clone, Debug)]
pub struct CellAssignmentService<R, P>
where
    R: CellAssignmentRepository,
    P: CellHealthProbe,
{
    repository: R,
    probe: P,
}

impl<R, P> CellAssignmentService<R, P>
where
    R: CellAssignmentRepository,
    P: CellHealthProbe,
{
    /// Wire the service to its ports.
    pub fn new(repository: R, probe: P) -> Self {
        Self { repository, probe }
    }

    /// Borrow the assignment record store.
    pub fn repository(&self) -> &R {
        &self.repository
    }

    /// Borrow the health probe.
    pub fn probe(&self) -> &P {
        &self.probe
    }

    /// Unwire the service back into its ports.
    pub fn into_parts(self) -> (R, P) {
        (self.repository, self.probe)
    }

    /// Observe one cell's live health, normalising any port failure to
    /// [`CellKernelError::ProbeFailed`].
    ///
    /// Failing to observe a cell is deliberately NOT the same as
    /// observing it unhealthy: an unobservable cell means the health
    /// pipeline is broken, and placing a tenant on that evidence would be
    /// a guess. The call fails instead.
    fn confirm_healthy(&self, cell: &CellId) -> Result<bool, CellKernelError> {
        self.probe
            .probe(cell)
            .map(crate::kernel::CellHealth::is_healthy)
            .map_err(|_| CellKernelError::ProbeFailed { cell: cell.clone() })
    }

    /// Replace each candidate's declared health with the health the probe
    /// reports right now.
    ///
    /// # Errors
    /// [`CellKernelError::ProbeFailed`] if any candidate cannot be
    /// observed.
    pub fn refresh_health(
        &self,
        candidates: &[CellCandidate],
    ) -> Result<Vec<CellCandidate>, CellKernelError> {
        let mut refreshed = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let health =
                self.probe
                    .probe(&candidate.cell)
                    .map_err(|_| CellKernelError::ProbeFailed {
                        cell: candidate.cell.clone(),
                    })?;
            refreshed.push(CellCandidate::new(
                candidate.cell.clone(),
                candidate.load_permille,
                health,
            )?);
        }
        Ok(refreshed)
    }

    /// Assign a tenant to the least-loaded healthy cell, confirming the
    /// choice against the live probe before recording it.
    ///
    /// Idempotent: a tenant that already has a recorded assignment keeps
    /// it and no write occurs. When the probe contradicts a candidate's
    /// declared health, that candidate is dropped and selection retries
    /// over what remains, so a stale candidate set degrades into a
    /// slightly worse placement rather than a bad one.
    ///
    /// # Errors
    /// - [`CellKernelError::PersistenceUnavailable`] from the record store.
    /// - [`CellKernelError::ProbeFailed`] if a cell cannot be observed.
    /// - [`CellKernelError::NoHealthyCellFor`] naming the tenant, if
    ///   nothing survives selection.
    /// - [`CellKernelError::LoadOutOfRange`] for an impossible candidate.
    pub fn assign(
        &self,
        tenant: &str,
        candidates: &[CellCandidate],
    ) -> Result<AssignmentOutcome, CellKernelError> {
        if let Some(existing) = self.repository.assigned_cell(tenant)? {
            return Ok(AssignmentOutcome::AlreadyAssigned(existing));
        }
        self.place_confirmed(tenant, candidates, select_least_loaded)
    }

    /// Assign a tenant by consistent hash: derive its shard key over
    /// `shard_count` and place it on the healthy cell that shard maps to.
    ///
    /// Returns the derived shard key alongside the outcome, because the
    /// shard key is the storage-facing coordinate the caller needs even
    /// when the assignment was already on record.
    ///
    /// # Errors
    /// Everything [`Self::assign`] can raise, plus
    /// [`CellKernelError::ZeroShardCount`] for an empty shard space.
    pub fn assign_by_shard(
        &self,
        tenant: &str,
        shard_count: u64,
        candidates: &[CellCandidate],
    ) -> Result<(ShardKey, AssignmentOutcome), CellKernelError> {
        let shard_key = derive_shard_key(tenant, shard_count)?;
        if let Some(existing) = self.repository.assigned_cell(tenant)? {
            return Ok((shard_key, AssignmentOutcome::AlreadyAssigned(existing)));
        }
        let outcome = self.place_confirmed(tenant, candidates, |remaining| {
            cell_for_shard(shard_key, remaining)
        })?;
        Ok((shard_key, outcome))
    }

    /// Select with `choose`, confirm the choice is live-healthy, and
    /// record it; on contradiction, drop that cell and select again.
    ///
    /// The bare [`CellKernelError::NoHealthyCell`] the selectors raise is
    /// re-stamped with the tenant and the size of the candidate set,
    /// because "no healthy cell" out of a forty-cell roster is not an
    /// actionable message without knowing who was being placed.
    fn place_confirmed(
        &self,
        tenant: &str,
        candidates: &[CellCandidate],
        choose: impl Fn(&[CellCandidate]) -> Result<CellId, CellKernelError>,
    ) -> Result<AssignmentOutcome, CellKernelError> {
        let considered = candidates.len();
        let mut remaining: Vec<CellCandidate> = candidates.to_vec();
        // Each pass either records an assignment or removes exactly one
        // candidate, so the loop is bounded by `candidates.len()`.
        loop {
            let chosen = match choose(&remaining) {
                Ok(cell) => cell,
                Err(CellKernelError::NoHealthyCell) => {
                    return Err(CellKernelError::NoHealthyCellFor {
                        tenant: tenant.to_owned(),
                        considered,
                    });
                }
                Err(other) => return Err(other),
            };
            if self.confirm_healthy(&chosen)? {
                self.repository.record_assignment(tenant, &chosen)?;
                return Ok(AssignmentOutcome::Assigned(chosen));
            }
            remaining.retain(|candidate| candidate.cell != chosen);
        }
    }

    /// Drop a tenant's recorded assignment, the offboarding half of
    /// [`Self::assign`].
    ///
    /// Returns whether a record was actually removed. Without this, the
    /// record store keeps a row for every tenant that has ever existed
    /// and a re-issued tenant id silently inherits its namesake's cell —
    /// possibly one that has since been decommissioned — through the
    /// idempotency short-circuit in [`Self::assign`].
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] from the record store.
    pub fn release(&self, tenant: &str) -> Result<bool, CellKernelError> {
        self.repository.forget_assignment(tenant)
    }

    /// Plan a rebalance using live probe health rather than the health
    /// the caller happened to observe when it assembled `candidates`.
    ///
    /// # Errors
    /// [`CellKernelError::ProbeFailed`] plus everything
    /// [`plan_rebalance`] can raise.
    pub fn plan_rebalance(
        &self,
        placement: &Placement,
        candidates: &[CellCandidate],
    ) -> Result<RebalancePlan, CellKernelError> {
        let refreshed = self.refresh_health(candidates)?;
        plan_rebalance(placement, &refreshed)
    }

    /// Apply a plan to a placement and record every resulting assignment.
    ///
    /// The moves are simulated against a copy and integrity-verified
    /// BEFORE anything is written and before the caller's placement is
    /// touched, so a plan that would lose or duplicate a tenant leaves
    /// both the record store and the placement untouched.
    ///
    /// # Errors
    /// - [`CellKernelError::IntegrityMismatch`] /
    ///   [`CellKernelError::TaskNotApplied`] /
    ///   [`CellKernelError::PlacementDiverged`] if the simulated
    ///   placement is not what the plan predicted.
    /// - [`CellKernelError::PartialPlanExecution`] when the record store
    ///   fails part-way through. Execution here is not transactional (see
    ///   the crate-level Gaps note), so the rows already written stay
    ///   written — and the error therefore carries the index of the task
    ///   that failed, which is the whole reconciliation key the operator
    ///   needs: `plan.tasks()[..committed]` are durable at the NEW cell,
    ///   `plan.tasks()[committed..]` are still at the OLD one, and the
    ///   caller's `placement` is untouched and still describes the old
    ///   world.
    /// - Any [`Placement::apply`] error for a malformed task.
    pub fn execute_plan(
        &self,
        placement: &mut Placement,
        plan: &RebalancePlan,
    ) -> Result<(), CellKernelError> {
        let mut working = placement.clone();
        // Admit the cells the planner validated as healthy candidates but
        // that this placement has never held a tenant in.
        plan.prepare(&mut working);
        for task in plan.tasks() {
            working.apply(task)?;
        }
        plan.verify_applied(&working)?;
        for (committed, task) in plan.tasks().iter().enumerate() {
            if let Err(cause) = self
                .repository
                .record_assignment(&task.tenant, &task.to_cell)
            {
                return Err(CellKernelError::PartialPlanExecution {
                    committed,
                    total: plan.len(),
                    cause: Box::new(cause),
                });
            }
        }
        *placement = working;
        Ok(())
    }
}
