//! One page of an object set: `POST /v1/object-sets/page`.
//!
//! The page is authorized against the tenant's own view rather than any
//! object on it, as a listing is: a set names no single object to authorize.
//! Every row is built by the same pinned view that answers a single read and
//! a listing, and a leaf that filters passes the listing's own pin guard, so
//! a set cannot answer a question the other two refuse.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_spine::{MAX_SET_LEAVES, MAX_SET_MEMBERS, SetError, materialize_object_set};

use crate::composition::AppState;
use crate::listing::refuse_page_error;
use crate::listing_query::QueryRefusal;
use crate::object_set_body::SetPageRequest;
use crate::read_dto::{ListingBody, pinned_body};
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized, declared_type, refuse};

pub async fn page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let (caller, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    let request = match SetPageRequest::parse(&body) {
        Ok(request) => request,
        Err(refusal) => {
            state.metrics.read_refused();
            return refuse(StatusCode::BAD_REQUEST, "surface", refusal.cause());
        }
    };
    let tenant = tenant.lock().await;
    let Some(entity_type) = declared_type(
        &tenant.projection.engine,
        &caller.tenant_id,
        &request.entity_type,
    ) else {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "this tenant declares no entity type by that id",
        );
    };
    let revision = request.revision;
    let page = request.page.clone();
    match materialize_object_set(
        &*tenant.projection_store,
        &tenant.projection.engine,
        &caller.tenant_id,
        &request.set(entity_type),
        revision,
        &page,
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
        // A leaf refuses for the reasons a listing's page refuses, with the
        // statuses the listing gives them.
        Err(SetError::Leaf(error)) => refuse_page_error(&state, &error),
        Err(SetError::TooManyMembers) => {
            state.metrics.read_refused();
            refuse(
                StatusCode::BAD_REQUEST,
                "surface",
                &format!(
                    "no step of a set may take more than {MAX_SET_MEMBERS} objects; narrow it"
                ),
            )
        }
        // This route's grammar admits a limit of 1..=1000, so it cannot reach
        // this arm; a caller that builds its own page request can, and gets the
        // same cause the grammar would have given.
        Err(SetError::UnusablePage) => {
            state.metrics.read_refused();
            refuse(
                StatusCode::BAD_REQUEST,
                "surface",
                QueryRefusal::UnusableLimit.cause(),
            )
        }
        Err(SetError::TooManyLeaves) => {
            state.metrics.read_refused();
            refuse(
                StatusCode::BAD_REQUEST,
                "surface",
                &format!("a set may read at most {MAX_SET_LEAVES} leaves"),
            )
        }
    }
}
