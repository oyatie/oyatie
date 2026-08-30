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
    fn try_from_indices<C>(
        graph: &DependencyGraphV1,
        candidate: &DependencyCandidateV1,
        root_indices: &[usize],
        affected_node_indices: &[usize],
        affected_edge_indices: &[usize],
        control: &mut DependencyImpactControlV1<C>,
    ) -> Result<Self, LifecycleFailureV1>
    where
        C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
    {
        let mut root_nodes = Vec::with_capacity(root_indices.len());
        for index in root_indices {
            root_nodes.push(graph.nodes()[*index].clone());
            control.materialize_root_node()?;
        }
        control.checkpoint_and_reset()?;
        let mut affected_nodes = Vec::with_capacity(affected_node_indices.len());
        for index in affected_node_indices {
            affected_nodes.push(graph.nodes()[*index].clone());
            control.materialize_node()?;
        }
        control.checkpoint_and_reset()?;
        let mut affected_edges = Vec::with_capacity(affected_edge_indices.len());
        for index in affected_edge_indices {
            affected_edges.push(graph.edges()[*index].clone());
            control.materialize_edge()?;
        }
        control.checkpoint_and_reset()?;
        let graph_identity_sha256 = graph.identity_sha256();
        let fact_envelope = graph.envelope().clone();
        let candidate_identity_sha256 = candidate.identity_sha256();
        let current_release_identity_sha256 = candidate.current().identity_sha256();
        let identity_context = DependencyImpactIdentityContextV1 {
            graph_identity_sha256,
            fact_envelope_identity_sha256: fact_envelope.identity_sha256(),
            candidate_identity_sha256,
            current_release_identity_sha256,
        };
        let identity_sha256 = dependency_impact_identity(
            identity_context,
            &root_nodes,
            &affected_nodes,
            &affected_edges,
            control,
        )?;
        Ok(Self {
            graph_identity_sha256,
            fact_envelope,
            candidate_identity_sha256,
            current_release_identity_sha256,
            root_nodes: root_nodes.into_boxed_slice(),
            affected_nodes: affected_nodes.into_boxed_slice(),
            affected_edges: affected_edges.into_boxed_slice(),
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

fn dependency_impact_batch<C>(
    graph: &DependencyGraphV1,
    impacts: Vec<DependencyImpactV1>,
    control: &mut DependencyImpactControlV1<C>,
) -> Result<DependencyImpactBatchV1, LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let mut hash = CanonicalHasherV1::new(b"build.dependency-impact-batch.v1\0");
    hash.digest(graph.identity_sha256());
    hash.digest(graph.envelope().identity_sha256());
    hash.u64(lifecycle_len(impacts.len())?);
    for impact in &impacts {
        hash.digest(impact.identity_sha256());
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;
    Ok(DependencyImpactBatchV1 {
        graph_identity_sha256: graph.identity_sha256(),
        fact_envelope: graph.envelope().clone(),
        impacts: impacts.into_boxed_slice(),
        identity_sha256: hash.finish(),
    })
}
