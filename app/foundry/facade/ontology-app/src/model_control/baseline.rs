use super::authorization::{Grant, after_wait, authorize, current, model};
use super::transport::{Refusal, encoded, malformed, source_error, unavailable};
use super::*;
use axum::{
    extract::{Path, State, rejection::PathRejection},
    http::{HeaderMap, StatusCode, Uri},
    response::Response,
};

pub(crate) async fn read(
    State(state): State<ModelState>,
    path: Result<Path<(String, String)>, PathRejection>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let Ok(Path((tenant, id))) = path else {
        return *malformed();
    };
    run(&state, &tenant, &id, uri.query(), &headers)
        .await
        .unwrap_or_else(|refusal| *refusal)
}

fn expected(query: Option<&str>) -> Result<PrepareModelDraft, Refusal> {
    let mut version = None;
    let mut base = None;
    for pair in query.ok_or_else(malformed)?.split('&') {
        let (key, value) = pair.split_once('=').ok_or_else(malformed)?;
        let number = ModelNumber::parse(value).map_err(|_| malformed())?;
        match key {
            "expected_version" if version.is_none() => version = Some(number),
            "expected_base" if base.is_none() => base = Some(number),
            _ => return Err(malformed()),
        }
    }
    Ok(PrepareModelDraft {
        expected_version: version.ok_or_else(malformed)?,
        expected_base: base.ok_or_else(malformed)?,
    })
}

async fn run(
    state: &ModelState,
    tenant: &str,
    id: &str,
    query: Option<&str>,
    headers: &HeaderMap,
) -> Result<Response, Refusal> {
    let first = authorize(state, tenant, headers, Grant::Read).await?;
    let id = ModelNumber::parse(id).map_err(|_| malformed())?.get();
    let expected = expected(query)?;
    let mut handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, headers, Grant::Read, &first).await?;
    let model = model(&mut handle, &authorization)?;
    let draft = model
        .store
        .open_draft(&authorization.caller, id)
        .map_err(source_error)?;
    draft
        .check_owner(&model.binding, &authorization.caller)
        .map_err(source_error)?;
    if draft.id != id {
        return Err(unavailable());
    }
    if draft.version != expected.expected_version.get() {
        return Err(source_error(ModelError::DraftConflict {
            current_version: draft.version,
        }));
    }
    if draft.base_generation != expected.expected_base.get() {
        return Err(source_error(ModelError::HeadConflict {
            current_generation: draft.base_generation,
        }));
    }
    let history = model
        .store
        .history(&authorization.caller)
        .map_err(|_| unavailable())?;
    if history.binding != model.binding {
        return Err(unavailable());
    }
    history.registry().map_err(|_| unavailable())?;
    draft.check_record(&history).map_err(|_| unavailable())?;
    let publication = history
        .publications
        .into_iter()
        .find(|row| row.generation == draft.base_generation)
        .ok_or_else(unavailable)?;
    current(&authorization)?;
    encoded(
        StatusCode::OK,
        ModelDraftBase {
            draft_id: ModelNumber::new(id).map_err(|_| unavailable())?,
            draft_version: expected.expected_version,
            base_generation: expected.expected_base,
            document: publication.document,
        }
        .encode(),
    )
}
