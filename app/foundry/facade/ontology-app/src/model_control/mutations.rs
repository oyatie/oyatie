use super::authorization::{Grant, after_wait, authorize, current, model};
use super::transport::{Refusal, encoded, malformed, mutation_body, source_error, unavailable};
use super::*;
use axum::{
    extract::{Path, Request, State, rejection::PathRejection},
    http::StatusCode,
    response::Response,
};

#[derive(Clone, Copy)]
enum Mutation {
    Create,
    Save,
    Validate,
}
enum Input {
    Create(CreateModelDraft),
    Save(SaveModelDraft),
    Validate(PrepareModelDraft),
}

impl Mutation {
    fn grant(self) -> Grant {
        match self {
            Self::Create | Self::Save => Grant::Author,
            Self::Validate => Grant::Validate,
        }
    }
    fn limit(self) -> Result<usize, Refusal> {
        let limits = model_wire_limits().map_err(|_| unavailable())?;
        Ok(match self {
            Self::Create => limits.create,
            Self::Save => limits.save,
            Self::Validate => limits.prepare,
        })
    }
    fn decode(self, bytes: &[u8]) -> Result<Input, Refusal> {
        match self {
            Self::Create => CreateModelDraft::decode(bytes).map(Input::Create),
            Self::Save => SaveModelDraft::decode(bytes).map(Input::Save),
            Self::Validate => PrepareModelDraft::decode(bytes).map(Input::Validate),
        }
        .map_err(source_error)
    }
}

pub(crate) async fn create(
    State(state): State<ModelState>,
    path: Result<Path<String>, PathRejection>,
    request: Request,
) -> Response {
    let Ok(Path(tenant)) = path else {
        return *malformed();
    };
    run(&state, &tenant, None, Mutation::Create, request)
        .await
        .unwrap_or_else(|refusal| *refusal)
}
pub(crate) async fn save(
    State(state): State<ModelState>,
    path: Result<Path<(String, String)>, PathRejection>,
    request: Request,
) -> Response {
    let Ok(Path((tenant, id))) = path else {
        return *malformed();
    };
    run(&state, &tenant, Some(&id), Mutation::Save, request)
        .await
        .unwrap_or_else(|refusal| *refusal)
}
pub(crate) async fn validate(
    State(state): State<ModelState>,
    path: Result<Path<(String, String)>, PathRejection>,
    request: Request,
) -> Response {
    let Ok(Path((tenant, id))) = path else {
        return *malformed();
    };
    run(&state, &tenant, Some(&id), Mutation::Validate, request)
        .await
        .unwrap_or_else(|refusal| *refusal)
}

async fn run(
    state: &ModelState,
    tenant: &str,
    id: Option<&str>,
    mutation: Mutation,
    request: Request,
) -> Result<Response, Refusal> {
    let headers = request.headers().clone();
    let grant = mutation.grant();
    let first = authorize(state, tenant, &headers, grant).await?;
    let id = id
        .map(|id| ModelNumber::parse(id).map_err(|_| malformed()))
        .transpose()?;
    let bytes = mutation_body(state, &headers, request.into_body(), mutation.limit()?).await?;
    let input = mutation.decode(&bytes)?;
    let mut handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, &headers, grant, &first).await?;
    if handle.publication_unknown.is_some() && !matches!(mutation, Mutation::Validate) {
        return Err(unavailable());
    }
    let model = model(&mut handle, &authorization)?;
    let caller = &authorization.caller;
    let (status, draft) = match input {
        Input::Create(input) => (
            StatusCode::CREATED,
            model
                .store
                .create_draft(caller, input.expected_generation.get(), input.document),
        ),
        Input::Save(input) => (
            StatusCode::OK,
            model.store.save_draft(
                caller,
                id.ok_or_else(malformed)?.get(),
                input.expected_version.get(),
                input.expected_base.get(),
                input.document,
            ),
        ),
        Input::Validate(input) => {
            let id = id.ok_or_else(malformed)?;
            let result = model.store.prepare(
                caller,
                id.get(),
                input.expected_version.get(),
                input.expected_base.get(),
            );
            let diagnostics = match result {
                Ok(_) => vec![],
                Err(error) => match super::diagnostics::diagnostic(error.clone()) {
                    Some(diagnostic) => vec![diagnostic],
                    None => return Err(source_error(error)),
                },
            };
            current(&authorization)?;
            return encoded(
                StatusCode::OK,
                ModelValidation {
                    draft_id: id,
                    draft_version: input.expected_version,
                    base_generation: input.expected_base,
                    valid: diagnostics.is_empty(),
                    diagnostics,
                }
                .encode(),
            );
        }
    };
    let draft = draft.map_err(source_error)?;
    draft
        .check_owner(&model.binding, caller)
        .map_err(source_error)?;
    current(&authorization)?;
    encoded(
        status,
        ModelDraftWire::try_from(&draft)
            .map_err(|_| unavailable())?
            .encode(),
    )
}
