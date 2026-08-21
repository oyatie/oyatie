//! Cell-assignment kernel: the entities and ports IP-008 assigns to
//! `oya-tenancy-cell-assignment-kernel`, collapsed into a module of this
//! crate (the capability is capped at 12 crates).
//!
//! Nothing here reads a clock, draws randomness, or performs I/O. Every
//! observation a decision depends on — a cell's load, a cell's health —
//! arrives as a parameter or through a port, so the domain and usecase
//! layers above are pure functions of their inputs.

/// The longest legal [`CellId`], matching the DNS name length limit that
/// bounds a Citus/Patroni node name.
pub const MAX_CELL_ID_LEN: usize = 253;

/// A cell: one blast-radius-bounded unit of the cellular architecture
/// (ADR-0248). Tenants are assigned to exactly one cell at a time.
///
/// `Ord` is derived deliberately: cell ids are the deterministic
/// tie-breaker for every selection and planning decision in this crate,
/// so their ordering is part of the published contract.
///
/// # Well-formedness
///
/// This crate is where a cell id is minted, and it is therefore the only
/// place a syntactic constraint can be enforced once for every consumer.
/// The downstream Citus adapter IP-008 specifies interpolates the node
/// name straight into SQL (`citus_move_shard_placement(.., '{source}',
/// '{target}')`), so an id carrying a quote, a semicolon or a newline is
/// an injection vector rather than a cosmetic problem.
///
/// A well-formed id is non-empty, at most [`MAX_CELL_ID_LEN`] bytes, and
/// built only from ASCII letters, ASCII digits, `-`, `_` and `.`. Use
/// [`CellId::parse`] to reject a malformed id at the boundary;
/// [`CellId::new`] stays infallible because the inner `String` is public
/// and an infallible constructor is what the rest of the crate's tests
/// and fixtures are written against. Every id that reaches a *decision*
/// is re-checked: [`CellCandidate::new`] rejects a malformed id, and the
/// domain re-validates candidates built as struct literals.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CellId(pub String); // data_class: INTERNAL_ONLY

impl CellId {
    /// Build a cell id from anything string-like, without validating it.
    ///
    /// Prefer [`CellId::parse`] anywhere the id comes from configuration,
    /// an operator, or another service.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Build a cell id, rejecting one that is not well-formed.
    ///
    /// # Errors
    /// [`CellKernelError::MalformedCellId`] when the id is empty, longer
    /// than [`MAX_CELL_ID_LEN`], or contains a byte outside
    /// `[A-Za-z0-9._-]`.
    pub fn parse(id: impl Into<String>) -> Result<Self, CellKernelError> {
        let candidate = Self(id.into());
        if candidate.is_well_formed() {
            Ok(candidate)
        } else {
            Err(CellKernelError::MalformedCellId { cell: candidate.0 })
        }
    }

    /// Whether this id satisfies the well-formedness rule documented on
    /// [`CellId`].
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        !self.0.is_empty()
            && self.0.len() <= MAX_CELL_ID_LEN
            && self
                .0
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    }

    /// Borrow the underlying identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for CellId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The consistent-hash shard a tenant hashes onto, in `0..shard_count`.
///
/// The shard key is the stable, storage-facing coordinate; the cell a
/// tenant currently *lives* in may move under rebalance while its shard
/// key does not.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ShardKey(pub u64); // data_class: INTERNAL_ONLY

/// Observed health of a cell, as reported by [`CellHealthProbe`].
///
/// This is a three-state machine and each state means something
/// different to the planner:
///
/// - [`CellHealth::Healthy`] — serving, and the only state eligible to
///   *receive* a tenant.
/// - [`CellHealth::Degraded`] — serving. It **keeps** the tenants it
///   holds and simply takes no new ones. Degradation is transient by
///   nature (a slow replica, elevated latency), and evacuating on it
///   would turn a one-cell wobble into a multi-cell stampede — exactly
///   the blast-radius event the cellular architecture exists to prevent.
/// - [`CellHealth::Unhealthy`] — not serving. Its tenants are drained.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CellHealth {
    /// Serving normally; eligible to receive tenants.
    Healthy,
    /// Serving, but not eligible to receive new tenants. Keeps its own.
    Degraded,
    /// Not serving; existing tenants are drained off it.
    Unhealthy,
}

impl CellHealth {
    /// Whether this cell may receive a tenant.
    #[must_use]
    pub fn is_healthy(self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Whether the tenants this cell holds must be moved off it.
    ///
    /// True for [`CellHealth::Unhealthy`] only: a `Degraded` cell keeps
    /// what it has. This is the predicate the rebalance planner drains
    /// on, and it is deliberately NOT the negation of
    /// [`CellHealth::is_healthy`].
    #[must_use]
    pub fn is_drained(self) -> bool {
        matches!(self, Self::Unhealthy)
    }

    /// Stable lowercase label, for logs and rebalance reasons.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}

/// The largest legal value of [`CellCandidate::load_permille`].
///
/// Load is carried in permille (parts per thousand) rather than a float
/// so that comparison is total and reproducible — no `NaN`, no
/// platform-dependent rounding, and therefore no ordering that can differ
/// between the planner and the code that verifies its plan.
pub const MAX_LOAD_PERMILLE: u32 = 1000;

/// A cell offered to the selector, with the load and health observed for
/// it at the moment the caller assembled the candidate set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCandidate {
    /// Which cell this candidate describes.
    pub cell: CellId, // data_class: INTERNAL_ONLY
    /// Observed utilisation in parts per thousand, `0..=MAX_LOAD_PERMILLE`.
    pub load_permille: u32, // data_class: INTERNAL_ONLY
    /// Observed health at candidate-assembly time.
    pub health: CellHealth, // data_class: INTERNAL_ONLY
}

impl CellCandidate {
    /// Build a validated candidate.
    ///
    /// # Errors
    /// - [`CellKernelError::LoadOutOfRange`] when `load_permille` exceeds
    ///   [`MAX_LOAD_PERMILLE`].
    /// - [`CellKernelError::MalformedCellId`] when `cell` is not
    ///   well-formed; see [`CellId`].
    pub fn new(
        cell: CellId,
        load_permille: u32,
        health: CellHealth,
    ) -> Result<Self, CellKernelError> {
        if !cell.is_well_formed() {
            return Err(CellKernelError::MalformedCellId { cell: cell.0 });
        }
        if load_permille > MAX_LOAD_PERMILLE {
            return Err(CellKernelError::LoadOutOfRange);
        }
        Ok(Self {
            cell,
            load_permille,
            health,
        })
    }
}

/// One tenant move in a rebalance plan.
///
/// A task is a *pure move*: the tenant leaves `from_cell` and arrives in
/// `to_cell`. It can neither create nor destroy a tenant, which is what
/// makes the before/after placement checksum a meaningful integrity
/// check on a whole plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceTask {
    /// The tenant being moved.
    pub tenant: String, // data_class: INTERNAL_ONLY
    /// The cell the tenant currently occupies.
    pub from_cell: CellId, // data_class: INTERNAL_ONLY
    /// The cell the tenant is moved to.
    pub to_cell: CellId, // data_class: INTERNAL_ONLY
    /// Why the planner emitted this move; see the `REASON_*` constants.
    pub reason: String, // data_class: INTERNAL_ONLY
}

impl RebalanceTask {
    /// Build a move task.
    pub fn new(
        tenant: impl Into<String>,
        from_cell: CellId,
        to_cell: CellId,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            tenant: tenant.into(),
            from_cell,
            to_cell,
            reason: reason.into(),
        }
    }
}

/// Durable record of which cell each tenant is assigned to.
///
/// Deliberately synchronous: this is the seam the Citus/`pg_dist_shard`
/// adapter plugs into later (see the crate-level Gaps note).
pub trait CellAssignmentRepository {
    /// The cell a tenant is currently assigned to, if any.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] when the record store
    /// cannot be read.
    fn assigned_cell(&self, tenant: &str) -> Result<Option<CellId>, CellKernelError>;

    /// Durably record `tenant -> cell`, replacing any prior assignment.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] when the record store
    /// cannot be written.
    fn record_assignment(&self, tenant: &str, cell: &CellId) -> Result<(), CellKernelError>;

    /// Drop a tenant's assignment, if it has one.
    ///
    /// Returns whether a row was actually removed. Offboarding is the
    /// other half of assignment: without it the record store grows with
    /// every tenant that has *ever* existed, and a re-issued tenant id
    /// silently inherits the retired cell of its namesake through the
    /// idempotency short-circuit in
    /// [`crate::usecase::CellAssignmentService::assign`].
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] when the record store
    /// cannot be written.
    fn forget_assignment(&self, tenant: &str) -> Result<bool, CellKernelError>;
}

/// Liveness check for a single cell.
///
/// Deliberately synchronous and one-shot: the 1s cadence probe *loop* of
/// IP-008 is a worker concern that needs an async runtime, and this port
/// is the seam it drives (see the crate-level Gaps note).
pub trait CellHealthProbe {
    /// Observe the health of one cell right now.
    ///
    /// An implementation that has never contacted the cell MUST fail
    /// rather than answer [`CellHealth::Healthy`]: "I have no
    /// observation" is not evidence of health, and treating it as such
    /// is how a typo'd or decommissioned cell id gets rubber-stamped as
    /// a placement target.
    ///
    /// # Errors
    /// [`CellKernelError::ProbeFailed`] when the cell could not be
    /// observed at all — which is distinct from observing it as
    /// [`CellHealth::Unhealthy`].
    fn probe(&self, cell: &CellId) -> Result<CellHealth, CellKernelError>;
}

/// Every way a cell-assignment decision can fail.
///
/// Plain enum implementing [`core::fmt::Display`] and
/// [`std::error::Error`]; no `thiserror`, no `anyhow`. Variants that
/// describe a *specific* subject carry it, so the operator reading the
/// error does not have to re-derive which tenant, which cell, or how far
/// a plan got — IP-008's `IntegrityCheckFailed { shard_id }` shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellKernelError {
    /// No candidate cell was both healthy and eligible.
    ///
    /// Raised by the bare selection functions, which are not told what
    /// they are selecting *for*. Call sites that know the tenant raise
    /// [`CellKernelError::NoHealthyCellFor`] instead.
    NoHealthyCell,
    /// No healthy cell was eligible to receive a named tenant, out of
    /// `considered` candidates.
    NoHealthyCellFor {
        /// The tenant that could not be homed.
        tenant: String,
        /// How many candidates were on the table when it failed.
        considered: usize,
    },
    /// A cell could not be observed at all, which is distinct from
    /// observing it as [`CellHealth::Unhealthy`].
    ProbeFailed {
        /// The cell whose health could not be established, so the
        /// operator does not have to re-derive which of a forty-cell
        /// roster went dark.
        cell: CellId,
    },
    /// The assignment record store could not be read or written.
    PersistenceUnavailable,
    /// A rebalance plan is internally malformed — a self-move, or a plan
    /// that fails to converge within its bounded iteration budget.
    RebalanceConflict,
    /// A shard key was requested against a shard space of size zero.
    ZeroShardCount,
    /// A candidate reported a load above [`MAX_LOAD_PERMILLE`].
    LoadOutOfRange,
    /// A cell id is not well-formed; see [`CellId`].
    MalformedCellId {
        /// The rejected id, verbatim, so the operator can find its source.
        cell: String,
    },
    /// A task names a `from_cell` that the placement does not know.
    UnknownSourceCell,
    /// A task names a `to_cell` that the placement does not know.
    UnknownTargetCell,
    /// A task claims to move a tenant out of a cell that does not hold it
    /// — applying it would lose the tenant.
    TenantNotInSourceCell,
    /// A task would place a tenant into a cell that already holds it —
    /// applying it would duplicate the tenant.
    TenantDuplicated,
    /// The placement holds tenants in a cell the candidate roster does
    /// not mention, so its health is unknown and no plan can be made.
    ///
    /// This exists because the alternative — inferring "not listed"
    /// means "not healthy" — turns a truncated or partially-failed cell
    /// listing into a full evacuation of the cells it forgot to mention.
    PlacementCellNotInRoster {
        /// The occupied cell missing from the roster.
        cell: CellId,
    },
    /// The placement checksum taken after a plan differs from the one
    /// taken before it: the plan did not conserve tenants.
    IntegrityMismatch,
    /// A named cell does not hold, after execution, the tenant set the
    /// plan predicted for it.
    ///
    /// This is the cell-aware half of the integrity check: a whole-
    /// placement fingerprint is invariant under *any* permutation of
    /// tenants across cells and therefore cannot see a misroute.
    PlacementDiverged {
        /// The first cell (in ascending id order) that disagrees.
        cell: CellId,
    },
    /// Execution did not leave a planned tenant in its planned cell.
    TaskNotApplied {
        /// The tenant the plan moved.
        tenant: String,
        /// Where the plan said it should have landed.
        expected_cell: CellId,
    },
    /// Recording a plan's moves failed part-way through.
    ///
    /// `committed` is the number of leading tasks of the plan that are
    /// durable, so `plan.tasks()[..committed]` are applied and
    /// `plan.tasks()[committed]` is the one that failed. The caller's
    /// in-memory placement is left untouched; reconciling it against the
    /// store needs only that index, not a full table scan.
    PartialPlanExecution {
        /// How many leading tasks were durably recorded.
        committed: usize,
        /// How many tasks the plan contained.
        total: usize,
        /// Why the `committed`-th task failed.
        cause: Box<CellKernelError>,
    },
}

impl core::fmt::Display for CellKernelError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoHealthyCell => {
                formatter.write_str("no healthy cell is eligible to receive the tenant")
            }
            Self::NoHealthyCellFor { tenant, considered } => write!(
                formatter,
                "no healthy cell among {considered} candidate(s) can receive tenant {tenant}"
            ),
            Self::ProbeFailed { cell } => {
                write!(
                    formatter,
                    "the cell health probe could not observe cell {cell}"
                )
            }
            Self::PersistenceUnavailable => {
                formatter.write_str("the cell assignment record store is unavailable")
            }
            Self::RebalanceConflict => {
                formatter.write_str("the rebalance plan is malformed or failed to converge")
            }
            Self::ZeroShardCount => {
                formatter.write_str("a shard key was requested against zero shards")
            }
            Self::LoadOutOfRange => {
                formatter.write_str("a cell reported a load above 1000 permille")
            }
            Self::MalformedCellId { cell } => {
                write!(formatter, "cell id {cell:?} is not well-formed")
            }
            Self::UnknownSourceCell => {
                formatter.write_str("the rebalance source cell is not part of the placement")
            }
            Self::UnknownTargetCell => {
                formatter.write_str("the rebalance target cell is not part of the placement")
            }
            Self::TenantNotInSourceCell => formatter
                .write_str("the rebalance source cell does not hold the tenant being moved"),
            Self::TenantDuplicated => {
                formatter.write_str("the rebalance target cell already holds the tenant")
            }
            Self::PlacementCellNotInRoster { cell } => write!(
                formatter,
                "cell {cell} holds tenants but is absent from the candidate roster, so its health is unknown"
            ),
            Self::IntegrityMismatch => formatter.write_str(
                "the placement checksum changed across the plan: tenants were lost or duplicated",
            ),
            Self::PlacementDiverged { cell } => write!(
                formatter,
                "cell {cell} does not hold the tenant set the plan predicted for it"
            ),
            Self::TaskNotApplied {
                tenant,
                expected_cell,
            } => write!(
                formatter,
                "tenant {tenant} was not left in its planned cell {expected_cell}"
            ),
            Self::PartialPlanExecution {
                committed,
                total,
                cause,
            } => write!(
                formatter,
                "recording the rebalance plan failed at task {committed} of {total}: {cause}"
            ),
        }
    }
}

impl std::error::Error for CellKernelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PartialPlanExecution { cause, .. } => Some(cause.as_ref()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant, for exhaustive checks over the error surface.
    fn every_error() -> Vec<CellKernelError> {
        vec![
            CellKernelError::NoHealthyCell,
            CellKernelError::NoHealthyCellFor {
                tenant: "ten_one".to_owned(),
                considered: 4,
            },
            CellKernelError::ProbeFailed {
                cell: CellId::new("cell-dark"),
            },
            CellKernelError::PersistenceUnavailable,
            CellKernelError::RebalanceConflict,
            CellKernelError::ZeroShardCount,
            CellKernelError::LoadOutOfRange,
            CellKernelError::MalformedCellId {
                cell: "bad id".to_owned(),
            },
            CellKernelError::UnknownSourceCell,
            CellKernelError::UnknownTargetCell,
            CellKernelError::TenantNotInSourceCell,
            CellKernelError::TenantDuplicated,
            CellKernelError::PlacementCellNotInRoster {
                cell: CellId::new("cell-a"),
            },
            CellKernelError::IntegrityMismatch,
            CellKernelError::PlacementDiverged {
                cell: CellId::new("cell-b"),
            },
            CellKernelError::TaskNotApplied {
                tenant: "ten_two".to_owned(),
                expected_cell: CellId::new("cell-c"),
            },
            CellKernelError::PartialPlanExecution {
                committed: 7,
                total: 12,
                cause: Box::new(CellKernelError::PersistenceUnavailable),
            },
        ]
    }

    #[test]
    fn candidate_rejects_load_above_full_scale() {
        let error = CellCandidate::new(
            CellId::new("cell-a"),
            MAX_LOAD_PERMILLE + 1,
            CellHealth::Healthy,
        )
        .expect_err("load above full scale is not a legal observation");
        assert_eq!(error, CellKernelError::LoadOutOfRange);
    }

    #[test]
    fn candidate_accepts_full_scale_load() {
        let candidate = CellCandidate::new(
            CellId::new("cell-a"),
            MAX_LOAD_PERMILLE,
            CellHealth::Degraded,
        )
        .expect("a fully loaded cell is still a legal observation");
        assert_eq!(candidate.load_permille, MAX_LOAD_PERMILLE);
        assert!(!candidate.health.is_healthy());
    }

    #[test]
    fn only_healthy_cells_are_eligible() {
        assert!(CellHealth::Healthy.is_healthy());
        assert!(!CellHealth::Degraded.is_healthy());
        assert!(!CellHealth::Unhealthy.is_healthy());
        assert_eq!(CellHealth::Unhealthy.label(), "unhealthy");
    }

    #[test]
    fn only_an_unhealthy_cell_is_drained_and_it_is_not_the_negation_of_healthy() {
        assert!(CellHealth::Unhealthy.is_drained());
        assert!(!CellHealth::Healthy.is_drained());
        // The whole point of the three-state machine: Degraded is
        // neither eligible to receive NOR a reason to evacuate.
        assert!(!CellHealth::Degraded.is_healthy());
        assert!(!CellHealth::Degraded.is_drained());
    }

    #[test]
    fn a_cell_id_carrying_sql_metacharacters_is_rejected() {
        // The exact shape the IP-008 Citus adapter would interpolate.
        let injection = "node1', 'node2'); DROP TABLE pg_dist_shard; --";
        assert!(!CellId::new(injection).is_well_formed());
        assert_eq!(
            CellId::parse(injection).expect_err("a quoted id is not well-formed"),
            CellKernelError::MalformedCellId {
                cell: injection.to_owned()
            }
        );
        // And it cannot enter a decision through a candidate either.
        assert_eq!(
            CellCandidate::new(CellId::new(injection), 0, CellHealth::Healthy)
                .expect_err("a malformed id is not a legal candidate"),
            CellKernelError::MalformedCellId {
                cell: injection.to_owned()
            }
        );
    }

    #[test]
    fn cell_id_well_formedness_covers_the_boundaries() {
        assert!(CellId::new("cell-a.eu_west.01").is_well_formed());
        assert!(!CellId::new("").is_well_formed(), "empty is not an id");
        assert!(!CellId::new("cell b").is_well_formed(), "space is illegal");
        assert!(
            !CellId::new("cell\nb").is_well_formed(),
            "newline is illegal"
        );
        assert!(!CellId::new("cell-b ").is_well_formed(), "trailing space");
        assert!(CellId::new("c".repeat(MAX_CELL_ID_LEN)).is_well_formed());
        assert!(!CellId::new("c".repeat(MAX_CELL_ID_LEN + 1)).is_well_formed());
    }

    #[test]
    fn every_error_variant_renders_a_distinct_message() {
        let variants = every_error();
        let rendered: std::collections::BTreeSet<String> =
            variants.iter().map(ToString::to_string).collect();
        assert_eq!(rendered.len(), variants.len());
        // The Display text is the operator-facing surface, so it must not
        // be empty for any variant.
        assert!(rendered.iter().all(|message| !message.is_empty()));
    }

    #[test]
    fn context_carrying_variants_name_their_subject_in_the_message() {
        // The point of the payload is that it reaches the operator, not
        // just the `match` arm.
        let rendered = CellKernelError::PartialPlanExecution {
            committed: 7,
            total: 12,
            cause: Box::new(CellKernelError::PersistenceUnavailable),
        }
        .to_string();
        assert!(rendered.contains('7'), "{rendered}");
        assert!(rendered.contains("12"), "{rendered}");
        assert!(
            rendered.contains("record store is unavailable"),
            "{rendered}"
        );

        let rendered = CellKernelError::PlacementDiverged {
            cell: CellId::new("cell-x"),
        }
        .to_string();
        assert!(rendered.contains("cell-x"), "{rendered}");

        let rendered = CellKernelError::ProbeFailed {
            cell: CellId::new("cell-dark"),
        }
        .to_string();
        assert!(rendered.contains("cell-dark"), "{rendered}");

        let rendered = CellKernelError::NoHealthyCellFor {
            tenant: "ten_alpha".to_owned(),
            considered: 40,
        }
        .to_string();
        assert!(rendered.contains("ten_alpha"), "{rendered}");
        assert!(rendered.contains("40"), "{rendered}");
    }

    #[test]
    fn only_partial_execution_exposes_a_source_error() {
        use std::error::Error as _;

        for error in every_error() {
            match &error {
                CellKernelError::PartialPlanExecution { cause, .. } => {
                    let source = error.source().expect("the cause is reachable");
                    assert_eq!(source.to_string(), cause.to_string());
                }
                other => assert!(other.source().is_none(), "{other:?} has no cause to expose"),
            }
        }
    }
}
