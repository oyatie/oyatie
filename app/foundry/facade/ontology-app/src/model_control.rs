use crate::composition::AppState;
use crate::human_session::{HumanSessionConfig, HumanSessionConfigError};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use foundry_model_api::control::*;
use foundry_model_api::*;
use iam_identity_user_session::UserSessionAuthorizer;
use std::sync::Arc;

mod authorization;
mod diagnostics;
mod lifecycle;
mod mutations;
mod publication;
mod reads;
mod transport;

pub struct EstablishedModelStore {
    pub(crate) binding: ModelBinding,
    pub(crate) store: Box<dyn ModelStore + Send>,
}

impl AppState {
    /// Explicit composition of established metadata; never initializes or activates it.
    pub fn inject_established_model_store(
        &mut self,
        tenant_id: &str,
        binding: ModelBinding,
        baseline: ModelBaseline,
        store: Box<dyn ModelStore + Send>,
    ) -> Result<(), ModelError> {
        if tenant_id != binding.tenant_id {
            return Err(ModelError::TenantMismatch);
        }
        let tenant = self
            .tenants
            .get_mut(tenant_id)
            .ok_or(ModelError::TenantMismatch)?
            .get_mut();
        if tenant.model_store.is_some() {
            return Err(ModelError::AlreadyInitialized);
        }
        let caller = ModelCaller {
            tenant_id: tenant_id.into(),
            owner: "model-composition".into(),
        };
        let history = store.history(&caller)?;
        if history.binding != binding || history.baseline != baseline {
            return Err(ModelError::BindingMismatch);
        }
        history.registry()?;
        tenant.invalidate_model_installation();
        tenant.model_store = Some(EstablishedModelStore { binding, store });
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ModelMutationOrigin(String);
impl ModelMutationOrigin {
    pub fn https(value: &str) -> Result<Self, HumanSessionConfigError> {
        HumanSessionConfig::https(value).map(|config| Self(config.exact_origin()))
    }
    pub fn loopback_http(value: &str) -> Result<Self, HumanSessionConfigError> {
        HumanSessionConfig::loopback_http(value).map(|config| Self(config.exact_origin()))
    }
}

#[derive(Clone)]
pub(crate) struct ModelState {
    app: Arc<AppState>,
    sessions: Arc<dyn UserSessionAuthorizer>,
    origin: Option<ModelMutationOrigin>,
}

pub(crate) fn router(
    app: Arc<AppState>,
    sessions: Arc<dyn UserSessionAuthorizer>,
    origin: Option<ModelMutationOrigin>,
) -> Router {
    Router::new()
        .route(MODEL_WORKSPACE_ROUTE, get(reads::workspace))
        .route(
            MODEL_DRAFTS_ROUTE,
            get(reads::drafts).post(mutations::create),
        )
        .route(MODEL_DRAFT_ROUTE, get(reads::draft))
        .route(MODEL_SAVE_ROUTE, post(mutations::save))
        .route(MODEL_VALIDATE_ROUTE, post(mutations::validate))
        .route(MODEL_PUBLISH_ROUTE, post(publication::publish))
        .route(MODEL_LIFECYCLE_ROUTE, get(lifecycle::read))
        .layer(middleware::map_response(|response| async {
            crate::human_session::no_store(response)
        }))
        .with_state(ModelState {
            app,
            sessions,
            origin,
        })
}
