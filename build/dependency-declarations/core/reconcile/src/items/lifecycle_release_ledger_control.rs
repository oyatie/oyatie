/// Monotonic progress exposed while constructing a release ledger.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ReleaseLedgerProgressV1 {
    completed_source_batches: u64,
    completed_items: u64,
    completed_dispositions: u64,
}

impl ReleaseLedgerProgressV1 {
    #[must_use]
    pub const fn completed_source_batches(self) -> u64 {
        self.completed_source_batches
    }

    #[must_use]
    pub const fn completed_items(self) -> u64 {
        self.completed_items
    }

    #[must_use]
    pub const fn completed_dispositions(self) -> u64 {
        self.completed_dispositions
    }
}

struct ReleaseLedgerControlV1<C> {
    work: LifecycleWorkControlV1<C, ReleaseLedgerProgressV1>,
}

impl<C> ReleaseLedgerControlV1<C>
where
    C: FnMut(ReleaseLedgerProgressV1) -> LifecycleControlDecisionV1,
{
    fn try_new(callback: C) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            work: LifecycleWorkControlV1::try_new(
                callback,
                ReleaseLedgerProgressV1::default(),
                LifecycleFailureClassV1::ReleaseLedgerCancelled,
                LifecycleFailureClassV1::ReleaseLedgerDeadlineExceeded,
            )?,
        })
    }

    fn complete_source_batch(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_source_batches = progress
            .completed_source_batches
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_item(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_items = progress
            .completed_items
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_disposition(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_dispositions = progress
            .completed_dispositions
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
