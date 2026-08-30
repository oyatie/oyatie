use super::advisory::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

fn advisory_records(count: usize) -> Vec<AdvisoryRecordV1> {
    let source = record_source(
        LifecycleComponentV1::UpstreamAdvisory,
        AdvisoryAuthorityV1::Upstream(
            AdvisoryAuthorityNameV1::try_new("control-upstream").unwrap(),
        ),
        "advisory-control-revision",
        qualified(),
    );
    (0..count)
        .map(|index| {
            active_record(
                source.clone(),
                identifier(
                    AdvisoryNamespaceV1::Upstream,
                    &format!("UPSTREAM-{index:06}"),
                ),
                Vec::new(),
                AdvisoryAffectedSetV1::reference_only(digest(&format!("reference-{index:06}"))),
                200,
            )
        })
        .collect()
}

#[test]
fn advisory_normalization_cancellation_refuses_before_record_work() {
    let mut checkpoints = Vec::new();
    let failure = AdvisoryLedgerV1::try_normalize(advisory_records(1), |progress| {
        checkpoints.push(progress);
        LifecycleControlDecisionV1::Cancel
    })
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::AdvisoryNormalizationCancelled
    );
    assert_eq!(
        checkpoints,
        vec![AdvisoryNormalizationProgressV1::default()]
    );
}

#[test]
fn advisory_normalization_deadline_refuses_at_a_bounded_identifier_checkpoint() {
    let mut checkpoints = Vec::new();
    let failure = AdvisoryLedgerV1::try_normalize(advisory_records(4_096), |progress| {
        checkpoints.push(progress);
        if progress.completed_identifier_occurrences() >= 1_024 {
            LifecycleControlDecisionV1::DeadlineExceeded
        } else {
            LifecycleControlDecisionV1::Continue
        }
    })
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::AdvisoryNormalizationDeadlineExceeded
    );
    assert!(
        checkpoints
            .last()
            .unwrap()
            .completed_identifier_occurrences()
            >= 1_024
    );
    assert!(checkpoints.len() < 32);
}

#[test]
fn continuing_advisory_control_reports_complete_progress() {
    let mut final_progress = None;
    let ledger = AdvisoryLedgerV1::try_normalize(advisory_records(3), |progress| {
        final_progress = Some(progress);
        LifecycleControlDecisionV1::Continue
    })
    .unwrap();

    assert_eq!(ledger.record_count(), 3);
    let progress = final_progress.unwrap();
    assert_eq!(progress.completed_records(), 3);
    assert_eq!(progress.completed_identifier_occurrences(), 3);
    assert_eq!(progress.completed_package_claims(), 0);
    assert_eq!(progress.completed_ranges(), 0);
}
