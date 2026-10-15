use crate::composition::{AppState, TenantState};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
#[cfg(test)]
use data_ontology_kernel::{OntologyEngine, ValueTypeDeclaration};

pub(crate) async fn response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
) -> Response {
    let tenant = tenant.lock().await;
    match foundry_model_api::schema_bytes(&tenant.projection.engine, tenant_id) {
        Ok(bytes) => {
            state.metrics.read_served();
            ([(header::CONTENT_TYPE, "application/json")], bytes).into_response()
        }
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
#[cfg(test)]
mod tests;
