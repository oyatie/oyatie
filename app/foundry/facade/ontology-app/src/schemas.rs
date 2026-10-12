use axum::Json;
use axum::response::{IntoResponse, Response};
use data_ontology_kernel::{OntologyEngine, ValueTypeDeclaration};
use serde::Serialize;

use crate::composition::{AppState, TenantState};

pub(crate) async fn response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
) -> Response {
    let tenant = tenant.lock().await;
    let rows = rows(&tenant.projection.engine, tenant_id);
    state.metrics.read_served();
    Json(rows).into_response()
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

fn rows(engine: &OntologyEngine, tenant_id: &str) -> Vec<EntitySchemaRow> {
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

#[cfg(test)]
mod tests;
