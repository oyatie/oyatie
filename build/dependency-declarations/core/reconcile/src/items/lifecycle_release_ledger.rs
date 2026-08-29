/// Completeness state of normalized release evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReleaseLedgerCompletenessV1 {
    ReleasedComplete,
    UnqualifiedExtraction,
    Provisional,
}

/// Complete, provenance-bound release facts and dispositions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseLedgerV1 {
    batches: Box<[ReleaseSourceBatchV1]>,
    items: Box<[ReleaseItemV1]>,
    dispositions: Box<[ReleaseDispositionV1]>,
    completeness: ReleaseLedgerCompletenessV1,
    identity_sha256: DigestV1,
}

impl ReleaseLedgerV1 {
    pub fn try_new(
        mut batches: Vec<ReleaseSourceBatchV1>,
        mut items: Vec<ReleaseItemV1>,
        mut dispositions: Vec<ReleaseDispositionV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_lifecycle_collection_bounds(&batches, &items, &dispositions)?;
        batches.sort_by_key(|batch| batch.source().identity_sha256());
        if batches.windows(2).any(|pair| {
            pair[0].source().identity_sha256() == pair[1].source().identity_sha256()
        }) {
            return Err(duplicate_lifecycle_identity());
        }
        items.sort_by(|left, right| {
            (left.source_identity, left.stable_key.as_bytes())
                .cmp(&(right.source_identity, right.stable_key.as_bytes()))
        });
        if items.windows(2).any(|pair| {
            pair[0].source_identity == pair[1].source_identity
                && pair[0].stable_key == pair[1].stable_key
        }) {
            return Err(duplicate_lifecycle_identity());
        }
        validate_source_coverage(&batches, &items)?;
        dispositions.sort_by_key(ReleaseDispositionV1::item_identity);
        if dispositions
            .windows(2)
            .any(|pair| pair[0].item_identity == pair[1].item_identity)
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateDisposition,
            ));
        }
        validate_dispositions(&items, &dispositions)?;

        let completeness = if batches
            .iter()
            .any(|batch| batch.source().maturity() == SourceMaturityV1::Provisional)
        {
            ReleaseLedgerCompletenessV1::Provisional
        } else if batches
            .iter()
            .any(|batch| !batch.extraction().qualification().is_qualified())
        {
            ReleaseLedgerCompletenessV1::UnqualifiedExtraction
        } else {
            ReleaseLedgerCompletenessV1::ReleasedComplete
        };
        let identity_sha256 =
            release_ledger_identity(&batches, &items, &dispositions, completeness)?;
        Ok(Self {
            batches: batches.into_boxed_slice(),
            items: items.into_boxed_slice(),
            dispositions: dispositions.into_boxed_slice(),
            completeness,
            identity_sha256,
        })
    }

    pub fn require_released_complete(&self) -> Result<(), LifecycleFailureV1> {
        match self.completeness {
            ReleaseLedgerCompletenessV1::ReleasedComplete => Ok(()),
            ReleaseLedgerCompletenessV1::UnqualifiedExtraction => Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnqualifiedExtraction,
            )),
            ReleaseLedgerCompletenessV1::Provisional => Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ProvisionalSource,
            )),
        }
    }

    #[must_use]
    pub const fn completeness(&self) -> ReleaseLedgerCompletenessV1 {
        self.completeness
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }

    #[must_use]
    pub fn items(&self) -> &[ReleaseItemV1] {
        &self.items
    }

    #[must_use]
    pub fn dispositions(&self) -> &[ReleaseDispositionV1] {
        &self.dispositions
    }

    #[must_use]
    pub fn batches(&self) -> &[ReleaseSourceBatchV1] {
        &self.batches
    }
}
