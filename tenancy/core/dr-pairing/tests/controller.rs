//! End-to-end DR-pairing controller behaviour: residency enforcement, the
//! failover state machine, optimistic concurrency under a stale writer,
//! probe-failure handling, and the audit trail a promotion leaves behind.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tenancy_dr_pairing::inmemory::{
    InMemoryCellCatalog, InMemoryDrPairRepository, RecordingEventSink, StaticDrSloProbe,
    StaticResidencyPolicy,
};
use tenancy_dr_pairing::{
    DrCellCandidate, DrPair, DrPairEventKind, DrPairRepository, DrPairingController,
    DrPairingError, DrSloProbe, FailoverCommand, PairAssignment, PairState, PromotionDecision,
    reason,
};

const TENANT: &str = "ten_alpha";
const HOME: &str = "cell-eu-1";
const DR: &str = "cell-eu-2";
const FOREIGN: &str = "cell-us-1";

fn cell(
    cell_id: &str,
    jurisdiction: &str,
    fault_domain: &str,
    load_percent: u8,
) -> DrCellCandidate {
    DrCellCandidate {
        cell_id: cell_id.to_owned(),
        jurisdiction: jurisdiction.to_owned(),
        fault_domain: fault_domain.to_owned(),
        healthy: true,
        load_percent,
    }
}

fn catalog() -> InMemoryCellCatalog {
    InMemoryCellCatalog::new()
        .with_cell(cell(HOME, "eu", "dc-1", 20))
        .expect("home cell is well formed")
        .with_cell(cell(DR, "eu", "dc-2", 30))
        .expect("dr cell is well formed")
        .with_cell(cell("cell-eu-3", "eu", "dc-1", 5))
        .expect("same-domain cell is well formed")
        .with_cell(cell(FOREIGN, "us", "dc-9", 1))
        .expect("foreign cell is well formed")
}

fn assignment(expected_version: Option<u32>) -> PairAssignment {
    PairAssignment {
        tenant_id: TENANT.to_owned(),
        home_cell: HOME.to_owned(),
        jurisdiction: "eu".to_owned(),
        home_fault_domain: "dc-1".to_owned(),
        rpo_seconds: 30,
        rto_seconds: 300,
        expected_version,
        at_millis: 1_700_000_000_000,
        correlation_id: "corr-1".to_owned(),
    }
}

fn command(expected_version: u32) -> FailoverCommand {
    FailoverCommand {
        tenant_id: TENANT.to_owned(),
        expected_version,
        at_millis: 1_700_000_001_000,
        correlation_id: "corr-2".to_owned(),
    }
}

/// The five ports a controller needs, owned together so tests can borrow
/// them all at once.
struct Rig {
    repo: InMemoryDrPairRepository,
    probe: StaticDrSloProbe,
    catalog: InMemoryCellCatalog,
    policy: StaticResidencyPolicy,
    sink: RecordingEventSink,
}

impl Rig {
    fn new(probe: StaticDrSloProbe, policy: StaticResidencyPolicy) -> Self {
        Self {
            repo: InMemoryDrPairRepository::new(),
            probe,
            catalog: catalog(),
            policy,
            sink: RecordingEventSink::new(),
        }
    }

    /// Every cell healthy on the side it is actually asked about: the DR
    /// candidates as DR replicas, the home cell as a home primary. The home
    /// cell is deliberately absent from the DR-health sets, so any code
    /// that asks the DR question about it fails the probe instead of
    /// quietly getting an answer.
    fn healthy() -> Self {
        Self::new(
            StaticDrSloProbe::new()
                .with_healthy(DR)
                .with_healthy("cell-eu-3")
                .with_home_healthy(HOME),
            StaticResidencyPolicy::permitting(),
        )
    }

    fn controller(
        &self,
    ) -> DrPairingController<
        '_,
        InMemoryDrPairRepository,
        StaticDrSloProbe,
        InMemoryCellCatalog,
        StaticResidencyPolicy,
        RecordingEventSink,
    > {
        DrPairingController::new(
            &self.repo,
            &self.probe,
            &self.catalog,
            &self.policy,
            &self.sink,
        )
    }
}

/// Assign, then activate, leaving the pair at `HomeActive` version 2.
fn paired_and_active(rig: &Rig) -> DrPair {
    let controller = rig.controller();
    let assigned = controller
        .assign_pair_to(&assignment(None), DR)
        .expect("first assignment succeeds");
    controller
        .activate_pair(&command(assigned.pair_version))
        .expect("activation succeeds")
}

#[test]
fn a_cross_jurisdiction_pair_is_refused_by_the_public_write_path() {
    let rig = Rig::healthy();
    let error = rig
        .controller()
        .assign_pair_to(&assignment(None), FOREIGN)
        .expect_err("a US DR cell for an EU home cell is a residency violation");
    assert_eq!(error, DrPairingError::JurisdictionMismatch);
    assert_eq!(rig.repo.is_empty(), Ok(true), "nothing may be persisted");
    assert!(
        rig.sink.events().expect("sink readable").is_empty(),
        "a refused assignment emits no audit event"
    );
}

#[test]
fn a_dr_cell_that_is_the_home_cell_is_refused() {
    let rig = Rig::healthy();
    let error = rig
        .controller()
        .assign_pair_to(&assignment(None), HOME)
        .expect_err("a DR cell that is the home cell is not a DR cell");
    assert_eq!(error, DrPairingError::HomeCellIsDrCell);
    assert_eq!(rig.repo.is_empty(), Ok(true));
}

#[test]
fn selection_never_offers_a_foreign_or_home_cell_and_prefers_fault_separation() {
    let rig = Rig::healthy();
    let assigned = rig
        .controller()
        .assign_pair(&assignment(None))
        .expect("an eligible EU cell exists");
    // cell-eu-3 is less loaded but shares dc-1 with the home cell;
    // cell-eu-2 is in dc-2 and wins on fault-domain separation.
    assert_eq!(assigned.dr_cell, DR);
    assert_eq!(assigned.jurisdiction, "eu");
    assert_eq!(assigned.state, PairState::Planned);
    assert_eq!(assigned.pair_version, 1);
}

#[test]
fn a_residency_policy_refusal_blocks_assignment_even_inside_one_jurisdiction() {
    let rig = Rig::new(
        StaticDrSloProbe::new().with_healthy(DR),
        StaticResidencyPolicy::refusing(),
    );
    let error = rig
        .controller()
        .assign_pair_to(&assignment(None), DR)
        .expect_err("same jurisdiction is necessary, not sufficient");
    assert_eq!(error, DrPairingError::ResidencyPolicyDenied);
}

#[test]
fn promotion_is_refused_until_the_pair_is_activated() {
    let rig = Rig::healthy();
    let controller = rig.controller();
    let assigned = controller
        .assign_pair_to(&assignment(None), DR)
        .expect("assignment succeeds");
    assert_eq!(
        controller.assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_ACTIVATED
        })
    );
    let error = controller
        .promote(&command(assigned.pair_version))
        .expect_err("a planned pair does not promote");
    assert_eq!(
        error,
        DrPairingError::PromotionBlocked {
            reason_code: reason::PAIR_NOT_ACTIVATED
        }
    );
}

#[test]
fn a_full_failover_and_failback_cycle_bumps_the_version_every_step() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    assert_eq!(active.state, PairState::HomeActive);
    assert_eq!(active.pair_version, 2);

    let controller = rig.controller();
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("a healthy activated pair promotes");
    assert_eq!(promoted.state, PairState::DrActive);
    assert_eq!(promoted.pair_version, 3);
    assert_eq!(promoted.serving_cell(), DR);

    let restored = controller
        .restore(&command(promoted.pair_version))
        .expect("failback succeeds when home is healthy");
    assert_eq!(restored.state, PairState::HomeActive);
    assert_eq!(restored.pair_version, 4);
    assert_eq!(restored.serving_cell(), HOME);
}

#[test]
fn a_stale_promotion_write_is_refused_instead_of_clobbering_a_newer_pair() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    let controller = rig.controller();

    // Two controllers both read version 2 and both decide to promote.
    let first = controller
        .promote(&command(active.pair_version))
        .expect("the first promotion wins");
    assert_eq!(first.pair_version, 3);

    let second = controller
        .promote(&command(active.pair_version))
        .expect_err("the second promotion carries a stale version");
    assert_eq!(second, DrPairingError::StalePairVersion);

    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored.pair_version, 3, "the winner's write survives");
    assert_eq!(stored.state, PairState::DrActive);
}

#[test]
fn a_stale_reassignment_is_refused_and_leaves_the_stored_pair_untouched() {
    let rig = Rig::healthy();
    let assigned = rig
        .controller()
        .assign_pair_to(&assignment(None), DR)
        .expect("first assignment succeeds");
    // A writer that still believes no pair exists must not overwrite one.
    let error = rig
        .controller()
        .assign_pair_to(&assignment(None), "cell-eu-3")
        .expect_err("expected_version None no longer matches");
    assert_eq!(error, DrPairingError::StalePairVersion);
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored, assigned);
}

#[test]
fn a_probe_failure_fails_closed_as_an_error_not_a_block() {
    let rig = Rig::new(
        StaticDrSloProbe::new()
            .with_home_healthy(HOME)
            .with_probe_failure(DR),
        StaticResidencyPolicy::permitting(),
    );
    let active = paired_and_active(&rig);
    let controller = rig.controller();
    assert_eq!(
        controller.assess_promotion(TENANT),
        Err(DrPairingError::SloProbeFailed),
        "an unanswerable probe is neither healthy nor a plain block"
    );
    assert_eq!(
        controller.promote(&command(active.pair_version)),
        Err(DrPairingError::SloProbeFailed)
    );
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored.state, PairState::HomeActive, "no promotion happened");
}

#[test]
fn an_unhealthy_replica_and_a_failed_probe_carry_different_outcomes() {
    let unhealthy = Rig::new(
        StaticDrSloProbe::new()
            .with_home_healthy(HOME)
            .with_unhealthy(DR),
        StaticResidencyPolicy::permitting(),
    );
    paired_and_active(&unhealthy);
    assert_eq!(
        unhealthy.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::DR_REPLICA_UNHEALTHY
        })
    );

    let failing = Rig::new(
        StaticDrSloProbe::new()
            .with_home_healthy(HOME)
            .with_probe_failure(DR),
        StaticResidencyPolicy::permitting(),
    );
    paired_and_active(&failing);
    assert_eq!(
        failing.controller().assess_promotion(TENANT),
        Err(DrPairingError::SloProbeFailed)
    );
}

#[test]
fn each_blocking_condition_yields_its_own_distinct_reason_code() {
    // Missing pair.
    let missing = Rig::healthy();
    assert_eq!(
        missing.controller().assess_promotion("ten_ghost"),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_FOUND
        })
    );

    // Planned, never activated.
    let planned = Rig::healthy();
    planned
        .controller()
        .assign_pair_to(&assignment(None), DR)
        .expect("assignment succeeds");
    assert_eq!(
        planned.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_ACTIVATED
        })
    );

    // Already promoted.
    let promoted = Rig::healthy();
    let active = paired_and_active(&promoted);
    promoted
        .controller()
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    assert_eq!(
        promoted.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::ALREADY_PROMOTED
        })
    );

    // Unhealthy DR replica.
    let sick = Rig::new(
        StaticDrSloProbe::new()
            .with_home_healthy(HOME)
            .with_unhealthy(DR),
        StaticResidencyPolicy::permitting(),
    );
    paired_and_active(&sick);
    assert_eq!(
        sick.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::DR_REPLICA_UNHEALTHY
        })
    );

    // Residency policy withdrawn after assignment.
    let denied = Rig::healthy();
    paired_and_active(&denied);
    let denied = Rig {
        policy: StaticResidencyPolicy::refusing(),
        ..denied
    };
    assert_eq!(
        denied.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::RESIDENCY_POLICY_DENIED
        })
    );

    // Degenerate stored row from an older writer.
    let degenerate = Rig::healthy();
    degenerate
        .repo
        .seed_unchecked(&DrPair {
            tenant_id: TENANT.to_owned(),
            home_cell: HOME.to_owned(),
            dr_cell: HOME.to_owned(),
            jurisdiction: "eu".to_owned(),
            pair_version: 9,
            state: PairState::HomeActive,
            rpo_seconds: 30,
            rto_seconds: 300,
        })
        .expect("seeding a legacy row succeeds");
    assert_eq!(
        degenerate.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::DEGENERATE_PAIR
        })
    );
}

#[test]
fn a_cell_that_left_the_catalog_blocks_promotion_rather_than_being_assumed() {
    let rig = Rig::healthy();
    paired_and_active(&rig);
    let rig = Rig {
        catalog: InMemoryCellCatalog::new()
            .with_cell(cell(HOME, "eu", "dc-1", 20))
            .expect("home cell is well formed"),
        ..rig
    };
    assert_eq!(
        rig.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::CELL_UNKNOWN
        })
    );
}

#[test]
fn a_cell_that_changed_jurisdiction_blocks_promotion_as_drift() {
    let rig = Rig::healthy();
    paired_and_active(&rig);
    let rig = Rig {
        catalog: InMemoryCellCatalog::new()
            .with_cell(cell(HOME, "eu", "dc-1", 20))
            .expect("home cell is well formed")
            // The DR cell was re-homed into another jurisdiction.
            .with_cell(cell(DR, "us", "dc-2", 30))
            .expect("dr cell is well formed"),
        ..rig
    };
    assert_eq!(
        rig.controller().assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::JURISDICTION_DRIFT
        })
    );
}

#[test]
fn failback_is_refused_when_the_home_replica_is_unhealthy() {
    let rig = Rig::new(
        StaticDrSloProbe::new()
            .with_home_unhealthy(HOME)
            .with_healthy(DR),
        StaticResidencyPolicy::permitting(),
    );
    let active = paired_and_active(&rig);
    let controller = rig.controller();
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("promotion succeeds while home is sick");
    let error = controller
        .restore(&command(promoted.pair_version))
        .expect_err("failing back onto a sick home cell is refused");
    assert_eq!(
        error,
        DrPairingError::PromotionBlocked {
            reason_code: reason::HOME_REPLICA_UNHEALTHY
        }
    );
}

#[test]
fn illegal_state_machine_edges_are_refused() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    let controller = rig.controller();

    // Activating an already-active pair is not an edge.
    assert_eq!(
        controller.activate_pair(&command(active.pair_version)),
        Err(DrPairingError::IllegalTransition)
    );
    // Failing back a pair that never failed over is not an edge.
    assert_eq!(
        controller.restore(&command(active.pair_version)),
        Err(DrPairingError::IllegalTransition)
    );
    // Re-assigning mid-failover is not an edge.
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    let mut reassign = assignment(Some(promoted.pair_version));
    reassign.correlation_id = "corr-3".to_owned();
    assert_eq!(
        controller.assign_pair_to(&reassign, "cell-eu-3"),
        Err(DrPairingError::IllegalTransition)
    );
}

#[test]
fn a_missing_pair_is_an_error_for_a_transition_command() {
    let rig = Rig::healthy();
    let controller = rig.controller();
    assert_eq!(
        controller.promote(&command(1)),
        Err(DrPairingError::PairNotFound)
    );
    assert_eq!(
        controller.restore(&command(1)),
        Err(DrPairingError::PairNotFound)
    );
    assert_eq!(
        controller.activate_pair(&command(1)),
        Err(DrPairingError::PairNotFound)
    );
}

#[test]
fn the_promotion_event_carries_what_an_auditor_needs() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    rig.controller()
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");

    let events = rig.sink.events().expect("sink readable");
    assert_eq!(events.len(), 3, "assign, activate, promote");
    let promoted = &events[2];
    assert_eq!(promoted.kind, DrPairEventKind::Promoted);
    assert_eq!(promoted.kind.label(), "oya.tenancy.dr-pairing-promoted");
    assert_eq!(promoted.tenant_id, TENANT);
    assert_eq!(promoted.home_cell, HOME);
    assert_eq!(promoted.dr_cell, DR);
    assert_eq!(promoted.jurisdiction, "eu");
    assert_eq!(promoted.from_state, PairState::HomeActive);
    assert_eq!(promoted.to_state, PairState::DrActive);
    assert_eq!(promoted.from_version, 2);
    assert_eq!(promoted.to_version, 3);
    assert_eq!(promoted.decision, PromotionDecision::Eligible);
    assert_eq!(promoted.at_millis, 1_700_000_001_000);
    assert_eq!(promoted.correlation_id, "corr-2");
    assert!(promoted.idempotency_key.starts_with("drp-"));
}

#[test]
fn a_promotion_that_committed_unnarrated_says_so_and_is_recoverable() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    rig.sink.set_failing(true).expect("sink toggleable");
    let error = rig
        .controller()
        .promote(&command(active.pair_version))
        .expect_err("a sink outage is reported, not swallowed");
    // Not `AuditEmitUnavailable` and emphatically not `StalePairVersion`:
    // the caller is told their write LANDED, and at which version.
    assert_eq!(
        error,
        DrPairingError::NarrationPending {
            committed_version: 3
        }
    );

    // The state change committed before the narration was attempted.
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored.state, PairState::DrActive);
    assert_eq!(stored.pair_version, 3);

    // While the sink is still down, re-narration reports the sink, not a race.
    assert_eq!(
        rig.controller()
            .renarrate(DrPairEventKind::Promoted, &command(3)),
        Err(DrPairingError::AuditEmitUnavailable)
    );

    // After recovery the caller finishes the job it started. The rebuilt
    // event is the event that was lost, not a summary of it.
    rig.sink.set_failing(false).expect("sink toggleable");
    let recovered = rig
        .controller()
        .renarrate(DrPairEventKind::Promoted, &command(3))
        .expect("the committed transition is narratable after recovery");
    assert_eq!(recovered.kind, DrPairEventKind::Promoted);
    assert_eq!(recovered.from_state, PairState::HomeActive);
    assert_eq!(recovered.to_state, PairState::DrActive);
    assert_eq!(recovered.from_version, 2);
    assert_eq!(recovered.to_version, 3);
    assert_eq!(recovered.decision, PromotionDecision::Eligible);
    assert_eq!(recovered.correlation_id, "corr-2");
    assert_eq!(recovered.at_millis, 1_700_000_001_000);

    // Retrying it is free: the derived idempotency key deduplicates.
    rig.controller()
        .renarrate(DrPairEventKind::Promoted, &command(3))
        .expect("re-narration is idempotent");
    let promoted: Vec<_> = rig
        .sink
        .events()
        .expect("sink readable")
        .into_iter()
        .filter(|event| event.kind == DrPairEventKind::Promoted)
        .collect();
    assert_eq!(promoted.len(), 1, "exactly one promotion in the trail");
    assert_eq!(promoted[0], recovered);
}

#[test]
fn a_lost_assignment_narration_is_recoverable_too() {
    let rig = Rig::healthy();
    rig.sink.set_failing(true).expect("sink toggleable");
    let error = rig
        .controller()
        .assign_pair_to(&assignment(None), DR)
        .expect_err("the sink is down");
    assert_eq!(
        error,
        DrPairingError::NarrationPending {
            committed_version: 1
        }
    );
    rig.sink.set_failing(false).expect("sink toggleable");
    // The command carries the committed version plus the original instant
    // and correlation id from the assignment that produced it.
    let narration = FailoverCommand {
        tenant_id: TENANT.to_owned(),
        expected_version: 1,
        at_millis: 1_700_000_000_000,
        correlation_id: "corr-1".to_owned(),
    };
    let recovered = rig
        .controller()
        .renarrate(DrPairEventKind::PairAssigned, &narration)
        .expect("the assignment is narratable");
    assert_eq!(
        recovered.from_version, 0,
        "a first assignment has no predecessor"
    );
    assert_eq!(recovered.to_version, 1);
    assert_eq!(recovered.to_state, PairState::Planned);
    assert_eq!(
        recovered.decision,
        PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_ACTIVATED
        }
    );
    assert_eq!(rig.sink.events().expect("sink readable").len(), 1);
}

#[test]
fn renarration_refuses_to_invent_an_event_for_a_pair_that_moved_on() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    rig.controller()
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    let controller = rig.controller();
    // Version 2 is behind the stored 3: the state it described is gone.
    assert_eq!(
        controller.renarrate(DrPairEventKind::PairActivated, &command(2)),
        Err(DrPairingError::StalePairVersion)
    );
    // Right version, wrong kind: version 3 is a promotion, not a failback.
    assert_eq!(
        controller.renarrate(DrPairEventKind::Restored, &command(3)),
        Err(DrPairingError::IllegalTransition)
    );
    assert_eq!(
        controller.renarrate(
            DrPairEventKind::Promoted,
            &FailoverCommand {
                tenant_id: "ten_ghost".to_owned(),
                ..command(3)
            }
        ),
        Err(DrPairingError::PairNotFound)
    );
}

#[test]
fn an_activated_pair_is_never_demoted_by_a_reassignment() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    assert_eq!(active.state, PairState::HomeActive);
    let controller = rig.controller();

    // `HomeActive -> Planned` is not an edge an assignment may walk: it
    // would withdraw the tenant's failover capability in silence.
    let mut reassign = assignment(Some(active.pair_version));
    reassign.correlation_id = "corr-9".to_owned();
    assert_eq!(
        controller.assign_pair_to(&reassign, "cell-eu-3"),
        Err(DrPairingError::IllegalTransition)
    );

    // Nothing moved: the pair is still activated, still promotable, and
    // the trail gained no event describing an impossible edge.
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored, active);
    assert_eq!(
        controller.assess_promotion(TENANT),
        Ok(PromotionDecision::Eligible)
    );
    assert_eq!(rig.sink.events().expect("sink readable").len(), 2);
}

#[test]
fn replanning_withdraws_failover_explicitly_and_audibly() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    let controller = rig.controller();

    let mut replan = assignment(Some(active.pair_version));
    replan.correlation_id = "corr-9".to_owned();
    let replanned = controller
        .replan_pair_to(&replan, "cell-eu-3")
        .expect("re-cabling an activated pair is an explicit operation");
    assert_eq!(replanned.dr_cell, "cell-eu-3");
    assert_eq!(replanned.state, PairState::Planned);
    assert_eq!(replanned.pair_version, 3);

    // The capability really is withdrawn, and the trail says who withdrew it.
    assert_eq!(
        controller.assess_promotion(TENANT),
        Ok(PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_ACTIVATED
        })
    );
    let events = rig.sink.events().expect("sink readable");
    let last = events.last().expect("an event was emitted");
    assert_eq!(last.kind, DrPairEventKind::PairReplanned);
    assert_eq!(last.kind.label(), "oya.tenancy.dr-pairing-replanned");
    assert_eq!(last.from_state, PairState::HomeActive);
    assert_eq!(last.to_state, PairState::Planned);
    assert_eq!(last.correlation_id, "corr-9");
    assert_eq!(
        last.decision,
        PromotionDecision::Blocked {
            reason_code: reason::PAIR_NOT_ACTIVATED
        }
    );

    // Activating again restores it, on the new cell.
    let reactivated = controller
        .activate_pair(&command(replanned.pair_version))
        .expect("the re-cabled pair activates");
    assert_eq!(reactivated.state, PairState::HomeActive);
    assert_eq!(reactivated.dr_cell, "cell-eu-3");
}

#[test]
fn replanning_is_refused_mid_failover_and_for_a_tenant_with_no_pair() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    let controller = rig.controller();
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    let mut replan = assignment(Some(promoted.pair_version));
    replan.correlation_id = "corr-9".to_owned();
    assert_eq!(
        controller.replan_pair_to(&replan, "cell-eu-3"),
        Err(DrPairingError::IllegalTransition),
        "a pair serving from DR is not re-cabled"
    );

    let empty = Rig::healthy();
    assert_eq!(
        empty
            .controller()
            .replan_pair_to(&assignment(None), "cell-eu-3"),
        Err(DrPairingError::PairNotFound),
        "there is nothing to re-cable"
    );
}

#[test]
fn an_assignment_may_not_move_the_home_cell_of_a_stored_pair() {
    let rig = Rig::healthy();
    let assigned = rig
        .controller()
        .assign_pair_to(&assignment(None), DR)
        .expect("first assignment succeeds");
    let mut moved = assignment(Some(assigned.pair_version));
    moved.home_cell = "cell-eu-3".to_owned();
    assert_eq!(
        rig.controller().assign_pair_to(&moved, DR),
        Err(DrPairingError::HomeCellImmutable),
        "the pair records where the tenant is served from, not a wish"
    );
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored.serving_cell(), HOME);
}

#[test]
fn failback_asks_the_home_side_and_not_the_dr_side_about_the_home_cell() {
    // The probe knows the home cell ONLY as a home primary — exactly what a
    // faithful adapter reports for a cell that hosts a primary and no DR
    // replica. Asking `dr_replica_health` about it would fail the probe and
    // strand the tenant in its DR cell.
    let rig = Rig::new(
        StaticDrSloProbe::new()
            .with_healthy(DR)
            .with_home_healthy(HOME),
        StaticResidencyPolicy::permitting(),
    );
    let active = paired_and_active(&rig);
    let controller = rig.controller();
    assert_eq!(
        rig.probe.dr_replica_health(HOME),
        Err(DrPairingError::SloProbeFailed),
        "the home cell hosts no DR replica to ask about"
    );
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    let restored = controller
        .restore(&command(promoted.pair_version))
        .expect("failback succeeds on a healthy home primary");
    assert_eq!(restored.state, PairState::HomeActive);
    assert_eq!(restored.serving_cell(), HOME);
}

#[test]
fn a_failing_home_probe_fails_failback_closed_as_an_error() {
    let rig = Rig::new(
        StaticDrSloProbe::new()
            .with_healthy(DR)
            .with_home_probe_failure(HOME),
        StaticResidencyPolicy::permitting(),
    );
    let active = paired_and_active(&rig);
    let controller = rig.controller();
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    assert_eq!(
        controller.restore(&command(promoted.pair_version)),
        Err(DrPairingError::SloProbeFailed),
        "an unanswerable home probe is neither healthy nor a plain block"
    );
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored.state, PairState::DrActive, "no failback happened");
}

#[test]
fn no_event_claims_a_dr_replica_that_was_never_probed() {
    // The DR probe cannot answer at all in this rig, so no transition here
    // is entitled to record `Eligible`.
    let rig = Rig::new(
        StaticDrSloProbe::new()
            .with_home_healthy(HOME)
            .with_probe_failure(DR),
        StaticResidencyPolicy::permitting(),
    );
    paired_and_active(&rig);
    assert_eq!(
        rig.controller().assess_promotion(TENANT),
        Err(DrPairingError::SloProbeFailed),
        "the DR replica's health is unknown"
    );
    for event in rig.sink.events().expect("sink readable") {
        assert_eq!(
            event.decision,
            PromotionDecision::Blocked {
                reason_code: reason::PAIR_NOT_ACTIVATED
            },
            "{} recorded a health claim nobody checked",
            event.kind.label()
        );
    }
}

#[test]
fn a_restoration_records_the_posture_it_actually_had() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    let controller = rig.controller();
    let promoted = controller
        .promote(&command(active.pair_version))
        .expect("promotion succeeds");
    controller
        .restore(&command(promoted.pair_version))
        .expect("failback succeeds");
    let events = rig.sink.events().expect("sink readable");
    let restored = events.last().expect("an event was emitted");
    assert_eq!(restored.kind, DrPairEventKind::Restored);
    assert_eq!(
        restored.decision,
        PromotionDecision::Blocked {
            reason_code: reason::ALREADY_PROMOTED
        },
        "at the instant of failback the tenant was serving from DR"
    );
}

#[test]
fn a_direct_repository_write_cannot_roll_the_version_backwards() {
    let rig = Rig::healthy();
    let active = paired_and_active(&rig);
    let rolled_back = DrPair {
        pair_version: 1,
        ..active.clone()
    };
    assert_eq!(
        rig.repo.record(&rolled_back),
        Err(DrPairingError::NonMonotonicPairVersion),
        "a version chain that can go backwards is not a split-brain guard"
    );
    assert_eq!(
        rig.repo.record(&active),
        Err(DrPairingError::NonMonotonicPairVersion),
        "standing still is going backwards for a replayed command"
    );
    // The conditional write is not a loophole either: the expected version
    // may match and the write still refuse to park the chain in place.
    assert_eq!(
        rig.repo.compare_and_swap(Some(2), &rolled_back),
        Err(DrPairingError::NonMonotonicPairVersion)
    );
    let stored = rig
        .repo
        .current(TENANT)
        .expect("store readable")
        .expect("a pair is stored");
    assert_eq!(stored.pair_version, 2);
}

#[test]
fn a_padded_tenant_id_is_refused_rather_than_forked_into_a_second_chain() {
    let rig = Rig::healthy();
    rig.controller()
        .assign_pair_to(&assignment(None), DR)
        .expect("first assignment succeeds");
    let mut padded = assignment(None);
    padded.tenant_id = format!("{TENANT} ");
    assert_eq!(
        rig.controller().assign_pair_to(&padded, DR),
        Err(DrPairingError::InvalidTenantId),
        "a padded id would key a second, invisible version chain"
    );
    assert_eq!(rig.repo.len(), Ok(1));

    let mut padded_cell = assignment(None);
    padded_cell.home_cell = format!(" {HOME}");
    assert_eq!(
        rig.controller().assign_pair_to(&padded_cell, DR),
        Err(DrPairingError::InvalidCellId)
    );
}

#[test]
fn draining_the_sink_frees_the_buffer_without_forgetting_the_dedupe_keys() {
    let rig = Rig::healthy();
    paired_and_active(&rig);
    let drained = rig.sink.drain().expect("sink drainable");
    assert_eq!(drained.len(), 2, "assign and activate");
    assert!(
        rig.sink.events().expect("sink readable").is_empty(),
        "the buffer is reclaimed"
    );
    // A re-narration of an already-forwarded event stays deduped: the keys
    // outlive the buffer on purpose.
    rig.controller()
        .renarrate(DrPairEventKind::PairActivated, &command(2))
        .expect("re-narration is accepted by the sink");
    assert!(
        rig.sink.events().expect("sink readable").is_empty(),
        "a duplicate key adds nothing"
    );
}

#[test]
fn port_outages_surface_as_their_own_typed_errors() {
    let rig = Rig::healthy();
    paired_and_active(&rig);

    rig.repo.set_fail_reads(true).expect("store toggleable");
    assert_eq!(
        rig.controller().assess_promotion(TENANT),
        Err(DrPairingError::PersistenceUnavailable)
    );
    rig.repo.set_fail_reads(false).expect("store toggleable");

    let paired = Rig::healthy();
    paired_and_active(&paired);
    let blind = Rig {
        catalog: catalog().unavailable(),
        ..paired
    };
    assert_eq!(
        blind.controller().assess_promotion(TENANT),
        Err(DrPairingError::CellCatalogUnavailable)
    );

    let mute = Rig {
        policy: StaticResidencyPolicy::unavailable(),
        ..Rig::healthy()
    };
    assert_eq!(
        mute.controller().assign_pair_to(&assignment(None), DR),
        Err(DrPairingError::ResidencyPolicyUnavailable)
    );
}
