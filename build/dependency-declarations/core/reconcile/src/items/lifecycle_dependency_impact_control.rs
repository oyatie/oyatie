/// Monotonic progress exposed at bounded dependency-impact checkpoints.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DependencyImpactProgressV1 {
    completed_candidates: u64,
    visited_nodes: u64,
    visited_edges: u64,
    materialized_root_nodes: u64,
    materialized_nodes: u64,
    materialized_edges: u64,
}

impl DependencyImpactProgressV1 {
    #[must_use]
    pub const fn completed_candidates(self) -> u64 {
        self.completed_candidates
    }

    #[must_use]
    pub const fn visited_nodes(self) -> u64 {
        self.visited_nodes
    }

    #[must_use]
    pub const fn visited_edges(self) -> u64 {
        self.visited_edges
    }

    #[must_use]
    pub const fn materialized_root_nodes(self) -> u64 {
        self.materialized_root_nodes
    }

    #[must_use]
    pub const fn materialized_nodes(self) -> u64 {
        self.materialized_nodes
    }

    #[must_use]
    pub const fn materialized_edges(self) -> u64 {
        self.materialized_edges
    }
}

struct DependencyImpactControlV1<C> {
    work: LifecycleWorkControlV1<C, DependencyImpactProgressV1>,
}

impl<C> DependencyImpactControlV1<C>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    fn try_new(callback: C) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            work: LifecycleWorkControlV1::try_new(
                callback,
                DependencyImpactProgressV1::default(),
                LifecycleFailureClassV1::DependencyImpactCancelled,
                LifecycleFailureClassV1::DependencyImpactDeadlineExceeded,
            )?,
        })
    }

    fn visit_node(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.visited_nodes = progress
            .visited_nodes
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn visit_edge(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.visited_edges = progress
            .visited_edges
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn complete_candidate(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.completed_candidates = progress
            .completed_candidates
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.checkpoint_and_reset()
    }

    fn materialize_root_node(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.materialized_root_nodes = progress
            .materialized_root_nodes
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn materialize_node(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.materialized_nodes = progress
            .materialized_nodes
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.work.record_work()
    }

    fn materialize_edge(&mut self) -> Result<(), LifecycleFailureV1> {
        let progress = self.work.progress_mut();
        progress.materialized_edges = progress
            .materialized_edges
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

impl DependencyGraphV1 {
    /// Computes a complete batch or refuses at a bounded in-memory checkpoint.
    pub fn try_analyze_candidates<C>(
        &self,
        candidates: &[DependencyCandidateV1],
        now: LifecycleTimestampV1,
        control: C,
    ) -> Result<DependencyImpactBatchV1, LifecycleFailureV1>
    where
        C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
    {
        self.envelope().require_safe(now)?;
        if candidates.is_empty()
            || candidates.len() > LifecycleBoundsV1::MAX_DEPENDENCY_CANDIDATES_PER_ANALYSIS
        {
            return Err(lifecycle_bounds());
        }
        let mut control = DependencyImpactControlV1::try_new(control)?;
        let mut ordered_candidates = lifecycle_try_vec(candidates.len())?;
        for candidate in candidates {
            ordered_candidates.push(candidate);
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;
        ordered_candidates.sort_unstable_by_key(|candidate| candidate.identity_sha256());
        control.checkpoint_and_reset()?;
        for pair in ordered_candidates.windows(2) {
            if pair[0].identity_sha256() == pair[1].identity_sha256() {
                return Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::DuplicateIdentity,
                ));
            }
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;
        analyze_dependency_impact_batch(self, &ordered_candidates, &mut control)
    }
}
