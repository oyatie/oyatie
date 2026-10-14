//! Fresh IAM identity is an input to Foundry authorization, never a grant.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header::CACHE_CONTROL};
use axum::response::Response;
use foundry_caller_draft::Caller;
use iam_identity_user_session::{ActiveUserMembership, SessionRefusal, UserSessionAuthorizer};

use crate::composition::{AppState, TenantState};
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized_caller, refuse};

mod config;
mod cookie;
pub(crate) use cookie::session_token;
mod http;
pub(crate) mod reads;
pub(crate) mod relationships;
pub use config::{HumanSessionConfig, HumanSessionConfigError};
pub use http::HttpUserSessionAuthorizer;

#[derive(Clone)]
pub(crate) struct HumanReadState {
    pub(crate) app: Arc<AppState>,
    pub(crate) sessions: Arc<dyn UserSessionAuthorizer>,
}

pub(crate) fn now_epoch_seconds() -> Result<i64, SessionRefusal> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .ok_or(SessionRefusal::MembershipUnavailable)
}

pub(crate) async fn status(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let response = status_inner(&state, &tenant_id, &headers).await;
    no_store(response)
}

async fn status_inner(state: &HumanReadState, tenant_id: &str, headers: &HeaderMap) -> Response {
    let (caller, tenant) = match authorized(state, tenant_id, headers, TENANT_SCOPED_RESOURCE).await
    {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    crate::status::human_status_response(&state.app, &caller.tenant_id, tenant)
}

struct HumanAuthorization<'a> {
    caller: Caller,
    decision: data_ontology_kernel::ActionPolicyDecision,
    tenant: &'a tokio::sync::Mutex<TenantState>,
    membership: ActiveUserMembership,
}

async fn authorized<'a>(
    state: &'a HumanReadState,
    tenant_id: &str,
    headers: &HeaderMap,
    object_ref: &str,
) -> Result<(Caller, &'a tokio::sync::Mutex<TenantState>), Box<Response>> {
    let access = authorized_membership(state, tenant_id, headers, object_ref).await?;
    Ok((access.caller, access.tenant))
}

async fn authorized_membership<'a>(
    state: &'a HumanReadState,
    tenant_id: &str,
    headers: &HeaderMap,
    object_ref: &str,
) -> Result<HumanAuthorization<'a>, Box<Response>> {
    let token = match cookie::session_token(headers) {
        Ok(token) => token,
        Err(error) => return Err(Box::new(session_refusal(&state.app, error))),
    };
    let now = match now_epoch_seconds() {
        Ok(now) => now,
        Err(error) => return Err(Box::new(session_refusal(&state.app, error))),
    };
    let membership = match state.sessions.authorize(Some(token), tenant_id, now).await {
        Ok(membership) => membership,
        Err(error) => return Err(Box::new(session_refusal(&state.app, error))),
    };
    let current = now_epoch_seconds().and_then(|now| membership.ensure_current(now));
    if let Err(error) = current {
        return Err(Box::new(session_refusal(&state.app, error)));
    }
    if membership.tenant_id() != tenant_id {
        return Err(Box::new(session_refusal(
            &state.app,
            SessionRefusal::TenantMismatch,
        )));
    }
    let caller = Caller {
        tenant_id: membership.tenant_id().to_owned(),
        principal_id: membership.user_id().to_owned(),
        roles: membership.roles().to_vec(),
    };
    let (caller, decision, tenant) = authorized_caller(&state.app, caller, object_ref)?;
    Ok(HumanAuthorization {
        caller,
        decision,
        tenant,
        membership,
    })
}

fn session_refusal(state: &AppState, error: SessionRefusal) -> Response {
    state.metrics.read_refused();
    let status = match error {
        SessionRefusal::MembershipUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        SessionRefusal::TenantMismatch
        | SessionRefusal::MissingMembership
        | SessionRefusal::InactiveMembership
        | SessionRefusal::AmbiguousMembership => StatusCode::FORBIDDEN,
        _ => StatusCode::UNAUTHORIZED,
    };
    refuse(
        status,
        "session",
        "a current active IAM session for this tenant is required",
    )
}

pub(crate) fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
