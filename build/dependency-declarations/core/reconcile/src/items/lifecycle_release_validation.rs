fn validate_lifecycle_collection_shape(
    batches: &[ReleaseSourceBatchV1],
    items: &[ReleaseItemV1],
    dispositions: &[ReleaseDispositionV1],
) -> Result<(), LifecycleFailureV1> {
    if batches.is_empty()
        || batches.len() > LifecycleBoundsV1::MAX_SOURCE_OBJECTS
        || items.len() > LifecycleBoundsV1::MAX_RELEASE_ITEMS
        || dispositions.len() > LifecycleBoundsV1::MAX_DISPOSITIONS
    {
        return Err(lifecycle_bounds());
    }
    Ok(())
}

fn validate_lifecycle_source_bounds<C>(
    batches: &[ReleaseSourceBatchV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    let mut total_bytes = 0_u64;
    for batch in batches {
        total_bytes = total_bytes
            .checked_add(batch.source().length_bytes())
            .ok_or_else(lifecycle_bounds)?;
        control.complete_source_batch()?;
    }
    if total_bytes > LifecycleBoundsV1::MAX_TOTAL_SOURCE_BYTES {
        return Err(lifecycle_bounds());
    }
    Ok(())
}

fn canonicalize_release_batches<C>(
    batches: &mut [ReleaseSourceBatchV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    control.checkpoint_and_reset()?;
    batches.sort_by_key(|batch| batch.source().identity_sha256());
    control.checkpoint_and_reset()?;
    for pair in batches.windows(2) {
        if pair[0].source().identity_sha256() == pair[1].source().identity_sha256() {
            return Err(duplicate_lifecycle_identity());
        }
        control.record_work()?;
    }
    control.checkpoint_and_reset()
}

fn canonicalize_release_items<C>(
    items: &mut [ReleaseItemV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    control.checkpoint_and_reset()?;
    items.sort_by(|left, right| {
        (left.source_identity, left.stable_key.as_bytes())
            .cmp(&(right.source_identity, right.stable_key.as_bytes()))
    });
    control.checkpoint_and_reset()?;
    for pair in items.windows(2) {
        if pair[0].source_identity == pair[1].source_identity
            && pair[0].stable_key == pair[1].stable_key
        {
            return Err(duplicate_lifecycle_identity());
        }
        control.record_work()?;
    }
    control.checkpoint_and_reset()
}

fn validate_source_coverage<C>(
    batches: &[ReleaseSourceBatchV1],
    items: &[ReleaseItemV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    let mut grouped: std::collections::BTreeMap<DigestV1, Vec<DigestV1>> =
        std::collections::BTreeMap::new();
    for item in items {
        grouped
            .entry(item.source_identity())
            .or_default()
            .push(item.identity_sha256());
        control.complete_item()?;
    }
    control.checkpoint_and_reset()?;
    for batch in batches {
        let source_identity = batch.source().identity_sha256();
        let source_item_identities = grouped.remove(&source_identity).unwrap_or_default();
        let count = lifecycle_len(source_item_identities.len())?;
        let items_sha256 = release_identity_set_sha256_with_work(
            source_item_identities,
            || control.record_work(),
        )?;
        if count != batch.item_count()
            || items_sha256 != batch.items_sha256()
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::SourceCoverageMismatch,
            ));
        }
        control.checkpoint_and_reset()?;
    }
    if !grouped.is_empty() {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::MissingSource,
        ));
    }
    Ok(())
}

fn canonicalize_release_dispositions<C>(
    dispositions: &mut [ReleaseDispositionV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    control.checkpoint_and_reset()?;
    dispositions.sort_by_key(ReleaseDispositionV1::item_identity);
    control.checkpoint_and_reset()?;
    for pair in dispositions.windows(2) {
        if pair[0].item_identity == pair[1].item_identity {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateDisposition,
            ));
        }
        control.record_work()?;
    }
    control.checkpoint_and_reset()
}

fn validate_dispositions<C>(
    items: &[ReleaseItemV1],
    dispositions: &[ReleaseDispositionV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    let mut item_identities = Vec::with_capacity(items.len());
    for item in items {
        item_identities.push(item.identity_sha256());
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;
    item_identities.sort_unstable();
    control.checkpoint_and_reset()?;
    if item_identities.len() != dispositions.len() {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::MissingDisposition,
        ));
    }
    for (item, disposition) in item_identities.iter().zip(dispositions) {
        if *item != disposition.item_identity() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::MissingDisposition,
            ));
        }
        control.complete_disposition()?;
    }
    control.checkpoint_and_reset()
}

fn release_ledger_completeness<C>(
    batches: &[ReleaseSourceBatchV1],
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<ReleaseLedgerCompletenessV1, LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    let mut provisional = false;
    let mut unqualified = false;
    for batch in batches {
        provisional |= batch.source().maturity() == SourceMaturityV1::Provisional;
        unqualified |= !batch.extraction().qualification().is_qualified();
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;
    if provisional {
        Ok(ReleaseLedgerCompletenessV1::Provisional)
    } else if unqualified {
        Ok(ReleaseLedgerCompletenessV1::UnqualifiedExtraction)
    } else {
        Ok(ReleaseLedgerCompletenessV1::ReleasedComplete)
    }
}

fn release_ledger_identity<C>(
    batches: &[ReleaseSourceBatchV1],
    items: &[ReleaseItemV1],
    dispositions: &[ReleaseDispositionV1],
    completeness: ReleaseLedgerCompletenessV1,
    control: &mut ReleaseLedgerControlV1<C>,
) -> Result<DigestV1, LifecycleFailureV1>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    let mut hash = CanonicalHasherV1::new(b"build.release-ledger.v1\0");
    hash.tag(match completeness {
        ReleaseLedgerCompletenessV1::ReleasedComplete => 0,
        ReleaseLedgerCompletenessV1::UnqualifiedExtraction => 1,
        ReleaseLedgerCompletenessV1::Provisional => 2,
    });
    hash.u64(lifecycle_len(batches.len())?);
    for batch in batches {
        hash.digest(batch.source().identity_sha256());
        hash.digest(batch.extraction().identity_sha256());
        hash.digest(batch.receipt().identity_sha256());
        control.record_work()?;
    }
    hash.u64(lifecycle_len(items.len())?);
    for item in items {
        hash.digest(item.identity_sha256());
        control.record_work()?;
    }
    hash.u64(lifecycle_len(dispositions.len())?);
    for disposition in dispositions {
        hash.digest(disposition.identity_sha256);
        control.record_work()?;
    }
    Ok(hash.finish())
}

const fn duplicate_lifecycle_identity() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::DuplicateIdentity)
}
