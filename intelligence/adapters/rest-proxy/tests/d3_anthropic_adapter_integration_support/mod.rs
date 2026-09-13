use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use intelligence_kernel::{
    AuthzDecision, AuthzRequest, OAuthSubscription, Provider, SeatId, SelectionStrategy,
    SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest_proxy::{ProxyRequest, SecretProviderStore};

pub struct StubStore {
    pub token: String,
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

pub fn make_pool(tenant: &str, seat: &str) -> Arc<Mutex<SubscriptionPool>> {
    let t = TenantId::new(tenant).unwrap();
    let mut pool =
        SubscriptionPool::new(t.clone(), Provider::Anthropic, SelectionStrategy::FillFirst);
    pool.add_seat(OAuthSubscription::new(
        t,
        SeatId::new(seat).unwrap(),
        SubscriptionId::new(format!("{seat}-sub")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("{tenant}/{seat}"),
        0,
    ))
    .unwrap();
    Arc::new(Mutex::new(pool))
}

pub fn proxy_req(base_path: &str, tenant: &str) -> ProxyRequest {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    ProxyRequest {
        method: "POST".to_string(),
        path: base_path.to_string(),
        headers,
        body: br#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#.to_vec(),
        tenant_id: TenantId::new(tenant).unwrap(),
    }
}

pub fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

pub struct AllowGate;
impl intelligence_kernel::AuthzGate for AllowGate {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

pub fn struct_allow_gate() -> AllowGate {
    AllowGate
}
