use super::authorization::{Grant, after_wait, authorize, capabilities, current, model};
use super::transport::{Refusal, encoded, malformed, source_error, unavailable};
use super::*;
use axum::{
    extract::{Path, State, rejection::PathRejection},
    http::{HeaderMap, StatusCode, Uri},
    response::Response,
};

pub(crate) async fn workspace(
    State(state): State<ModelState>,
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
) -> Response {
    let Ok(Path(tenant)) = path else {
        return *malformed();
    };
    workspace_inner(&state, &tenant, &headers)
        .await
        .unwrap_or_else(|refusal| *refusal)
}

async fn workspace_inner(
    state: &ModelState,
    tenant: &str,
    headers: &HeaderMap,
) -> Result<Response, Refusal> {
    let first = authorize(state, tenant, headers, Grant::Read).await?;
    let mut handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, headers, Grant::Read, &first).await?;
    let capabilities = capabilities(state, &authorization)?;
    let published = model(&mut handle, &authorization)?
        .store
        .head(&authorization.caller)
        .map_err(source_error)?;
    current(&authorization)?;
    let workspace = ModelWorkspace {
        generation: ModelNumber::new(published.generation).map_err(|_| unavailable())?,
        document: published.document,
        capabilities,
    };
    encoded(StatusCode::OK, workspace.encode())
}

pub(crate) async fn drafts(
    State(state): State<ModelState>,
    path: Result<Path<String>, PathRejection>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let Ok(Path(tenant)) = path else {
        return *malformed();
    };
    drafts_inner(&state, &tenant, &headers, uri.query())
        .await
        .unwrap_or_else(|refusal| *refusal)
}

fn page(query: Option<&str>) -> Result<DraftPageRequest, Refusal> {
    let mut limit = None;
    let mut after = None;
    for pair in query.ok_or_else(malformed)?.split('&') {
        let (key, value) = pair.split_once('=').ok_or_else(malformed)?;
        let number = ModelNumber::parse(value).map_err(|_| malformed())?.get();
        match key {
            "limit" if limit.is_none() => {
                limit = Some(usize::try_from(number).map_err(|_| malformed())?);
            }
            "after" if after.is_none() => {
                after = Some(number);
            }
            _ => return Err(malformed()),
        }
    }
    let request = DraftPageRequest {
        limit: limit.ok_or_else(malformed)?,
        after,
    };
    request.validate().map_err(|_| malformed())?;
    Ok(request)
}

async fn drafts_inner(
    state: &ModelState,
    tenant: &str,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<Response, Refusal> {
    let first = authorize(state, tenant, headers, Grant::Read).await?;
    let page = page(query)?;
    let mut handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, headers, Grant::Read, &first).await?;
    let drafts = model(&mut handle, &authorization)?
        .store
        .owned_drafts(&authorization.caller, &page)
        .map_err(source_error)?;
    current(&authorization)?;
    encoded(
        StatusCode::OK,
        ModelDraftPage::try_from(drafts)
            .map_err(|_| unavailable())?
            .encode(),
    )
}

pub(crate) async fn draft(
    State(state): State<ModelState>,
    path: Result<Path<(String, String)>, PathRejection>,
    headers: HeaderMap,
) -> Response {
    let Ok(Path((tenant, id))) = path else {
        return *malformed();
    };
    draft_inner(&state, &tenant, &id, &headers)
        .await
        .unwrap_or_else(|refusal| *refusal)
}

async fn draft_inner(
    state: &ModelState,
    tenant: &str,
    id: &str,
    headers: &HeaderMap,
) -> Result<Response, Refusal> {
    let first = authorize(state, tenant, headers, Grant::Read).await?;
    let id = ModelNumber::parse(id).map_err(|_| malformed())?.get();
    let mut handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, headers, Grant::Read, &first).await?;
    let draft = model(&mut handle, &authorization)?
        .store
        .open_draft(&authorization.caller, id)
        .map_err(source_error)?;
    current(&authorization)?;
    encoded(
        StatusCode::OK,
        ModelDraftWire::try_from(&draft)
            .map_err(|_| unavailable())?
            .encode(),
    )
}
