/// Closure-complete in-memory impact for one dependency candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyImpactV1 {
    graph_identity_sha256: DigestV1,
    fact_envelope: FactEnvelopeV1,
    candidate_identity_sha256: DigestV1,
    current_release_identity_sha256: DigestV1,
    root_nodes: Box<[DependencyGraphNodeV1]>,
    affected_nodes: Box<[DependencyGraphNodeV1]>,
    affected_edges: Box<[DependencyGraphEdgeV1]>,
    identity_sha256: DigestV1,
}

impl DependencyImpactV1 {
    fn try_from_indices(
        graph: &DependencyGraphV1,
        candidate: &DependencyCandidateV1,
        root_indices: &[usize],
        affected_node_indices: &[usize],
        affected_edge_indices: &[usize],
    ) -> Result<Self, LifecycleFailureV1> {
        let root_nodes: Box<_> = root_indices
            .iter()
            .map(|index| graph.nodes()[*index].clone())
            .collect();
        let affected_nodes: Box<_> = affected_node_indices
            .iter()
            .map(|index| graph.nodes()[*index].clone())
            .collect();
        let affected_edges: Box<_> = affected_edge_indices
            .iter()
            .map(|index| graph.edges()[*index].clone())
            .collect();
        let graph_identity_sha256 = graph.identity_sha256();
        let fact_envelope = graph.envelope().clone();
        let candidate_identity_sha256 = candidate.identity_sha256();
        let current_release_identity_sha256 = candidate.current().identity_sha256();
        let identity_sha256 = dependency_impact_identity(
            graph_identity_sha256,
            fact_envelope.identity_sha256(),
            candidate_identity_sha256,
            current_release_identity_sha256,
            &root_nodes,
            &affected_nodes,
            &affected_edges,
        )?;
        Ok(Self {
            graph_identity_sha256,
            fact_envelope,
            candidate_identity_sha256,
            current_release_identity_sha256,
            root_nodes,
            affected_nodes,
            affected_edges,
            identity_sha256,
        })
    }

    #[must_use]
    pub const fn graph_identity_sha256(&self) -> DigestV1 {
        self.graph_identity_sha256
    }

    #[must_use]
    pub const fn fact_envelope_identity_sha256(&self) -> DigestV1 {
        self.fact_envelope.identity_sha256()
    }

    #[must_use]
    pub const fn fact_envelope(&self) -> &FactEnvelopeV1 {
        &self.fact_envelope
    }

    #[must_use]
    pub const fn candidate_identity_sha256(&self) -> DigestV1 {
        self.candidate_identity_sha256
    }

    #[must_use]
    pub const fn current_release_identity_sha256(&self) -> DigestV1 {
        self.current_release_identity_sha256
    }

    #[must_use]
    pub fn root_nodes(&self) -> &[DependencyGraphNodeV1] {
        &self.root_nodes
    }

    #[must_use]
    pub fn affected_nodes(&self) -> &[DependencyGraphNodeV1] {
        &self.affected_nodes
    }

    #[must_use]
    pub fn affected_edges(&self) -> &[DependencyGraphEdgeV1] {
        &self.affected_edges
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Canonical batch result over one already-materialized graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyImpactBatchV1 {
    graph_identity_sha256: DigestV1,
    fact_envelope: FactEnvelopeV1,
    impacts: Box<[DependencyImpactV1]>,
    identity_sha256: DigestV1,
}

impl DependencyImpactBatchV1 {
    #[must_use]
    pub fn impacts(&self) -> &[DependencyImpactV1] {
        &self.impacts
    }

    #[must_use]
    pub const fn graph_identity_sha256(&self) -> DigestV1 {
        self.graph_identity_sha256
    }

    #[must_use]
    pub const fn fact_envelope_identity_sha256(&self) -> DigestV1 {
        self.fact_envelope.identity_sha256()
    }

    #[must_use]
    pub const fn fact_envelope(&self) -> &FactEnvelopeV1 {
        &self.fact_envelope
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

impl DependencyGraphV1 {
    pub fn try_analyze_candidates(
        &self,
        candidates: &[DependencyCandidateV1],
        now: LifecycleTimestampV1,
    ) -> Result<DependencyImpactBatchV1, LifecycleFailureV1> {
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
                for position in self.reverse_range(dependency_index) {
                    let edge_index = self.reverse_edge(position);
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
        }
        dependency_impact_batch(self, impacts)
    }
}

fn mark_dependency_node(
    node_index: usize,
    generation: usize,
    node_marks: &mut [usize],
    affected_node_indices: &mut Vec<usize>,
    queue: &mut Vec<usize>,
) {
    if node_marks[node_index] != generation {
        node_marks[node_index] = generation;
        affected_node_indices.push(node_index);
        queue.push(node_index);
    }
}

fn checked_dependency_impact_total(
    current: usize,
    additional: usize,
    limit: usize,
) -> Result<usize, LifecycleFailureV1> {
    current
        .checked_add(additional)
        .filter(|total| *total <= limit)
        .ok_or_else(lifecycle_bounds)
}

fn dependency_impact_batch(
    graph: &DependencyGraphV1,
    impacts: Vec<DependencyImpactV1>,
) -> Result<DependencyImpactBatchV1, LifecycleFailureV1> {
    let mut hash = CanonicalHasherV1::new(b"build.dependency-impact-batch.v1\0");
    hash.digest(graph.identity_sha256());
    hash.digest(graph.envelope().identity_sha256());
    hash.u64(lifecycle_len(impacts.len())?);
    for impact in &impacts {
        hash.digest(impact.identity_sha256());
    }
    Ok(DependencyImpactBatchV1 {
        graph_identity_sha256: graph.identity_sha256(),
        fact_envelope: graph.envelope().clone(),
        impacts: impacts.into_boxed_slice(),
        identity_sha256: hash.finish(),
    })
}
