const DEPENDENCY_IMPACT_CHECKPOINT_WORK_UNITS: u64 = 1_024;

/// Caller-owned stop decision at one bounded dependency-impact checkpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyImpactControlDecisionV1 {
    Continue,
    Cancel,
    DeadlineExceeded,
}

/// Monotonic progress exposed at bounded dependency-impact checkpoints.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DependencyImpactProgressV1 {
    completed_candidates: u64,
    visited_nodes: u64,
    visited_edges: u64,
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
}

struct DependencyImpactControlV1<C> {
    callback: C,
    progress: DependencyImpactProgressV1,
    work_since_checkpoint: u64,
}

impl<C> DependencyImpactControlV1<C>
where
    C: FnMut(DependencyImpactProgressV1) -> DependencyImpactControlDecisionV1,
{
    fn try_new(callback: C) -> Result<Self, LifecycleFailureV1> {
        let mut control = Self {
            callback,
            progress: DependencyImpactProgressV1::default(),
            work_since_checkpoint: 0,
        };
        control.checkpoint()?;
        Ok(control)
    }

    fn visit_node(&mut self) -> Result<(), LifecycleFailureV1> {
        self.progress.visited_nodes = self
            .progress
            .visited_nodes
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.record_work()
    }

    fn visit_edge(&mut self) -> Result<(), LifecycleFailureV1> {
        self.progress.visited_edges = self
            .progress
            .visited_edges
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.record_work()
    }

    fn complete_candidate(&mut self) -> Result<(), LifecycleFailureV1> {
        self.progress.completed_candidates = self
            .progress
            .completed_candidates
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.checkpoint()?;
        self.work_since_checkpoint = 0;
        Ok(())
    }

    fn record_work(&mut self) -> Result<(), LifecycleFailureV1> {
        self.work_since_checkpoint = self
            .work_since_checkpoint
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        if self.work_since_checkpoint < DEPENDENCY_IMPACT_CHECKPOINT_WORK_UNITS {
            return Ok(());
        }
        self.checkpoint()?;
        self.work_since_checkpoint = 0;
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<(), LifecycleFailureV1> {
        match (self.callback)(self.progress) {
            DependencyImpactControlDecisionV1::Continue => Ok(()),
            DependencyImpactControlDecisionV1::Cancel => Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DependencyImpactCancelled,
            )),
            DependencyImpactControlDecisionV1::DeadlineExceeded => {
                Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::DependencyImpactDeadlineExceeded,
                ))
            }
        }
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
        C: FnMut(DependencyImpactProgressV1) -> DependencyImpactControlDecisionV1,
    {
        self.envelope().require_safe(now)?;
        if candidates.is_empty()
            || candidates.len() > LifecycleBoundsV1::MAX_DEPENDENCY_CANDIDATES_PER_ANALYSIS
        {
            return Err(lifecycle_bounds());
        }
        let mut candidates: Vec<_> = candidates.iter().collect();
        candidates.sort_by_key(|candidate| candidate.identity_sha256());
        if candidates
            .windows(2)
            .any(|pair| pair[0].identity_sha256() == pair[1].identity_sha256())
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let mut control = DependencyImpactControlV1::try_new(control)?;
        let mut node_marks = vec![0_usize; self.nodes().len()];
        let mut edge_marks = vec![0_usize; self.edges().len()];
        let mut queue = Vec::new();
        let mut root_indices = Vec::new();
        let mut affected_node_indices = Vec::new();
        let mut affected_edge_indices = Vec::new();
        let mut total_nodes = 0_usize;
        let mut total_edges = 0_usize;
        let mut impacts = Vec::with_capacity(candidates.len());

        for (candidate_index, candidate) in candidates.into_iter().enumerate() {
            let roots = self.release_roots(candidate.current().identity_sha256());
            if roots.is_empty() {
                return Err(LifecycleFailureV1::new(
                    LifecycleFailureClassV1::MissingDependencyRoot,
                ));
            }
            let generation = candidate_index + 1;
            queue.clear();
            root_indices.clear();
            affected_node_indices.clear();
            affected_edge_indices.clear();
            for root in roots {
                root_indices.push(root.node_index);
                mark_dependency_node(
                    root.node_index,
                    generation,
                    &mut node_marks,
                    &mut affected_node_indices,
                    &mut queue,
                );
            }
            let mut cursor = 0_usize;
            while cursor < queue.len() {
                let dependency_index = queue[cursor];
                cursor += 1;
                control.visit_node()?;
                for position in self.reverse_range(dependency_index) {
                    let edge_index = self.reverse_edge(position);
                    control.visit_edge()?;
                    if edge_marks[edge_index] != generation {
                        edge_marks[edge_index] = generation;
                        affected_edge_indices.push(edge_index);
                    }
                    mark_dependency_node(
                        self.reverse_dependent(position),
                        generation,
                        &mut node_marks,
                        &mut affected_node_indices,
                        &mut queue,
                    );
                }
            }
            root_indices.sort_unstable();
            affected_node_indices.sort_unstable();
            affected_edge_indices.sort_unstable();
            total_nodes = checked_dependency_impact_total(
                total_nodes,
                affected_node_indices.len(),
                LifecycleBoundsV1::MAX_TOTAL_DEPENDENCY_IMPACT_NODES,
            )?;
            total_edges = checked_dependency_impact_total(
                total_edges,
                affected_edge_indices.len(),
                LifecycleBoundsV1::MAX_TOTAL_DEPENDENCY_IMPACT_EDGES,
            )?;
            impacts.push(DependencyImpactV1::try_from_indices(
                self,
                candidate,
                &root_indices,
                &affected_node_indices,
                &affected_edge_indices,
            )?);
            control.complete_candidate()?;
        }
        dependency_impact_batch(self, impacts)
    }
}
