use super::ModelState;
use super::authorization::{Grant, after_wait, authorize, current};
use super::transport::{Refusal, encoded, malformed, source_error, unavailable};
use axum::{
    extract::{Path, State, rejection::PathRejection},
    http::{HeaderMap, StatusCode, Uri},
    response::Response,
};

pub(crate) async fn read(
    State(state): State<ModelState>,
    path: Result<Path<String>, PathRejection>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let Ok(Path(tenant)) = path else {
        return *malformed();
    };
    run(&state, &tenant, &headers, uri.query())
        .await
        .unwrap_or_else(|refusal| *refusal)
}

async fn run(
    state: &ModelState,
    tenant: &str,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<Response, Refusal> {
    let first = authorize(state, tenant, headers, Grant::Read).await?;
    if query.is_some() {
        return Err(malformed());
    }
    let handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, headers, Grant::Read, &first).await?;
    let lifecycle =
        crate::model_runtime::lifecycle(&handle, &authorization.caller).map_err(source_error)?;
    current(&authorization)?;
    encoded(StatusCode::OK, lifecycle.encode())
}
