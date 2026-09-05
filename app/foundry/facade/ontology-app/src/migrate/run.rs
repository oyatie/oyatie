//! `POST /v1/migrations/run` — execute a plan to its fixpoint.
//!
//! The executing half of the migration surfaces, and the one that WRITES. It
//! is gated on `Invoke` rather than `Use`, and every refusal returns before
//! the runner is reached: a migration that half-ran and then refused is worse
//! than one that never started, because the operator's next decision is made
//! against a population no plan describes.
//!
//! THE AUTHORITY IS THE CALLER'S OWN DECISION — and precisely which part of
//! it, because the looser claim was wrong. `MigrationAuthority` carries a
//! `decision_id`, the surfaces allowed and an autonomy tier, and the runner
//! stamps all three onto every upcast it writes. Only the decision and the
//! principal come from the PDP: `authz.rs` fills `allowed_surfaces` and
//! `autonomy_tier` with process constants on every Allow, so calling the
//! whole struct "the PDP's answer" would tell the next reader that two of its
//! fields carry an authority they do not.
//!
//! What must never happen is a `decision_id` this process invented. That puts
//! an authorization into the durable record that no policy engine granted —
//! an audit trail asserting a decision was made when none was — and nothing
//! downstream notices, because the kernel only checks the id is non-empty.
//! The suite pins it by VALUE against the seeded generator, having twice been
//! written in shapes that a fabrication satisfied.
//!
//! On `Invoke` versus `Use`: no credential this process can mint holds one
//! without the other, because `foundry-policies.cedar` scopes both permits to
//! `Role::"foundry-operator"`. That is a property of the policy rather than a
//! gap in these tests, and it is why the gate is pinned by a roleless caller
//! writing nothing rather than by a caller who may read but not act. The
//! surfaces stay distinct because the POLICY may one day separate them, and a
//! write asking for read permission would then be a silent escalation.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_spine::{MigrationAuthority, MigrationPlan, run_to_fixpoint};
use serde::Serialize;

use super::PlanRequest;
use crate::auth::{authenticate, bearer_token};
use crate::composition::AppState;
use crate::pdp::Surface;
use crate::reads::{TENANT_SCOPED_RESOURCE, refuse};

/// What one run did and where the population stands after it.
///
/// EVERY field the runner reports, not a chosen three. A run that stopped
/// short is the case an operator most needs to see, and it is invisible in a
/// body carrying only totals: `fixpoint` is the verdict, and `refused`,
/// `conflicted` and `poisoned` are the three ways an object can be owed and
/// stay owed. Reporting the flattering subset would have made a store outage
/// mid-migration look like a completed one.
#[derive(Debug, Serialize)]
pub(crate) struct RunBody {
    pub(crate) total: u64,       // data_class: INTERNAL_ONLY
    pub(crate) upcast: u64,      // data_class: INTERNAL_ONLY
    pub(crate) pending: u64,     // data_class: INTERNAL_ONLY
    pub(crate) refused: u64,     // data_class: INTERNAL_ONLY
    pub(crate) conflicted: u64,  // data_class: INTERNAL_ONLY
    pub(crate) unavailable: u64, // data_class: INTERNAL_ONLY
    pub(crate) poisoned: u64,    // data_class: INTERNAL_ONLY
    pub(crate) fixpoint: bool,   // data_class: INTERNAL_ONLY
}

pub async fn run(State(state): State<Arc<AppState>>, headers: HeaderMap, body: String) -> Response {
    let Some(token) = bearer_token(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    ) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "no bearer credential",
        );
    };
    let Some(caller) = authenticate(&state.operators, token) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "the presented credential is not recognized",
        );
    };
    let Ok(request) = serde_json::from_str::<PlanRequest>(&body) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the request is not a migration plan",
        );
    };

    // AUTHORIZE before anything is built, and keep the decision: it is the
    // authority the runner stamps, not merely a yes.
    let Ok(decision) = state
        .pep
        .decide(&caller, Surface::Invoke, TENANT_SCOPED_RESOURCE)
    else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision point refused this invocation",
        );
    };

    // The credential's tenant, checked rather than substituted.
    if request.tenant_id != caller.tenant_id {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the plan names a tenant other than the credential's",
        );
    }
    let Some(tenant) = state.tenants.get(&caller.tenant_id) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential names a tenant this process does not serve",
        );
    };
    let mut transforms = Vec::with_capacity(request.transforms.len());
    for wire in request.transforms {
        match wire.into_domain() {
            Ok(transform) => transforms.push(transform),
            Err(cause) => {
                state.metrics.submit_refused();
                return refuse(StatusCode::BAD_REQUEST, "surface", cause);
            }
        }
    }
    let plan = MigrationPlan {
        tenant_id: request.tenant_id,
        entity_type: request.entity_type,
        from_revision: request.from_revision,
        to_revision: request.to_revision,
        action_type: request.action_type,
        audit_event_type: request.audit_event_type,
        declared_at_epoch_seconds: request.declared_at_epoch_seconds,
        transforms,
    };
    let authority = MigrationAuthority {
        principal_id: decision.principal_id,
        decision_id: decision.decision_id,
        allowed_surfaces: decision.allowed_surfaces,
        autonomy_tier: decision.autonomy_tier,
    };

    let mut tenant = tenant.lock().await;
    let (log, denial_log, projection) = tenant.write_handles();
    match run_to_fixpoint(&plan, &authority, log, denial_log, projection) {
        Ok(status) => {
            state.metrics.submit_served();
            Json(RunBody {
                total: status.total,
                upcast: status.upcast,
                pending: status.pending,
                refused: status.refused,
                conflicted: status.conflicted,
                unavailable: status.unavailable,
                poisoned: status.poisoned,
                fixpoint: status.fixpoint,
            })
            .into_response()
        }
        Err(error) => {
            state.metrics.submit_refused();
            refuse(
                StatusCode::BAD_REQUEST,
                "surface",
                &format!("the plan is not executable: {error:?}"),
            )
        }
    }
}
