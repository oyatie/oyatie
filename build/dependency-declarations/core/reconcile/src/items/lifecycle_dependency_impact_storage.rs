struct DependencyImpactStorageV1 {
    graph_identity_sha256: DigestV1,
    fact_envelope: std::sync::Arc<FactEnvelopeV1>,
    graph_nodes: Vec<DependencyGraphNodeV1>,
    graph_edges: Vec<DependencyGraphEdgeV1>,
    root_node_indices: Vec<u32>,
    affected_node_indices: Vec<u32>,
    affected_edge_indices: Vec<u32>,
    selection_bytes: usize,
    retained_bytes_upper_bound: usize,
}

impl DependencyImpactStorageV1 {
    fn new(
        graph: &DependencyGraphV1,
        selections: DependencyImpactSelectionsV1,
        selection_bytes: usize,
        retained_bytes_upper_bound: usize,
    ) -> Self {
        Self {
            graph_identity_sha256: graph.identity_sha256(),
            fact_envelope: graph.shared_envelope(),
            graph_nodes: selections.graph_nodes,
            graph_edges: selections.graph_edges,
            root_node_indices: selections.root_nodes,
            affected_node_indices: selections.affected_nodes,
            affected_edge_indices: selections.affected_edges,
            selection_bytes,
            retained_bytes_upper_bound,
        }
    }

    fn root_nodes(&self, range: std::ops::Range<usize>) -> DependencyImpactNodesV1<'_> {
        DependencyImpactNodesV1::new(&self.graph_nodes, &self.root_node_indices[range])
    }

    fn affected_nodes(&self, range: std::ops::Range<usize>) -> DependencyImpactNodesV1<'_> {
        DependencyImpactNodesV1::new(&self.graph_nodes, &self.affected_node_indices[range])
    }

    fn affected_edges(&self, range: std::ops::Range<usize>) -> DependencyImpactEdgesV1<'_> {
        DependencyImpactEdgesV1::new(&self.graph_edges, &self.affected_edge_indices[range])
    }
}

/// Borrowed node selection over one shared immutable dependency graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencyImpactNodesV1<'a> {
    nodes: &'a [DependencyGraphNodeV1],
    indices: &'a [u32],
}

impl<'a> DependencyImpactNodesV1<'a> {
    fn new(nodes: &'a [DependencyGraphNodeV1], indices: &'a [u32]) -> Self {
        Self { nodes, indices }
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.indices.len()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.indices.is_empty()
    }

    pub fn iter(
        self,
    ) -> impl DoubleEndedIterator<Item = &'a DependencyGraphNodeV1> + ExactSizeIterator + 'a {
        self.indices
            .iter()
            .map(move |index| &self.nodes[*index as usize])
    }
}

/// Borrowed edge selection over one shared immutable dependency graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencyImpactEdgesV1<'a> {
    edges: &'a [DependencyGraphEdgeV1],
    indices: &'a [u32],
}

impl<'a> DependencyImpactEdgesV1<'a> {
    fn new(edges: &'a [DependencyGraphEdgeV1], indices: &'a [u32]) -> Self {
        Self { edges, indices }
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.indices.len()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.indices.is_empty()
    }

    pub fn iter(
        self,
    ) -> impl DoubleEndedIterator<Item = &'a DependencyGraphEdgeV1> + ExactSizeIterator + 'a {
        self.indices
            .iter()
            .map(move |index| &self.edges[*index as usize])
    }
}
