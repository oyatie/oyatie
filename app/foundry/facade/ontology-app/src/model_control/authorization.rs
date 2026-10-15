use super::transport::{Refusal, refuse, unavailable};
use super::*;
use crate::authz::{ModelDecision, ModelOperation, ModelPepError};
use crate::human_session::{now_epoch_seconds, session_token};
use axum::http::{HeaderMap, StatusCode};
use foundry_caller_draft::Caller;
use iam_identity_user_session::SessionRefusal;

#[derive(Clone, Copy)]
pub(super) enum Grant {
    Read,
    Author,
    Validate,
    Publish,
}

pub(super) struct Authorization {
    pub caller: ModelCaller,
    pub principal: Caller,
    pub decision: ModelDecision,
}

fn session_error(error: SessionRefusal) -> Refusal {
    match error {
        SessionRefusal::MembershipUnavailable => unavailable(),
        SessionRefusal::TenantMismatch
        | SessionRefusal::MissingMembership
        | SessionRefusal::InactiveMembership
        | SessionRefusal::AmbiguousMembership => {
            refuse(StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden)
        }
        _ => refuse(StatusCode::UNAUTHORIZED, ModelRefusalCode::Unauthenticated),
    }
}

fn policy_error(error: ModelPepError) -> Refusal {
    match error {
        ModelPepError::Denied => refuse(StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden),
        ModelPepError::Unavailable => unavailable(),
    }
}

pub(super) async fn authorize(
    state: &ModelState,
    tenant: &str,
    headers: &HeaderMap,
    grant: Grant,
) -> Result<Authorization, Refusal> {
    if !state.app.tenants.contains_key(tenant) {
        return Err(refuse(StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden));
    }
    let token = session_token(headers).map_err(session_error)?;
    let now = now_epoch_seconds().map_err(session_error)?;
    let membership = state
        .sessions
        .authorize(Some(token), tenant, now)
        .await
        .map_err(session_error)?;
    membership
        .ensure_current(now_epoch_seconds().map_err(session_error)?)
        .map_err(session_error)?;
    if membership.tenant_id() != tenant {
        return Err(session_error(SessionRefusal::TenantMismatch));
    }
    let principal = Caller {
        tenant_id: membership.tenant_id().into(),
        principal_id: membership.user_id().into(),
        roles: membership.roles().to_vec(),
    };
    let decide = |operation| {
        state.app.pep.decide_model(
            &principal,
            operation,
            tenant,
            membership.expires_at_epoch_seconds(),
        )
    };
    let decision = match grant {
        Grant::Read => decide(ModelOperation::Read),
        Grant::Author => decide(ModelOperation::Author),
        Grant::Publish => decide(ModelOperation::Publish),
        Grant::Validate => match decide(ModelOperation::Author) {
            Err(ModelPepError::Denied) => decide(ModelOperation::Publish),
            result => result,
        },
    }
    .map_err(policy_error)?;
    let authorization = Authorization {
        caller: ModelCaller {
            tenant_id: principal.tenant_id.clone(),
            owner: principal.principal_id.clone(),
        },
        principal,
        decision,
    };
    current(&authorization)?;
    Ok(authorization)
}

pub(super) fn current(authorization: &Authorization) -> Result<(), Refusal> {
    if now_epoch_seconds().map_err(session_error)? >= authorization.decision.session_expiry {
        return Err(session_error(SessionRefusal::Expired));
    }
    Ok(())
}

pub(super) async fn after_wait(
    state: &ModelState,
    tenant: &str,
    headers: &HeaderMap,
    grant: Grant,
    previous: &Authorization,
) -> Result<Authorization, Refusal> {
    current(previous)?;
    let authorization = authorize(state, tenant, headers, grant).await?;
    if authorization.caller != previous.caller {
        return Err(refuse(StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden));
    }
    current(previous)?;
    current(&authorization)?;
    Ok(authorization)
}

pub(super) fn capabilities(
    state: &ModelState,
    authorization: &Authorization,
) -> Result<ModelCapabilities, Refusal> {
    let allowed = |operation| match state.app.pep.decide_model(
        &authorization.principal,
        operation,
        &authorization.caller.tenant_id,
        authorization.decision.session_expiry,
    ) {
        Ok(_) => Ok(true),
        Err(ModelPepError::Denied) => Ok(false),
        Err(error) => Err(policy_error(error)),
    };
    let capabilities = ModelCapabilities {
        author: allowed(ModelOperation::Author)?,
        publish: allowed(ModelOperation::Publish)?,
    };
    current(authorization)?;
    Ok(capabilities)
}

pub(super) fn model<'a>(
    tenant: &'a mut crate::composition::TenantState,
    authorization: &Authorization,
) -> Result<&'a mut EstablishedModelStore, Refusal> {
    current(authorization)?;
    let model = tenant.model_store.as_mut().ok_or_else(unavailable)?;
    if model.binding.tenant_id != authorization.caller.tenant_id {
        return Err(unavailable());
    }
    Ok(model)
}
