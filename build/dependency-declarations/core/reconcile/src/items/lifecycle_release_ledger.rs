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
    pub fn try_new<C>(
        mut batches: Vec<ReleaseSourceBatchV1>,
        mut items: Vec<ReleaseItemV1>,
        mut dispositions: Vec<ReleaseDispositionV1>,
        control: C,
    ) -> Result<Self, LifecycleFailureV1>
    where
        C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
    {
        validate_lifecycle_collection_shape(&batches, &items, &dispositions)?;
        let mut control = ReleaseLedgerControlV1::try_new(control)?;
        validate_lifecycle_source_bounds(&batches, &mut control)?;
        canonicalize_release_batches(&mut batches, &mut control)?;
        canonicalize_release_items(&mut items, &mut control)?;
        validate_source_coverage(&batches, &items, &mut control)?;
        canonicalize_release_dispositions(&mut dispositions, &mut control)?;
        validate_dispositions(&items, &dispositions, &mut control)?;

        let completeness = release_ledger_completeness(&batches, &mut control)?;
        let identity_sha256 = release_ledger_identity(
            &batches,
            &items,
            &dispositions,
            completeness,
            &mut control,
        )?;
        control.checkpoint_and_reset()?;
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
