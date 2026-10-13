use axum::Json;
use axum::response::{IntoResponse, Response};
use data_ontology_kernel::{LinkCardinality, OntologyEngine};
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
struct LinkSchemaRow {
    link_type: String,         // data_class: INTERNAL_ONLY
    from_entity_type: String,  // data_class: INTERNAL_ONLY
    to_entity_type: String,    // data_class: INTERNAL_ONLY
    cardinality: &'static str, // data_class: INTERNAL_ONLY
    allow_cross_tenant: bool,  // data_class: INTERNAL_ONLY
    revision: u32,             // data_class: INTERNAL_ONLY
}

fn rows(engine: &OntologyEngine, tenant_id: &str) -> Vec<LinkSchemaRow> {
    engine
        .link_types(tenant_id)
        .map(|definition| LinkSchemaRow {
            link_type: definition.id.value.clone(),
            from_entity_type: definition.from_entity_type.value.clone(),
            to_entity_type: definition.to_entity_type.value.clone(),
            cardinality: match definition.cardinality {
                LinkCardinality::OneToOne => "one_to_one",
                LinkCardinality::OneToMany => "one_to_many",
                LinkCardinality::ManyToMany => "many_to_many",
            },
            allow_cross_tenant: definition.allow_cross_tenant,
            revision: definition.revision,
        })
        .collect()
}

#[cfg(test)]
mod tests;
