use super::support::property;
use crate::{
    EntityTypeDefinition, EntityTypeId, LinkCardinality, LinkInstanceOutcome, LinkTypeDefinition,
    LinkTypeId, OntologyEngine, OntologyEngineError,
};

fn engine() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    for tenant in ["ten_a", "ten_b"] {
        engine
            .register_entity_type(
                EntityTypeDefinition::new(
                    tenant,
                    EntityTypeId::new("ety_node").unwrap(),
                    "Node",
                    vec![property("name")],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        for (id, cardinality) in [
            ("lty_related", LinkCardinality::ManyToMany),
            ("lty_single", LinkCardinality::OneToOne),
        ] {
            engine
                .register_link_type(
                    LinkTypeDefinition::new(
                        tenant,
                        LinkTypeId::new(id).unwrap(),
                        EntityTypeId::new("ety_node").unwrap(),
                        EntityTypeId::new("ety_node").unwrap(),
                        cardinality,
                        false,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
    }
    engine
}

#[test]
fn instance_view_is_directed_tenant_scoped_unique_and_immutable() {
    let mut engine = engine();
    let relation = LinkTypeId::new("lty_related").unwrap();
    for (tenant, from, to) in [
        ("ten_a", "ent_from", "ent_to"),
        ("ten_a", "ent_to", "ent_to"),
        ("ten_b", "ent_to", "ent_from"),
    ] {
        assert_eq!(
            engine.register_link_instance(tenant, &relation, from, to),
            Ok(LinkInstanceOutcome::Registered)
        );
    }
    assert_eq!(
        engine.register_link_instance("ten_a", &relation, "ent_from", "ent_to"),
        Ok(LinkInstanceOutcome::AlreadyExists)
    );
    let before = engine.clone();
    let tuples: Vec<(&str, &str, &str)> = engine.link_instances("ten_a").collect();
    assert_eq!(
        tuples,
        vec![
            ("lty_related", "ent_from", "ent_to"),
            ("lty_related", "ent_to", "ent_to")
        ]
    );
    let incident = |reference| {
        tuples
            .iter()
            .copied()
            .filter(move |(_, from, to)| *from == reference || *to == reference)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        incident("ent_from")[0],
        incident("ent_to")[0],
        "either endpoint sees the original directed tuple"
    );
    assert_eq!(incident("ent_to").len(), 2, "a self-link is emitted once");
    assert_eq!(
        engine.link_instances("ten_b").collect::<Vec<_>>(),
        vec![("lty_related", "ent_to", "ent_from")]
    );
    assert_eq!(engine.link_instances("ten_absent").count(), 0);
    assert_eq!(engine, before);
}

#[test]
fn refused_cardinality_does_not_change_the_instance_view() {
    let mut engine = engine();
    let relation = LinkTypeId::new("lty_single").unwrap();
    engine
        .register_link_instance("ten_a", &relation, "ent_a", "ent_b")
        .unwrap();
    let before = engine.clone();
    assert_eq!(
        engine.register_link_instance("ten_a", &relation, "ent_a", "ent_c"),
        Err(OntologyEngineError::CardinalityViolation {
            cardinality: LinkCardinality::OneToOne
        })
    );
    assert_eq!(engine, before);
    assert_eq!(
        engine.link_instances("ten_a").collect::<Vec<_>>(),
        vec![("lty_single", "ent_a", "ent_b")]
    );
}
