use data_ontology_kernel::{EntityTypeId, LinkTypeDefinition, LinkTypeId};
use serde_json::json;

use super::*;

#[test]
fn current_link_declarations_preserve_native_metadata_without_foreign_tenant_rows() {
    let mut engine = OntologyEngine::default();
    for tenant in ["ten_a", "ten_b"] {
        let seed = crate::seed::registry_for(tenant).unwrap();
        engine
            .register_entity_type(
                seed.entity_type(tenant, &EntityTypeId::new("ety_record").unwrap())
                    .unwrap()
                    .clone(),
            )
            .unwrap();
        for (id, cardinality, label) in [
            ("lty_one", LinkCardinality::OneToOne, "one_to_one"),
            ("lty_many", LinkCardinality::OneToMany, "one_to_many"),
            ("lty_all", LinkCardinality::ManyToMany, "many_to_many"),
        ] {
            let mut link = LinkTypeDefinition::new(
                tenant,
                LinkTypeId::new(id).unwrap(),
                EntityTypeId::new("ety_record").unwrap(),
                EntityTypeId::new("ety_record").unwrap(),
                cardinality,
                tenant == "ten_b",
            )
            .unwrap();
            link.revision = if tenant == "ten_a" { 0 } else { 9 };
            engine.register_link_type(link).unwrap();
            let wire = serde_json::to_value(rows(&engine, tenant)).unwrap();
            let row = wire
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["link_type"] == id)
                .unwrap();
            assert_eq!(
                row,
                &json!({"link_type":id,"from_entity_type":"ety_record",
                "to_entity_type":"ety_record","cardinality":label,
                "allow_cross_tenant":tenant == "ten_b","revision":if tenant == "ten_a" {0} else {9}})
            );
        }
    }
    let rows = rows(&engine, "ten_a");
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter()
            .all(|row| row.revision == 0 && !row.allow_cross_tenant)
    );
    assert!(super::rows(&engine, "ten_absent").is_empty());
}
