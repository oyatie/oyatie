#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::{conformance::*, *};
use data_ontology_kernel::{EntityTypeId, OntologyEngineError, ValueTypeDeclaration};
fn prepare(model: ModelDocument) -> Result<PreparedPublication, ModelError> {
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let saved = store.create_draft(&caller(), 1, model).unwrap();
    store.prepare(&caller(), saved.id, saved.version, saved.base_generation)
}
#[test]
fn reusable_identity_survives_capture_retained_revisions_and_duplicate_labels_without_latest_resolution()
 {
    let mut model = named_string_model();
    let mut unused = model.named_string_types[0].clone();
    unused.id = "vty_test_unused".into();
    model.named_string_types.push(unused);
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let saved = store.create_draft(&caller(), 1, model.clone()).unwrap();
    let ready = store
        .prepare(&caller(), saved.id, saved.version, saved.base_generation)
        .unwrap();
    store.publish(&caller(), &ready).unwrap();
    let registry = store.history(&caller()).unwrap().registry().unwrap();
    assert_eq!(registry.named_string_types("ten_model").count(), 2);
    let id = EntityTypeId::new("ety_record").unwrap();
    let row = registry.entity_type("ten_model", &id).unwrap();
    let declarations: Vec<_> = row
        .properties
        .iter()
        .filter(|p| p.name.ends_with("test_status"))
        .map(|p| p.value_type.as_ref().unwrap())
        .collect();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0], declarations[1]);
    assert!(
        matches!(declarations[0],ValueTypeDeclaration::NamedString(d) if d.reference().id().as_str()=="vty_test_status" && d.reference().revision()==1)
    );
    let captured = ModelBaseline::capture(&registry, "ten_model").unwrap();
    assert_eq!(captured.registry("ten_model").unwrap(), registry);
    assert!(
        registry
            .entity_type_at_revision("ten_model", &id, 1)
            .unwrap()
            .properties
            .iter()
            .all(|p| !p.name.ends_with("test_status"))
    );
    let mut roundtrip = ModelDocument::capture(&registry, "ten_model").unwrap();
    roundtrip.named_string_types.sort_by(|a, b| a.id.cmp(&b.id));
    model.named_string_types.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(roundtrip, model);
    let schemas: serde_json::Value =
        serde_json::from_slice(&schema_bytes(&registry, "ten_model").unwrap()).unwrap();
    let properties = schemas[0]["properties"].as_array().unwrap();
    let used: Vec<_> = properties
        .iter()
        .filter(|p| p["name"].as_str().unwrap().ends_with("test_status"))
        .collect();
    assert_eq!(used[0]["value_type"], used[1]["value_type"]);
    assert_eq!(used[0]["value_type"]["kind"], "named_string");
    assert_eq!(
        used[0]["value_type"]["members"],
        serde_json::json!(["TestActive", "TestLeave"])
    );
    assert_eq!(schema_bytes(&registry, "ten_foreign").unwrap(), b"[]");
    let mut same = model.clone();
    same.named_string_types.reverse();
    same.named_string_types[0].members.reverse();
    let saved = store.create_draft(&caller(), 2, same).unwrap();
    assert_eq!(
        store.prepare(&caller(), saved.id, 1, 2).unwrap_err(),
        ModelError::NoChanges
    );
    for mutation in 0..6 {
        let mut changed = model.clone();
        match mutation {
            0 => changed.named_string_types.clear(),
            1 => changed.named_string_types[0]
                .members
                .push("TestOther".into()),
            2 => changed.named_string_types[0].display_name = "Changed".into(),
            3 => changed.named_string_types[0].revision = 2,
            4 => {
                changed.entities[0]
                    .properties
                    .last_mut()
                    .unwrap()
                    .value_type = Some(ModelValueType::NamedString {
                    type_id: "vty_test_unused".into(),
                    revision: 1,
                })
            }
            _ => {
                changed.entities[0].properties[0].value_type = Some(ModelValueType::NamedString {
                    type_id: "vty_test_status".into(),
                    revision: 1,
                });
            }
        };
        changed.entities[0].revision = 3;
        let saved = store.create_draft(&caller(), 2, changed).unwrap();
        assert!(
            store.prepare(&caller(), saved.id, 1, 2).is_err(),
            "mutation {mutation}"
        );
        assert_eq!(store.head(&caller()).unwrap().generation, 2);
    }
}
#[test]
fn invalid_but_bounded_named_drafts_preserve_repairs_and_refuse_admission_without_effects() {
    for mutation in 0..8 {
        let mut model = named_string_model();
        match mutation {
            0 => model.named_string_types[0].members.clear(),
            1 => model.named_string_types[0]
                .members
                .push("TestActive".into()),
            2 => model
                .named_string_types
                .push(model.named_string_types[0].clone()),
            3 => model.named_string_types[0].id = "invalid".into(),
            4 => model.named_string_types[0].revision = 0,
            5 => model.named_string_types.clear(),
            6 => {
                let p = model.entities[0].properties.last_mut().unwrap();
                p.tier = ModelTier::Vector;
                p.value_type = Some(ModelValueType::Array {
                    element: Box::new(p.value_type.take().unwrap()),
                });
            }
            _ => model.named_string_types[0].display_name.clear(),
        };
        let encoded = model.encode().unwrap();
        assert_eq!(ModelDocument::decode(&encoded).unwrap(), model);
        let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
        let before = store.history(&caller()).unwrap();
        let saved = store.create_draft(&caller(), 1, model.clone()).unwrap();
        assert_eq!(
            store.open_draft(&caller(), saved.id).unwrap().document,
            model
        );
        let err = store.prepare(&caller(), saved.id, 1, 1).unwrap_err();
        assert!(
            matches!(err,ModelError::Native{path,..} if path.starts_with("named_string_types")||path.contains(".value_type"))
        );
        assert_eq!(store.history(&caller()).unwrap(), before);
        let fixed = store
            .save_draft(&caller(), saved.id, 1, 1, named_string_model())
            .unwrap();
        assert!(store.prepare(&caller(), fixed.id, fixed.version, 1).is_ok());
    }
    let mut invalid = named_string_model();
    invalid.named_string_types[0].members = vec!["Test".into(); 257];
    assert!(invalid.encode().is_err());
}
#[test]
fn reused_large_member_metadata_is_refused_before_publication_at_the_actual_schema_response_bound()
{
    let mut model = named_string_model();
    model.named_string_types[0].members = (0..64)
        .map(|i| format!("Test{i:03}{}", "x".repeat(4000)))
        .collect();
    for index in 0..4 {
        let mut p = property(&format!("reuse_{index}"), false);
        p.value_type = Some(ModelValueType::NamedString {
            type_id: "vty_test_status".into(),
            revision: 1,
        });
        model.entities[0].properties.push(p);
    }
    assert!(model.encode().unwrap().len() < MAX_MODEL_BYTES);
    assert!(matches!(prepare(model),Err(ModelError::Limit{path}) if path=="schemas"));
    let mut accepted = named_string_model();
    accepted.named_string_types[0].members = (0..64)
        .map(|i| format!("Test{i:03}{}", "x".repeat(4000)))
        .collect();
    let registry = prepare(accepted).unwrap();
    assert!(
        schema_bytes(registry.model_registry(), "ten_model")
            .unwrap()
            .len()
            < MAX_ONTOLOGY_SCHEMA_BYTES
    );
}
#[test]
fn nested_reference_reports_its_exact_unsupported_semantic_path() {
    let mut model = named_string_model();
    let property = model.entities[0].properties.last_mut().unwrap();
    property.tier = ModelTier::Vector;
    property.value_type = Some(ModelValueType::Array {
        element: Box::new(property.value_type.take().unwrap()),
    });
    assert!(
        matches!(prepare(model),Err(ModelError::Native{path,cause:OntologyEngineError::UnsupportedNamedStringPosition}) if path=="entities.ety_record.properties.other_test_status.value_type.element")
    );
}

#[test]
fn actual_schema_bytes_accept_the_exact_limit_and_refuse_one_more_byte_before_commit() {
    let mut model = named_string_model();
    model.named_string_types[0].members = (0..128)
        .map(|i| format!("Test{i:03}{}", "x".repeat(4000)))
        .collect();
    let ready = prepare(model.clone()).unwrap();
    let length = schema_bytes(ready.model_registry(), "ten_model")
        .unwrap()
        .len();
    let mut needed = MAX_ONTOLOGY_SCHEMA_BYTES - length;
    if needed % 2 == 1 {
        model.entities[0]
            .properties
            .last_mut()
            .unwrap()
            .name
            .push('x');
        needed -= 1;
    }
    for member in &mut model.named_string_types[0].members {
        let extra = (4096 - member.len()).min(needed / 2);
        member.push_str(&"x".repeat(extra));
        needed -= extra * 2;
    }
    assert_eq!(needed, 0);
    let ready = prepare(model.clone()).unwrap();
    assert_eq!(
        schema_bytes(ready.model_registry(), "ten_model")
            .unwrap()
            .len(),
        MAX_ONTOLOGY_SCHEMA_BYTES
    );
    model.entities[0]
        .properties
        .last_mut()
        .unwrap()
        .name
        .push('x');
    assert!(matches!(prepare(model),Err(ModelError::Limit{path}) if path=="schemas"));
}

#[test]
fn an_existing_untyped_property_cannot_be_retargeted_to_a_named_reference() {
    let mut old = baseline();
    old.document.entities[0].properties[0].value_type = None;
    let mut store = ReferenceModelStore::initialize(binding(), old.clone()).unwrap();
    let mut model = old.document;
    model.entities[0].revision = 2;
    model.named_string_types = named_string_model().named_string_types;
    model.entities[0].properties[0].value_type = Some(ModelValueType::NamedString {
        type_id: "vty_test_status".into(),
        revision: 1,
    });
    let draft = store.create_draft(&caller(), 1, model).unwrap();
    assert!(matches!(
        store.prepare(&caller(), draft.id, 1, 1),
        Err(ModelError::Native {
            cause: OntologyEngineError::IncompatibleSchemaEvolution,
            ..
        })
    ));
    assert_eq!(store.head(&caller()).unwrap().generation, 1);
}
#[test]
fn catalogue_members_share_the_existing_global_document_node_budget() {
    let mut model = named_string_model();
    model.named_string_types = (0..16)
        .map(|i| ModelNamedStringType {
            id: format!("vty_test_{i}"),
            revision: 1,
            display_name: "Test status".into(),
            members: (0..256).map(|j| format!("Test{j}")).collect(),
        })
        .collect();
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    assert!(matches!(
        store.create_draft(&caller(), 1, model),
        Err(ModelError::Limit { .. })
    ));
    assert!(
        store
            .owned_drafts(
                &caller(),
                &DraftPageRequest {
                    limit: 1,
                    after: None
                }
            )
            .unwrap()
            .drafts
            .is_empty()
    );
}

mod wire;
