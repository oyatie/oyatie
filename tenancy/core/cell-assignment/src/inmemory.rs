//! In-memory adapters for the two cell-assignment ports.
//!
//! These are the reference implementations the usecase layer is tested
//! against, and they are usable for local development and for a
//! single-process deployment whose assignment table fits in memory and
//! need not survive a restart. They are NOT the Citus or Valkey adapters
//! IP-008 calls for — see the crate-level Gaps note.
//!
//! The ports take `&self`, so state lives behind [`RefCell`]. Every
//! borrow goes through `try_borrow`/`try_borrow_mut` and maps a conflict
//! onto [`CellKernelError::PersistenceUnavailable`]: a re-entrant borrow
//! is exactly what "the store is busy" means here, and production code in
//! this crate does not panic.
//!
//! Neither adapter carries a fault injector. Arming an arbitrary error on
//! a type the service hands out through
//! [`crate::usecase::CellAssignmentService::repository`] would be a
//! tampering affordance with no audit trail, so the error paths of the
//! usecase layer are exercised by test doubles that implement the ports
//! directly (see `tests/assignment.rs`) rather than by a switch compiled
//! into the shipped adapter.

use core::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::domain::Placement;
use crate::kernel::{
    CellAssignmentRepository, CellHealth, CellHealthProbe, CellId, CellKernelError,
};

/// An assignment record store held in process memory.
#[derive(Debug, Default)]
pub struct InMemoryCellAssignmentRepository {
    assignments: RefCell<BTreeMap<String, CellId>>,
}

impl InMemoryCellAssignmentRepository {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed an assignment without going through the port.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the store is
    /// already borrowed.
    pub fn seed(&self, tenant: &str, cell: &CellId) -> Result<(), CellKernelError> {
        self.assignments
            .try_borrow_mut()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .insert(tenant.to_owned(), cell.clone());
        Ok(())
    }

    /// How many tenants currently have a recorded assignment.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the store is
    /// already borrowed.
    pub fn len(&self) -> Result<usize, CellKernelError> {
        Ok(self
            .assignments
            .try_borrow()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .len())
    }

    /// Whether no tenant has a recorded assignment.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the store is
    /// already borrowed.
    pub fn is_empty(&self) -> Result<bool, CellKernelError> {
        Ok(self.len()? == 0)
    }

    /// A copy of every recorded assignment.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the store is
    /// already borrowed.
    pub fn snapshot(&self) -> Result<BTreeMap<String, CellId>, CellKernelError> {
        Ok(self
            .assignments
            .try_borrow()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .clone())
    }

    /// The recorded assignments viewed as a [`Placement`], which is the
    /// input rebalance planning needs.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the store is
    /// already borrowed.
    pub fn placement(&self) -> Result<Placement, CellKernelError> {
        let mut placement = Placement::new();
        for (tenant, cell) in self.snapshot()? {
            placement.place(&cell, &tenant);
        }
        Ok(placement)
    }
}

impl CellAssignmentRepository for InMemoryCellAssignmentRepository {
    fn assigned_cell(&self, tenant: &str) -> Result<Option<CellId>, CellKernelError> {
        Ok(self
            .assignments
            .try_borrow()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .get(tenant)
            .cloned())
    }

    fn record_assignment(&self, tenant: &str, cell: &CellId) -> Result<(), CellKernelError> {
        self.assignments
            .try_borrow_mut()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .insert(tenant.to_owned(), cell.clone());
        Ok(())
    }

    fn forget_assignment(&self, tenant: &str) -> Result<bool, CellKernelError> {
        Ok(self
            .assignments
            .try_borrow_mut()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .remove(tenant)
            .is_some())
    }
}

/// A health probe backed by an in-process table.
///
/// # The unknown-cell posture is fail-closed
///
/// [`Default`] builds a probe that has *no* opinion about a cell it has
/// never been told about: probing an unlisted cell returns
/// [`CellKernelError::ProbeFailed`], not a health. "I have never
/// contacted this cell" is not evidence that it is healthy, and
/// answering `Healthy` there is how a typo'd id (`"cell-b "`), or a
/// decommissioned one, gets confirmed as a placement target and has a
/// live tenant written onto it.
///
/// [`InMemoryCellHealthProbe::with_default`] deliberately opts *out* of
/// that posture by naming a health to report for unlisted cells. It is a
/// fixture convenience and a way to model a probe with a known blanket
/// answer; it is not what `Default::default()` gives you.
#[derive(Debug)]
pub struct InMemoryCellHealthProbe {
    health: RefCell<BTreeMap<CellId, CellHealth>>,
    unreachable: RefCell<BTreeSet<CellId>>,
    /// `None` means an unlisted cell is unobservable rather than assumed.
    default_health: Option<CellHealth>,
}

impl Default for InMemoryCellHealthProbe {
    fn default() -> Self {
        Self::strict()
    }
}

impl InMemoryCellHealthProbe {
    /// A probe that treats every cell it has not been told about as
    /// unobservable. This is [`Default`].
    #[must_use]
    pub fn strict() -> Self {
        Self {
            health: RefCell::new(BTreeMap::new()),
            unreachable: RefCell::new(BTreeSet::new()),
            default_health: None,
        }
    }

    /// A probe reporting `default_health` for every unlisted cell.
    ///
    /// Passing [`CellHealth::Healthy`] here is a fail-open posture; see
    /// the type docs before doing it outside a fixture.
    #[must_use]
    pub fn with_default(default_health: CellHealth) -> Self {
        Self {
            default_health: Some(default_health),
            ..Self::strict()
        }
    }

    /// Record the health this probe will report for a cell.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the table is
    /// already borrowed.
    pub fn set_health(&self, cell: &CellId, health: CellHealth) -> Result<(), CellKernelError> {
        self.health
            .try_borrow_mut()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .insert(cell.clone(), health);
        Ok(())
    }

    /// Make a cell unobservable, so probing it fails rather than
    /// reporting a health.
    ///
    /// # Errors
    /// [`CellKernelError::PersistenceUnavailable`] if the table is
    /// already borrowed.
    pub fn set_unreachable(&self, cell: &CellId) -> Result<(), CellKernelError> {
        self.unreachable
            .try_borrow_mut()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .insert(cell.clone());
        Ok(())
    }
}

impl CellHealthProbe for InMemoryCellHealthProbe {
    fn probe(&self, cell: &CellId) -> Result<CellHealth, CellKernelError> {
        if self
            .unreachable
            .try_borrow()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .contains(cell)
        {
            return Err(CellKernelError::ProbeFailed { cell: cell.clone() });
        }
        self.health
            .try_borrow()
            .map_err(|_| CellKernelError::PersistenceUnavailable)?
            .get(cell)
            .copied()
            .or(self.default_health)
            .ok_or_else(|| CellKernelError::ProbeFailed { cell: cell.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_round_trips_and_overwrites_an_assignment() {
        let repository = InMemoryCellAssignmentRepository::new();
        assert_eq!(
            repository.assigned_cell("ten_one").expect("store is idle"),
            None
        );
        repository
            .record_assignment("ten_one", &CellId::new("cell-a"))
            .expect("store is idle");
        repository
            .record_assignment("ten_one", &CellId::new("cell-b"))
            .expect("store is idle");
        assert_eq!(
            repository.assigned_cell("ten_one").expect("store is idle"),
            Some(CellId::new("cell-b"))
        );
    }

    #[test]
    fn forgetting_an_assignment_shrinks_the_store_and_reports_whether_it_hit() {
        let repository = InMemoryCellAssignmentRepository::new();
        repository
            .record_assignment("ten_one", &CellId::new("cell-a"))
            .expect("store is idle");
        assert_eq!(repository.len().expect("store is idle"), 1);

        assert!(
            repository
                .forget_assignment("ten_one")
                .expect("store is idle"),
            "the row existed"
        );
        assert!(repository.is_empty().expect("store is idle"));
        assert_eq!(
            repository.assigned_cell("ten_one").expect("store is idle"),
            None
        );
        assert!(
            !repository
                .forget_assignment("ten_one")
                .expect("store is idle"),
            "forgetting twice is not an error, it just reports no hit"
        );
        assert!(
            !repository
                .forget_assignment("ten_never")
                .expect("store is idle")
        );
    }

    #[test]
    fn repository_projects_its_rows_as_a_placement() {
        let repository = InMemoryCellAssignmentRepository::new();
        repository
            .seed("ten_one", &CellId::new("cell-a"))
            .expect("store is idle");
        repository
            .seed("ten_two", &CellId::new("cell-a"))
            .expect("store is idle");
        repository
            .seed("ten_three", &CellId::new("cell-b"))
            .expect("store is idle");

        let placement = repository.placement().expect("store is idle");
        assert_eq!(placement.occupancy(), 3);
        assert_eq!(placement.load_of(&CellId::new("cell-a")), 2);
        assert_eq!(placement.cell_of("ten_three"), Some(CellId::new("cell-b")));
    }

    #[test]
    fn probe_distinguishes_unhealthy_from_unobservable() {
        let probe = InMemoryCellHealthProbe::default();
        probe
            .set_health(&CellId::new("cell-sick"), CellHealth::Unhealthy)
            .expect("table is idle");
        probe
            .set_health(&CellId::new("cell-ok"), CellHealth::Healthy)
            .expect("table is idle");
        probe
            .set_unreachable(&CellId::new("cell-dark"))
            .expect("table is idle");

        assert_eq!(
            probe.probe(&CellId::new("cell-sick")).expect("observable"),
            CellHealth::Unhealthy
        );
        assert_eq!(
            probe.probe(&CellId::new("cell-ok")).expect("observable"),
            CellHealth::Healthy
        );
        assert_eq!(
            probe
                .probe(&CellId::new("cell-dark"))
                .expect_err("an unobservable cell has no health"),
            CellKernelError::ProbeFailed {
                cell: CellId::new("cell-dark")
            }
        );
    }

    #[test]
    fn the_default_probe_refuses_to_vouch_for_a_cell_it_has_never_seen() {
        // The fail-open case this guards: a typo'd id reported as Healthy
        // would be confirmed and written as a live tenant's home.
        let probe = InMemoryCellHealthProbe::default();
        for unknown in ["cell-b ", "cell-old", "", "cell-unlisted"] {
            assert_eq!(
                probe
                    .probe(&CellId::new(unknown))
                    .expect_err("an unheard-of cell has no observed health"),
                CellKernelError::ProbeFailed {
                    cell: CellId::new(unknown)
                },
                "probe vouched for {unknown:?}"
            );
        }
    }

    #[test]
    fn probe_default_posture_is_configurable() {
        let probe = InMemoryCellHealthProbe::with_default(CellHealth::Unhealthy);
        assert_eq!(
            probe
                .probe(&CellId::new("cell-unlisted"))
                .expect("unlisted cells take the default"),
            CellHealth::Unhealthy
        );
        // An explicit entry still wins over the blanket answer, and an
        // unreachable cell still fails rather than taking it.
        probe
            .set_health(&CellId::new("cell-ok"), CellHealth::Healthy)
            .expect("table is idle");
        probe
            .set_unreachable(&CellId::new("cell-dark"))
            .expect("table is idle");
        assert_eq!(
            probe.probe(&CellId::new("cell-ok")).expect("observable"),
            CellHealth::Healthy
        );
        assert_eq!(
            probe
                .probe(&CellId::new("cell-dark"))
                .expect_err("unreachable beats the default"),
            CellKernelError::ProbeFailed {
                cell: CellId::new("cell-dark")
            }
        );
    }
}
