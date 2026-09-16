#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod http;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{get, post};
use messenger_domain::{ENTERPRISE_ROOM_TYPE, Error};
use messenger_policy_api::{Action, Policy};
use messenger_policy_http::OyatiePolicy;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    pub policy_origin: String,
    pub policy_version: String,
    pub workload_identity_file: Option<String>,
    pub tenants: BTreeMap<String, Tenant>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Error> {
        let bytes = std::fs::read(path).map_err(|_| Error::Invalid("config unreadable".into()))?;
        serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("invalid config".into()))
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tenant {
    pub audit_bot: String,
    pub subjects: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub event_id: String,
    pub tenant: String,
    pub audit_bot: String,
    pub event: Value,
    pub create: Value,
}

pub struct AppState<P> {
    listen: SocketAddr,
    pub(crate) token_hash: [u8; 32],
    pub(crate) capacity: Arc<Semaphore>,
    tenants: BTreeMap<String, Tenant>,
    policy: P,
}

pub fn compose(config: Config, token: &str) -> Result<Arc<AppState<OyatiePolicy>>, Error> {
    let pem = config
        .workload_identity_file
        .as_ref()
        .map(std::fs::read)
        .transpose()
        .map_err(|_| Error::Invalid("workload identity unreadable".into()))?;
    let policy = OyatiePolicy::new(
        &config.policy_origin,
        &config.policy_version,
        pem.as_deref(),
    )?;
    AppState::bind(config, token, policy)
}

impl<P> AppState<P> {
    pub fn listen(&self) -> SocketAddr {
        self.listen
    }

    pub fn try_hold(&self) -> Option<OwnedSemaphorePermit> {
        self.capacity.clone().try_acquire_owned().ok()
    }
}

impl<P: Policy> AppState<P> {
    pub fn bind(config: Config, token: &str, policy: P) -> Result<Arc<Self>, Error> {
        if !config.listen.ip().is_loopback() || config.tenants.is_empty() {
            return Err(Error::Invalid(
                "loopback listen and a tenant roster are required".into(),
            ));
        }
        if token.len() < 32 {
            return Err(Error::Invalid(
                "admission token must contain at least 32 bytes".into(),
            ));
        }
        for tenant in config.tenants.values() {
            if !tenant.subjects.contains_key(&tenant.audit_bot)
                || tenant
                    .subjects
                    .values()
                    .any(|subject| subject.trim().is_empty())
            {
                return Err(Error::Invalid("invalid tenant identity roster".into()));
            }
        }
        Ok(Arc::new(Self {
            listen: config.listen,
            token_hash: Sha256::digest(token.as_bytes()).into(),
            capacity: Arc::new(Semaphore::new(64)),
            tenants: config.tenants,
            policy,
        }))
    }

    pub async fn authorize_admission(&self, request: &Admission) -> Result<(), Error> {
        let (tenant, subject, action, resource) = parc(&self.tenants, request)?;
        messenger_policy_usecase::authorize(&self.policy, &tenant, &subject, action, &resource)
            .await
    }
}

fn parc(
    tenants: &BTreeMap<String, Tenant>,
    request: &Admission,
) -> Result<(String, String, Action, String), Error> {
    let tenant = tenants.get(&request.tenant).ok_or(Error::Denied)?;
    let sender = request.event["sender"].as_str().ok_or(Error::Denied)?;
    if request.event["room_id"]
        .as_str()
        .is_none_or(|room| !room.starts_with('!'))
        || request.event_id.is_empty()
        || request.audit_bot != tenant.audit_bot
        || request.create["type"] != ENTERPRISE_ROOM_TYPE
        || request.create["m.federate"] != false
        || request.create["dev.oyatie.tenant"] != request.tenant
        || request.create["dev.oyatie.audit_bot"] != request.audit_bot
    {
        return Err(Error::Denied);
    }
    let resource = request.create["dev.oyatie.policy_resource"]
        .as_str()
        .map(str::trim)
        .filter(|resource| resource.len() <= 256)
        .ok_or(Error::Denied)?;
    let subject = tenant.subjects.get(sender).ok_or(Error::Denied)?;
    membership(tenant, &request.event)?;
    Ok((
        request.tenant.clone(),
        subject.clone(),
        action(&request.event)?,
        resource.to_owned(),
    ))
}

fn membership(tenant: &Tenant, event: &Value) -> Result<(), Error> {
    if event["type"] != "m.room.member" {
        return Ok(());
    }
    let target = event["state_key"].as_str().ok_or(Error::Denied)?;
    match event["content"]["membership"].as_str() {
        Some("leave" | "ban") => Ok(()),
        Some("invite" | "join" | "knock") if tenant.subjects.contains_key(target) => Ok(()),
        _ => Err(Error::Denied),
    }
}

fn action(event: &Value) -> Result<Action, Error> {
    match event["type"].as_str() {
        Some("m.room.create") => Ok(Action::CreateRoom),
        Some("m.room.encrypted") => Ok(Action::Send),
        Some("m.room.member") => match event["content"]["membership"].as_str() {
            Some("invite") => Ok(Action::Invite),
            Some("leave" | "ban" | "join" | "knock") => Ok(Action::ManageRoom),
            _ => Err(Error::Denied),
        },
        Some(
            "m.room.power_levels"
            | "m.room.encryption"
            | "m.room.join_rules"
            | "m.room.history_visibility"
            | "m.room.guest_access",
        ) => Ok(Action::ManageRoom),
        _ => Err(Error::Denied),
    }
}

pub fn router<P: Policy + 'static>(state: Arc<AppState<P>>) -> Router {
    Router::new()
        .route("/v1/messenger/admit", post(http::admit::<P>))
        .layer(DefaultBodyLimit::max(1_048_576))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            http::authenticate::<P>,
        ))
        .route("/healthz", get(ok))
        .route("/readyz", get(ok))
        .with_state(state)
}

async fn ok() -> StatusCode {
    StatusCode::OK
}
