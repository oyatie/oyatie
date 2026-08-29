pub(crate) fn dependency_impact_identity(
    graph_identity_sha256: DigestV1,
    fact_envelope_identity_sha256: DigestV1,
    candidate_identity_sha256: DigestV1,
    current_release_identity_sha256: DigestV1,
    root_nodes: &[DependencyGraphNodeV1],
    affected_nodes: &[DependencyGraphNodeV1],
    affected_edges: &[DependencyGraphEdgeV1],
) -> Result<DigestV1, LifecycleFailureV1> {
    let mut hash = CanonicalHasherV1::new(b"build.dependency-impact.v1\0");
    hash.digest(graph_identity_sha256);
    hash.digest(fact_envelope_identity_sha256);
    hash.digest(candidate_identity_sha256);
    hash.digest(current_release_identity_sha256);
    hash.u64(lifecycle_len(root_nodes.len())?);
    for node in root_nodes {
        hash.digest(node.identity_sha256());
    }
    hash.u64(lifecycle_len(affected_nodes.len())?);
    for node in affected_nodes {
        hash.digest(node.identity_sha256());
    }
    hash.u64(lifecycle_len(affected_edges.len())?);
    for edge in affected_edges {
        hash.digest(edge.identity_sha256());
    }
    Ok(hash.finish())
}
