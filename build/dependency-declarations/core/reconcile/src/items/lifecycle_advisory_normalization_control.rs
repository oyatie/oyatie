/// Monotonic progress exposed while normalizing advisory records.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct AdvisoryNormalizationProgressV1 {
    completed_records: u64,
    completed_identifier_occurrences: u64,
    completed_package_claims: u64,
    completed_ranges: u64,
}

impl AdvisoryNormalizationProgressV1 {
    #[must_use]
    pub const fn completed_records(self) -> u64 {
        self.completed_records
    }

    #[must_use]
    pub const fn completed_identifier_occurrences(self) -> u64 {
        self.completed_identifier_occurrences
    }

    #[must_use]
    pub const fn completed_package_claims(self) -> u64 {
        self.completed_package_claims
    }

    #[must_use]
    pub const fn completed_ranges(self) -> u64 {
        self.completed_ranges
    }
}

struct AdvisoryNormalizationControlV1<C> {
    work: LifecycleWorkControlV1<C, AdvisoryNormalizationProgressV1>,
}

impl<C> AdvisoryNormalizationControlV1<C>
where
    C: FnMut(AdvisoryNormalizationProgressV1) -> LifecycleControlDecisionV1,
{
    fn try_new(callback: C) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            work: LifecycleWorkControlV1::try_new(
                callback,
                AdvisoryNormalizationProgressV1::default(),
                LifecycleFailureClassV1::AdvisoryNormalizationCancelled,
                LifecycleFailureClassV1::AdvisoryNormalizationDeadlineExceeded,
            )?,
        })
    }

    fn complete_record(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_records = progress
            .completed_records
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_identifier_occurrence(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_identifier_occurrences = progress
            .completed_identifier_occurrences
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_package_claim(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_package_claims = progress
            .completed_package_claims
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_range(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_ranges = progress
            .completed_ranges
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn record_work(&mut self) -> Result<(), LifecycleFailureV1> {
        self.work.record_work()
    }

    fn checkpoint_and_reset(&mut self) -> Result<(), LifecycleFailureV1> {
        self.work.checkpoint_and_reset()
    }
}
