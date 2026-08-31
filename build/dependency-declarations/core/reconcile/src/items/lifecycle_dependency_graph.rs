#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DependencyReleaseRootV1 {
    release_identity_sha256: DigestV1,
    node_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReverseDependencyIndexV1 {
    offsets: Box<[usize]>,
    dependents: Box<[usize]>,
    dependencies_by_edge: Box<[u32]>,
}

/// Canonical bounded graph with shared immutable indexes.
#[derive(Clone)]
pub struct DependencyGraphV1 {
    envelope: std::sync::Arc<FactEnvelopeV1>,
    nodes: std::sync::Arc<[DependencyGraphNodeV1]>,
    edges: std::sync::Arc<[DependencyGraphEdgeV1]>,
    reverse: std::sync::Arc<ReverseDependencyIndexV1>,
    closure: std::sync::Arc<DependencyClosureIndexV1>,
    release_roots: std::sync::Arc<[DependencyReleaseRootV1]>,
    identity_sha256: DigestV1,
    retained_bytes_upper_bound: usize,
}

impl std::fmt::Debug for DependencyGraphV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DependencyGraphV1")
            .field("fact_envelope_identity_sha256", &self.envelope.identity_sha256())
            .field("node_count", &self.nodes.len())
            .field("edge_count", &self.edges.len())
            .field("component_count", &self.closure.component_count())
            .field("release_root_count", &self.release_roots.len())
            .field("identity_sha256", &self.identity_sha256)
            .field("retained_bytes_upper_bound", &self.retained_bytes_upper_bound)
            .finish()
    }
}

impl PartialEq for DependencyGraphV1 {
    fn eq(&self, other: &Self) -> bool {
        self.identity_sha256 == other.identity_sha256
            && self.envelope == other.envelope
            && self.nodes == other.nodes
            && self.edges == other.edges
    }
}

impl Eq for DependencyGraphV1 {}

impl DependencyGraphV1 {
    pub fn try_new<C>(
        envelope: FactEnvelopeV1,
        mut nodes: Vec<DependencyGraphNodeV1>,
        mut edges: Vec<DependencyGraphEdgeV1>,
        control: C,
    ) -> Result<Self, LifecycleFailureV1>
    where
        C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
    {
        validate_dependency_graph_bounds(&nodes, &edges)?;
        let retained_bytes_upper_bound = validate_dependency_graph_memory(&nodes, &edges)?;
        let mut control = DependencyGraphConstructionControlV1::try_new(control)?;
        nodes.sort_unstable_by_key(DependencyGraphNodeV1::unit_identity_sha256);
        control.checkpoint_and_reset()?;
        for (index, node) in nodes.iter().enumerate() {
            let duplicate = index > 0
                && nodes[index - 1].unit_identity_sha256() == node.unit_identity_sha256();
            let misplaced_release = node.package_release_identity_sha256().is_some()
                && node.kind() != DependencyGraphNodeKindV1::CargoPackage;
            if duplicate || misplaced_release {
                return Err(invalid_dependency_graph());
            }
            control.complete_node()?;
        }
        control.checkpoint_and_reset()?;
        edges.sort_unstable_by_key(DependencyGraphEdgeV1::semantic_key);
        control.checkpoint_and_reset()?;
        for (index, edge) in edges.iter().enumerate() {
            if index > 0 && edges[index - 1].semantic_key() == edge.semantic_key() {
                return Err(invalid_dependency_graph());
            }
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;

        let mut node_indices = std::collections::HashMap::new();
        node_indices
            .try_reserve(nodes.len())
            .map_err(|_| lifecycle_bounds())?;
        for (index, node) in nodes.iter().enumerate() {
            node_indices.insert(node.unit_identity_sha256(), index);
            control.record_work()?;
        }
        control.checkpoint_and_reset()?;
        let reverse = std::sync::Arc::new(build_reverse_index(
            &nodes,
            &edges,
            &node_indices,
            &mut control,
        )?);
        let closure = std::sync::Arc::new(build_dependency_closure_index(
            nodes.len(),
            &reverse,
            &mut control,
        )?);
        let release_roots = dependency_release_roots(&nodes, &mut control)?;
        let identity_sha256 =
            dependency_graph_identity(&envelope, &nodes, &edges, &mut control)?;
        control.checkpoint_and_reset()?;
        Ok(Self {
            envelope: std::sync::Arc::new(envelope),
            nodes: std::sync::Arc::from(nodes.into_boxed_slice()),
            edges: std::sync::Arc::from(edges.into_boxed_slice()),
            reverse,
            closure,
            release_roots: std::sync::Arc::from(release_roots),
            identity_sha256,
            retained_bytes_upper_bound,
        })
    }

    #[must_use]
    pub fn envelope(&self) -> &FactEnvelopeV1 {
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

    #[must_use]
    pub const fn retained_bytes_upper_bound(&self) -> usize {
        self.retained_bytes_upper_bound
    }

    pub(crate) fn shared_envelope(&self) -> std::sync::Arc<FactEnvelopeV1> {
        std::sync::Arc::clone(&self.envelope)
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

    pub(crate) fn edge_dependency_node(&self, edge_index: usize) -> usize {
        self.reverse.dependencies_by_edge[edge_index] as usize
    }

    pub(crate) fn closure_index(&self) -> &DependencyClosureIndexV1 {
        &self.closure
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

fn build_reverse_index<C>(
    nodes: &[DependencyGraphNodeV1],
    edges: &[DependencyGraphEdgeV1],
    node_indices: &DependencyNodeIndexV1,
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<ReverseDependencyIndexV1, LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let mut counts = lifecycle_try_filled_vec(nodes.len(), 0_usize)?;
    let mut endpoints = lifecycle_try_vec(edges.len())?;
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
        control.complete_edge()?;
    }
    control.checkpoint_and_reset()?;

    let capacity = nodes.len().checked_add(1).ok_or_else(lifecycle_bounds)?;
    let mut offsets = lifecycle_try_vec(capacity)?;
    offsets.push(0_usize);
    for count in counts {
        let next = offsets
            .last()
            .copied()
            .and_then(|offset| offset.checked_add(count))
            .ok_or_else(lifecycle_bounds)?;
        offsets.push(next);
        control.record_work()?;
    }
    if offsets.last().copied() != Some(edges.len()) {
        return Err(lifecycle_internal());
    }

    let mut cursors = lifecycle_try_vec(nodes.len())?;
    cursors.extend_from_slice(&offsets[..nodes.len()]);
    let mut dependents = lifecycle_try_filled_vec(edges.len(), 0_usize)?;
    let mut dependencies_by_edge = lifecycle_try_filled_vec(edges.len(), 0_u32)?;
    for (edge_index, (dependent, dependency)) in endpoints.into_iter().enumerate() {
        let position = cursors[dependency];
        dependents[position] = dependent;
        dependencies_by_edge[edge_index] = lifecycle_u32_index(dependency)?;
        cursors[dependency] = position.checked_add(1).ok_or_else(lifecycle_bounds)?;
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;
    Ok(ReverseDependencyIndexV1 {
        offsets: offsets.into_boxed_slice(),
        dependents: dependents.into_boxed_slice(),
        dependencies_by_edge: dependencies_by_edge.into_boxed_slice(),
    })
}

fn dependency_release_roots<C>(
    nodes: &[DependencyGraphNodeV1],
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<Box<[DependencyReleaseRootV1]>, LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let mut roots = lifecycle_try_vec(nodes.len())?;
    for (node_index, node) in nodes.iter().enumerate() {
        if let Some(release_identity_sha256) = node.package_release_identity_sha256() {
            roots.push(DependencyReleaseRootV1 {
                release_identity_sha256,
                node_index,
            });
        }
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;
    roots.sort_unstable_by_key(|root| (root.release_identity_sha256, root.node_index));
    control.checkpoint_and_reset()?;
    Ok(roots.into_boxed_slice())
}

fn dependency_graph_identity<C>(
    envelope: &FactEnvelopeV1,
    nodes: &[DependencyGraphNodeV1],
    edges: &[DependencyGraphEdgeV1],
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<DigestV1, LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let mut hash = CanonicalHasherV1::new(b"build.dependency-graph.v1\0");
    hash.digest(envelope.identity_sha256());
    hash.u64(lifecycle_len(nodes.len())?);
    for node in nodes {
        hash.digest(node.identity_sha256());
        control.record_work()?;
    }
    hash.u64(lifecycle_len(edges.len())?);
    for edge in edges {
        hash.digest(edge.identity_sha256());
        control.record_work()?;
    }
    Ok(hash.finish())
}
