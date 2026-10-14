use application_ontology_read::{
    LinkCardinality, LinkSchema, OntologyReadError, RelationshipDirection, RelationshipEdge,
    RelationshipGraph, RelationshipNode, RelationshipQuery,
};

use super::{FixtureOntologyReadSource, check_revision, fixture};

pub(super) fn declaration() -> LinkSchema {
    LinkSchema {
        link_type: "lty_fixture_related".into(),
        from_entity_type: FixtureOntologyReadSource::ENTITY_TYPE.into(),
        to_entity_type: FixtureOntologyReadSource::ENTITY_TYPE.into(),
        cardinality: LinkCardinality::OneToMany,
        allow_cross_tenant: false,
        revision: 1,
    }
}

pub(super) fn read(
    source: &FixtureOntologyReadSource,
    tenant: &str,
    query: &RelationshipQuery,
) -> Result<RelationshipGraph, OntologyReadError> {
    source.check(tenant)?;
    query.validate_for(&declaration())?;
    if !source.relationships {
        return Err(OntologyReadError::InvalidQuery);
    }
    check_revision(query.root.revision)?;
    let objects = fixture::objects();
    let root = objects
        .iter()
        .find(|o| o.object_ref == query.root.object_ref)
        .ok_or(OntologyReadError::InvalidQuery)?;
    let connected = match query.direction {
        RelationshipDirection::Outbound => root.object_ref == "ent_fixture_alpha",
        RelationshipDirection::Inbound => root.object_ref == "ent_fixture_beta",
    };
    let graph = RelationshipGraph {
        nodes: objects
            .iter()
            .filter(|object| connected || object.object_ref == root.object_ref)
            .map(|object| RelationshipNode {
                entity_id: object.object_ref.clone(),
                entity_type_id: FixtureOntologyReadSource::ENTITY_TYPE.into(),
                written_revision: object.written_revision,
            })
            .collect(),
        edges: if connected {
            vec![RelationshipEdge {
                from_entity_id: "ent_fixture_alpha".into(),
                to_entity_id: "ent_fixture_beta".into(),
                edge_type_id: query.link_type.clone(),
            }]
        } else {
            vec![]
        },
        observed_at_epoch_seconds: query.observed_at_epoch_seconds,
        truncated: false,
    };
    graph.validate(query, &declaration())?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use application_ontology_read::{OntologyReadSource, RelationshipRoot};

    fn query() -> RelationshipQuery {
        RelationshipQuery {
            idempotency_key: "fixture-one-hop".into(),
            root: RelationshipRoot {
                entity_type: FixtureOntologyReadSource::ENTITY_TYPE.into(),
                object_ref: "ent_fixture_alpha".into(),
                revision: 1,
            },
            link_type: declaration().link_type,
            direction: RelationshipDirection::Outbound,
            observed_at_epoch_seconds: 123,
        }
    }

    #[tokio::test]
    async fn explicit_sample_preserves_direction_and_each_objects_own_revision() {
        let source = FixtureOntologyReadSource::relationship_sample();
        let mut query = query();
        let graph = source
            .search_around(FixtureOntologyReadSource::TENANT, &query)
            .await
            .unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.nodes[1].written_revision, 2);
        assert_eq!(graph.edges.len(), 1);
        query.direction = RelationshipDirection::Inbound;
        let empty = source
            .search_around(FixtureOntologyReadSource::TENANT, &query)
            .await
            .unwrap();
        assert_eq!(empty.nodes.len(), 1);
        assert!(empty.edges.is_empty());
        query.root.object_ref = "ent_fixture_beta".into();
        assert_eq!(
            source
                .search_around(FixtureOntologyReadSource::TENANT, &query)
                .await
                .unwrap()
                .nodes
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn scope_source_pin_and_catalog_refusals_never_become_empty_graphs() {
        let tenant = FixtureOntologyReadSource::TENANT;
        let mut query = query();
        assert_eq!(
            FixtureOntologyReadSource::sample()
                .search_around(tenant, &query)
                .await,
            Err(OntologyReadError::InvalidQuery)
        );
        let source = FixtureOntologyReadSource::relationship_sample();
        assert_eq!(
            source.search_around("ten_other", &query).await,
            Err(OntologyReadError::Forbidden)
        );
        query.root.revision = 99;
        assert_eq!(
            source.search_around(tenant, &query).await,
            Err(OntologyReadError::RevisionConflict)
        );
        for error in [
            OntologyReadError::Unauthenticated,
            OntologyReadError::Forbidden,
            OntologyReadError::SourceUnavailable,
        ] {
            assert_eq!(
                FixtureOntologyReadSource::refusing(error)
                    .search_around(tenant, &query)
                    .await,
                Err(error)
            );
        }
        query.root.revision = 1;
        query.root.object_ref = "ent_missing".into();
        assert_eq!(
            source.search_around(tenant, &query).await,
            Err(OntologyReadError::InvalidQuery)
        );
    }
}
