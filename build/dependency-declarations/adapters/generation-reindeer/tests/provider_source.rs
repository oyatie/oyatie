#[path = "provider_source/artifact.rs"]
mod artifact;
#[path = "provider_source/build.rs"]
mod build;
#[path = "provider_source/cargo_build.rs"]
mod cargo_build;
#[path = "provider_source/collision.rs"]
mod collision;
#[path = "provider_source/identity.rs"]
mod identity;
#[path = "provider_source/qualification.rs"]
mod qualification;
#[path = "provider_source/snapshot.rs"]
mod snapshot;
#[path = "provider_source/support.rs"]
mod support;
#[path = "provider_source/workspace_roots.rs"]
mod workspace_roots;

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn exact_pinned_upstream_source_is_qualified_once() {
    let snapshot = support::source_snapshot(&support::pinned_source_root());
    identity::exact_source_batch_produces_one_deterministic_adaptation(&snapshot);
    identity::source_discovery_order_does_not_change_the_adaptation(&snapshot);
    identity::exact_revision_with_changed_source_bytes_refuses(&snapshot);
    identity::exact_revision_with_changed_unadapted_source_refuses(&snapshot);
    #[cfg(unix)]
    identity::exact_revision_with_unadapted_mode_change_refuses(&snapshot);
    identity::unsupported_revision_batch_and_presence_refuse(&snapshot);
    identity::oversized_source_refuses_before_digest_comparison(&snapshot);

    let provider = support::qualify_provider(&snapshot);
    build::adapted_source_builds_as_the_pinned_provider(&snapshot, &provider);
    artifact::one_adapted_binary_produces_distinct_equivalent_whole_graph_runs(&provider);
    collision::artifact_public_aliases_are_version_qualified_when_logical_names_collide(&provider);
    workspace_roots::artifact_traverses_non_workspace_dependency_kinds_without_generating_workspace_rules(
        &provider,
    );
}
