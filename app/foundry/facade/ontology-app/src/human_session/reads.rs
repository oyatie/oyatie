use axum::extract::{Path, RawQuery, State};
use axum::http::HeaderMap;
use axum::response::Response;

use super::{HumanReadState, authorized};
use crate::reads::TENANT_SCOPED_RESOURCE;

pub(crate) async fn type_labels(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) =
        match authorized(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(authorized) => authorized,
            Err(response) => return *response,
        };
    crate::type_labels::response(&state.app, &caller.tenant_id, tenant).await
}

pub(crate) async fn types(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) =
        match authorized(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(authorized) => authorized,
            Err(response) => return *response,
        };
    crate::reads::types_response(&state.app, &caller.tenant_id, tenant).await
}

pub(crate) async fn schemas(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) =
        match authorized(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(authorized) => authorized,
            Err(response) => return *response,
        };
    crate::schemas::response(&state.app, &caller.tenant_id, tenant).await
}

pub(crate) async fn link_types(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) =
        match authorized(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(authorized) => authorized,
            Err(response) => return *response,
        };
    crate::link_types::response(&state.app, &caller.tenant_id, tenant).await
}

pub(crate) async fn objects(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) =
        match authorized(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(authorized) => authorized,
            Err(response) => return *response,
        };
    crate::listing::objects_response(&state.app, &caller.tenant_id, tenant, raw_query).await
}

pub(crate) async fn object(
    State(state): State<HumanReadState>,
    Path((tenant_id, object_ref)): Path<(String, String)>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) = match authorized(&state, &tenant_id, &headers, &object_ref).await {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    crate::reads::object_response(&state.app, &caller.tenant_id, tenant, object_ref, raw_query)
        .await
}

pub(crate) async fn history(
    State(state): State<HumanReadState>,
    Path((tenant_id, object_ref)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) = match authorized(&state, &tenant_id, &headers, &object_ref).await {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    crate::reads::history_response(&state.app, &caller.tenant_id, tenant, &object_ref).await
}

pub(crate) async fn audit(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) =
        match authorized(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(authorized) => authorized,
            Err(response) => return *response,
        };
    crate::reads::audit_response(&state.app, &caller.tenant_id, tenant).await
}
