use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use intelligence_kernel::{
    AuthzDecision, AuthzRequest, OAuthSubscription, Provider, SeatId, SelectionStrategy,
    SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest_proxy::{AppState, ProxyRequest, SecretProviderStore};

pub struct StubStore {
    pub token: String, // data_class: INTERNAL_ONLY
}

impl SecretProviderStore for StubStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _: &'a str,
    ) -> intelligence_rest_proxy::SecretProviderFuture<'a, String> {
        Box::pin(async move { Ok(self.token.clone()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
    ) -> intelligence_rest_proxy::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

pub struct AlwaysAllow;
impl intelligence_kernel::AuthzGate for AlwaysAllow {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

pub struct NoopSink;
impl intelligence_kernel::EventSink for NoopSink {
    fn emit(&self, _: intelligence_kernel::LlmGatewayEvent) {}
}

pub fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

pub fn proxy_req_json(base_url_path: &str, tenant: &str) -> ProxyRequest {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    ProxyRequest {
        method: "POST".to_string(),
        path: base_url_path.to_string(),
        headers,
        body: br#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#.to_vec(),
        tenant_id: TenantId::new(tenant).unwrap(),
    }
}

pub fn proxy_req_sse(base_url_path: &str, tenant: &str) -> ProxyRequest {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("accept".to_string(), "text/event-stream".to_string());
    ProxyRequest {
        method: "POST".to_string(),
        path: base_url_path.to_string(),
        headers,
        body: br#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[],"stream":true}"#
            .to_vec(),
        tenant_id: TenantId::new(tenant).unwrap(),
    }
}

pub fn make_pool_2_seats(tenant: &str) -> Arc<Mutex<SubscriptionPool>> {
    let t = TenantId::new(tenant).unwrap();
    let mut pool = SubscriptionPool::new(
        t.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    for i in 1..=2 {
        let seat = SeatId::new(format!("seat-{i}")).unwrap();
        pool.add_seat(OAuthSubscription::new(
            t.clone(),
            seat.clone(),
            SubscriptionId::new(format!("sub-{i}")).unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            format!("{tenant}/{}", seat.as_str()),
            0,
        ))
        .unwrap();
    }
    Arc::new(Mutex::new(pool))
}

pub fn make_app_state(
    base_url: String,
    pool: Arc<Mutex<SubscriptionPool>>,
    tenant: &str,
) -> Arc<AppState> {
    Arc::new(
        AppState::new(
            pool,
            Arc::new(AlwaysAllow),
            Arc::new(NoopSink),
            Arc::new(StubStore {
                token: "refresh-tok".to_string(),
            }),
            base_url,
            TenantId::new(tenant).unwrap(),
        )
        .unwrap()
        .with_ingress_bearer_token(Some("ingress-token".to_string())),
    )
}
