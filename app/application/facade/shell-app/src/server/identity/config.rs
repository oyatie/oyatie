use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use application_ontology_card_http::HumanOntologyCardSource;
use iam_identity_user_session::AuthenticatedSubject;
use reqwest::{Client, Url};

#[derive(Clone)]
pub struct SignInConfig {
    pub(super) public_origin: String,
    pub(super) iam: Url,
    pub(super) client: Client,
    pub(super) ontology: HumanOntologyCardSource,
}

impl SignInConfig {
    /// Deployment origins are explicit; there is no endpoint discovery.
    pub fn new(public_origin: &str, iam: &str, foundry: &str) -> Result<Self, &'static str> {
        Self::configured(public_origin, iam, foundry, false)
    }

    /// Enables only explicit loopback HTTP fixtures alongside HTTPS origins.
    pub fn for_loopback(
        public_origin: &str,
        iam: &str,
        foundry: &str,
    ) -> Result<Self, &'static str> {
        Self::configured(public_origin, iam, foundry, true)
    }

    fn configured(
        public_origin: &str,
        iam: &str,
        foundry: &str,
        loopback: bool,
    ) -> Result<Self, &'static str> {
        let origin = parse_origin(public_origin, loopback)?;
        if origin.origin().ascii_serialization() != public_origin {
            return Err("public origin must be exact");
        }
        let iam = parse_origin(iam, loopback)?;
        let ontology = if loopback {
            HumanOntologyCardSource::for_loopback(foundry)
        } else {
            HumanOntologyCardSource::new(foundry)
        }
        .map_err(|_| "invalid Foundry origin")?;
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| "identity HTTP client unavailable")?;
        Ok(Self {
            public_origin: public_origin.into(),
            iam,
            client,
            ontology,
        })
    }

    pub(in crate::server) fn from_environment() -> Option<Self> {
        let origin = env::var("OYATIE_SHELL_PUBLIC_ORIGIN").ok()?;
        let iam = env::var("OYATIE_SHELL_IAM_BASE_URL").ok()?;
        let foundry = env::var("OYATIE_SHELL_FOUNDRY_BASE_URL").ok()?;
        let loopback = env::var("OYATIE_SHELL_ALLOW_LOOPBACK_HTTP").ok().as_deref() == Some("true");
        match Self::configured(&origin, &iam, &foundry, loopback) {
            Ok(config) => Some(config),
            Err(reason) => {
                eprintln!("Oyatie sign-in disabled: {reason}");
                None
            }
        }
    }
}

fn parse_origin(value: &str, loopback: bool) -> Result<Url, &'static str> {
    let url = Url::parse(value).map_err(|_| "invalid service origin")?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https"
            || (loopback
                && url.scheme() == "http"
                && matches!(url.host_str(), Some("localhost" | "127.0.0.1"))))
    {
        return Err("service origin must be secure and fixed");
    }
    Ok(url)
}

pub(super) fn canonical_tenant(tenant: &str) -> bool {
    AuthenticatedSubject::new("usr_shell", tenant, i64::MAX).is_ok()
}

#[cfg(test)]
tokio::task_local! {
    pub(super) static TEST_CLOCK: std::sync::Arc<std::sync::atomic::AtomicI64>;
}

pub(super) fn now() -> i64 {
    #[cfg(test)]
    if let Ok(now) = TEST_CLOCK.try_with(|clock| clock.load(std::sync::atomic::Ordering::SeqCst)) {
        return now;
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(i64::MAX)
}

#[cfg(test)]
#[tokio::test]
async fn scoped_test_clocks_are_isolated_and_restored() {
    use std::sync::{Arc, atomic::AtomicI64};
    let first = TEST_CLOCK.scope(Arc::new(AtomicI64::new(11)), async {
        tokio::task::yield_now().await;
        assert_eq!(now(), 11);
    });
    let second = TEST_CLOCK.scope(Arc::new(AtomicI64::new(22)), async {
        tokio::task::yield_now().await;
        assert_eq!(now(), 22);
    });
    tokio::join!(first, second);
    assert!(TEST_CLOCK.try_with(|_| ()).is_err());
    assert_ne!(now(), 11);
    assert_ne!(now(), 22);
}
