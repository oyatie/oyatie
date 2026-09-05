//! The migration surfaces.
//!
//! `POST /v1/migrations/attest` reports what a plan still owes. It is a READ
//! of the projection dressed as a POST, because a plan does not fit in a
//! query string — so it is gated on `Use`, not `Invoke`, and it mutates
//! nothing. The attestation is pure and recomputable at any time.
//!
//! THE TENANT IS THE CREDENTIAL'S. `MigrationPlan` carries its own
//! `tenant_id`, and the write path already settled what that means: nothing
//! in the body may move the tenant. Here the stakes are a read — a plan
//! naming another tenant would otherwise report that tenant's pending
//! objects and its poisoned ordinals to a caller who may not read them — so
//! a plan whose tenant disagrees with the credential is refused rather than
//! silently rewritten. Refused, not corrected, because a caller who names
//! the wrong tenant has asked a question this process should not answer.
//!
//! The plan is VALIDATED against the tenant's registry before it is
//! attested. `MigrationPlan::validate` is the same check the runner makes,
//! so an attestation can never claim a fixpoint over a plan the runner would
//! refuse to execute.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_edits::WireDouble;
use foundry_spine::{
    DefaultValue, MigrationPlan, UpcastTransform, ValueConversion, migration_attestation,
};
use serde::{Deserialize, Serialize};

use crate::composition::AppState;
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized, refuse, tenant_of};

#[derive(Debug, Deserialize)]
pub(crate) struct AttestRequest {
    pub(crate) tenant_id: String,              // data_class: INTERNAL_ONLY
    pub(crate) entity_type: String,            // data_class: INTERNAL_ONLY
    pub(crate) from_revision: u32,             // data_class: INTERNAL_ONLY
    pub(crate) to_revision: u32,               // data_class: INTERNAL_ONLY
    pub(crate) action_type: String,            // data_class: INTERNAL_ONLY
    pub(crate) audit_event_type: String,       // data_class: INTERNAL_ONLY
    pub(crate) declared_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub(crate) transforms: Vec<WireTransform>, // data_class: INTERNAL_ONLY
}

/// The transform vocabulary, mirrored on the wire.
///
/// The spine's `UpcastTransform` carries no serde, and giving it some would
/// make a kernel type's field names a public wire contract that could not
/// then be refactored. The mirror is the seam: it is a facade concern that
/// this vocabulary is expressible in JSON at all, and the mapping below is
/// where a wire value becomes a domain one — including the refusals, which
/// a derive would have had nowhere to put.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum WireTransform {
    CopyAs {
        from: String, // data_class: INTERNAL_ONLY
        to: String,   // data_class: INTERNAL_ONLY
    },
    ConvertAs {
        from: String,       // data_class: INTERNAL_ONLY
        to: String,         // data_class: INTERNAL_ONLY
        conversion: String, // data_class: INTERNAL_ONLY
    },
    DefaultTo {
        to: String,         // data_class: INTERNAL_ONLY
        value: WireDefault, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum WireDefault {
    String { value: String },        // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Integer { value: i64 },          // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Boolean { value: bool },         // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Double { value: f64 },           // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    Timestamp { epoch_millis: i64 }, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

impl WireDefault {
    /// A non-finite double is refused here rather than canonicalised. NaN
    /// has no total order and no canonical bytes, so a plan carrying one
    /// could not produce a stable digest — `WireDouble::new` is the law and
    /// this surface reports it rather than routing around it.
    ///
    /// No JSON request can reach that refusal, and that is measured, not
    /// assumed: the grammar has no `NaN` or `Infinity` literal, and
    /// `serde_json` fails an out-of-range exponent (`1e400`) with "number
    /// out of range" rather than yielding an infinity. The arm stays because
    /// the law belongs to `WireDouble`, not to the codec that happens to
    /// front it today; a second codec would arrive to find it already held.
    fn into_domain(self) -> Result<DefaultValue, &'static str> {
        Ok(match self {
            WireDefault::String { value } => DefaultValue::String(value),
            WireDefault::Integer { value } => DefaultValue::Integer(value),
            WireDefault::Boolean { value } => DefaultValue::Boolean(value),
            WireDefault::Double { value } => DefaultValue::Double(
                WireDouble::new(value).map_err(|_| "a default double must be finite")?,
            ),
            WireDefault::Timestamp { epoch_millis } => DefaultValue::Timestamp { epoch_millis },
        })
    }
}

impl WireTransform {
    fn into_domain(self) -> Result<UpcastTransform, &'static str> {
        Ok(match self {
            WireTransform::CopyAs { from, to } => UpcastTransform::CopyAs { from, to },
            WireTransform::ConvertAs {
                from,
                to,
                conversion,
            } => UpcastTransform::ConvertAs {
                from,
                to,
                conversion: match conversion.as_str() {
                    "integer_to_string" => ValueConversion::IntegerToString,
                    "boolean_to_integer" => ValueConversion::BooleanToInteger,
                    _ => return Err("that conversion is not one this process performs"),
                },
            },
            WireTransform::DefaultTo { to, value } => UpcastTransform::DefaultTo {
                to,
                value: value.into_domain()?,
            },
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct AttestBody {
    pub(crate) fixpoint: bool,       // data_class: INTERNAL_ONLY
    pub(crate) pending: Vec<String>, // data_class: INTERNAL_ONLY
    pub(crate) poisoned: Vec<u64>,   // data_class: INTERNAL_ONLY
}

pub async fn attest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let caller = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(caller) => caller,
        Err(response) => return *response,
    };
    let Ok(request) = serde_json::from_str::<AttestRequest>(&body) else {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the request is not a migration plan",
        );
    };
    // The credential's tenant, checked rather than substituted.
    if request.tenant_id != caller.tenant_id {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the plan names a tenant other than the credential's",
        );
    }
    let tenant = match tenant_of(&state, &caller) {
        Ok(tenant) => tenant,
        Err(response) => return *response,
    };
    let mut transforms = Vec::with_capacity(request.transforms.len());
    for wire in request.transforms {
        match wire.into_domain() {
            Ok(transform) => transforms.push(transform),
            Err(cause) => {
                state.metrics.read_refused();
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
    let tenant = tenant.lock().await;
    if let Err(error) = plan.validate(&tenant.projection.engine) {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            &format!("the plan is not executable: {error:?}"),
        );
    }
    let attestation = migration_attestation(&tenant.projection, &plan);
    state.metrics.read_served();
    Json(AttestBody {
        fixpoint: attestation.fixpoint,
        pending: attestation.pending,
        poisoned: attestation.poisoned,
    })
    .into_response()
}
