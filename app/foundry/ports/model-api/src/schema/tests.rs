#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use crate::{conformance::*, *};

#[test]
fn legacy_schema_rows_and_nested_declaration_bytes_remain_exact() {
    eprintln!(
        "schema JSON map order: {}",
        if *LEGACY_INSERTION_ORDER {
            "insertion"
        } else {
            "sorted"
        }
    );
    let engine = baseline().registry("ten_model").unwrap();
    assert_eq!(schema_bytes(&engine,"ten_model").unwrap(), br#"[{"entity_type":"ety_record","revision":1,"properties":[{"name":"name","tier":"scalar","data_class":"INTERNAL_ONLY","required":true,"value_type":{"kind":"string"}}],"primary_key_property":null,"title_property":"name"}]"#);
    let declaration = ValueTypeDeclaration::Struct(data_ontology_kernel::StructSchema {
        fields: vec![data_ontology_kernel::StructFieldDeclaration {
            name: "tags".into(),
            required: false,
            value_type: ValueTypeDeclaration::Array {
                element: Box::new(ValueTypeDeclaration::Scalar(
                    data_ontology_kernel::ScalarType::Date,
                )),
            },
        }],
    });
    let legacy = serde_json::json!({"kind":"struct", "fields":[{"name":"tags", "required":false, "value_type":{"kind":"array", "element":{"kind":"date"}}}]});
    assert_eq!(
        serde_json::to_vec(&DeclaredValueType(&declaration)).unwrap(),
        serde_json::to_vec(&legacy).unwrap()
    );
    use data_ontology_kernel::{NamedStringTypeDefinition, NamedStringTypeId, NamedStringTypeRef};
    let named = ValueTypeDeclaration::NamedString(
        NamedStringTypeDefinition::new(
            NamedStringTypeRef::new(
                "ten_model",
                NamedStringTypeId::new("vty_test_status").unwrap(),
                1,
            )
            .unwrap(),
            "Test status",
            vec!["TestActive".into(), "TestLeave".into()],
        )
        .unwrap(),
    );
    let legacy = serde_json::json!({"kind":"named_string", "type_id":"vty_test_status", "revision":1, "display_name":"Test status", "members":["TestActive","TestLeave"]});
    assert_eq!(
        serde_json::to_vec(&DeclaredValueType(&named)).unwrap(),
        serde_json::to_vec(&legacy).unwrap()
    );
}

#[test]
fn bounded_writer_refuses_before_allocation_and_keeps_real_serializer_errors_distinct() {
    let mut output = SchemaBuffer::default();
    assert!(
        output
            .write(&vec![0; MAX_ONTOLOGY_SCHEMA_BYTES + 1])
            .is_err()
    );
    assert!(output.exceeded);
    assert_eq!(output.bytes.capacity(), 0);
    let mut output = SchemaBuffer::default();
    for _ in 0..MAX_ONTOLOGY_SCHEMA_BYTES / 4096 {
        output.write_all(&[b'x'; 4096]).unwrap();
        assert!(output.bytes.capacity() <= MAX_ONTOLOGY_SCHEMA_BYTES);
    }
    assert_eq!(output.bytes.len(), MAX_ONTOLOGY_SCHEMA_BYTES);
    assert!(output.write_all(b"x").is_err());
    assert_eq!(output.bytes.len(), MAX_ONTOLOGY_SCHEMA_BYTES);
    assert_eq!(output.bytes.capacity(), MAX_ONTOLOGY_SCHEMA_BYTES);
    struct Invalid;
    impl Serialize for Invalid {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("injected serializer failure"))
        }
    }
    let mut output = SchemaBuffer::default();
    assert!(serde_json::to_writer(&mut output, &Invalid).is_err());
    assert!(!output.exceeded);
}

#[test]
fn high_fanout_uses_shared_canonical_members_and_stops_schema_output_at_the_read_cap() {
    let mut model = baseline().document;
    model.named_string_types.push(ModelNamedStringType {
        id: "vty_test_status".into(),
        revision: 1,
        display_name: "Test status".into(),
        members: (0..64)
            .map(|index| format!("Test{index:03}{}", "x".repeat(4000)))
            .collect(),
    });
    for index in 0..16 {
        let mut entity = baseline().document.entities.remove(0);
        entity.id = format!("ety_test_{index}");
        entity.properties = (0..250)
            .map(|p| {
                let mut property = property(&format!("status_{p:03}"), false);
                property.value_type = Some(ModelValueType::NamedString {
                    type_id: "vty_test_status".into(),
                    revision: 1,
                });
                property
            })
            .collect();
        entity.title_property = Some("status_000".into());
        model.entities.push(entity);
    }
    assert!(model.encode().unwrap().len() < MAX_MODEL_BYTES);
    let mut registry = baseline().registry("ten_model").unwrap();
    assert!(
        matches!(crate::admission::stage_document(&mut registry,"ten_model",&model),Err(ModelError::Limit{path}) if path=="schemas")
    );
    let canonical = registry.named_string_types("ten_model").next().unwrap();
    let mut count = 0;
    for property in registry
        .entity_types("ten_model")
        .flat_map(|e| &e.properties)
    {
        if let Some(ValueTypeDeclaration::NamedString(definition)) = &property.value_type {
            assert!(std::ptr::eq(canonical.members(), definition.members()));
            count += 1;
        }
    }
    assert_eq!(count, 4000);
    let mut output = SchemaBuffer::default();
    assert!(serde_json::to_writer(&mut output, &SchemaRows(&registry, "ten_model")).is_err());
    assert!(output.exceeded);
    assert!(output.bytes.len() <= MAX_ONTOLOGY_SCHEMA_BYTES);
    assert!(output.bytes.capacity() <= MAX_ONTOLOGY_SCHEMA_BYTES);
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let draft = store.create_draft(&caller(), 1, model).unwrap();
    let before = store.history(&caller()).unwrap();
    assert!(
        matches!(store.prepare(&caller(),draft.id,1,1),Err(ModelError::Limit{path}) if path=="schemas")
    );
    assert_eq!(store.history(&caller()).unwrap(), before);
    assert!(
        store
            .open_draft(&caller(), draft.id)
            .unwrap()
            .publication
            .is_none()
    );
}
