use data_boundary_kernel::Classified;
use data_ontology_kernel::EntityTypeId;
use serde_json::json;

use super::*;

#[test]
fn labels_preserve_native_names_revisions_and_tenant_scope_without_reclassifying_values() {
    let mut engine = OntologyEngine::default();
    for (tenant, name, revision) in [("ten_a", "Record", 0), ("ten_b", "기록 <script>", 9)] {
        let seed = crate::seed::registry_for(tenant).unwrap();
        let mut definition = seed
            .entity_type(tenant, &EntityTypeId::new("ety_record").unwrap())
            .unwrap()
            .clone();
        definition.display_name.value = name.to_owned();
        definition.revision = revision;
        engine.register_entity_type(definition).unwrap();
        assert_eq!(
            serde_json::to_value(rows(&engine, tenant).unwrap()).unwrap(),
            json!([{
                "entity_type":"ety_record", "revision":revision, "display_name":name
            }])
        );
    }
    assert!(rows(&engine, "ten_absent").unwrap().is_empty());
    for label in [
        Classified::new("Private name".to_owned(), DataClass::Phi),
        Classified::new("\u{2003}".to_owned(), DataClass::InternalOnly),
    ] {
        let mut engine = OntologyEngine::default();
        let seed = crate::seed::registry_for("ten_a").unwrap();
        let mut definition = seed
            .entity_type("ten_a", &EntityTypeId::new("ety_record").unwrap())
            .unwrap()
            .clone();
        definition.display_name = label;
        engine.register_entity_type(definition).unwrap();
        assert!(
            rows(&engine, "ten_a").is_err(),
            "unsupported classifications and blank names fail closed"
        );
    }
}
