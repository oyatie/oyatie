//! Authorized, not merely authenticated, even though its aggregate counts
//! are already visible on, or derivable from, `/metrics`: the per-tenant
//! blocks, and the seed digests and attestations this surface is designed
//! to grow, are not public, and a surface that starts unauthorized has to be
//! taken back later.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::composition::{AppState, TenantState};
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized};
use crate::seed;

#[derive(Debug, Serialize)]
pub(crate) struct StatusBody {
    pub(crate) policy_version: String,  // data_class: INTERNAL_ONLY
    pub(crate) served_tenants: u64,     // data_class: INTERNAL_ONLY
    pub(crate) observed_tenants: u64,   // data_class: INTERNAL_ONLY
    pub(crate) projection_lag: u64,     // data_class: INTERNAL_ONLY
    pub(crate) poisoned_entries: u64,   // data_class: INTERNAL_ONLY
    pub(crate) contended_tenants: u64,  // data_class: INTERNAL_ONLY
    pub(crate) unreadable_tenants: u64, // data_class: INTERNAL_ONLY
    /// `null` when the tenant was locked and the list could not be read.
    /// An empty list would be a lie: the seed always registers at least one
    /// type, so `[]` cannot honestly mean "declares none", and this read is
    /// a SECOND pass that may disagree with the aggregate above. The two
    /// blocks below are read under the same acquisition and go `null` with it.
    pub(crate) entity_types: Option<Vec<String>>, // data_class: TENANT_SCOPED
    /// Also `null` when the log head or the projection store is unreadable:
    /// a sync status with a guessed side would be a lie.
    pub(crate) tenant: Option<TenantSync>, // data_class: INTERNAL_ONLY
    /// The registry snapshot the caller's tenant's fold was seeded with,
    /// which is the one its writes are stamped from. Not the store-side
    /// registry identity the design's Limit 1 names; nothing records that yet.
    pub(crate) registry: Option<Registry>, // data_class: TENANT_SCOPED
}

#[derive(Debug, Serialize)]
pub(crate) struct TenantSync {
    pub(crate) log_head: u64,                       // data_class: INTERNAL_ONLY
    pub(crate) applied_ordinal: u64,                // data_class: INTERNAL_ONLY
    pub(crate) lag: u64,                            // data_class: INTERNAL_ONLY
    pub(crate) poisoned_entries: u64,               // data_class: INTERNAL_ONLY
    pub(crate) first_poisoned_ordinal: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Debug, Serialize)]
pub(crate) struct Registry {
    pub(crate) entity_types: Vec<DeclaredType>, // data_class: TENANT_SCOPED
    pub(crate) action_types: Vec<DeclaredType>, // data_class: TENANT_SCOPED
}

#[derive(Debug, Serialize)]
pub(crate) struct DeclaredType {
    pub(crate) id: String,    // data_class: TENANT_SCOPED
    pub(crate) revision: u32, // data_class: TENANT_SCOPED
}

pub async fn statusz(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let (caller, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    let seen = crate::observation::observe(&state);
    let (entity_types, sync, registry) = match facts_without_waiting(tenant, &caller.tenant_id) {
        Some((entity_types, sync, registry)) => (Some(entity_types), sync, Some(registry)),
        None => (None, None, None),
    };
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
        tenant: sync,
        registry,
    })
    .into_response()
}

type TenantFacts = (Vec<String>, Option<TenantSync>, Registry);

fn facts_without_waiting(
    tenant: &tokio::sync::Mutex<TenantState>,
    tenant_id: &str,
) -> Option<TenantFacts> {
    let tenant = tenant.try_lock().ok()?;
    let engine = &tenant.projection.engine;
    let entity_types = seed::declared_entity_types(engine, tenant_id);
    // The fold input, not `engine`: it is the registry the writer stamps
    // revisions from, so a revision served here predicts what a write does.
    let registry_input = &tenant.projection.registry_input;
    let sync = tenant.sync_status(tenant_id).ok().map(|status| TenantSync {
        log_head: status.head,
        applied_ordinal: status.applied_ordinal,
        lag: status.lag,
        poisoned_entries: status.poisoned_count,
        first_poisoned_ordinal: status.first_poisoned_ordinal,
    });
    let registry = Registry {
        entity_types: seed::declared_entity_types(registry_input, tenant_id)
            .into_iter()
            .map(|definition| DeclaredType {
                id: definition.id.value.clone(),
                revision: definition.revision,
            })
            .collect(),
        action_types: seed::declared_action_types(registry_input, tenant_id)
            .into_iter()
            .map(|definition| DeclaredType {
                id: definition.id.value.clone(),
                revision: definition.revision,
            })
            .collect(),
    };
    let names = entity_types
        .into_iter()
        .map(|definition| definition.id.value.clone())
        .collect();
    Some((names, sync, registry))
}
