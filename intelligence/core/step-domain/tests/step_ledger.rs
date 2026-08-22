// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_step_domain::{
    StepDisposition, StepError, StepKind, StepLedger, StepStart, StepState,
};
use data_boundary_kernel::{DataClass, PrivacyDataClass, privacy_data_classes_from};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

#[test]
fn step_ledger_assigns_monotonic_sequences_per_run_and_completes_steps() {
    let mut ledger = StepLedger::default();
    let first = ledger
        .start(valid_start("run_000000000001", StepKind::ToolCall, 1_000))
        .expect("first step is valid");
    let second = ledger
        .start(valid_start(
            "run_000000000001",
            StepKind::ProviderCall,
            1_001,
        ))
        .expect("second step is valid");
    let other_run = ledger
        .start(valid_start("run_000000000002", StepKind::Reasoning, 1_002))
        .expect("other run starts its own sequence");

    assert_eq!(first.step_id.value, "step_000000000001_000001");
    assert_eq!(first.sequence.value, 1);
    assert_eq!(second.sequence.value, 2);
    assert_eq!(other_run.sequence.value, 1);
    assert_eq!(
        first.touched_privacy_data_classes(),
        privacy_data_classes(vec![DataClass::InternalOnly]).as_slice()
    );
    assert_eq!(
        first.legacy_touched_data_classes(),
        vec![DataClass::InternalOnly]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            first.touched_data_classes(),
            first.legacy_touched_data_classes()
        );
    }

    let completed = ledger
        .complete(&first.step_id.value, StepDisposition::Succeeded, 42, 1_003)
        .expect("running step completes");
    assert_eq!(completed.state.value, StepState::Succeeded);
    assert_eq!(
        completed.disposition.value,
        Some(StepDisposition::Succeeded)
    );
    assert_eq!(completed.latency_ms.value, Some(42));
}

#[test]
fn step_ledger_validates_shape_and_rejects_non_running_completion() {
    let start = valid_start("run_000000000001", StepKind::ToolCall, 999);
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
        StepStart::new(
            "run-bad".into(),
            StepKind::ToolCall,
            "provider.demo".into(),
            Some("model.demo".into()),
            Some(1),
            Some(2),
            privacy_data_classes(vec![DataClass::InternalOnly]),
            1_000,
        ),
        Err(StepError::InvalidRunId)
    );

    let mut ledger = StepLedger::default();
    let step = ledger
        .start(valid_start("run_000000000001", StepKind::ToolCall, 1_000))
        .unwrap();
    ledger
        .complete(&step.step_id.value, StepDisposition::Succeeded, 12, 1_001)
        .unwrap();
    assert_eq!(
        ledger.complete(&step.step_id.value, StepDisposition::Succeeded, 1, 1_002),
        Err(StepError::StepNotRunning)
    );

    let mut missing_classes = valid_start("run_000000000002", StepKind::ProviderCall, 1_010);
    missing_classes.data_classes_touched.value.clear();
    assert_eq!(
        ledger.start(missing_classes),
        Err(StepError::MissingDataClasses)
    );

    let mut inconsistent_class = valid_start("run_000000000002", StepKind::ProviderCall, 1_011);
    inconsistent_class.data_class.value = DataClass::Public;
    assert_eq!(
        ledger.start(inconsistent_class),
        Err(StepError::InvalidStepHistory)
    );

    for marker in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            StepStart::try_from_legacy_data_classes_touched(
                "run_000000000002".into(),
                StepKind::ToolCall,
                "provider.demo".into(),
                Some("model.demo".into()),
                Some(1),
                Some(2),
                vec![marker],
                1_020,
            ),
            Err(StepError::InvalidDataClass)
        );
    }
}

#[test]
fn step_ledger_replays_valid_history_and_rejects_tampered_records() {
    let mut ledger = StepLedger::default();
    let first = ledger
        .start(valid_start("run_000000000001", StepKind::ToolCall, 1_000))
        .expect("first step is valid");
    ledger
        .complete(&first.step_id.value, StepDisposition::Succeeded, 12, 1_001)
        .expect("first step completion is valid");
    ledger
        .start(valid_start(
            "run_000000000001",
            StepKind::ProviderCall,
            1_002,
        ))
        .expect("second step is valid");

    let mut restored = StepLedger::from_steps(ledger.steps().to_vec()).expect("history replays");
    assert_eq!(restored.steps(), ledger.steps());
    let next = restored
        .start(valid_start("run_000000000001", StepKind::Reasoning, 1_003))
        .expect("next step continues sequence");
    assert_eq!(next.step_id.value, "step_000000000001_000003");

    let mut tampered = ledger.steps().to_vec();
    tampered[0].data_class.value = DataClass::Public;
    assert_eq!(
        StepLedger::from_steps(tampered),
        Err(StepError::InvalidStepHistory)
    );
}

fn valid_start(run_id: &str, kind: StepKind, started_at: u64) -> StepStart {
    StepStart::new(
        run_id.into(),
        kind,
        "provider.demo".into(),
        Some("model.demo".into()),
        Some(12),
        Some(34),
        privacy_data_classes(vec![DataClass::InternalOnly]),
        started_at,
    )
    .unwrap()
}
