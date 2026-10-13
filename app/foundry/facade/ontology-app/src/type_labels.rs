use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use data_boundary_kernel::{DataClass, DataClassification};
use data_ontology_kernel::OntologyEngine;
use serde::Serialize;

use crate::composition::{AppState, TenantState};

pub(crate) async fn response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
) -> Response {
    let tenant = tenant.lock().await;
    match rows(&tenant.projection.engine, tenant_id) {
        Ok(rows) => {
            state.metrics.read_served();
            Json(rows).into_response()
        }
        Err(()) => {
            state.metrics.read_refused();
            crate::reads::refuse(
                StatusCode::SERVICE_UNAVAILABLE,
                "registry",
                "native type labels are unavailable",
            )
        }
    }
}

#[derive(Debug, Serialize)]
struct EntityTypeLabelRow {
    entity_type: String,  // data_class: INTERNAL_ONLY
    revision: u32,        // data_class: INTERNAL_ONLY
    display_name: String, // data_class: INTERNAL_ONLY
}

fn rows(engine: &OntologyEngine, tenant_id: &str) -> Result<Vec<EntityTypeLabelRow>, ()> {
    engine
        .entity_types(tenant_id)
        .map(|definition| {
            if definition.display_name.data_class
                != DataClassification::from(DataClass::InternalOnly)
                || definition.display_name.value.trim().is_empty()
            {
                return Err(());
            }
            Ok(EntityTypeLabelRow {
                entity_type: definition.id.value.clone(),
                revision: definition.revision,
                display_name: definition.display_name.value.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
