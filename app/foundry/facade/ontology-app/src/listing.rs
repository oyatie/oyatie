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
use data_ontology_kernel::EntityTypeId;
use foundry_projection_draft::{PageRequest, ProjectionCursor};
use foundry_spine::{ViewError, objects_of_type_at_revision};

use crate::composition::AppState;
use crate::read_dto::{ListingBody, pinned_body};
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized, refuse};

/// The page size a caller that names none is served.
pub(crate) const DEFAULT_PAGE_LIMIT: usize = 100;
/// The largest page this surface will serve, so one request cannot ask the
/// store for a tenant's whole type.
pub(crate) const MAX_PAGE_LIMIT: usize = 1000;

/// `?type=&revision=&limit=&cursor=`, parsed from the RAW query string
/// inside the handler for the reason `RevisionPin` states: a typed
/// extractor refuses a malformed value before the credential is ever read.
///
/// A key this route does not define is refused rather than ignored, so a
/// parameter the surface drops can never be mistaken for one it honoured.
pub(crate) struct ListingQuery {
    pub(crate) entity_type: String,
    pub(crate) revision: u32,
    pub(crate) limit: usize,
    pub(crate) cursor: Option<ProjectionCursor>,
}

/// Why a query string cannot be served. Each names what was measured, so
/// no refusal stands for two different mistakes.
pub(crate) enum QueryRefusal {
    UnknownParameter,
    RepeatedParameter,
    MissingType,
    UnusableRevision,
    UnusableLimit,
    UnusableCursor,
}

impl QueryRefusal {
    pub(crate) fn cause(&self) -> &'static str {
        match self {
            Self::UnknownParameter => "this listing defines type, revision, limit and cursor only",
            Self::RepeatedParameter => "a parameter given twice has no single honest answer",
            Self::MissingType => "a listing must name the entity type it lists: ?type=ety_...",
            Self::UnusableRevision => "a read must pin the revision it understands: ?revision=N",
            Self::UnusableLimit => "?limit= must be a whole number from 1 to 1000",
            Self::UnusableCursor => "?cursor= must be the object reference a page ended on",
        }
    }
}

impl ListingQuery {
    /// Canonical form only: keys and values are matched literally and are
    /// NOT percent-decoded, the narrowing `RevisionPin::parse` documents.
    pub(crate) fn parse(raw: Option<&str>) -> Result<Self, QueryRefusal> {
        let (mut entity_type, mut revision, mut limit, mut cursor) = (None, None, None, None);
        for pair in raw.unwrap_or_default().split('&').filter(|p| !p.is_empty()) {
            let Some((key, value)) = pair.split_once('=') else {
                return Err(QueryRefusal::UnknownParameter);
            };
            let slot = match key {
                "type" => &mut entity_type,
                "revision" => &mut revision,
                "limit" => &mut limit,
                "cursor" => &mut cursor,
                _ => return Err(QueryRefusal::UnknownParameter),
            };
            if slot.replace(value.to_owned()).is_some() {
                return Err(QueryRefusal::RepeatedParameter);
            }
        }
        let entity_type = entity_type.ok_or(QueryRefusal::MissingType)?;
        let revision = revision
            .ok_or(QueryRefusal::UnusableRevision)?
            .parse::<u32>()
            .map_err(|_| QueryRefusal::UnusableRevision)?;
        let limit = match limit {
            None => DEFAULT_PAGE_LIMIT,
            Some(text) => match text.parse::<usize>() {
                Ok(limit) if (1..=MAX_PAGE_LIMIT).contains(&limit) => limit,
                _ => return Err(QueryRefusal::UnusableLimit),
            },
        };
        let cursor = match cursor {
            None => None,
            Some(after_object_ref) if after_object_ref.starts_with("ent_") => {
                Some(ProjectionCursor { after_object_ref })
            }
            Some(_) => return Err(QueryRefusal::UnusableCursor),
        };
        Ok(Self {
            entity_type,
            revision,
            limit,
            cursor,
        })
    }

    fn page(&self) -> PageRequest {
        match &self.cursor {
            None => PageRequest::first(self.limit),
            Some(cursor) => PageRequest::after(self.limit, cursor.clone()),
        }
    }
}

pub async fn objects(
    State(state): State<Arc<AppState>>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let (caller, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
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
    // A malformed id and an undeclared one are one refusal: which of the
    // two a caller's guess was is not this surface's to disclose. It is
    // separate from the pin refusal below, so a type this tenant has never
    // declared is never reported as a revision it never accepted.
    let declared = EntityTypeId::new(query.entity_type.clone())
        .ok()
        .filter(|id| {
            tenant
                .projection
                .engine
                .entity_type(&caller.tenant_id, id)
                .is_some()
        });
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
        // The paged view resolves the pin and then reads a page; it has no
        // single object to miss, so `UnknownObject` is unreachable here.
        Err(ViewError::UnretainedRevision | ViewError::UnknownObject) => {
            state.metrics.read_refused();
            refuse(
                StatusCode::CONFLICT,
                "surface",
                "that revision was never accepted for this entity type",
            )
        }
        Err(ViewError::StoreUnreadable(_)) => {
            state.metrics.read_refused();
            refuse(
                StatusCode::SERVICE_UNAVAILABLE,
                "store",
                "the projection store could not be read",
            )
        }
    }
}
