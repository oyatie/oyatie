/// Monotonic progress exposed while constructing a dependency graph.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DependencyGraphConstructionProgressV1 {
    completed_nodes: u64,
    completed_edges: u64,
}

impl DependencyGraphConstructionProgressV1 {
    #[must_use]
    pub const fn completed_nodes(self) -> u64 {
        self.completed_nodes
    }

    #[must_use]
    pub const fn completed_edges(self) -> u64 {
        self.completed_edges
    }
}

struct DependencyGraphConstructionControlV1<C> {
    work: LifecycleWorkControlV1<C, DependencyGraphConstructionProgressV1>,
}

impl<C> DependencyGraphConstructionControlV1<C>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    fn try_new(callback: C) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            work: LifecycleWorkControlV1::try_new(
                callback,
                DependencyGraphConstructionProgressV1::default(),
                LifecycleFailureClassV1::DependencyGraphConstructionCancelled,
                LifecycleFailureClassV1::DependencyGraphConstructionDeadlineExceeded,
            )?,
        })
    }

    fn complete_node(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_nodes = progress
            .completed_nodes
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_edge(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_edges = progress
            .completed_edges
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
