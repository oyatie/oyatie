#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyImpactRangesV1 {
    root_nodes: std::ops::Range<usize>,
    affected_nodes: std::ops::Range<usize>,
    affected_edges: std::ops::Range<usize>,
}

/// Closure-complete impact for one candidate over shared graph storage.
#[derive(Clone)]
pub struct DependencyImpactV1 {
    storage: std::sync::Arc<DependencyImpactStorageV1>,
    candidate_identity_sha256: DigestV1,
    current_release_identity_sha256: DigestV1,
    ranges: DependencyImpactRangesV1,
    identity_sha256: DigestV1,
}

impl std::fmt::Debug for DependencyImpactV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DependencyImpactV1")
            .field("graph_identity_sha256", &self.graph_identity_sha256())
            .field(
                "fact_envelope_identity_sha256",
                &self.fact_envelope_identity_sha256(),
            )
            .field("candidate_identity_sha256", &self.candidate_identity_sha256)
            .field(
                "current_release_identity_sha256",
                &self.current_release_identity_sha256,
            )
            .field("root_node_count", &self.root_nodes().len())
            .field("affected_node_count", &self.affected_nodes().len())
            .field("affected_edge_count", &self.affected_edges().len())
            .field("identity_sha256", &self.identity_sha256)
            .finish()
    }
}

impl PartialEq for DependencyImpactV1 {
    fn eq(&self, other: &Self) -> bool {
        self.graph_identity_sha256() == other.graph_identity_sha256()
            && self.fact_envelope() == other.fact_envelope()
            && self.candidate_identity_sha256 == other.candidate_identity_sha256
            && self.current_release_identity_sha256 == other.current_release_identity_sha256
            && self.identity_sha256 == other.identity_sha256
            && self.root_nodes().iter().eq(other.root_nodes().iter())
            && self
                .affected_nodes()
                .iter()
                .eq(other.affected_nodes().iter())
            && self
                .affected_edges()
                .iter()
                .eq(other.affected_edges().iter())
    }
}

impl Eq for DependencyImpactV1 {}

impl DependencyImpactV1 {
    fn try_from_ranges<C>(
        storage: std::sync::Arc<DependencyImpactStorageV1>,
        candidate: &DependencyCandidateV1,
        ranges: DependencyImpactRangesV1,
        control: &mut DependencyImpactControlV1<C>,
    ) -> Result<Self, LifecycleFailureV1>
    where
        C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
    {
        let candidate_identity_sha256 = candidate.identity_sha256();
        let current_release_identity_sha256 = candidate.current().identity_sha256();
        let identity_context = DependencyImpactIdentityContextV1 {
            graph_identity_sha256: storage.graph_identity_sha256,
            fact_envelope_identity_sha256: storage.fact_envelope.identity_sha256(),
            candidate_identity_sha256,
            current_release_identity_sha256,
        };
        let identity_sha256 = dependency_impact_identity(
            identity_context,
            storage.root_nodes(ranges.root_nodes.clone()),
            storage.affected_nodes(ranges.affected_nodes.clone()),
            storage.affected_edges(ranges.affected_edges.clone()),
            control,
        )?;
        Ok(Self {
            storage,
            candidate_identity_sha256,
            current_release_identity_sha256,
            ranges,
            identity_sha256,
        })
    }

    #[must_use]
    pub fn graph_identity_sha256(&self) -> DigestV1 {
        self.storage.graph_identity_sha256
    }

    #[must_use]
    pub fn fact_envelope_identity_sha256(&self) -> DigestV1 {
        self.storage.fact_envelope.identity_sha256()
    }

    #[must_use]
    pub fn fact_envelope(&self) -> &FactEnvelopeV1 {
        &self.storage.fact_envelope
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
    pub fn root_nodes(&self) -> DependencyImpactNodesV1<'_> {
        self.storage.root_nodes(self.ranges.root_nodes.clone())
    }

    #[must_use]
    pub fn affected_nodes(&self) -> DependencyImpactNodesV1<'_> {
        self.storage
            .affected_nodes(self.ranges.affected_nodes.clone())
    }

    #[must_use]
    pub fn affected_edges(&self) -> DependencyImpactEdgesV1<'_> {
        self.storage
            .affected_edges(self.ranges.affected_edges.clone())
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Canonical candidate batch over one shared immutable dependency graph.
#[derive(Clone)]
pub struct DependencyImpactBatchV1 {
    storage: std::sync::Arc<DependencyImpactStorageV1>,
    impacts: Box<[DependencyImpactV1]>,
    identity_sha256: DigestV1,
}

impl std::fmt::Debug for DependencyImpactBatchV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DependencyImpactBatchV1")
            .field("graph_identity_sha256", &self.graph_identity_sha256())
            .field(
                "fact_envelope_identity_sha256",
                &self.fact_envelope_identity_sha256(),
            )
            .field("impact_count", &self.impacts.len())
            .field("selection_bytes", &self.selection_bytes())
            .field(
                "retained_bytes_upper_bound",
                &self.retained_bytes_upper_bound(),
            )
            .field("identity_sha256", &self.identity_sha256)
            .finish()
    }
}

impl PartialEq for DependencyImpactBatchV1 {
    fn eq(&self, other: &Self) -> bool {
        self.graph_identity_sha256() == other.graph_identity_sha256()
            && self.fact_envelope() == other.fact_envelope()
            && self.selection_bytes() == other.selection_bytes()
            && self.identity_sha256 == other.identity_sha256
            && self.impacts == other.impacts
    }
}

impl Eq for DependencyImpactBatchV1 {}

impl DependencyImpactBatchV1 {
    #[must_use]
    pub fn impacts(&self) -> &[DependencyImpactV1] {
        &self.impacts
    }

    #[must_use]
    pub fn graph_identity_sha256(&self) -> DigestV1 {
        self.storage.graph_identity_sha256
    }

    #[must_use]
    pub fn fact_envelope_identity_sha256(&self) -> DigestV1 {
        self.storage.fact_envelope.identity_sha256()
    }

    #[must_use]
    pub fn fact_envelope(&self) -> &FactEnvelopeV1 {
        &self.storage.fact_envelope
    }

    #[must_use]
    pub fn selection_bytes(&self) -> usize {
        self.storage.selection_bytes
    }

    #[must_use]
    pub fn retained_bytes_upper_bound(&self) -> usize {
        self.storage.retained_bytes_upper_bound
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn dependency_impact_batch<C>(
    storage: std::sync::Arc<DependencyImpactStorageV1>,
    impacts: Vec<DependencyImpactV1>,
    control: &mut DependencyImpactControlV1<C>,
) -> Result<DependencyImpactBatchV1, LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let mut hash = CanonicalHasherV1::new(b"build.dependency-impact-batch.v1\0");
    hash.digest(storage.graph_identity_sha256);
    hash.digest(storage.fact_envelope.identity_sha256());
    hash.u64(lifecycle_len(impacts.len())?);
    for impact in &impacts {
        hash.digest(impact.identity_sha256());
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;
    Ok(DependencyImpactBatchV1 {
        storage,
        impacts: impacts.into_boxed_slice(),
        identity_sha256: hash.finish(),
    })
}
