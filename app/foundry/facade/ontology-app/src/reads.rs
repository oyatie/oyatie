//! Cross-tenant refusals are deliberately indistinguishable from one
//! another: a caller outside the tenant gets the same answer whether the
//! object exists or not, because a distinguishable "not found" would make
//! this surface an existence oracle for a tenant the caller was never
//! entitled to ask about.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_records_draft::SealedEnvelope;
use foundry_spine::{
    PinnedObject, ViewError, audit_view, object_at_revision_in_store, object_history,
};

use crate::composition::{AppState, TenantState};
use crate::dto::RefusalBody;
use crate::read_dto::{AuditRow, EntityTypeRow, HistoryRow, RevisionPin, pinned_body};

mod authorization;
pub(crate) use authorization::{authorized, authorized_caller};

pub async fn object(
    State(state): State<Arc<AppState>>,
    Path(object_ref): Path<String>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let (caller, _decision, tenant) = match authorized(&state, &headers, &object_ref) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    object_response(&state, &caller.tenant_id, tenant, object_ref, raw_query).await
}

pub(crate) async fn object_response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
    object_ref: String,
    raw_query: Option<String>,
) -> Response {
    let RevisionPin::Pinned(revision) = RevisionPin::parse(raw_query.as_deref()) else {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "a read must pin the revision it understands: ?revision=N",
        );
    };
    let viewed = object_view(tenant, tenant_id, &object_ref, revision, None).await;
    object_view_response(state, object_ref, viewed)
}

pub(crate) async fn object_view(
    tenant: &tokio::sync::Mutex<TenantState>,
    tenant_id: &str,
    object_ref: &str,
    revision: u32,
    expected_type: Option<&data_ontology_kernel::EntityTypeId>,
) -> Result<PinnedObject, ViewError> {
    let tenant = tenant.lock().await;
    // The object is the store's: a reader sees its last taken write, and a
    // write the store did not take is lag on the sync surface until a
    // catch-up repairs it. The definitions the pin resolves against are
    // still the fold's registry, which is where revisions are retained.
    object_at_revision_in_store(
        &*tenant.projection_store,
        &tenant.projection.engine,
        tenant_id,
        object_ref,
        revision,
        expected_type,
    )
}

pub(crate) fn object_view_response(
    state: &AppState,
    object_ref: String,
    viewed: Result<PinnedObject, ViewError>,
) -> Response {
    match viewed {
        Ok(pinned) => {
            state.metrics.read_served();
            Json(pinned_body(object_ref, &pinned)).into_response()
        }
        Err(ViewError::UnknownObject) => {
            state.metrics.read_refused();
            refuse(
                StatusCode::NOT_FOUND,
                "surface",
                "the durable projection holds no object at this reference",
            )
        }
        Err(ViewError::UnretainedRevision) => {
            state.metrics.read_refused();
            refuse(StatusCode::CONFLICT, "surface", UNRETAINED_REVISION_CAUSE)
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

fn entries_or_refuse(
    state: &AppState,
    tenant: &crate::composition::TenantState,
    tenant_id: &str,
) -> Result<Vec<SealedEnvelope>, Box<Response>> {
    tenant.entries_now(tenant_id).map_err(|_| {
        state.metrics.read_refused();
        Box::new(refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "log",
            "the action log could not be read; this view is unavailable",
        ))
    })
}

pub async fn history(
    State(state): State<Arc<AppState>>,
    Path(object_ref): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (caller, _decision, tenant) = match authorized(&state, &headers, &object_ref) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    history_response(&state, &caller.tenant_id, tenant, &object_ref).await
}

pub(crate) async fn history_response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
    object_ref: &str,
) -> Response {
    let tenant = tenant.lock().await;
    let entries = match entries_or_refuse(state, &tenant, tenant_id) {
        Ok(entries) => entries,
        Err(response) => return *response,
    };
    let rows: Vec<HistoryRow> = object_history(&tenant.projection, &entries, object_ref)
        .into_iter()
        .map(|entry| HistoryRow {
            ordinal: entry.ordinal,
            object_sequence: entry.object_sequence,
            principal_id: entry.principal_id,
            decision_id: entry.decision_id,
            action_type: entry.action_type,
            audit_event_type: entry.audit_event_type,
            schema_revision: entry.schema_revision,
        })
        .collect();
    state.metrics.read_served();
    Json(rows).into_response()
}

pub async fn audit(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let (caller, _decision, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    audit_response(&state, &caller.tenant_id, tenant).await
}

pub(crate) async fn audit_response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
) -> Response {
    let tenant = tenant.lock().await;
    let entries = match entries_or_refuse(state, &tenant, tenant_id) {
        Ok(entries) => entries,
        Err(response) => return *response,
    };
    let rows: Vec<AuditRow> = audit_view(&tenant.projection, &entries)
        .into_iter()
        .map(|entry| {
            let (disposition, poison_reason) = match &entry.disposition {
                foundry_spine::AuditDisposition::Applied => ("applied", None),
                foundry_spine::AuditDisposition::Poisoned(reason) => {
                    ("poisoned", Some(format!("{reason:?}")))
                }
            };
            AuditRow {
                ordinal: entry.ordinal,
                object_ref: entry.object_ref,
                action_type: entry.action_type,
                disposition,
                poison_reason,
                principal_id: entry.principal_id,
                decision_id: entry.decision_id,
            }
        })
        .collect();
    state.metrics.read_served();
    Json(rows).into_response()
}

pub async fn types(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let (caller, _decision, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    types_response(&state, &caller.tenant_id, tenant).await
}

pub(crate) async fn types_response(
    state: &AppState,
    tenant_id: &str,
    tenant: &tokio::sync::Mutex<TenantState>,
) -> Response {
    let tenant = tenant.lock().await;
    let rows: Vec<EntityTypeRow> =
        crate::seed::declared_entity_types(&tenant.projection.engine, tenant_id)
            .into_iter()
            .map(|definition| EntityTypeRow {
                entity_type: definition.id.value.clone(),
                revision: definition.revision,
                properties: definition
                    .properties
                    .iter()
                    .map(|property| property.name.clone())
                    .collect(),
            })
            .collect();
    state.metrics.read_served();
    Json(rows).into_response()
}

/// The resource a tenant-wide read is authorized against. It is an
/// `ent_`-shaped id because the kernel's authorization vocabulary requires
/// one; it names the tenant's own view, not any object in it.
pub(crate) const TENANT_SCOPED_RESOURCE: &str = "ent_tenant_view";

/// The cause a read gives for a revision its tenant never accepted. Shared, so
/// the single-object read and the paged reads cannot answer one state with two
/// sentences.
pub(crate) const UNRETAINED_REVISION_CAUSE: &str =
    "that revision was never accepted for this entity type";

/// The entity type `raw` names, if this tenant declares it at any revision.
/// A malformed id and an undeclared one are one answer: which of the two a
/// caller's guess was is not this surface's to disclose. Separate from a pin
/// refusal, so a type a tenant never declared is never reported as a revision
/// it never accepted.
pub(crate) fn declared_type(
    engine: &data_ontology_kernel::OntologyEngine,
    tenant_id: &str,
    raw: &str,
) -> Option<data_ontology_kernel::EntityTypeId> {
    data_ontology_kernel::EntityTypeId::new(raw)
        .ok()
        .filter(|id| engine.entity_type(tenant_id, id).is_some())
}

pub(crate) fn refuse(status: StatusCode, gate: &str, cause: &str) -> Response {
    (
        status,
        Json(RefusalBody {
            gate: gate.to_owned(),
            cause: cause.to_owned(),
        }),
    )
        .into_response()
}
