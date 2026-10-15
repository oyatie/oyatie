use super::authorization::{Grant, after_wait, authorize, current, model};
use super::transport::{Refusal, encoded, malformed, mutation_body, source_error, unavailable};
use super::*;
use crate::model_runtime::{StagedPublication, StagedRecovery};
use axum::{
    extract::{Path, Request, State, rejection::PathRejection},
    http::StatusCode,
    response::Response,
};

pub(crate) async fn publish(
    State(state): State<ModelState>,
    path: Result<Path<(String, String)>, PathRejection>,
    request: Request,
) -> Response {
    let Ok(Path((tenant, id))) = path else {
        return *malformed();
    };
    run(&state, &tenant, &id, request)
        .await
        .unwrap_or_else(|refusal| *refusal)
}

async fn run(
    state: &ModelState,
    tenant: &str,
    id: &str,
    request: Request,
) -> Result<Response, Refusal> {
    let headers = request.headers().clone();
    let first = authorize(state, tenant, &headers, Grant::Publish).await?;
    let id = ModelNumber::parse(id).map_err(|_| malformed())?.get();
    let limit = model_wire_limits().map_err(|_| unavailable())?.prepare;
    let bytes = mutation_body(state, &headers, request.into_body(), limit).await?;
    let input = PrepareModelDraft::decode(&bytes).map_err(source_error)?;
    let mut handle = state
        .app
        .tenants
        .get(tenant)
        .ok_or_else(unavailable)?
        .lock()
        .await;
    let authorization = after_wait(state, tenant, &headers, Grant::Publish, &first).await?;
    if handle.observed_model_generation().is_none() {
        return Err(unavailable());
    }
    if let Some(pending) = &handle.publication_unknown
        && (pending.owner != authorization.caller.owner
            || pending.draft_id != id
            || pending.draft_version != input.expected_version.get()
            || pending.prior_generation != input.expected_base.get())
    {
        return Err(unavailable());
    }
    // A saved document may already be closed, including below a newer head.
    let receipt = model(&mut handle, &authorization)?
        .store
        .receipt(
            &authorization.caller,
            id,
            input.expected_version.get(),
            input.expected_base.get(),
        )
        .map_err(source_error)?;
    if let Some(receipt) = &receipt {
        if receipt.owner != authorization.caller.owner
            || receipt.draft_id != id
            || receipt.draft_version != input.expected_version.get()
            || receipt.prior_generation != input.expected_base.get()
        {
            return Err(unavailable());
        }
        ModelReceiptWire::try_from(receipt)
            .and_then(|wire| wire.check_bounds())
            .map_err(|_| unavailable())?;
    }
    if handle.publication_unknown.is_some() {
        let recovery = StagedRecovery::prepare(&handle, &authorization.caller, receipt.as_ref())
            .map_err(source_error)?;
        after_wait(state, tenant, &headers, Grant::Publish, &authorization).await?;
        // An uncommitted retry clears the freeze, but does not also start a new commit.
        return recovery
            .install(&mut handle)
            .ok_or_else(unavailable)
            .and_then(|bytes| encoded(StatusCode::OK, Ok(bytes)));
    }
    if let Some(receipt) = receipt {
        let lifecycle = crate::model_runtime::lifecycle(&handle, &authorization.caller)
            .map_err(source_error)?;
        let response = ModelPublished {
            receipt: ModelReceiptWire::try_from(&receipt).map_err(|_| unavailable())?,
            lifecycle,
        }
        .encode();
        current(&authorization)?;
        return encoded(StatusCode::OK, response);
    }
    let staged = StagedPublication::prepare(&handle, &authorization.caller, id, &input)
        .map_err(source_error)?;
    let authorization = after_wait(state, tenant, &headers, Grant::Publish, &authorization).await?;
    // Commit and cached install are synchronous under the same tenant and native ownership.
    encoded(
        StatusCode::OK,
        staged.commit(&mut handle, &authorization.caller),
    )
}
