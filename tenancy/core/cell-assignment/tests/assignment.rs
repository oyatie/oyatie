//! End-to-end cell-assignment behaviour over the in-memory ports:
//! idempotency, probe contradiction, port-failure mapping, shard-sticky
//! placement, offboarding, and integrity-verified rebalance execution.
//!
//! The fault-injection cases drive a test double defined here rather than
//! a switch compiled into the shipped adapter — arming an arbitrary error
//! on a type the service hands out publicly would be a tampering
//! affordance in production, so the port trait is implemented locally
//! instead.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use core::cell::Cell;
use std::collections::BTreeMap;

use tenancy_cell_assignment::{
    AssignmentOutcome, CellAssignmentRepository, CellAssignmentService, CellCandidate, CellHealth,
    CellId, CellKernelError, InMemoryCellAssignmentRepository, InMemoryCellHealthProbe, Placement,
    RebalancePlan, RebalanceTask, derive_shard_key,
};

type Service = CellAssignmentService<InMemoryCellAssignmentRepository, InMemoryCellHealthProbe>;

/// A probe that has observed exactly the cells it is told about. The
/// default posture is fail-closed, so every cell a test expects to be
/// confirmable must be listed.
fn probe_for(observed: &[(&str, CellHealth)]) -> InMemoryCellHealthProbe {
    let probe = InMemoryCellHealthProbe::default();
    for (id, health) in observed {
        probe
            .set_health(&CellId::new(*id), *health)
            .expect("table is idle");
    }
    probe
}

fn service_for(observed: &[(&str, CellHealth)]) -> Service {
    CellAssignmentService::new(InMemoryCellAssignmentRepository::new(), probe_for(observed))
}

fn candidate(id: &str, load: u32, health: CellHealth) -> CellCandidate {
    CellCandidate::new(CellId::new(id), load, health).expect("fixture id and load are legal")
}

/// A record store that can be made to fail on a chosen port call.
#[derive(Debug, Default)]
struct FaultyRepository {
    inner: InMemoryCellAssignmentRepository,
    fail_next_read: Cell<bool>,
    /// Successful writes remaining before writes start failing.
    writes_before_failure: Cell<Option<usize>>,
}

impl FaultyRepository {
    fn failing_read() -> Self {
        let repository = Self::default();
        repository.fail_next_read.set(true);
        repository
    }

    fn failing_write_after(successes: usize) -> Self {
        let repository = Self::default();
        repository.writes_before_failure.set(Some(successes));
        repository
    }

    fn snapshot(&self) -> BTreeMap<String, CellId> {
        self.inner.snapshot().expect("store is idle")
    }
}

impl CellAssignmentRepository for FaultyRepository {
    fn assigned_cell(&self, tenant: &str) -> Result<Option<CellId>, CellKernelError> {
        if self.fail_next_read.replace(false) {
            return Err(CellKernelError::PersistenceUnavailable);
        }
        self.inner.assigned_cell(tenant)
    }

    fn record_assignment(&self, tenant: &str, cell: &CellId) -> Result<(), CellKernelError> {
        match self.writes_before_failure.get() {
            Some(0) => return Err(CellKernelError::PersistenceUnavailable),
            Some(remaining) => self.writes_before_failure.set(Some(remaining - 1)),
            None => {}
        }
        self.inner.record_assignment(tenant, cell)
    }

    fn forget_assignment(&self, tenant: &str) -> Result<bool, CellKernelError> {
        self.inner.forget_assignment(tenant)
    }
}

#[test]
fn assignment_records_the_least_loaded_healthy_cell() {
    let service = service_for(&[
        ("cell-a", CellHealth::Healthy),
        ("cell-b", CellHealth::Healthy),
        ("cell-c", CellHealth::Unhealthy),
    ]);
    let candidates = vec![
        candidate("cell-a", 800, CellHealth::Healthy),
        candidate("cell-b", 120, CellHealth::Healthy),
        candidate("cell-c", 5, CellHealth::Unhealthy),
    ];

    let outcome = service
        .assign("ten_alpha", &candidates)
        .expect("a healthy cell exists");
    assert_eq!(outcome, AssignmentOutcome::Assigned(CellId::new("cell-b")));
    assert!(outcome.is_new());
    assert_eq!(
        service
            .repository()
            .assigned_cell("ten_alpha")
            .expect("store is idle"),
        Some(CellId::new("cell-b"))
    );
}

#[test]
fn assignment_is_idempotent_and_does_not_move_a_live_tenant() {
    let service = service_for(&[
        ("cell-a", CellHealth::Healthy),
        ("cell-b", CellHealth::Healthy),
    ]);
    let candidates = vec![
        candidate("cell-a", 900, CellHealth::Healthy),
        candidate("cell-b", 0, CellHealth::Healthy),
    ];
    service
        .repository()
        .seed("ten_alpha", &CellId::new("cell-a"))
        .expect("store is idle");

    let outcome = service
        .assign("ten_alpha", &candidates)
        .expect("an existing assignment is honoured");
    // cell-b is far quieter, but a live tenant is not relocated by an
    // assignment call — that is what rebalance planning is for.
    assert_eq!(
        outcome,
        AssignmentOutcome::AlreadyAssigned(CellId::new("cell-a"))
    );
    assert!(!outcome.is_new());
}

#[test]
fn releasing_a_tenant_lets_a_reissued_id_be_placed_afresh() {
    // Without offboarding, a re-created tenant id short-circuits onto the
    // row of its namesake — pinned, silently, to a retired cell.
    let service = service_for(&[
        ("cell-retired", CellHealth::Unhealthy),
        ("cell-new", CellHealth::Healthy),
    ]);
    service
        .repository()
        .seed("ten_recycled", &CellId::new("cell-retired"))
        .expect("store is idle");
    let candidates = vec![candidate("cell-new", 0, CellHealth::Healthy)];

    assert_eq!(
        service
            .assign("ten_recycled", &candidates)
            .expect("the stale row wins while it exists"),
        AssignmentOutcome::AlreadyAssigned(CellId::new("cell-retired"))
    );

    assert!(
        service.release("ten_recycled").expect("store is idle"),
        "a row existed to release"
    );
    assert!(
        !service.release("ten_recycled").expect("store is idle"),
        "releasing again is a no-op, not an error"
    );
    assert!(
        service
            .repository()
            .is_empty()
            .expect("the store shrank, it did not just tombstone")
    );

    assert_eq!(
        service
            .assign("ten_recycled", &candidates)
            .expect("a released tenant is placeable again"),
        AssignmentOutcome::Assigned(CellId::new("cell-new"))
    );
}

#[test]
fn a_probe_that_contradicts_the_candidate_falls_through_to_the_next_cell() {
    let service = service_for(&[
        ("cell-quiet", CellHealth::Unhealthy),
        ("cell-busy", CellHealth::Healthy),
    ]);
    let candidates = vec![
        candidate("cell-quiet", 10, CellHealth::Healthy),
        candidate("cell-busy", 700, CellHealth::Healthy),
    ];

    let outcome = service
        .assign("ten_alpha", &candidates)
        .expect("the busy cell still qualifies");
    assert_eq!(
        outcome,
        AssignmentOutcome::Assigned(CellId::new("cell-busy"))
    );
}

#[test]
fn assignment_fails_when_every_candidate_is_contradicted() {
    let service = service_for(&[
        ("cell-a", CellHealth::Degraded),
        ("cell-b", CellHealth::Degraded),
    ]);
    let candidates = vec![
        candidate("cell-a", 10, CellHealth::Healthy),
        candidate("cell-b", 20, CellHealth::Healthy),
    ];

    let error = service
        .assign("ten_alpha", &candidates)
        .expect_err("nothing survives confirmation");
    // The error names who could not be placed and how big the roster was.
    assert_eq!(
        error,
        CellKernelError::NoHealthyCellFor {
            tenant: "ten_alpha".to_owned(),
            considered: 2,
        }
    );
    assert_eq!(
        service
            .repository()
            .assigned_cell("ten_alpha")
            .expect("store is idle"),
        None
    );
}

#[test]
fn a_cell_the_probe_has_never_observed_is_never_recorded_as_a_home() {
    // Two shapes of the same mistake. A syntactically illegal id cannot
    // even become a candidate...
    assert_eq!(
        CellCandidate::new(CellId::new("cell-b "), 0, CellHealth::Healthy)
            .expect_err("a trailing space is not a legal cell id"),
        CellKernelError::MalformedCellId {
            cell: "cell-b ".to_owned()
        }
    );

    // ...and a well-formed id for a cell nobody has ever contacted (a
    // decommissioned one, say) fails confirmation rather than being
    // rubber-stamped because it reports load 0.
    let service = service_for(&[("cell-live", CellHealth::Healthy)]);
    let candidates = vec![
        candidate("cell-old", 0, CellHealth::Healthy),
        candidate("cell-live", 900, CellHealth::Healthy),
    ];
    let error = service
        .assign("ten_alpha", &candidates)
        .expect_err("an unobserved cell has no health to confirm");
    // And the error names which cell went dark, so the operator does not
    // have to re-derive it from a forty-cell roster.
    assert_eq!(
        error,
        CellKernelError::ProbeFailed {
            cell: CellId::new("cell-old")
        }
    );
    assert!(
        service.repository().is_empty().expect("store is idle"),
        "nothing may be written on the strength of a cell nobody probed"
    );
}

#[test]
fn an_unobservable_cell_fails_the_call_rather_than_guessing() {
    let service = service_for(&[
        ("cell-dark", CellHealth::Healthy),
        ("cell-lit", CellHealth::Healthy),
    ]);
    service
        .probe()
        .set_unreachable(&CellId::new("cell-dark"))
        .expect("table is idle");
    let candidates = vec![
        candidate("cell-dark", 0, CellHealth::Healthy),
        candidate("cell-lit", 500, CellHealth::Healthy),
    ];

    let error = service
        .assign("ten_alpha", &candidates)
        .expect_err("an unobservable cell is not the same as an unhealthy one");
    assert_eq!(
        error,
        CellKernelError::ProbeFailed {
            cell: CellId::new("cell-dark")
        }
    );
}

#[test]
fn a_record_store_failure_maps_onto_the_kernel_error() {
    let service = CellAssignmentService::new(
        FaultyRepository::failing_read(),
        probe_for(&[("cell-a", CellHealth::Healthy)]),
    );

    let error = service
        .assign("ten_alpha", &[candidate("cell-a", 0, CellHealth::Healthy)])
        .expect_err("the read of the existing assignment fails");
    assert_eq!(error, CellKernelError::PersistenceUnavailable);
}

#[test]
fn shard_assignment_is_sticky_for_a_stable_healthy_set() {
    let service = service_for(&[
        ("cell-a", CellHealth::Healthy),
        ("cell-b", CellHealth::Healthy),
        ("cell-c", CellHealth::Healthy),
    ]);
    let candidates = vec![
        candidate("cell-a", 900, CellHealth::Healthy),
        candidate("cell-b", 0, CellHealth::Healthy),
        candidate("cell-c", 0, CellHealth::Healthy),
    ];

    let (key, outcome) = service
        .assign_by_shard("ten_alpha", 128, &candidates)
        .expect("healthy cells exist");
    assert_eq!(
        key,
        derive_shard_key("ten_alpha", 128).expect("128 shards is legal")
    );
    let placed = outcome.cell().clone();

    // A replay reports the recorded cell and the same shard key.
    let (replay_key, replay) = service
        .assign_by_shard("ten_alpha", 128, &candidates)
        .expect("healthy cells exist");
    assert_eq!(replay_key, key);
    assert_eq!(replay, AssignmentOutcome::AlreadyAssigned(placed.clone()));

    // And the recorded home is what survives a membership change, which
    // is precisely what recomputing the shard mapping would NOT give: a
    // shrunken healthy set remaps the key, but the tenant does not move.
    let shrunk = vec![
        candidate("cell-b", 0, CellHealth::Healthy),
        candidate("cell-c", 0, CellHealth::Healthy),
    ];
    let (_, after_change) = service
        .assign_by_shard("ten_alpha", 128, &shrunk)
        .expect("the recorded assignment answers");
    assert_eq!(after_change, AssignmentOutcome::AlreadyAssigned(placed));
}

#[test]
fn shard_assignment_rejects_an_empty_shard_space_before_touching_the_store() {
    let service = service_for(&[("cell-a", CellHealth::Healthy)]);
    let error = service
        .assign_by_shard(
            "ten_alpha",
            0,
            &[candidate("cell-a", 0, CellHealth::Healthy)],
        )
        .expect_err("zero shards has no legal key");
    assert_eq!(error, CellKernelError::ZeroShardCount);
    assert_eq!(
        service
            .repository()
            .assigned_cell("ten_alpha")
            .expect("store is idle"),
        None
    );
}

#[test]
fn planning_uses_live_probe_health_over_the_declared_candidate_health() {
    let service = service_for(&[
        ("cell-sick", CellHealth::Unhealthy),
        ("cell-ok", CellHealth::Healthy),
    ]);

    let mut placement = Placement::new();
    placement.place(&CellId::new("cell-sick"), "ten_one");
    placement.place(&CellId::new("cell-sick"), "ten_two");
    placement.place(&CellId::new("cell-ok"), "ten_three");

    // The caller believes cell-sick is fine; the probe knows better.
    let candidates = vec![
        candidate("cell-sick", 100, CellHealth::Healthy),
        candidate("cell-ok", 100, CellHealth::Healthy),
    ];

    let plan = service
        .plan_rebalance(&placement, &candidates)
        .expect("cell-ok can absorb the drained tenants");
    assert_eq!(plan.len(), 2);

    service
        .execute_plan(&mut placement, &plan)
        .expect("the plan conserves the tenant set");
    assert_eq!(placement.load_of(&CellId::new("cell-sick")), 0);
    assert_eq!(placement.load_of(&CellId::new("cell-ok")), 3);

    // Executing a plan writes the new home of every moved tenant.
    let recorded = service.repository().snapshot().expect("store is idle");
    assert_eq!(recorded.len(), 2);
    assert!(
        recorded
            .values()
            .all(|cell| *cell == CellId::new("cell-ok"))
    );
}

#[test]
fn a_degraded_cell_is_not_evacuated_end_to_end() {
    // The blast-radius guarantee, through the service: the probe reports
    // a transient degradation and NOTHING moves.
    let service = service_for(&[
        ("cell-slow", CellHealth::Degraded),
        ("cell-ok", CellHealth::Healthy),
    ]);
    let mut placement = Placement::new();
    for index in 0..5 {
        placement.place(&CellId::new("cell-slow"), &format!("ten_{index}"));
    }
    placement.place(&CellId::new("cell-ok"), "ten_ok");
    let before = placement.clone();
    let candidates = vec![
        candidate("cell-slow", 900, CellHealth::Healthy),
        candidate("cell-ok", 100, CellHealth::Healthy),
    ];

    let plan = service
        .plan_rebalance(&placement, &candidates)
        .expect("a degraded cell is plannable");
    assert!(plan.is_empty(), "nothing moves: {:?}", plan.tasks());

    service
        .execute_plan(&mut placement, &plan)
        .expect("an empty plan executes trivially");
    assert_eq!(placement, before);
    assert_eq!(placement.load_of(&CellId::new("cell-slow")), 5);
    assert!(service.repository().is_empty().expect("store is idle"));
}

#[test]
fn planning_refuses_when_the_placement_holds_a_cell_the_roster_omits() {
    // A paginated or partially-failed cell listing must not read as a
    // drain order for the cells it forgot to mention.
    let service = service_for(&[
        ("cell-a", CellHealth::Healthy),
        ("cell-b", CellHealth::Healthy),
    ]);
    let mut placement = Placement::new();
    placement.place(&CellId::new("cell-a"), "ten_one");
    placement.place(&CellId::new("cell-b"), "ten_two");
    for index in 0..8 {
        placement.place(&CellId::new("cell-c"), &format!("ten_c_{index}"));
    }
    let first_page = vec![
        candidate("cell-a", 100, CellHealth::Healthy),
        candidate("cell-b", 100, CellHealth::Healthy),
    ];

    let error = service
        .plan_rebalance(&placement, &first_page)
        .expect_err("cell-c was never probed, so its health is unknown");
    assert_eq!(
        error,
        CellKernelError::PlacementCellNotInRoster {
            cell: CellId::new("cell-c")
        }
    );
    assert!(service.repository().is_empty().expect("store is idle"));
}

#[test]
fn a_partial_plan_execution_names_the_task_that_failed() {
    let service = CellAssignmentService::new(
        FaultyRepository::failing_write_after(1),
        probe_for(&[
            ("cell-a", CellHealth::Healthy),
            ("cell-b", CellHealth::Healthy),
        ]),
    );
    let mut placement = Placement::new();
    for index in 0..6 {
        placement.place(&CellId::new("cell-a"), &format!("ten_{index}"));
    }
    let before = placement.clone();
    let candidates = vec![
        candidate("cell-a", 900, CellHealth::Healthy),
        candidate("cell-b", 0, CellHealth::Healthy),
    ];

    let plan = service
        .plan_rebalance(&placement, &candidates)
        .expect("levelling succeeds");
    assert_eq!(plan.len(), 3);

    let error = service
        .execute_plan(&mut placement, &plan)
        .expect_err("the second write fails");
    assert_eq!(
        error,
        CellKernelError::PartialPlanExecution {
            committed: 1,
            total: 3,
            cause: Box::new(CellKernelError::PersistenceUnavailable),
        }
    );

    // The index is the reconciliation key: task 0 is durable, tasks 1..3
    // are not, and the caller's placement still describes the old world.
    assert_eq!(placement, before, "the caller's placement is untouched");
    let recorded = service.repository().snapshot();
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded.get(&plan.tasks()[0].tenant),
        Some(&plan.tasks()[0].to_cell)
    );
    for task in &plan.tasks()[1..] {
        assert!(
            !recorded.contains_key(&task.tenant),
            "{} must not be recorded",
            task.tenant
        );
    }
}

#[test]
fn executing_a_lossy_plan_leaves_the_store_and_the_placement_untouched() {
    let service = service_for(&[]);
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
    .expect("the move itself is legal");

    // The world drifted after the plan was built: the tenant is already
    // gone from cell-a, so replaying the plan would lose it.
    let mut drifted = Placement::new();
    drifted.register_cell(&CellId::new("cell-a"));
    drifted.register_cell(&CellId::new("cell-b"));
    let before = drifted.clone();

    let error = service
        .execute_plan(&mut drifted, &plan)
        .expect_err("the move no longer applies");
    assert_eq!(error, CellKernelError::TenantNotInSourceCell);
    assert_eq!(drifted, before);
    assert!(service.repository().is_empty().expect("store is idle"));
}

#[test]
fn refreshing_health_reports_the_probe_verdict_for_every_candidate() {
    let service = service_for(&[
        ("cell-a", CellHealth::Healthy),
        ("cell-b", CellHealth::Degraded),
    ]);

    let refreshed = service
        .refresh_health(&[
            candidate("cell-a", 10, CellHealth::Unhealthy),
            candidate("cell-b", 20, CellHealth::Healthy),
        ])
        .expect("both cells are observable");

    assert_eq!(refreshed[0].health, CellHealth::Healthy);
    assert_eq!(refreshed[1].health, CellHealth::Degraded);
    // Load is the caller's observation and is carried through untouched.
    assert_eq!(refreshed[0].load_permille, 10);
    assert_eq!(refreshed[1].load_permille, 20);
}

#[test]
fn a_full_lifecycle_assigns_then_rebalances_without_losing_a_tenant() {
    let service = service_for(&[
        ("cell-a", CellHealth::Healthy),
        ("cell-b", CellHealth::Healthy),
    ]);
    let candidates = vec![
        candidate("cell-a", 0, CellHealth::Healthy),
        candidate("cell-b", 1000, CellHealth::Healthy),
    ];

    // Six tenants all land on cell-a, because cell-b is saturated.
    for index in 0..6 {
        let outcome = service
            .assign(&format!("ten_{index}"), &candidates)
            .expect("cell-a is healthy");
        assert_eq!(outcome, AssignmentOutcome::Assigned(CellId::new("cell-a")));
    }

    let mut placement = service.repository().placement().expect("store is idle");
    let before = placement.checksum();
    assert_eq!(before.occupancy, 6);

    // Once load is reported evenly, planning levels them out.
    let levelled = vec![
        candidate("cell-a", 600, CellHealth::Healthy),
        candidate("cell-b", 0, CellHealth::Healthy),
    ];
    let plan = service
        .plan_rebalance(&placement, &levelled)
        .expect("levelling succeeds");
    service
        .execute_plan(&mut placement, &plan)
        .expect("the plan conserves the tenant set");

    assert_eq!(placement.checksum(), before);
    assert_eq!(placement.load_of(&CellId::new("cell-a")), 3);
    assert_eq!(placement.load_of(&CellId::new("cell-b")), 3);
    assert_eq!(
        service.repository().placement().expect("store is idle"),
        placement
    );
}
