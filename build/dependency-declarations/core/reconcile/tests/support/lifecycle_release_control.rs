use super::lifecycle_support::*;
use dependency_declarations_reconcile::*;

fn release_facts(
    item_count: usize,
) -> (
    Vec<ReleaseSourceBatchV1>,
    Vec<ReleaseItemV1>,
    Vec<ReleaseDispositionV1>,
) {
    let source = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-release",
    );
    let items: Vec<_> = (0..item_count)
        .map(|index| {
            release_item(
                &source,
                &format!("rust#release-item-{index:06}"),
                ReleaseItemKindV1::Compiler,
            )
        })
        .collect();
    let dispositions = items
        .iter()
        .map(|item| disposition(item, ReleaseDecisionV1::Benchmark))
        .collect();
    let batches = vec![qualified_batch(source, &items)];
    (batches, items, dispositions)
}

fn source_batch_facts(
    item_count: usize,
) -> (
    LifecycleSourceV1,
    ReleaseExtractionProfileV1,
    Vec<ReleaseItemV1>,
) {
    let source = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-release-batch",
    );
    let items = (0..item_count)
        .map(|index| {
            release_item(
                &source,
                &format!("rust#batch-item-{index:06}"),
                ReleaseItemKindV1::Compiler,
            )
        })
        .collect();
    let extraction = extraction(
        &source,
        ReleaseExtractionQualificationV1::Qualified {
            qualification_receipt_sha256: digest("batch-extraction-qualification"),
        },
    );
    (source, extraction, items)
}

#[test]
fn release_source_batch_cancellation_refuses_before_item_work() {
    let (source, extraction, items) = source_batch_facts(8);
    let mut checkpoints = Vec::new();
    let failure = ReleaseSourceBatchV1::try_from_items(
        source,
        extraction,
        &items,
        digest("cancelled-batch"),
        |progress| {
            checkpoints.push(progress);
            LifecycleControlDecisionV1::Cancel
        },
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ReleaseSourceBatchCancelled
    );
    assert_eq!(checkpoints, vec![ReleaseSourceBatchProgressV1::default()]);
}

#[test]
fn release_source_batch_deadline_refuses_at_a_bounded_item_checkpoint() {
    let (source, extraction, items) = source_batch_facts(4_096);
    let mut checkpoints = Vec::new();
    let failure = ReleaseSourceBatchV1::try_from_items(
        source,
        extraction,
        &items,
        digest("deadline-batch"),
        |progress| {
            checkpoints.push(progress);
            if progress.completed_items() >= 1_024 {
                LifecycleControlDecisionV1::DeadlineExceeded
            } else {
                LifecycleControlDecisionV1::Continue
            }
        },
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ReleaseSourceBatchDeadlineExceeded
    );
    assert_eq!(checkpoints.last().unwrap().completed_items(), 1_024);
}

#[test]
fn release_ledger_cancellation_refuses_before_work() {
    let (batches, items, dispositions) = release_facts(8);
    let mut checkpoints = Vec::new();
    let failure = ReleaseLedgerV1::try_new(batches, items, dispositions, |progress| {
        checkpoints.push(progress);
        LifecycleControlDecisionV1::Cancel
    })
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ReleaseLedgerCancelled
    );
    assert_eq!(checkpoints, vec![ReleaseLedgerProgressV1::default()]);
}

#[test]
fn release_ledger_deadline_refuses_at_a_bounded_item_checkpoint() {
    let (batches, items, dispositions) = release_facts(4_096);
    let mut checkpoints = Vec::new();
    let failure = ReleaseLedgerV1::try_new(batches, items, dispositions, |progress| {
        checkpoints.push(progress);
        if progress.completed_items() >= 1_024 {
            LifecycleControlDecisionV1::DeadlineExceeded
        } else {
            LifecycleControlDecisionV1::Continue
        }
    })
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ReleaseLedgerDeadlineExceeded
    );
    assert!(checkpoints.len() > 1);
    assert!(checkpoints.len() < 16);
    assert_eq!(checkpoints.last().unwrap().completed_items(), 1_024);
    assert_eq!(checkpoints.last().unwrap().completed_dispositions(), 0);
}

#[test]
fn continuing_release_control_preserves_canonical_identity_and_final_progress() {
    let (batches, items, dispositions) = release_facts(3);
    let mut final_progress = None;
    let controlled = ReleaseLedgerV1::try_new(
        batches.clone(),
        items.clone(),
        dispositions.clone(),
        |progress| {
            final_progress = Some(progress);
            LifecycleControlDecisionV1::Continue
        },
    )
    .unwrap();
    let reversed = ReleaseLedgerV1::try_new(
        batches.into_iter().rev().collect(),
        items.into_iter().rev().collect(),
        dispositions.into_iter().rev().collect(),
        |_| LifecycleControlDecisionV1::Continue,
    )
    .unwrap();

    assert_eq!(controlled.identity_sha256(), reversed.identity_sha256());
    let progress = final_progress.unwrap();
    assert_eq!(progress.completed_source_batches(), 1);
    assert_eq!(progress.completed_items(), 3);
    assert_eq!(progress.completed_dispositions(), 3);
}
