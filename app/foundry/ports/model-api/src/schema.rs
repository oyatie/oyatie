use crate::ModelError;
use data_ontology_kernel::{OntologyEngine, ValueTypeDeclaration};
use serde::Serialize;

pub const MAX_ONTOLOGY_SCHEMA_BYTES: usize = 1_048_576;

pub fn schema_bytes(engine: &OntologyEngine, tenant: &str) -> Result<Vec<u8>, ModelError> {
    let bytes = serde_json::to_vec(&schema_rows(engine, tenant))
        .map_err(|_| ModelError::MalformedDocument)?;
    if bytes.len() > MAX_ONTOLOGY_SCHEMA_BYTES {
        return Err(ModelError::Limit {
            path: "schemas".into(),
        });
    }
    Ok(bytes)
}

#[derive(Debug, Serialize)]
struct EntitySchemaRow {
    entity_type: String,                    // data_class: INTERNAL_ONLY
    revision: u32,                          // data_class: INTERNAL_ONLY
    properties: Vec<PropertyDefinitionRow>, // data_class: INTERNAL_ONLY
    primary_key_property: Option<String>,   // data_class: INTERNAL_ONLY
    title_property: Option<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Debug, Serialize)]
struct PropertyDefinitionRow {
    name: String,                          // data_class: INTERNAL_ONLY
    tier: &'static str,                    // data_class: INTERNAL_ONLY
    data_class: &'static str,              // data_class: INTERNAL_ONLY
    required: bool,                        // data_class: INTERNAL_ONLY
    value_type: Option<serde_json::Value>, // data_class: INTERNAL_ONLY
}

fn schema_rows(engine: &OntologyEngine, tenant_id: &str) -> Vec<EntitySchemaRow> {
    engine
        .entity_types(tenant_id)
        .map(|definition| EntitySchemaRow {
            entity_type: definition.id.value.clone(),
            revision: definition.revision,
            properties: definition
                .properties
                .iter()
                .map(|property| PropertyDefinitionRow {
                    name: property.name.clone(),
                    tier: property.tier.wire_label(),
                    data_class: property.data_class.label(),
                    required: property.required,
                    value_type: property.value_type.as_ref().map(declared_value_type),
                })
                .collect(),
            primary_key_property: definition.primary_key_property.clone(),
            title_property: definition.title_property.clone(),
        })
        .collect()
}

fn declared_value_type(declaration: &ValueTypeDeclaration) -> serde_json::Value {
    match declaration {
        ValueTypeDeclaration::NamedString(definition) => serde_json::json!({
            "kind":"named_string", "type_id":definition.reference().id().as_str(),
            "revision":definition.reference().revision(), "display_name":definition.display_name().value,
            "members":definition.members(),
        }),
        ValueTypeDeclaration::Scalar(_) => serde_json::json!({"kind":declaration.type_label()}),
        ValueTypeDeclaration::Array { element } => serde_json::json!({
            "kind":"array", "element":declared_value_type(element)
        }),
        ValueTypeDeclaration::Struct(schema) => serde_json::json!({
            "kind":"struct", "fields":schema.fields.iter().map(|field| serde_json::json!({
                "name":field.name, "required":field.required,
                "value_type":declared_value_type(&field.value_type)
            })).collect::<Vec<_>>()
        }),
    }
}
