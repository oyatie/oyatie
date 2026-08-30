const LIFECYCLE_CHECKPOINT_WORK_UNITS: u64 = 1_024;

/// Caller-owned stop decision at one bounded lifecycle-work checkpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LifecycleControlDecisionV1 {
    Continue,
    Cancel,
    DeadlineExceeded,
}

struct LifecycleWorkControlV1<C, P> {
    callback: C,
    progress: P,
    work_since_checkpoint: u64,
    cancelled: LifecycleFailureClassV1,
    deadline_exceeded: LifecycleFailureClassV1,
}

impl<C, P> LifecycleWorkControlV1<C, P>
where
    C: FnMut(P) -> LifecycleControlDecisionV1,
    P: Copy,
{
    fn try_new(
        callback: C,
        progress: P,
        cancelled: LifecycleFailureClassV1,
        deadline_exceeded: LifecycleFailureClassV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let mut control = Self {
            callback,
            progress,
            work_since_checkpoint: 0,
            cancelled,
            deadline_exceeded,
        };
        control.checkpoint()?;
        Ok(control)
    }

    fn progress_mut(&mut self) -> &mut P {
        &mut self.progress
    }

    fn record_work(&mut self) -> Result<(), LifecycleFailureV1> {
        self.work_since_checkpoint = self
            .work_since_checkpoint
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        if self.work_since_checkpoint < LIFECYCLE_CHECKPOINT_WORK_UNITS {
            return Ok(());
        }
        self.checkpoint_and_reset()
    }

    fn checkpoint_and_reset(&mut self) -> Result<(), LifecycleFailureV1> {
        self.checkpoint()?;
        self.work_since_checkpoint = 0;
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<(), LifecycleFailureV1> {
        match (self.callback)(self.progress) {
            LifecycleControlDecisionV1::Continue => Ok(()),
            LifecycleControlDecisionV1::Cancel => {
                Err(LifecycleFailureV1::new(self.cancelled))
            }
            LifecycleControlDecisionV1::DeadlineExceeded => {
                Err(LifecycleFailureV1::new(self.deadline_exceeded))
            }
        }
    }
}
