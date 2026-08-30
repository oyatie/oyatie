pub(crate) struct DependencyImpactIdentityContextV1 {
    pub(crate) graph_identity_sha256: DigestV1,
    pub(crate) fact_envelope_identity_sha256: DigestV1,
    pub(crate) candidate_identity_sha256: DigestV1,
    pub(crate) current_release_identity_sha256: DigestV1,
}

pub(crate) fn dependency_impact_identity<C>(
    context: DependencyImpactIdentityContextV1,
    root_nodes: &[DependencyGraphNodeV1],
    affected_nodes: &[DependencyGraphNodeV1],
    affected_edges: &[DependencyGraphEdgeV1],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<DigestV1, LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let mut hash = CanonicalHasherV1::new(b"build.dependency-impact.v1\0");
    hash.digest(context.graph_identity_sha256);
    hash.digest(context.fact_envelope_identity_sha256);
    hash.digest(context.candidate_identity_sha256);
    hash.digest(context.current_release_identity_sha256);
    hash.u64(lifecycle_len(root_nodes.len())?);
    for node in root_nodes {
        hash.digest(node.identity_sha256());
        control.record_work()?;
    }
    hash.u64(lifecycle_len(affected_nodes.len())?);
    for node in affected_nodes {
        hash.digest(node.identity_sha256());
        control.record_work()?;
    }
    hash.u64(lifecycle_len(affected_edges.len())?);
    for edge in affected_edges {
        hash.digest(edge.identity_sha256());
        control.record_work()?;
    }
    Ok(hash.finish())
}
