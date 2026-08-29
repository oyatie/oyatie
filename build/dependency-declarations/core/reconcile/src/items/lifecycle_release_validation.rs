fn validate_lifecycle_collection_bounds(
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
    let mut total_bytes = 0_u64;
    for batch in batches {
        total_bytes = total_bytes
            .checked_add(batch.source().length_bytes())
            .ok_or_else(lifecycle_bounds)?;
    }
    if total_bytes > LifecycleBoundsV1::MAX_TOTAL_SOURCE_BYTES {
        return Err(lifecycle_bounds());
    }
    Ok(())
}

fn validate_source_coverage(
    batches: &[ReleaseSourceBatchV1],
    items: &[ReleaseItemV1],
) -> Result<(), LifecycleFailureV1> {
    let mut grouped: std::collections::BTreeMap<DigestV1, Vec<DigestV1>> =
        std::collections::BTreeMap::new();
    for item in items {
        grouped
            .entry(item.source_identity())
            .or_default()
            .push(item.identity_sha256());
    }
    for batch in batches {
        let source_identity = batch.source().identity_sha256();
        let source_item_identities = grouped.remove(&source_identity).unwrap_or_default();
        let count = lifecycle_len(source_item_identities.len())?;
        if count != batch.item_count()
            || release_identity_set_sha256(source_item_identities)? != batch.items_sha256()
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::SourceCoverageMismatch,
            ));
        }
    }
    if !grouped.is_empty() {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::MissingSource,
        ));
    }
    Ok(())
}

fn validate_dispositions(
    items: &[ReleaseItemV1],
    dispositions: &[ReleaseDispositionV1],
) -> Result<(), LifecycleFailureV1> {
    let mut item_identities: Vec<DigestV1> =
        items.iter().map(ReleaseItemV1::identity_sha256).collect();
    item_identities.sort_unstable();
    if item_identities.len() != dispositions.len()
        || item_identities
            .iter()
            .zip(dispositions)
            .any(|(item, disposition)| *item != disposition.item_identity())
    {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::MissingDisposition,
        ));
    }
    Ok(())
}

fn release_ledger_identity(
    batches: &[ReleaseSourceBatchV1],
    items: &[ReleaseItemV1],
    dispositions: &[ReleaseDispositionV1],
    completeness: ReleaseLedgerCompletenessV1,
) -> Result<DigestV1, LifecycleFailureV1> {
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
    }
    hash.u64(lifecycle_len(items.len())?);
    for item in items {
        hash.digest(item.identity_sha256());
    }
    hash.u64(lifecycle_len(dispositions.len())?);
    for disposition in dispositions {
        hash.digest(disposition.identity_sha256);
    }
    Ok(hash.finish())
}

const fn duplicate_lifecycle_identity() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::DuplicateIdentity)
}
