//! A tenant's objects of one entity type, one page at a time.
//!
//! The page is authorized against the tenant's own view rather than any
//! object on it, because a listing has no single object to name. Every row
//! is filtered to the pinned revision by the same view that answers a
//! single read, so a listed object and a fetched one never disagree.

use std::sync::Arc;

use axum::Json;
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_spine::{PageError, objects_of_type_at_revision};

use crate::composition::AppState;
use crate::listing_query::ListingQuery;
use crate::read_dto::{ListingBody, pinned_body};
use crate::reads::{
    TENANT_SCOPED_RESOURCE, UNRETAINED_REVISION_CAUSE, authorized, declared_type, refuse,
};

pub async fn objects(
    State(state): State<Arc<AppState>>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let (caller, _decision, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    let query = match ListingQuery::parse(raw_query.as_deref()) {
        Ok(query) => query,
        Err(refusal) => {
            state.metrics.read_refused();
            return refuse(StatusCode::BAD_REQUEST, "surface", refusal.cause());
        }
    };
    let tenant = tenant.lock().await;
    let declared = declared_type(
        &tenant.projection.engine,
        &caller.tenant_id,
        &query.entity_type,
    );
    let Some(entity_type) = declared else {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "this tenant declares no entity type by that id",
        );
    };
    match objects_of_type_at_revision(
        &*tenant.projection_store,
        &tenant.projection.engine,
        &caller.tenant_id,
        &entity_type,
        query.revision,
        &query.page(),
        query.filter.as_ref(),
    ) {
        Ok(page) => {
            state.metrics.read_served();
            Json(ListingBody {
                objects: page
                    .objects
                    .iter()
                    .map(|(object_ref, pinned)| pinned_body(object_ref.clone(), pinned))
                    .collect(),
                next: page.next.map(|cursor| cursor.after_object_ref),
            })
            .into_response()
        }
        Err(error) => refuse_page_error(&state, &error),
    }
}

/// The refusal a page error is answered with. Shared by every route that serves
/// a PAGE of pinned rows, so one cause cannot take two statuses. The
/// single-object read answers a `ViewError`, which has an arm no page can reach,
/// and shares this mapping only through [`UNRETAINED_REVISION_CAUSE`].
pub(crate) fn refuse_page_error(state: &AppState, error: &PageError) -> Response {
    state.metrics.read_refused();
    match error {
        PageError::UnretainedRevision => {
            refuse(StatusCode::CONFLICT, "surface", UNRETAINED_REVISION_CAUSE)
        }
        PageError::UndeclaredFilterProperty => refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the pinned revision declares no property by that name",
        ),
        // The caller's bounds met a stored value of another kind. Not a
        // store fault, so not a 503.
        PageError::FilterKindMismatch { .. } => refuse(
            StatusCode::CONFLICT,
            "surface",
            "the stored values of that property are of another kind",
        ),
        PageError::StoreUnreadable(_) => refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "store",
            "the projection store could not be read",
        ),
    }
}
