//! The operator status surface.
//!
//! It shipped as a hardcoded refusal reading "no policy decision point is
//! composed". That stopped being true the moment one was, and every read
//! route now calls `state.pep.decide` — so the process was explaining itself
//! with a sentence it no longer checked.
//!
//! Seven of its eight fields are things this caller may ALREADY see: the
//! aggregate observation `/metrics` publishes unauthenticated — `lag`,
//! `poisoned`, `contended` and `served` directly, with
//! `observed = served − lag_unknown` and
//! `unreadable = lag_unknown − contended` as exact differences of those
//! series — and the entity types `/v1/types` already returns them.
//! Composing those changes no disclosure boundary.
//!
//! `policy_version` is the exception and is NEW here — nothing else in this
//! facade exposes it. It is the version the guard actually loaded rather
//! than the constant `compose` asked for, so a process that fell back cannot
//! report its intent as its state, and an operator diagnosing a policy
//! refusal needs it. Naming the exception rather than folding it into the
//! union, because the union was the security argument.
//!
//! It is authorized rather than merely authenticated anyway, on the same bar
//! as `/v1/audit` and `/v1/types`. The fields this surface is designed to
//! grow — seed digests, attestations — are not public, and a surface that
//! starts unauthorized is one that has to be taken back later.
//!
//! The observation is process-wide and unlabelled by tenant, deliberately.
//! Per-tenant lag on an authenticated surface would still be a tenancy
//! oracle for any operator whose roster is narrower than the process's.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::composition::AppState;
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized, tenant_of};

#[derive(Debug, Serialize)]
pub(crate) struct StatusBody {
    /// The version the guard actually loaded, not the constant compose asked
    /// for — a process that fell back would otherwise report its intent.
    pub(crate) policy_version: String, // data_class: INTERNAL_ONLY
    pub(crate) served_tenants: u64, // data_class: INTERNAL_ONLY
    /// Tenants whose status this answer could actually read. A lag of zero
    /// over zero observed tenants is not health, and the two travel together
    /// so a reader cannot take one without the other.
    pub(crate) observed_tenants: u64, // data_class: INTERNAL_ONLY
    pub(crate) projection_lag: u64, // data_class: INTERNAL_ONLY
    pub(crate) poisoned_entries: u64, // data_class: INTERNAL_ONLY
    pub(crate) contended_tenants: u64, // data_class: INTERNAL_ONLY
    pub(crate) unreadable_tenants: u64, // data_class: INTERNAL_ONLY
    /// The caller's own tenant's declared types — the same set `/v1/types`
    /// serves this caller, named here so an operator can see what the
    /// process believes it is serving without a second request.
    /// `null` when the tenant was locked and the list could not be read.
    /// An empty list would be a lie: the seed always registers at least one
    /// type, so `[]` cannot honestly mean "declares none", and this read is
    /// a SECOND pass that may disagree with the aggregate above.
    pub(crate) entity_types: Option<Vec<String>>, // data_class: TENANT_SCOPED
}

pub async fn statusz(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let caller = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    // The SAME two gates the other tenant-wide views run, in the same order.
    // Authorization alone let a credential naming an unserved tenant read
    // this surface, which `/v1/audit` and `/v1/types` both refuse — and it
    // gave `entity_types: []` a third meaning on top of "contended" and
    // "none declared".
    let tenant = match tenant_of(&state, &caller) {
        Ok(tenant) => tenant,
        Err(response) => return *response,
    };
    let seen = crate::observation::observe(&state);
    // TRY, never wait. This surface is what an operator reaches for when the
    // process is wedged, and a wedge is precisely a tenant lock held across a
    // long replay — so blocking here would hang the one view that could
    // explain the hang.
    //
    // `null` rather than `[]` when it could not be read. This is a SECOND
    // pass: the aggregate above was one observation, this is another, and a
    // lock free during the first and held during the second would otherwise
    // publish `contended_tenants: 0` beside an empty list — a body claiming
    // the tenant declares no types, which the seed makes impossible. The
    // field carries its own honesty rather than borrowing it from a number
    // taken at a different instant.
    let entity_types = tenant.try_lock().ok().map(|tenant| {
        crate::seed::declared_entity_types(&tenant.projection.engine, &caller.tenant_id)
            .into_iter()
            .map(|definition| definition.id.value.clone())
            .collect()
    });
    state.metrics.read_served();
    Json(StatusBody {
        policy_version: state.pep.loaded_policy_version().as_str().to_owned(),
        served_tenants: state.tenant_count() as u64,
        observed_tenants: seen.observed,
        projection_lag: seen.lag,
        poisoned_entries: seen.poisoned,
        contended_tenants: seen.contended,
        unreadable_tenants: seen.unreadable,
        entity_types,
    })
    .into_response()
}
