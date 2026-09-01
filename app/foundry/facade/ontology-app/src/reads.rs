//! The read surface: pinned object reads, per-object history, the
//! governance audit view, and the type registry.
//!
//! Every read is authorized by the READ action, separately from the write
//! action — a read-only operator must be able to open what the shell
//! renders for them, and recognizing a credential is not the same as
//! permitting it.
//!
//! Cross-tenant refusals are deliberately indistinguishable from one
//! another: a caller outside the tenant gets the same answer whether the
//! object exists or not, because a distinguishable "not found" would make
//! this surface an existence oracle for a tenant the caller was never
//! entitled to ask about.
//!
//! Reads serve the in-memory fold plus the per-tenant entries mirror. The
//! durable indexed store is a separate lane's evidence; nothing here
//! claims `store == fold(log)`.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_spine::{ViewError, audit_view, object_at_revision, object_history};

use crate::auth::{authenticate, bearer_token};
use crate::authz::Caller;
use crate::composition::AppState;
use crate::dto::RefusalBody;
use crate::pdp::Surface;
use crate::read_dto::{
    AuditRow, EntityTypeRow, HistoryRow, PinnedObjectBody, PropertyBody, RevisionPin, json_value,
};

/// Authenticate, then authorize the read action. Both failures are the
/// caller's answer; neither reveals whether the addressed object exists.
fn authorized(
    state: &AppState,
    headers: &HeaderMap,
    object_ref: &str,
) -> Result<Caller, Box<Response>> {
    let Some(token) = bearer_token(headers.get("authorization").and_then(|v| v.to_str().ok()))
    else {
        return Err(Box::new(refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "no bearer credential",
        )));
    };
    let Some(caller) = authenticate(&state.operators, token) else {
        return Err(Box::new(refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "the presented credential is not recognized",
        )));
    };
    if state.pep.decide(&caller, Surface::Use, object_ref).is_err() {
        return Err(Box::new(refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision point refused this read",
        )));
    }
    Ok(caller)
}

/// The tenant is the credential's; a caller can never read another's.
fn tenant_of<'a>(
    state: &'a AppState,
    caller: &Caller,
) -> Result<&'a tokio::sync::Mutex<crate::composition::TenantState>, Box<Response>> {
    state.tenants.get(&caller.tenant_id).ok_or_else(|| {
        Box::new(refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential names a tenant this process does not serve",
        ))
    })
}

/// `GET /v1/objects/{object_ref}?revision=N`
pub async fn object(
    State(state): State<Arc<AppState>>,
    Path(object_ref): Path<String>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let caller = match authorized(&state, &headers, &object_ref) {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    let tenant = match tenant_of(&state, &caller) {
        Ok(tenant) => tenant,
        Err(response) => return *response,
    };
    let RevisionPin::Pinned(revision) = RevisionPin::parse(raw_query.as_deref()) else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "a read must pin the revision it understands: ?revision=N",
        );
    };
    let tenant = tenant.lock().await;
    match object_at_revision(&tenant.projection, &object_ref, revision, None) {
        Ok(pinned) => Json(PinnedObjectBody {
            object_ref,
            written_revision: pinned.written_revision,
            upcast_state: match pinned.upcast_state {
                foundry_spine::UpcastState::Current => "current",
                foundry_spine::UpcastState::UpcastPending => "upcast_pending",
            },
            properties: pinned
                .properties
                .iter()
                .map(|(name, property)| {
                    (
                        name.clone(),
                        PropertyBody {
                            value_type: property.value.value.type_label(),
                            data_class: property.value.data_class.label(),
                            value: json_value(&property.value.value),
                        },
                    )
                })
                .collect(),
        })
        .into_response(),
        Err(ViewError::UnknownObject) => refuse(
            StatusCode::NOT_FOUND,
            "surface",
            "no applied entry ever bound this object",
        ),
        Err(ViewError::UnretainedRevision) => refuse(
            StatusCode::CONFLICT,
            "surface",
            "that revision was never accepted for this entity type",
        ),
    }
}

/// `GET /v1/objects/{object_ref}/history`
pub async fn history(
    State(state): State<Arc<AppState>>,
    Path(object_ref): Path<String>,
    headers: HeaderMap,
) -> Response {
    let caller = match authorized(&state, &headers, &object_ref) {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    let tenant = match tenant_of(&state, &caller) {
        Ok(tenant) => tenant,
        Err(response) => return *response,
    };
    let tenant = tenant.lock().await;
    let rows: Vec<HistoryRow> = object_history(&tenant.projection, &tenant.entries, &object_ref)
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
    Json(rows).into_response()
}

/// `GET /v1/audit`
pub async fn audit(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    // The audit view is tenant-wide, so it is authorized against the
    // tenant itself rather than any one object.
    let caller = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    let tenant = match tenant_of(&state, &caller) {
        Ok(tenant) => tenant,
        Err(response) => return *response,
    };
    let tenant = tenant.lock().await;
    let rows: Vec<AuditRow> = audit_view(&tenant.projection, &tenant.entries)
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
    Json(rows).into_response()
}

/// `GET /v1/types` — the registry the writer stamps against, so a reader
/// knows which revisions exist to pin. This is the Ontology Manager seam.
pub async fn types(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let caller = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    let tenant = match tenant_of(&state, &caller) {
        Ok(tenant) => tenant,
        Err(response) => return *response,
    };
    let tenant = tenant.lock().await;
    let rows: Vec<EntityTypeRow> =
        crate::seed::declared_entity_types(&tenant.projection.engine, &caller.tenant_id)
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
    Json(rows).into_response()
}

/// The resource a tenant-wide read is authorized against. It is an
/// `ent_`-shaped id because the kernel's authorization vocabulary requires
/// one; it names the tenant's own view, not any object in it.
const TENANT_SCOPED_RESOURCE: &str = "ent_tenant_view";

fn refuse(status: StatusCode, gate: &str, cause: &str) -> Response {
    (
        status,
        Json(RefusalBody {
            gate: gate.to_owned(),
            cause: cause.to_owned(),
        }),
    )
        .into_response()
}
