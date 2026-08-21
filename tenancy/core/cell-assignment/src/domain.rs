//! Cell-assignment domain: shard derivation, cell selection, placement
//! bookkeeping, and rebalance planning.
//!
//! Everything in this module is a pure function of its arguments. There
//! is no clock read and no randomness anywhere below this line, so a plan
//! computed from a given placement is byte-identical on every run and on
//! every host — which is what makes the before/after integrity checksum
//! worth anything.

use std::collections::{BTreeMap, BTreeSet};

use crate::kernel::{
    CellCandidate, CellHealth, CellId, CellKernelError, MAX_LOAD_PERMILLE, RebalanceTask, ShardKey,
};

/// FNV-1a 64-bit offset basis.
pub const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64-bit prime.
pub const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;

/// Reason stamped on moves emitted to empty a cell reported unhealthy.
pub const REASON_DRAIN_UNHEALTHY: &str = "drain_unhealthy_cell";

/// Reason stamped on moves emitted to even out load across healthy cells.
pub const REASON_LEVEL_LOAD: &str = "level_load";

/// FNV-1a over `bytes`, 64-bit.
///
/// IP-008 specifies `blake3`. blake3 is an external dependency and this
/// lane's lockfile is frozen, so the substitution is FNV-1a — see the
/// crate-level Gaps note for the consequences.
#[must_use]
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS_64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    hash
}

/// Widen a length to `u64` without an `as` cast that could truncate.
///
/// `usize` is never wider than `u64` on any target this repo builds for,
/// so the saturating branch is defensive only.
fn widen(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Narrow an already-bounded index to `usize` without an `as` cast.
///
/// Callers only pass values already reduced modulo a `usize`-derived
/// count, so the saturating branch is defensive only.
fn narrow(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// Derive a tenant's shard key over a shard space of `shard_count`.
///
/// Deterministic and stateless: the same tenant id always yields the same
/// shard key for a given `shard_count`.
///
/// # Errors
/// [`CellKernelError::ZeroShardCount`] when `shard_count` is zero — a
/// typed error rather than a division-by-zero panic.
pub fn derive_shard_key(tenant: &str, shard_count: u64) -> Result<ShardKey, CellKernelError> {
    if shard_count == 0 {
        return Err(CellKernelError::ZeroShardCount);
    }
    Ok(ShardKey(fnv1a_64(tenant.as_bytes()) % shard_count))
}

/// The candidate set folded into one validated observation per cell.
///
/// Validation is deliberately re-done here even though
/// [`CellCandidate::new`] already performs it: `CellCandidate` has public
/// fields, so a struct literal can carry an impossible load or a
/// malformed cell id straight past the constructor, and a decision made
/// on either is a decision made on a broken observation pipeline.
///
/// Duplicate entries for one cell are folded to the **most conservative**
/// reading — the highest reported load and the most severe health
/// ([`CellHealth`] orders `Healthy < Degraded < Unhealthy`). Folding
/// rather than last-wins is what keeps planning independent of the order
/// the caller happened to assemble its candidates in.
///
/// # Errors
/// - [`CellKernelError::LoadOutOfRange`] for a load above
///   [`MAX_LOAD_PERMILLE`].
/// - [`CellKernelError::MalformedCellId`] for an id that is not
///   well-formed; see [`CellId`].
fn validated_roster(
    candidates: &[CellCandidate],
) -> Result<BTreeMap<CellId, (u32, CellHealth)>, CellKernelError> {
    let mut roster: BTreeMap<CellId, (u32, CellHealth)> = BTreeMap::new();
    for candidate in candidates {
        if !candidate.cell.is_well_formed() {
            return Err(CellKernelError::MalformedCellId {
                cell: candidate.cell.0.clone(),
            });
        }
        if candidate.load_permille > MAX_LOAD_PERMILLE {
            return Err(CellKernelError::LoadOutOfRange);
        }
        let entry = roster
            .entry(candidate.cell.clone())
            .or_insert((candidate.load_permille, candidate.health));
        entry.0 = entry.0.max(candidate.load_permille);
        entry.1 = entry.1.max(candidate.health);
    }
    Ok(roster)
}

/// The healthy candidates, deduplicated and sorted by cell id.
///
/// # Errors
/// Anything [`validated_roster`] can raise — including for a candidate
/// that is *not* healthy, because one broken observation means none of
/// them is trustworthy.
fn healthy_sorted(candidates: &[CellCandidate]) -> Result<Vec<CellCandidate>, CellKernelError> {
    Ok(validated_roster(candidates)?
        .into_iter()
        .filter(|(_, (_, health))| health.is_healthy())
        .map(|(cell, (load_permille, health))| CellCandidate {
            cell,
            load_permille,
            health,
        })
        .collect())
}

/// Pick the least loaded healthy cell.
///
/// Tie-breaking rule, and it is part of the contract: among cells with
/// equal `load_permille`, the lexicographically smallest [`CellId`] wins.
/// Two callers holding the same candidate set therefore always choose the
/// same cell, regardless of the order the set was assembled in.
///
/// # Errors
/// - [`CellKernelError::LoadOutOfRange`] if any candidate reports a load
///   above [`MAX_LOAD_PERMILLE`].
/// - [`CellKernelError::MalformedCellId`] if any candidate carries an
///   ill-formed cell id.
/// - [`CellKernelError::NoHealthyCell`] if no candidate is healthy.
pub fn select_least_loaded(candidates: &[CellCandidate]) -> Result<CellId, CellKernelError> {
    let healthy = healthy_sorted(candidates)?;
    healthy
        .into_iter()
        // `min_by_key` keeps the FIRST minimum, and the list is already
        // sorted by cell id, so this is exactly the documented rule.
        .min_by_key(|candidate| candidate.load_permille)
        .map(|candidate| candidate.cell)
        .ok_or(CellKernelError::NoHealthyCell)
}

/// Map a shard key onto a healthy cell by `shard_key % healthy_count`
/// over the healthy cells sorted by id.
///
/// # This is modulo placement, not consistent hashing
///
/// Despite [`ShardKey`]'s "consistent-hash" framing, the mapping here is
/// plain modulo over the *current* healthy set. A membership change
/// remaps roughly every key, not roughly `1/N` of them, which has two
/// consequences worth stating plainly:
///
/// - It is a **placement** function, not a **lookup** function. Routing a
///   live tenant's traffic by recomputing `cell_for_shard` is a defect:
///   the healthy set that answered when the tenant was placed is not the
///   healthy set that answers now, so the recomputed cell can be one that
///   has never held the tenant's data. Read the recorded assignment from
///   [`crate::kernel::CellAssignmentRepository`] instead —
///   [`crate::usecase::CellAssignmentService::assign_by_shard`] is safe
///   only because its `AlreadyAssigned` short-circuit runs first.
/// - Real ring-based consistent hashing (virtual nodes, minimal
///   disruption on membership change) is a deliberate gap; see the
///   crate-level Gaps note.
///
/// # Errors
/// - [`CellKernelError::LoadOutOfRange`] if any candidate reports a load
///   above [`MAX_LOAD_PERMILLE`].
/// - [`CellKernelError::MalformedCellId`] if any candidate carries an
///   ill-formed cell id.
/// - [`CellKernelError::NoHealthyCell`] if no candidate is healthy.
pub fn cell_for_shard(
    shard_key: ShardKey,
    candidates: &[CellCandidate],
) -> Result<CellId, CellKernelError> {
    let healthy = healthy_sorted(candidates)?;
    let count = widen(healthy.len());
    if count == 0 {
        return Err(CellKernelError::NoHealthyCell);
    }
    let position = narrow(shard_key.0 % count);
    healthy
        .get(position)
        .map(|candidate| candidate.cell.clone())
        .ok_or(CellKernelError::NoHealthyCell)
}

/// An order-independent integrity fingerprint of a set of tenants.
///
/// `occupancy` counts every tenant *placement* (so a duplicated tenant
/// counts twice) and `digest` is the wrapping sum of [`fnv1a_64`] over
/// every placed tenant id.
///
/// # What it can and cannot see
///
/// Folded across a whole placement it is invariant under a pure move, so
/// it detects loss and duplication — but, being cell-agnostic, it is
/// equally invariant under any *permutation* of tenants across cells and
/// therefore cannot see a misroute on its own. That is why
/// [`Placement::checksum_by_cell`] exists and why
/// [`RebalancePlan::verify_applied`] compares per cell as well as in
/// total.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementChecksum {
    /// Total number of tenant placements across all cells.
    pub occupancy: usize, // data_class: INTERNAL_ONLY
    /// Order-independent fold over the placed tenant ids.
    pub digest: u64, // data_class: INTERNAL_ONLY
}

impl PlacementChecksum {
    /// The fingerprint of a cell holding nothing, which is also the
    /// fingerprint of a cell that is not registered at all.
    pub const EMPTY: Self = Self {
        occupancy: 0,
        digest: 0,
    };
}

/// Fingerprint one cell's tenant set.
fn checksum_of(tenants: &BTreeSet<String>) -> PlacementChecksum {
    let mut digest: u64 = 0;
    for tenant in tenants {
        digest = digest.wrapping_add(fnv1a_64(tenant.as_bytes()));
    }
    PlacementChecksum {
        occupancy: tenants.len(),
        digest,
    }
}

/// Which tenants live in which cells right now.
///
/// Modelled as cell -> tenant set (rather than tenant -> cell) precisely
/// so that a *duplicate* is representable and therefore detectable: a
/// tenant-keyed map would silently swallow the very defect the integrity
/// checksum exists to catch.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Placement {
    cells: BTreeMap<CellId, BTreeSet<String>>,
}

impl Placement {
    /// An empty placement with no cells.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a cell with no tenants, so it can receive moves.
    ///
    /// Idempotent: registering a cell that already exists leaves its
    /// tenants untouched.
    pub fn register_cell(&mut self, cell: &CellId) {
        self.cells.entry(cell.clone()).or_default();
    }

    /// Place a tenant into a cell, registering the cell if needed.
    ///
    /// Returns `true` if the tenant was not already in that cell.
    pub fn place(&mut self, cell: &CellId, tenant: &str) -> bool {
        self.cells
            .entry(cell.clone())
            .or_default()
            .insert(tenant.to_owned())
    }

    /// Remove a tenant from a cell.
    ///
    /// This is the only way to *lose* a tenant from a placement, and it
    /// exists so integrity verification can be exercised against a
    /// tampered placement.
    ///
    /// # Errors
    /// - [`CellKernelError::UnknownSourceCell`] if the cell is unknown.
    /// - [`CellKernelError::TenantNotInSourceCell`] if it does not hold
    ///   the tenant.
    pub fn evict(&mut self, cell: &CellId, tenant: &str) -> Result<(), CellKernelError> {
        let tenants = self
            .cells
            .get_mut(cell)
            .ok_or(CellKernelError::UnknownSourceCell)?;
        if tenants.remove(tenant) {
            Ok(())
        } else {
            Err(CellKernelError::TenantNotInSourceCell)
        }
    }

    /// Every registered cell, in ascending cell-id order.
    #[must_use]
    pub fn cell_ids(&self) -> Vec<CellId> {
        self.cells.keys().cloned().collect()
    }

    /// The tenants held by a cell, if the cell is registered.
    #[must_use]
    pub fn tenants_of(&self, cell: &CellId) -> Option<&BTreeSet<String>> {
        self.cells.get(cell)
    }

    /// How many tenants a cell holds (zero for an unknown cell).
    #[must_use]
    pub fn load_of(&self, cell: &CellId) -> usize {
        self.cells.get(cell).map_or(0, BTreeSet::len)
    }

    /// Whether a cell holds a tenant.
    #[must_use]
    pub fn contains(&self, cell: &CellId, tenant: &str) -> bool {
        self.cells
            .get(cell)
            .is_some_and(|tenants| tenants.contains(tenant))
    }

    /// The first cell (in ascending cell-id order) holding a tenant.
    #[must_use]
    pub fn cell_of(&self, tenant: &str) -> Option<CellId> {
        self.cells
            .iter()
            .find(|(_, tenants)| tenants.contains(tenant))
            .map(|(cell, _)| cell.clone())
    }

    /// Total tenant placements across all cells; a duplicated tenant is
    /// counted once per cell that holds it.
    #[must_use]
    pub fn occupancy(&self) -> usize {
        self.cells.values().map(BTreeSet::len).sum()
    }

    /// Whether no cell holds any tenant.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.occupancy() == 0
    }

    /// The whole-placement integrity fingerprint. See
    /// [`PlacementChecksum`], including what it deliberately cannot see.
    #[must_use]
    pub fn checksum(&self) -> PlacementChecksum {
        let mut digest: u64 = 0;
        let mut occupancy: usize = 0;
        for tenants in self.cells.values() {
            let cell_sum = checksum_of(tenants);
            digest = digest.wrapping_add(cell_sum.digest);
            occupancy += cell_sum.occupancy;
        }
        PlacementChecksum { occupancy, digest }
    }

    /// One fingerprint per registered cell.
    ///
    /// This is the cell-aware fingerprint: two placements holding the
    /// same tenants in *different* cells agree on [`Placement::checksum`]
    /// but disagree here, which is what makes a misrouted move visible.
    /// A cell registered with no tenants fingerprints as
    /// [`PlacementChecksum::EMPTY`], the same as a cell that is absent,
    /// so merely registering a cell never counts as divergence.
    #[must_use]
    pub fn checksum_by_cell(&self) -> BTreeMap<CellId, PlacementChecksum> {
        self.cells
            .iter()
            .map(|(cell, tenants)| (cell.clone(), checksum_of(tenants)))
            .collect()
    }

    /// Apply one move task in place.
    ///
    /// Both endpoints must already be registered: an unregistered target
    /// would let a typo silently mint a cell.
    ///
    /// # Errors
    /// - [`CellKernelError::RebalanceConflict`] for a self-move.
    /// - [`CellKernelError::UnknownSourceCell`] / [`CellKernelError::UnknownTargetCell`]
    ///   for endpoints the placement does not know.
    /// - [`CellKernelError::TenantNotInSourceCell`] when the move would
    ///   lose a tenant.
    /// - [`CellKernelError::TenantDuplicated`] when it would duplicate one.
    pub fn apply(&mut self, task: &RebalanceTask) -> Result<(), CellKernelError> {
        if task.from_cell == task.to_cell {
            return Err(CellKernelError::RebalanceConflict);
        }
        if !self.cells.contains_key(&task.from_cell) {
            return Err(CellKernelError::UnknownSourceCell);
        }
        if !self.cells.contains_key(&task.to_cell) {
            return Err(CellKernelError::UnknownTargetCell);
        }
        if self.contains(&task.to_cell, &task.tenant) {
            return Err(CellKernelError::TenantDuplicated);
        }
        self.evict(&task.from_cell, &task.tenant)?;
        self.cells
            .entry(task.to_cell.clone())
            .or_default()
            .insert(task.tenant.clone());
        Ok(())
    }
}

/// A verified set of moves, carrying the placement fingerprints taken
/// before the moves and after simulating them.
///
/// A plan can only be built by a constructor that simulates every move
/// against a real placement, so an ill-formed plan is a construction-time
/// typed error rather than a runtime surprise.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalancePlan {
    tasks: Vec<RebalanceTask>,
    required_cells: BTreeSet<CellId>,
    before: PlacementChecksum,
    after: PlacementChecksum,
    after_by_cell: BTreeMap<CellId, PlacementChecksum>,
}

/// Every cell a task list touches, at either end.
fn required_cells_of(tasks: &[RebalanceTask]) -> BTreeSet<CellId> {
    let mut cells = BTreeSet::new();
    for task in tasks {
        cells.insert(task.from_cell.clone());
        cells.insert(task.to_cell.clone());
    }
    cells
}

impl RebalancePlan {
    /// The moves, in the order they must be applied.
    #[must_use]
    pub fn tasks(&self) -> &[RebalanceTask] {
        &self.tasks
    }

    /// How many moves the plan contains.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Whether the placement was already balanced (no moves needed).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Every cell the plan touches, at either end of a move.
    #[must_use]
    pub fn required_cells(&self) -> &BTreeSet<CellId> {
        &self.required_cells
    }

    /// Register every cell this plan touches on `placement`.
    ///
    /// [`Placement::apply`] refuses to invent a cell, so that a mistyped
    /// destination cannot silently mint one. A plan produced by
    /// [`plan_rebalance`] may legitimately move tenants onto a healthy
    /// cell the placement has never held a tenant in, and this is the
    /// explicit, auditable way to admit those cells — the planner having
    /// already validated every one of them as a healthy candidate.
    pub fn prepare(&self, placement: &mut Placement) {
        for cell in &self.required_cells {
            placement.register_cell(cell);
        }
    }

    /// The checksum of the placement the plan was computed against.
    #[must_use]
    pub fn before_checksum(&self) -> PlacementChecksum {
        self.before
    }

    /// The checksum of the simulated placement after all moves.
    #[must_use]
    pub fn after_checksum(&self) -> PlacementChecksum {
        self.after
    }

    /// The simulated post-plan fingerprint of every cell the plan expects
    /// to exist — the cell-aware post-condition
    /// [`RebalancePlan::verify_applied`] checks against.
    #[must_use]
    pub fn after_checksum_by_cell(&self) -> &BTreeMap<CellId, PlacementChecksum> {
        &self.after_by_cell
    }

    /// Validate a hand-written task list against a placement, producing a
    /// plan only if every move is legal.
    ///
    /// # Errors
    /// Any error [`Placement::apply`] can raise, plus
    /// [`CellKernelError::IntegrityMismatch`] if the simulated placement
    /// does not fingerprint identically to the starting one.
    ///
    /// The `IntegrityMismatch` arm is defence in depth rather than a
    /// reachable outcome today: [`Placement::apply`] performs exactly one
    /// checked `evict` and one checked insert, so it conserves the
    /// checksum by construction and this guard cannot fire while that
    /// holds. It is kept so that a future `apply` that stops conserving
    /// is caught here instead of in production. The *reachable*
    /// protection is `apply`'s own per-task validation.
    pub fn from_tasks(
        placement: &Placement,
        tasks: Vec<RebalanceTask>,
    ) -> Result<Self, CellKernelError> {
        let before = placement.checksum();
        let mut working = placement.clone();
        for task in &tasks {
            working.apply(task)?;
        }
        let after = working.checksum();
        if after != before {
            return Err(CellKernelError::IntegrityMismatch);
        }
        Ok(Self {
            required_cells: required_cells_of(&tasks),
            tasks,
            before,
            after,
            after_by_cell: working.checksum_by_cell(),
        })
    }

    /// Check a placement that this plan was actually executed against.
    ///
    /// This is the post-condition half of the IP-008 integrity check, and
    /// it is deliberately three checks rather than one, because each sees
    /// something the others are blind to. They run coarsest-first, so the
    /// error the caller gets is the most specific one that still explains
    /// the whole discrepancy:
    ///
    /// 1. the whole-placement fingerprint, which catches a tenant lost or
    ///    duplicated by execution;
    /// 2. an exact membership check that each moved tenant actually sits
    ///    in its planned cell — no hashing at all, so no digest collision
    ///    can defeat it, and it names the tenant;
    /// 3. the per-cell fingerprint, which catches *collateral* movement:
    ///    an unplanned tenant shuffled between cells conserves both the
    ///    total fold and every planned endpoint, and check 1 is
    ///    structurally blind to any permutation across cells.
    ///
    /// # Errors
    /// - [`CellKernelError::IntegrityMismatch`] when execution lost or
    ///   duplicated a tenant.
    /// - [`CellKernelError::TaskNotApplied`] naming a planned tenant that
    ///   is not in its planned cell.
    /// - [`CellKernelError::PlacementDiverged`] naming the first cell
    ///   whose tenant set is not what the plan predicted.
    pub fn verify_applied(&self, applied: &Placement) -> Result<(), CellKernelError> {
        if applied.checksum() != self.after {
            return Err(CellKernelError::IntegrityMismatch);
        }
        for task in &self.tasks {
            if !applied.contains(&task.to_cell, &task.tenant) {
                return Err(CellKernelError::TaskNotApplied {
                    tenant: task.tenant.clone(),
                    expected_cell: task.to_cell.clone(),
                });
            }
        }
        let observed = applied.checksum_by_cell();
        let mut cells: BTreeSet<&CellId> = self.after_by_cell.keys().collect();
        cells.extend(observed.keys());
        for cell in cells {
            let expected = self
                .after_by_cell
                .get(cell)
                .copied()
                .unwrap_or(PlacementChecksum::EMPTY);
            let actual = observed
                .get(cell)
                .copied()
                .unwrap_or(PlacementChecksum::EMPTY);
            if expected != actual {
                return Err(CellKernelError::PlacementDiverged { cell: cell.clone() });
            }
        }
        Ok(())
    }
}

/// The healthy cell, among `healthy`, that currently holds the fewest
/// tenants and does not already hold `tenant`.
///
/// Ties are broken by the lexicographically smallest cell id, matching
/// [`select_least_loaded`].
///
/// # Errors
/// [`CellKernelError::NoHealthyCellFor`] naming the tenant, when no
/// healthy cell can take it.
fn least_loaded_recipient(
    working: &Placement,
    healthy: &BTreeSet<CellId>,
    tenant: &str,
) -> Result<CellId, CellKernelError> {
    let mut ranked: Vec<(usize, CellId)> = healthy
        .iter()
        .filter(|cell| !working.contains(cell, tenant))
        .map(|cell| (working.load_of(cell), cell.clone()))
        .collect();
    // Sorting on the (load, cell id) pair IS the documented tie-break:
    // fewest tenants first, smallest cell id to settle a draw.
    ranked.sort();
    ranked
        .into_iter()
        .next()
        .map(|(_, cell)| cell)
        .ok_or_else(|| CellKernelError::NoHealthyCellFor {
            tenant: tenant.to_owned(),
            considered: healthy.len(),
        })
}

/// Plan the moves that carry `placement` toward balance across the
/// healthy cells in `candidates`.
///
/// # Roster coverage is mandatory
///
/// Every cell that holds a tenant must appear in `candidates`. A cell
/// that is merely *absent* is not treated as unhealthy: a truncated,
/// paginated or partially-failed cell listing is an incomplete
/// observation, not a health signal, and inferring "unlisted means
/// drain" would let it evacuate the cells it forgot to mention. An
/// occupied cell missing from the roster is
/// [`CellKernelError::PlacementCellNotInRoster`] instead. (Cells that
/// are registered but hold nothing need no coverage — there is nothing
/// to lose.)
///
/// # The two phases, both deterministic
///
/// 1. **Drain.** Every placement cell the roster reports
///    [`CellHealth::Unhealthy`] is emptied, tenant by tenant in ascending
///    tenant-id order, onto the currently least-loaded healthy cell. A
///    [`CellHealth::Degraded`] cell is **not** drained: per the
///    [`CellHealth`] contract it keeps its tenants and simply takes no
///    new ones, so a transient wobble cannot stampede a whole cell's
///    population onto its neighbours. It also takes no part in phase 2.
/// 2. **Level.** While the busiest healthy cell holds more than one
///    tenant more than the quietest, one tenant moves from the busiest to
///    the quietest. Donor and recipient ties break on the smallest cell
///    id; the tenant moved is the lexicographically smallest one in the
///    donor that the recipient does not already hold.
///
/// # Errors
/// - [`CellKernelError::LoadOutOfRange`] for an impossible candidate load.
/// - [`CellKernelError::MalformedCellId`] for an ill-formed candidate id.
/// - [`CellKernelError::PlacementCellNotInRoster`] for an occupied cell
///   the roster does not mention.
/// - [`CellKernelError::NoHealthyCellFor`] naming the tenant, if a tenant
///   that must move has nowhere healthy to go.
/// - [`CellKernelError::RebalanceConflict`] if levelling exceeds its
///   iteration budget, and [`CellKernelError::IntegrityMismatch`] if the
///   simulated moves did not conserve the tenant set. Both are defence in
///   depth, not reachable outcomes: every accepted levelling move
///   strictly reduces the donor's excess over the target so the move
///   count is bounded by the occupancy (well inside the
///   `occupancy * 4 + 8` budget), and [`Placement::apply`] conserves the
///   checksum by construction. They are kept so that a future change
///   which breaks either property fails here rather than in production —
///   a caller should not expect to observe them.
pub fn plan_rebalance(
    placement: &Placement,
    candidates: &[CellCandidate],
) -> Result<RebalancePlan, CellKernelError> {
    let before = placement.checksum();
    let roster = validated_roster(candidates)?;
    let healthy: BTreeSet<CellId> = roster
        .iter()
        .filter(|(_, (_, health))| health.is_healthy())
        .map(|(cell, _)| cell.clone())
        .collect();

    // Coverage check, before any move is contemplated: an unknown health
    // is not a licence to move anybody.
    for cell in placement.cell_ids() {
        if placement.load_of(&cell) > 0 && !roster.contains_key(&cell) {
            return Err(CellKernelError::PlacementCellNotInRoster { cell });
        }
    }

    let mut working = placement.clone();
    for cell in &healthy {
        working.register_cell(cell);
    }

    let mut tasks: Vec<RebalanceTask> = Vec::new();

    // Phase 1 — drain the cells the roster reports as Unhealthy. NOT
    // "everything that is not healthy": see the doc comment above.
    let draining: Vec<CellId> = working
        .cell_ids()
        .into_iter()
        .filter(|cell| {
            roster
                .get(cell)
                .is_some_and(|(_, health)| health.is_drained())
        })
        .collect();
    for cell in draining {
        let tenants: Vec<String> = working
            .tenants_of(&cell)
            .map(|held| held.iter().cloned().collect())
            .unwrap_or_default();
        for tenant in tenants {
            let target = least_loaded_recipient(&working, &healthy, &tenant)?;
            let task = RebalanceTask::new(tenant, cell.clone(), target, REASON_DRAIN_UNHEALTHY);
            working.apply(&task)?;
            tasks.push(task);
        }
    }

    // Phase 2 — level load across the healthy cells. Every accepted move
    // shrinks (busiest - quietest) monotonically, so the loop terminates;
    // the budget is a defensive bound, not the termination argument.
    let budget = working.occupancy().saturating_mul(4).saturating_add(8);
    let mut iterations = 0usize;
    loop {
        let mut ranked: Vec<(usize, CellId)> = healthy
            .iter()
            .map(|cell| (working.load_of(cell), cell.clone()))
            .collect();
        ranked.sort();
        // Quietest cell: first in (load, cell id) order.
        let Some((recipient_load, recipient)) = ranked.first().cloned() else {
            break;
        };
        // Busiest cell: the FIRST entry carrying the maximum load, so the
        // tie-break is the smallest cell id, not the largest.
        let Some(&(donor_load, _)) = ranked.last() else {
            break;
        };
        let Some((_, donor)) = ranked.iter().find(|(load, _)| *load == donor_load).cloned() else {
            break;
        };
        if donor == recipient || donor_load <= recipient_load + 1 {
            break;
        }
        let moved = working.tenants_of(&donor).and_then(|held| {
            held.iter()
                .find(|tenant| !working.contains(&recipient, tenant))
                .cloned()
        });
        // No movable tenant means every tenant in the donor already lives
        // in the recipient too; levelling can make no further progress.
        let Some(tenant) = moved else { break };
        let task = RebalanceTask::new(tenant, donor, recipient, REASON_LEVEL_LOAD);
        working.apply(&task)?;
        tasks.push(task);

        iterations += 1;
        if iterations > budget {
            return Err(CellKernelError::RebalanceConflict);
        }
    }

    let after = working.checksum();
    if after != before {
        return Err(CellKernelError::IntegrityMismatch);
    }
    Ok(RebalancePlan {
        required_cells: required_cells_of(&tasks),
        tasks,
        before,
        after,
        after_by_cell: working.checksum_by_cell(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, load: u32, health: CellHealth) -> CellCandidate {
        CellCandidate::new(CellId::new(id), load, health).expect("fixture load is in range")
    }

    #[test]
    fn fnv1a_matches_the_published_reference_vectors() {
        // Canonical FNV-1a 64 test vectors: the empty string hashes to the
        // offset basis, and "a" / "foobar" have published digests. This
        // pins the substitution so a refactor cannot quietly change every
        // tenant's shard.
        assert_eq!(fnv1a_64(b""), FNV_OFFSET_BASIS_64);
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a_64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn shard_derivation_is_stable_across_calls() {
        let first = derive_shard_key("ten_alpha", 64).expect("64 shards is a legal shard space");
        let second = derive_shard_key("ten_alpha", 64).expect("64 shards is a legal shard space");
        assert_eq!(first, second);
        assert!(first.0 < 64);
    }

    #[test]
    fn shard_derivation_spreads_distinct_tenants() {
        let shards: BTreeSet<ShardKey> = (0..256)
            .map(|index| {
                derive_shard_key(&format!("ten_{index:04}"), 16)
                    .expect("16 shards is a legal shard space")
            })
            .collect();
        // 256 tenants over 16 shards: a hash that collapsed would occupy
        // only a handful of shards. Require full coverage.
        assert_eq!(shards.len(), 16);
    }

    #[test]
    fn shard_derivation_rejects_a_zero_shard_space() {
        let error =
            derive_shard_key("ten_alpha", 0).expect_err("zero shards has no legal shard key");
        assert_eq!(error, CellKernelError::ZeroShardCount);
    }

    #[test]
    fn least_loaded_picks_the_quietest_healthy_cell() {
        let candidates = vec![
            candidate("cell-c", 100, CellHealth::Healthy),
            candidate("cell-a", 900, CellHealth::Healthy),
            candidate("cell-b", 10, CellHealth::Unhealthy),
        ];
        let chosen = select_least_loaded(&candidates).expect("one healthy cell exists");
        assert_eq!(chosen, CellId::new("cell-c"));
    }

    #[test]
    fn least_loaded_breaks_ties_on_the_smallest_cell_id() {
        let forward = vec![
            candidate("cell-b", 500, CellHealth::Healthy),
            candidate("cell-a", 500, CellHealth::Healthy),
        ];
        let reversed: Vec<CellCandidate> = forward.iter().rev().cloned().collect();
        assert_eq!(
            select_least_loaded(&forward).expect("healthy cells exist"),
            CellId::new("cell-a")
        );
        assert_eq!(
            select_least_loaded(&reversed).expect("healthy cells exist"),
            CellId::new("cell-a")
        );
    }

    #[test]
    fn least_loaded_rejects_a_set_with_no_healthy_cell() {
        let candidates = vec![
            candidate("cell-a", 10, CellHealth::Degraded),
            candidate("cell-b", 10, CellHealth::Unhealthy),
        ];
        let error = select_least_loaded(&candidates).expect_err("no cell is healthy");
        assert_eq!(error, CellKernelError::NoHealthyCell);
    }

    #[test]
    fn selection_rejects_an_impossible_load_observation() {
        let bogus = CellCandidate {
            cell: CellId::new("cell-a"),
            load_permille: MAX_LOAD_PERMILLE + 1,
            health: CellHealth::Healthy,
        };
        let error = select_least_loaded(&[bogus]).expect_err("load above full scale is impossible");
        assert_eq!(error, CellKernelError::LoadOutOfRange);
    }

    #[test]
    fn selection_rejects_a_malformed_cell_id_smuggled_in_by_struct_literal() {
        // `CellCandidate::new` refuses this, but the fields are public, so
        // the domain re-validates rather than trusting the constructor.
        let bogus = CellCandidate {
            cell: CellId::new("node1'; DROP TABLE pg_dist_shard; --"),
            load_permille: 0,
            health: CellHealth::Healthy,
        };
        assert_eq!(
            select_least_loaded(std::slice::from_ref(&bogus))
                .expect_err("a quoted cell id is not a legal decision input"),
            CellKernelError::MalformedCellId {
                cell: "node1'; DROP TABLE pg_dist_shard; --".to_owned()
            }
        );
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        assert!(matches!(
            plan_rebalance(&placement, &[bogus]),
            Err(CellKernelError::MalformedCellId { .. })
        ));
    }

    #[test]
    fn a_duplicated_candidate_folds_to_the_most_severe_reading() {
        // Two scrapes of the same cell disagree. Whichever order they
        // arrive in, the conservative reading wins, so planning stays
        // order-independent.
        let forward = vec![
            candidate("cell-a", 100, CellHealth::Healthy),
            candidate("cell-a", 900, CellHealth::Unhealthy),
            candidate("cell-b", 0, CellHealth::Healthy),
        ];
        let reversed: Vec<CellCandidate> = forward.iter().rev().cloned().collect();
        assert_eq!(
            select_least_loaded(&forward).expect("cell-b is healthy"),
            CellId::new("cell-b")
        );
        assert_eq!(
            select_least_loaded(&reversed).expect("cell-b is healthy"),
            CellId::new("cell-b")
        );
    }

    #[test]
    fn shard_placement_is_sticky_and_ignores_load() {
        let candidates = vec![
            candidate("cell-a", 900, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
        ];
        let key = derive_shard_key("ten_alpha", 2).expect("2 shards is a legal shard space");
        let first = cell_for_shard(key, &candidates).expect("healthy cells exist");
        let second = cell_for_shard(key, &candidates).expect("healthy cells exist");
        assert_eq!(first, second);
        // Whichever cell it lands on, it must be one of the healthy ones,
        // chosen by shard and not by load.
        assert!(first == CellId::new("cell-a") || first == CellId::new("cell-b"));
    }

    #[test]
    fn shard_placement_pins_the_exact_shard_to_cell_mapping() {
        // Pins the mapping itself, not merely that it is stable: a
        // constant selector (ignore the shard key, always take the first
        // healthy cell) would satisfy stickiness and must fail here.
        let candidates = vec![
            candidate("cell-a", 0, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
            candidate("cell-c", 0, CellHealth::Healthy),
        ];
        for (key, expected) in [
            (0u64, "cell-a"),
            (1, "cell-b"),
            (2, "cell-c"),
            (7, "cell-b"),
        ] {
            assert_eq!(
                cell_for_shard(ShardKey(key), &candidates).expect("healthy cells exist"),
                CellId::new(expected),
                "shard {key} must map to {expected}"
            );
        }
    }

    #[test]
    fn shard_placement_covers_every_healthy_cell_across_the_shard_space() {
        // A collapsed mapping (every shard onto one cell) is the failure
        // this catches; the modulo mapping must hit all three.
        let candidates = vec![
            candidate("cell-a", 0, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
            candidate("cell-c", 0, CellHealth::Healthy),
        ];
        let reached: BTreeSet<CellId> = (0..64)
            .map(|key| cell_for_shard(ShardKey(key), &candidates).expect("healthy cells exist"))
            .collect();
        assert_eq!(reached.len(), 3);
    }

    #[test]
    fn shard_placement_needs_a_healthy_cell() {
        let error = cell_for_shard(ShardKey(3), &[candidate("cell-a", 0, CellHealth::Degraded)])
            .expect_err("a degraded cell may not receive a shard");
        assert_eq!(error, CellKernelError::NoHealthyCell);
    }

    #[test]
    fn checksum_is_invariant_under_a_move_and_changes_on_loss() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-a"), "ten_two");
        placement.register_cell(&CellId::new("cell-b"));
        let before = placement.checksum();

        placement
            .apply(&RebalanceTask::new(
                "ten_one",
                CellId::new("cell-a"),
                CellId::new("cell-b"),
                REASON_LEVEL_LOAD,
            ))
            .expect("the move is legal");
        assert_eq!(placement.checksum(), before);

        placement
            .evict(&CellId::new("cell-b"), "ten_one")
            .expect("cell-b holds ten_one after the move");
        assert_ne!(placement.checksum(), before);
        assert_eq!(placement.checksum().occupancy, before.occupancy - 1);
    }

    #[test]
    fn the_per_cell_checksum_sees_a_permutation_the_total_cannot() {
        let mut left = Placement::new();
        left.place(&CellId::new("cell-a"), "ten_one");
        left.place(&CellId::new("cell-b"), "ten_two");
        let mut right = Placement::new();
        right.place(&CellId::new("cell-a"), "ten_two");
        right.place(&CellId::new("cell-b"), "ten_one");

        assert_eq!(
            left.checksum(),
            right.checksum(),
            "the total fold is blind to which cell holds what"
        );
        assert_ne!(left.checksum_by_cell(), right.checksum_by_cell());
        // A cell registered but empty must fingerprint as absent, so
        // `prepare` can never manufacture a divergence.
        let mut padded = left.clone();
        padded.register_cell(&CellId::new("cell-z"));
        assert_eq!(
            padded.checksum_by_cell().get(&CellId::new("cell-z")),
            Some(&PlacementChecksum::EMPTY)
        );
    }

    #[test]
    fn apply_rejects_a_move_that_would_lose_a_tenant() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.register_cell(&CellId::new("cell-b"));
        let error = placement
            .apply(&RebalanceTask::new(
                "ten_ghost",
                CellId::new("cell-a"),
                CellId::new("cell-b"),
                REASON_LEVEL_LOAD,
            ))
            .expect_err("cell-a does not hold ten_ghost");
        assert_eq!(error, CellKernelError::TenantNotInSourceCell);
    }

    #[test]
    fn apply_rejects_a_move_that_would_duplicate_a_tenant() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-b"), "ten_one");
        let error = placement
            .apply(&RebalanceTask::new(
                "ten_one",
                CellId::new("cell-a"),
                CellId::new("cell-b"),
                REASON_LEVEL_LOAD,
            ))
            .expect_err("cell-b already holds ten_one");
        assert_eq!(error, CellKernelError::TenantDuplicated);
    }

    #[test]
    fn apply_rejects_unknown_endpoints_and_self_moves() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");

        assert_eq!(
            placement
                .apply(&RebalanceTask::new(
                    "ten_one",
                    CellId::new("cell-a"),
                    CellId::new("cell-a"),
                    REASON_LEVEL_LOAD,
                ))
                .expect_err("a self-move is malformed"),
            CellKernelError::RebalanceConflict
        );
        assert_eq!(
            placement
                .apply(&RebalanceTask::new(
                    "ten_one",
                    CellId::new("cell-a"),
                    CellId::new("cell-zzz"),
                    REASON_LEVEL_LOAD,
                ))
                .expect_err("cell-zzz is not registered"),
            CellKernelError::UnknownTargetCell
        );
        assert_eq!(
            placement
                .apply(&RebalanceTask::new(
                    "ten_one",
                    CellId::new("cell-yyy"),
                    CellId::new("cell-a"),
                    REASON_LEVEL_LOAD,
                ))
                .expect_err("cell-yyy is not registered"),
            CellKernelError::UnknownSourceCell
        );
    }

    #[test]
    fn planning_levels_a_lopsided_placement() {
        let mut placement = Placement::new();
        for index in 0..6 {
            placement.place(&CellId::new("cell-a"), &format!("ten_{index}"));
        }
        let candidates = vec![
            candidate("cell-a", 900, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
        ];

        let plan = plan_rebalance(&placement, &candidates).expect("the placement can be levelled");
        assert_eq!(plan.len(), 3);
        assert!(
            plan.tasks()
                .iter()
                .all(|task| task.reason == REASON_LEVEL_LOAD)
        );
        assert_eq!(plan.before_checksum(), plan.after_checksum());

        plan.prepare(&mut placement);
        for task in plan.tasks() {
            placement.apply(task).expect("planned moves are legal");
        }
        plan.verify_applied(&placement)
            .expect("execution conserved the tenant set");
        assert_eq!(placement.load_of(&CellId::new("cell-a")), 3);
        assert_eq!(placement.load_of(&CellId::new("cell-b")), 3);
    }

    #[test]
    fn planning_drains_an_unhealthy_cell_completely() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-sick"), "ten_one");
        placement.place(&CellId::new("cell-sick"), "ten_two");
        placement.place(&CellId::new("cell-ok"), "ten_three");
        let candidates = vec![
            candidate("cell-sick", 200, CellHealth::Unhealthy),
            candidate("cell-ok", 100, CellHealth::Healthy),
            candidate("cell-new", 0, CellHealth::Healthy),
        ];

        let plan = plan_rebalance(&placement, &candidates).expect("healthy cells can absorb");
        assert!(
            plan.tasks()
                .iter()
                .filter(|task| task.reason == REASON_DRAIN_UNHEALTHY)
                .count()
                >= 2
        );
        plan.prepare(&mut placement);
        for task in plan.tasks() {
            placement.apply(task).expect("planned moves are legal");
        }
        assert_eq!(placement.load_of(&CellId::new("cell-sick")), 0);
        plan.verify_applied(&placement)
            .expect("draining conserved the tenant set");
    }

    #[test]
    fn a_drained_tenant_lands_on_the_least_loaded_healthy_cell() {
        // Pins the recipient rule itself. Inverting it — piling the
        // drained tenants onto the busiest eligible cell, which is the
        // cascade this planner exists to avoid — must fail here.
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-sick"), "ten_one");
        for index in 0..4 {
            placement.place(&CellId::new("cell-busy"), &format!("ten_busy_{index}"));
        }
        placement.place(&CellId::new("cell-quiet"), "ten_quiet");
        let candidates = vec![
            candidate("cell-sick", 900, CellHealth::Unhealthy),
            candidate("cell-busy", 10, CellHealth::Healthy),
            candidate("cell-quiet", 990, CellHealth::Healthy),
        ];

        let plan = plan_rebalance(&placement, &candidates).expect("healthy cells can absorb");
        let drain: Vec<&RebalanceTask> = plan
            .tasks()
            .iter()
            .filter(|task| task.reason == REASON_DRAIN_UNHEALTHY)
            .collect();
        assert_eq!(drain.len(), 1);
        // cell-quiet holds ONE tenant and cell-busy holds four; the
        // recipient rule is by actual occupancy, not by the reported
        // load_permille, so the drained tenant goes to cell-quiet.
        assert_eq!(drain[0].tenant, "ten_one");
        assert_eq!(drain[0].to_cell, CellId::new("cell-quiet"));
    }

    #[test]
    fn a_degraded_cell_keeps_its_tenants_and_receives_none() {
        // The blast-radius guarantee: a transient degradation must not
        // stampede a whole cell's population onto its neighbours.
        let mut placement = Placement::new();
        for index in 0..5 {
            placement.place(&CellId::new("cell-a"), &format!("ten_{index}"));
        }
        let candidates = vec![
            candidate("cell-a", 900, CellHealth::Degraded),
            candidate("cell-b", 0, CellHealth::Healthy),
        ];

        let plan = plan_rebalance(&placement, &candidates).expect("a degraded cell is plannable");
        assert!(
            plan.is_empty(),
            "a degraded cell keeps its tenants: {:?}",
            plan.tasks()
        );

        // And it takes no new ones either: levelling never uses it as a
        // recipient, even when it is by far the quietest.
        let mut lopsided = Placement::new();
        for index in 0..6 {
            lopsided.place(&CellId::new("cell-b"), &format!("ten_{index}"));
        }
        lopsided.register_cell(&CellId::new("cell-a"));
        let candidates = vec![
            candidate("cell-a", 0, CellHealth::Degraded),
            candidate("cell-b", 900, CellHealth::Healthy),
        ];
        let plan = plan_rebalance(&lopsided, &candidates).expect("a degraded cell is plannable");
        assert!(
            plan.is_empty(),
            "a degraded cell must not receive: {:?}",
            plan.tasks()
        );
    }

    #[test]
    fn an_unhealthy_cell_is_drained_but_a_degraded_one_beside_it_is_not() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-sick"), "ten_sick");
        placement.place(&CellId::new("cell-slow"), "ten_slow");
        placement.place(&CellId::new("cell-ok"), "ten_ok");
        let candidates = vec![
            candidate("cell-sick", 500, CellHealth::Unhealthy),
            candidate("cell-slow", 500, CellHealth::Degraded),
            candidate("cell-ok", 500, CellHealth::Healthy),
        ];

        let plan = plan_rebalance(&placement, &candidates).expect("cell-ok can absorb");
        assert_eq!(plan.len(), 1);
        assert_eq!(plan.tasks()[0].tenant, "ten_sick");
        assert_eq!(plan.tasks()[0].from_cell, CellId::new("cell-sick"));
        assert_eq!(plan.tasks()[0].to_cell, CellId::new("cell-ok"));
        assert_eq!(plan.tasks()[0].reason, REASON_DRAIN_UNHEALTHY);
    }

    #[test]
    fn an_occupied_cell_absent_from_the_roster_is_a_typed_error_not_an_evacuation() {
        // An incomplete inventory (a truncated page, a scrape miss) must
        // not read as "drain everything you forgot to mention".
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-b"), "ten_two");
        for index in 0..8 {
            placement.place(&CellId::new("cell-c"), &format!("ten_c_{index}"));
        }
        let truncated = vec![
            candidate("cell-a", 100, CellHealth::Healthy),
            candidate("cell-b", 100, CellHealth::Healthy),
        ];

        let error = plan_rebalance(&placement, &truncated)
            .expect_err("cell-c's health is unknown, so no plan is possible");
        assert_eq!(
            error,
            CellKernelError::PlacementCellNotInRoster {
                cell: CellId::new("cell-c")
            }
        );
        // A cell that is registered but holds nothing needs no coverage:
        // there is no tenant to lose.
        let mut with_empty = Placement::new();
        with_empty.place(&CellId::new("cell-a"), "ten_one");
        with_empty.place(&CellId::new("cell-b"), "ten_two");
        with_empty.register_cell(&CellId::new("cell-empty"));
        plan_rebalance(&with_empty, &truncated).expect("an empty stray cell is harmless");
    }

    #[test]
    fn planning_is_deterministic_across_candidate_ordering() {
        let mut placement = Placement::new();
        for index in 0..7 {
            placement.place(&CellId::new("cell-a"), &format!("ten_{index}"));
        }
        let forward = vec![
            candidate("cell-a", 700, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
            candidate("cell-c", 0, CellHealth::Healthy),
        ];
        let reversed: Vec<CellCandidate> = forward.iter().rev().cloned().collect();
        let first = plan_rebalance(&placement, &forward).expect("levelling succeeds");
        let second = plan_rebalance(&placement, &reversed).expect("levelling succeeds");
        assert_eq!(first, second);
    }

    #[test]
    fn planning_refuses_when_no_cell_is_healthy() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        let error = plan_rebalance(
            &placement,
            &[candidate("cell-a", 10, CellHealth::Unhealthy)],
        )
        .expect_err("a tenant cannot be drained onto nothing");
        // And the error names the tenant that had nowhere to go.
        assert_eq!(
            error,
            CellKernelError::NoHealthyCellFor {
                tenant: "ten_one".to_owned(),
                considered: 0,
            }
        );
    }

    #[test]
    fn planning_an_already_balanced_placement_emits_no_moves() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-b"), "ten_two");
        let candidates = vec![
            candidate("cell-a", 500, CellHealth::Healthy),
            candidate("cell-b", 500, CellHealth::Healthy),
        ];
        let plan = plan_rebalance(&placement, &candidates).expect("a balanced placement plans");
        assert!(plan.is_empty());
        assert_eq!(plan.before_checksum(), plan.after_checksum());
    }

    #[test]
    fn a_hand_written_plan_that_drops_a_tenant_is_a_typed_error() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.register_cell(&CellId::new("cell-b"));
        let error = RebalancePlan::from_tasks(
            &placement,
            vec![RebalanceTask::new(
                "ten_absent",
                CellId::new("cell-a"),
                CellId::new("cell-b"),
                "hand-written",
            )],
        )
        .expect_err("a move of an absent tenant would lose it");
        assert_eq!(error, CellKernelError::TenantNotInSourceCell);
    }

    #[test]
    fn verification_catches_execution_that_lost_a_tenant() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-a"), "ten_two");
        placement.register_cell(&CellId::new("cell-b"));
        let candidates = vec![
            candidate("cell-a", 800, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
        ];
        let plan = plan_rebalance(&placement, &candidates).expect("levelling succeeds");
        plan.prepare(&mut placement);
        for task in plan.tasks() {
            placement.apply(task).expect("planned moves are legal");
        }
        // Simulate a lossy executor: the shard move "succeeded" but a
        // tenant never arrived.
        placement
            .evict(&CellId::new("cell-a"), "ten_two")
            .expect("cell-a still holds ten_two");
        let error = plan
            .verify_applied(&placement)
            .expect_err("a lost tenant must not verify");
        assert_eq!(error, CellKernelError::IntegrityMismatch);
    }

    #[test]
    fn verification_catches_a_misroute_the_total_checksum_is_blind_to() {
        // cell-x is being drained onto cell-a. The executor misfires and
        // moves the OTHER tenant the other way instead. Occupancy and the
        // whole-placement digest are unchanged, so only a cell-aware
        // check can see it.
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-x"), "ten_a");
        placement.place(&CellId::new("cell-a"), "ten_b");
        let candidates = vec![
            candidate("cell-x", 500, CellHealth::Unhealthy),
            candidate("cell-a", 500, CellHealth::Healthy),
        ];
        let plan = plan_rebalance(&placement, &candidates).expect("cell-a can absorb");
        assert_eq!(plan.len(), 1);

        let mut misrouted = Placement::new();
        misrouted.place(&CellId::new("cell-x"), "ten_a");
        misrouted.place(&CellId::new("cell-x"), "ten_b");
        misrouted.register_cell(&CellId::new("cell-a"));
        assert_eq!(
            misrouted.checksum(),
            plan.after_checksum(),
            "the total fold cannot tell these apart — that is the point"
        );

        let error = plan
            .verify_applied(&misrouted)
            .expect_err("a misroute must not verify");
        // Names the tenant and where it should have landed, not just
        // "something is wrong somewhere".
        assert_eq!(
            error,
            CellKernelError::TaskNotApplied {
                tenant: "ten_a".to_owned(),
                expected_cell: CellId::new("cell-a"),
            }
        );
    }

    #[test]
    fn verification_catches_a_planned_tenant_left_where_it_started() {
        // A no-op executor that also happens to have swapped two other
        // tenants between two cells would slip past the fingerprints;
        // the exact endpoint check does not hash at all.
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.register_cell(&CellId::new("cell-b"));
        let plan = RebalancePlan::from_tasks(
            &placement,
            vec![RebalanceTask::new(
                "ten_one",
                CellId::new("cell-a"),
                CellId::new("cell-b"),
                "hand-written",
            )],
        )
        .expect("the move is legal");

        let error = plan
            .verify_applied(&placement)
            .expect_err("nothing was executed");
        assert_eq!(
            error,
            CellKernelError::TaskNotApplied {
                tenant: "ten_one".to_owned(),
                expected_cell: CellId::new("cell-b"),
            }
        );
    }

    #[test]
    fn verification_catches_collateral_movement_of_an_unplanned_tenant() {
        // Every planned move landed, and the total fold is conserved, but
        // the executor also shuffled a tenant nobody asked it to touch.
        // Only the per-cell fingerprint can see that.
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-a"), "ten_two");
        placement.place(&CellId::new("cell-c"), "ten_three");
        placement.place(&CellId::new("cell-d"), "ten_four");
        placement.register_cell(&CellId::new("cell-b"));
        let plan = RebalancePlan::from_tasks(
            &placement,
            vec![RebalanceTask::new(
                "ten_one",
                CellId::new("cell-a"),
                CellId::new("cell-b"),
                "hand-written",
            )],
        )
        .expect("the move is legal");

        let mut applied = placement.clone();
        for task in plan.tasks() {
            applied.apply(task).expect("planned moves are legal");
        }
        // Collateral: ten_three and ten_four swap cells behind our back.
        applied
            .evict(&CellId::new("cell-c"), "ten_three")
            .expect("cell-c holds ten_three");
        applied
            .evict(&CellId::new("cell-d"), "ten_four")
            .expect("cell-d holds ten_four");
        applied.place(&CellId::new("cell-c"), "ten_four");
        applied.place(&CellId::new("cell-d"), "ten_three");

        assert_eq!(
            applied.checksum(),
            plan.after_checksum(),
            "the total fold is conserved by the swap"
        );
        let error = plan
            .verify_applied(&applied)
            .expect_err("collateral movement must not verify");
        assert_eq!(
            error,
            CellKernelError::PlacementDiverged {
                cell: CellId::new("cell-c")
            }
        );
    }

    #[test]
    fn an_empty_registered_cell_never_counts_as_divergence() {
        let mut placement = Placement::new();
        placement.place(&CellId::new("cell-a"), "ten_one");
        placement.place(&CellId::new("cell-a"), "ten_two");
        let candidates = vec![
            candidate("cell-a", 900, CellHealth::Healthy),
            candidate("cell-b", 0, CellHealth::Healthy),
        ];
        let plan = plan_rebalance(&placement, &candidates).expect("levelling succeeds");
        plan.prepare(&mut placement);
        for task in plan.tasks() {
            placement.apply(task).expect("planned moves are legal");
        }
        // An unrelated, empty cell appearing in the applied placement is
        // bookkeeping, not divergence.
        placement.register_cell(&CellId::new("cell-spare"));
        plan.verify_applied(&placement)
            .expect("an empty stray cell is not a divergence");
    }
}
