// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::{DataClass, PrivacyDataClass, privacy_data_classes_from};
use intelligence_capability_domain::AutonomyTier;
use intelligence_run_domain::{RunDisposition, RunError, RunLedger, RunStart, RunState};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

#[test]
fn run_ledger_starts_and_completes_successful_run() {
    let mut ledger = RunLedger::default();
    let run = ledger
        .start(valid_start("idem-001", 1_000, AutonomyTier::T2Advisory))
        .expect("run start is valid");
    assert_eq!(run.run_id.value, "run_000000000001");
    assert_eq!(run.state.value, RunState::Running);
    assert_eq!(
        run.touched_privacy_data_classes(),
        privacy_data_classes(vec![DataClass::InternalOnly]).as_slice()
    );
    assert_eq!(
        run.legacy_touched_data_classes(),
        vec![DataClass::InternalOnly]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            run.touched_data_classes(),
            run.legacy_touched_data_classes()
        );
    }

    let completed = ledger
        .complete(&run.run_id.value, RunDisposition::Success, 1_005)
        .expect("running run can complete");
    assert_eq!(completed.state.value, RunState::Succeeded);
    assert_eq!(completed.disposition.value, Some(RunDisposition::Success));
    assert_eq!(completed.completed_at_epoch_seconds.value, Some(1_005));
    assert_eq!(ledger.runs().len(), 1);
}

#[test]
fn run_ledger_rejects_invalid_shape_and_non_running_completion() {
    let mut ledger = RunLedger::default();
    let start = valid_start("idem-projection", 999, AutonomyTier::T2Advisory);
    assert_eq!(
        start.touched_privacy_data_classes(),
        privacy_data_classes(vec![DataClass::InternalOnly]).as_slice()
    );
    assert_eq!(
        start.legacy_touched_data_classes(),
        vec![DataClass::InternalOnly]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            start.touched_data_classes(),
            start.legacy_touched_data_classes()
        );
    }

    assert_eq!(
        RunStart::new(
            "tenant-alpha".into(),
            "cap.demo.invoke".into(),
            "usr_admin".into(),
            AutonomyTier::T1ViewOnly,
            privacy_data_classes(vec![DataClass::InternalOnly]),
            "region-home".into(),
            "idem-001".into(),
            1_000,
        ),
        Err(RunError::InvalidTenantId)
    );

    let rejected = ledger
        .reject(
            valid_start("idem-002", 1_010, AutonomyTier::T1ViewOnly),
            RunDisposition::FailureBudget,
        )
        .expect("rejection is valid");
    assert_eq!(rejected.state.value, RunState::RejectedBudget);
    assert_eq!(
        ledger.complete(&rejected.run_id.value, RunDisposition::Success, 1_011),
        Err(RunError::RunNotRunning)
    );

    let mut invalid_start = valid_start("idem-003", 1_020, AutonomyTier::T2Advisory);
    invalid_start.data_classes_touched.value.clear();
    assert_eq!(
        ledger.start(invalid_start),
        Err(RunError::MissingDataClasses)
    );

    for marker in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            RunStart::try_from_legacy_data_classes_touched(
                "ten_alpha".into(),
                "cap.demo.invoke".into(),
                "usr_admin".into(),
                AutonomyTier::T1ViewOnly,
                vec![marker],
                "region-home".into(),
                "idem-004".into(),
                1_030,
            ),
            Err(RunError::InvalidDataClass)
        );
    }
}

#[test]
fn run_ledger_replays_valid_history_and_rejects_tampered_records() {
    let mut ledger = RunLedger::default();
    let running = ledger
        .start(valid_start("idem-003", 1_020, AutonomyTier::T2Advisory))
        .expect("run start is valid");
    ledger
        .complete(&running.run_id.value, RunDisposition::Success, 1_021)
        .expect("run completion is valid");
    ledger
        .reject(
            valid_start("idem-004", 1_030, AutonomyTier::T1ViewOnly),
            RunDisposition::FailureAutonomy,
        )
        .expect("run rejection is valid");

    let mut restored =
        RunLedger::from_runs(ledger.runs().to_vec()).expect("history can be replayed");
    assert_eq!(restored.runs(), ledger.runs());
    let next = restored
        .start(valid_start("idem-005", 1_040, AutonomyTier::T2Advisory))
        .expect("next run continues sequence");
    assert_eq!(next.run_id.value, "run_000000000003");

    let mut tampered = ledger.runs().to_vec();
    tampered[0].data_class.value = DataClass::Public;
    assert_eq!(
        RunLedger::from_runs(tampered),
        Err(RunError::InvalidRunHistory)
    );
}

fn valid_start(idempotency_key: &str, started_at: u64, tier: AutonomyTier) -> RunStart {
    RunStart::new(
        "ten_alpha".into(),
        "cap.demo.invoke".into(),
        "usr_admin".into(),
        tier,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "region-home".into(),
        idempotency_key.into(),
        started_at,
    )
    .unwrap()
}
