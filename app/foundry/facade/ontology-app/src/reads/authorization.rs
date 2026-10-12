use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use foundry_caller_draft::Caller;

use super::refuse;
use crate::auth::bearer_token;
use crate::composition::AppState;
use crate::pdp::Surface;

/// Verify the credential, resolve the tenant this process serves for it,
/// then ask the policy decision point with that served tenant as the
/// resource's tenant. The roster is consulted before the PDP because the
/// served tenant is an input to the decision, not a consequence of it.
pub(crate) fn authorized<'a>(
    state: &'a AppState,
    headers: &HeaderMap,
    object_ref: &str,
) -> Result<
    (
        Caller,
        data_ontology_kernel::ActionPolicyDecision,
        &'a tokio::sync::Mutex<crate::composition::TenantState>,
    ),
    Box<Response>,
> {
    let Some(token) = bearer_token(headers.get("authorization").and_then(|v| v.to_str().ok()))
    else {
        state.metrics.read_refused();
        return Err(Box::new(refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "no bearer credential",
        )));
    };
    let Some(caller) = state.verifier.verify(Some(token)) else {
        state.metrics.read_refused();
        return Err(Box::new(refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "the presented credential is not recognized",
        )));
    };
    authorized_caller(state, caller, object_ref)
}

/// Both human and operator credentials enter the same tenant and PDP gate.
pub(crate) fn authorized_caller<'a>(
    state: &'a AppState,
    caller: Caller,
    object_ref: &str,
) -> Result<
    (
        Caller,
        data_ontology_kernel::ActionPolicyDecision,
        &'a tokio::sync::Mutex<crate::composition::TenantState>,
    ),
    Box<Response>,
> {
    let Some((served_tenant, tenant)) = state.tenants.get_key_value(&caller.tenant_id) else {
        state.metrics.read_refused();
        return Err(Box::new(refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential names a tenant this process does not serve",
        )));
    };
    // The decision is returned, not discarded: it is the authority a query
    // execution records, and a route that needed one would otherwise ask the
    // policy engine a second time for an answer this gate already has.
    let Ok(decision) = state
        .pep
        .decide(&caller, Surface::Use, object_ref, served_tenant)
    else {
        state.metrics.read_refused();
        return Err(Box::new(refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision point refused this read",
        )));
    };
    Ok((caller, decision, tenant))
}
