use crate::*;

use super::support::property;

fn definition(tenant: &str, id: &str, revision: u32) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        tenant,
        EntityTypeId::new(id).unwrap(),
        "Record",
        vec![property("zeta"), property("alpha")],
        revision,
    )
    .unwrap()
}

#[test]
fn current_registry_enumeration_preserves_tenant_scope_and_native_property_order() {
    let mut engine = OntologyEngine::default();
    for (tenant, id) in [
        ("ten_a", "ety_zeta"),
        ("ten_b", "ety_foreign"),
        ("ten_b", "ety_alpha"),
        ("ten_a", "ety_alpha"),
    ] {
        let mut candidate = definition(tenant, id, 0);
        if tenant == "ten_b" {
            candidate.properties[0].name = "foreign_only".to_owned();
        }
        engine.register_entity_type(candidate).unwrap();
    }
    let rows: Vec<_> = engine.entity_types("ten_a").collect();
    assert_eq!(
        rows.iter().map(|d| d.id.value.as_str()).collect::<Vec<_>>(),
        vec!["ety_alpha", "ety_zeta"]
    );
    assert_eq!(
        rows[0]
            .properties
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        vec!["zeta", "alpha"]
    );
    assert!(engine.entity_types("ten_absent").next().is_none());
    assert_eq!(rows[0].revision, 0);
}

#[test]
fn ambiguous_or_blank_names_never_enter_first_registration_or_retained_revisions() {
    for evolve in [false, true] {
        for blank in [false, true] {
            let mut engine = OntologyEngine::default();
            let mut candidate = definition("ten_a", "ety_record", 1);
            candidate.properties[1].name = if blank { " " } else { "zeta" }.to_owned();
            let id = candidate.id.clone();
            let expected = if blank {
                OntologyEngineError::EmptyPropertyName
            } else {
                OntologyEngineError::DuplicatePropertyName {
                    name: "zeta".to_owned(),
                }
            };
            let result = if evolve {
                engine.evolve_entity_type(candidate)
            } else {
                engine.register_entity_type(candidate)
            };
            assert_eq!(result, Err(expected));
            assert_eq!(engine.entity_type("ten_a", &id), None);
            assert_eq!(engine.entity_type_at_revision("ten_a", &id, 1), None);
            assert!(engine.entity_types("ten_a").next().is_none());
        }
    }
}

#[test]
fn rejected_evolution_preserves_prior_definition_and_does_not_retain_candidate() {
    let mut engine = OntologyEngine::default();
    let prior = definition("ten_a", "ety_record", 1);
    engine.register_entity_type(prior.clone()).unwrap();
    for blank in [false, true] {
        let mut candidate = prior.clone();
        candidate.revision = 2;
        candidate.properties[1].name = if blank { "" } else { "zeta" }.to_owned();
        assert!(engine.evolve_entity_type(candidate).is_err());
        assert_eq!(engine.entity_type("ten_a", &prior.id), Some(&prior));
        assert_eq!(
            engine.entity_type_at_revision("ten_a", &prior.id, 1),
            Some(&prior)
        );
        assert_eq!(engine.entity_type_at_revision("ten_a", &prior.id, 2), None);
    }
}
