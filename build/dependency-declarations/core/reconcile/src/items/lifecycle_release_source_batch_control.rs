/// Monotonic progress exposed while validating one release-source batch.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ReleaseSourceBatchProgressV1 {
    completed_items: u64,
}

impl ReleaseSourceBatchProgressV1 {
    #[must_use]
    pub const fn completed_items(self) -> u64 {
        self.completed_items
    }
}

struct ReleaseSourceBatchControlV1<C> {
    work: LifecycleWorkControlV1<C, ReleaseSourceBatchProgressV1>,
}

impl<C> ReleaseSourceBatchControlV1<C>
where
    C: FnMut(ReleaseSourceBatchProgressV1) -> LifecycleControlDecisionV1,
{
    fn try_new(callback: C) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            work: LifecycleWorkControlV1::try_new(
                callback,
                ReleaseSourceBatchProgressV1::default(),
                LifecycleFailureClassV1::ReleaseSourceBatchCancelled,
                LifecycleFailureClassV1::ReleaseSourceBatchDeadlineExceeded,
            )?,
        })
    }

    fn complete_item(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_items = progress
            .completed_items
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
