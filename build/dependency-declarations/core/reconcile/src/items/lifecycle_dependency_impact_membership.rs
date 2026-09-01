struct DependencyCandidateMembershipIterV1<'a> {
    words: &'a [u64],
    word_index: usize,
    active_word: u64,
    candidate_count: usize,
}

impl<'a> DependencyCandidateMembershipIterV1<'a> {
    fn new(words: &'a [u64], candidate_count: usize) -> Self {
        Self {
            words,
            word_index: 0,
            active_word: 0,
            candidate_count,
        }
    }
}

impl Iterator for DependencyCandidateMembershipIterV1<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.active_word != 0 {
                let bit = self.active_word.trailing_zeros() as usize;
                self.active_word &= self.active_word - 1;
                let candidate = (self.word_index - 1) * u64::BITS as usize + bit;
                return (candidate < self.candidate_count).then_some(candidate);
            }
            self.active_word = *self.words.get(self.word_index)?;
            self.word_index += 1;
        }
    }
}

fn dependency_candidate_memberships<C>(
    graph: &DependencyGraphV1,
    candidates: &[&DependencyCandidateV1],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<(usize, Vec<u64>, usize), LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let word_count = candidates
        .len()
        .checked_add(u64::BITS as usize - 1)
        .ok_or_else(lifecycle_bounds)?
        / u64::BITS as usize;
    let membership_count = graph
        .closure_index()
        .component_count()
        .checked_mul(word_count)
        .ok_or_else(lifecycle_bounds)?;
    let membership_bytes = lifecycle_allocation_bytes::<u64>(membership_count)?;
    if membership_bytes > LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_WORKING_BYTES {
        return Err(lifecycle_bounds());
    }
    let mut memberships = lifecycle_try_filled_vec(membership_count, 0_u64)?;
    seed_dependency_candidates(graph, candidates, word_count, &mut memberships, control)?;
    propagate_dependency_candidates(graph, word_count, &mut memberships, control)?;
    Ok((word_count, memberships, membership_bytes))
}

fn seed_dependency_candidates<C>(
    graph: &DependencyGraphV1,
    candidates: &[&DependencyCandidateV1],
    word_count: usize,
    memberships: &mut [u64],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        let roots = graph.release_roots(candidate.current().identity_sha256());
        if roots.is_empty() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::MissingDependencyRoot,
            ));
        }
        let word = candidate_index / u64::BITS as usize;
        let bit = candidate_index % u64::BITS as usize;
        for root in roots {
            let component = graph.closure_index().component_for_node(root.node_index);
            memberships[component * word_count + word] |= 1_u64 << bit;
            control.record_work()?;
        }
    }
    control.checkpoint_and_reset()
}

fn propagate_dependency_candidates<C>(
    graph: &DependencyGraphV1,
    word_count: usize,
    memberships: &mut [u64],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<(), LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    for component in graph.closure_index().topological_components() {
        let component = *component as usize;
        let source = component * word_count;
        if memberships[source..source + word_count]
            .iter()
            .all(|word| *word == 0)
        {
            control.record_work()?;
            continue;
        }
        for dependent in graph.closure_index().dependent_components(component) {
            let destination = *dependent as usize * word_count;
            for word in 0..word_count {
                memberships[destination + word] |= memberships[source + word];
                control.record_work()?;
            }
        }
    }
    control.checkpoint_and_reset()
}

fn dependency_node_membership<'a>(
    graph: &DependencyGraphV1,
    node_index: usize,
    word_count: usize,
    memberships: &'a [u64],
) -> &'a [u64] {
    let component = graph.closure_index().component_for_node(node_index);
    let start = component * word_count;
    &memberships[start..start + word_count]
}
