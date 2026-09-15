use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};

use messenger_domain::Error;
use messenger_policy_api::Policy;
use messenger_policy_http::OyatiePolicy;
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

use crate::config::{Config, Tenant};
use crate::translate::{Admission, parc};

pub struct AppState<P> {
    pub(crate) listen: SocketAddr,
    pub(crate) token_hash: [u8; 32],
    pub(crate) capacity: Arc<Semaphore>,
    pub(crate) metrics: Metrics,
    pub(crate) tenants: BTreeMap<String, Tenant>,
    policy: P,
}

#[derive(Default)]
pub(crate) struct Metrics {
    pub requests: AtomicU64,
    pub denials: AtomicU64,
    pub unavailable: AtomicU64,
    pub elapsed_micros: AtomicU64,
    pub policy_unavailable: AtomicBool,
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
            metrics: Metrics::default(),
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
