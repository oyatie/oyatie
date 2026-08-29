#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DependencyReleaseRootV1 {
    release_identity_sha256: DigestV1,
    node_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReverseDependencyIndexV1 {
    offsets: Box<[usize]>,
    dependents: Box<[usize]>,
    edges: Box<[usize]>,
}

/// Canonical bounded graph with one reverse adjacency index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyGraphV1 {
    envelope: FactEnvelopeV1,
    nodes: Box<[DependencyGraphNodeV1]>,
    edges: Box<[DependencyGraphEdgeV1]>,
    reverse: ReverseDependencyIndexV1,
    release_roots: Box<[DependencyReleaseRootV1]>,
    identity_sha256: DigestV1,
}

impl DependencyGraphV1 {
    pub fn try_new(
        envelope: FactEnvelopeV1,
        mut nodes: Vec<DependencyGraphNodeV1>,
        mut edges: Vec<DependencyGraphEdgeV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_dependency_graph_bounds(&nodes, &edges)?;
        nodes.sort_by_key(DependencyGraphNodeV1::unit_identity_sha256);
        if nodes.windows(2).any(|pair| {
            pair[0].unit_identity_sha256() == pair[1].unit_identity_sha256()
        }) || nodes.iter().any(|node| {
            node.package_release_identity_sha256().is_some()
                && node.kind() != DependencyGraphNodeKindV1::CargoPackage
        }) {
            return Err(invalid_dependency_graph());
        }
        edges.sort_by_key(DependencyGraphEdgeV1::semantic_key);
        if edges
            .windows(2)
            .any(|pair| pair[0].semantic_key() == pair[1].semantic_key())
        {
            return Err(invalid_dependency_graph());
        }

        let mut node_indices = std::collections::HashMap::with_capacity(nodes.len());
        for (index, node) in nodes.iter().enumerate() {
            node_indices.insert(node.unit_identity_sha256(), index);
        }
        let reverse = build_reverse_index(&nodes, &edges, &node_indices)?;
        let release_roots = dependency_release_roots(&nodes);
        let identity_sha256 = dependency_graph_identity(&envelope, &nodes, &edges)?;
        Ok(Self {
            envelope,
            nodes: nodes.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
            reverse,
            release_roots,
            identity_sha256,
        })
    }

    #[must_use]
    pub const fn envelope(&self) -> &FactEnvelopeV1 {
        &self.envelope
    }

    #[must_use]
    pub fn nodes(&self) -> &[DependencyGraphNodeV1] {
        &self.nodes
    }

    #[must_use]
    pub fn edges(&self) -> &[DependencyGraphEdgeV1] {
        &self.edges
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }

    pub(crate) fn release_roots(
        &self,
        release_identity_sha256: DigestV1,
    ) -> &[DependencyReleaseRootV1] {
        let start = self
            .release_roots
            .partition_point(|root| root.release_identity_sha256 < release_identity_sha256);
        let end = self
            .release_roots
            .partition_point(|root| root.release_identity_sha256 <= release_identity_sha256);
        &self.release_roots[start..end]
    }

    pub(crate) fn reverse_range(&self, dependency_index: usize) -> std::ops::Range<usize> {
        self.reverse.offsets[dependency_index]..self.reverse.offsets[dependency_index + 1]
    }

    pub(crate) const fn reverse_dependent(&self, position: usize) -> usize {
        self.reverse.dependents[position]
    }

    pub(crate) const fn reverse_edge(&self, position: usize) -> usize {
        self.reverse.edges[position]
    }
}

fn validate_dependency_graph_bounds(
    nodes: &[DependencyGraphNodeV1],
    edges: &[DependencyGraphEdgeV1],
) -> Result<(), LifecycleFailureV1> {
    if nodes.is_empty() {
        return Err(invalid_dependency_graph());
    }
    if nodes.len() > LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_NODES
        || edges.len() > LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_EDGES
    {
        return Err(lifecycle_bounds());
    }
    Ok(())
}

type DependencyNodeIndexV1 = std::collections::HashMap<DigestV1, usize>;

fn build_reverse_index(
    nodes: &[DependencyGraphNodeV1],
    edges: &[DependencyGraphEdgeV1],
    node_indices: &DependencyNodeIndexV1,
) -> Result<ReverseDependencyIndexV1, LifecycleFailureV1> {
    let mut counts = vec![0_usize; nodes.len()];
    let mut endpoints = Vec::with_capacity(edges.len());
    for edge in edges {
        let dependent = node_indices
            .get(&edge.dependent_unit_sha256())
            .copied()
            .ok_or_else(invalid_dependency_graph)?;
        let dependency = node_indices
            .get(&edge.dependency_unit_sha256())
            .copied()
            .ok_or_else(invalid_dependency_graph)?;
        counts[dependency] = counts[dependency]
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        endpoints.push((dependent, dependency));
    }

    let capacity = nodes.len().checked_add(1).ok_or_else(lifecycle_bounds)?;
    let mut offsets = Vec::with_capacity(capacity);
    offsets.push(0_usize);
    for count in counts {
        let next = offsets
            .last()
            .copied()
            .and_then(|offset| offset.checked_add(count))
            .ok_or_else(lifecycle_bounds)?;
        offsets.push(next);
    }
    if offsets.last().copied() != Some(edges.len()) {
        return Err(lifecycle_internal());
    }

    let mut cursors = offsets[..nodes.len()].to_vec();
    let mut dependents = vec![0_usize; edges.len()];
    let mut edge_indices = vec![0_usize; edges.len()];
    for (edge_index, (dependent, dependency)) in endpoints.into_iter().enumerate() {
        let position = cursors[dependency];
        dependents[position] = dependent;
        edge_indices[position] = edge_index;
        cursors[dependency] = position.checked_add(1).ok_or_else(lifecycle_bounds)?;
    }
    Ok(ReverseDependencyIndexV1 {
        offsets: offsets.into_boxed_slice(),
        dependents: dependents.into_boxed_slice(),
        edges: edge_indices.into_boxed_slice(),
    })
}

fn dependency_release_roots(
    nodes: &[DependencyGraphNodeV1],
) -> Box<[DependencyReleaseRootV1]> {
    let mut roots: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter_map(|(node_index, node)| {
            node.package_release_identity_sha256()
                .map(|release_identity_sha256| DependencyReleaseRootV1 {
                    release_identity_sha256,
                    node_index,
                })
        })
        .collect();
    roots.sort_by_key(|root| (root.release_identity_sha256, root.node_index));
    roots.into_boxed_slice()
}

fn dependency_graph_identity(
    envelope: &FactEnvelopeV1,
    nodes: &[DependencyGraphNodeV1],
    edges: &[DependencyGraphEdgeV1],
) -> Result<DigestV1, LifecycleFailureV1> {
    let mut hash = CanonicalHasherV1::new(b"build.dependency-graph.v1\0");
    hash.digest(envelope.identity_sha256());
    hash.u64(lifecycle_len(nodes.len())?);
    for node in nodes {
        hash.digest(node.identity_sha256());
    }
    hash.u64(lifecycle_len(edges.len())?);
    for edge in edges {
        hash.digest(edge.identity_sha256());
    }
    Ok(hash.finish())
}
