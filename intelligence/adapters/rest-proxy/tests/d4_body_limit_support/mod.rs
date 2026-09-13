use intelligence_kernel::{AuthzDecision, AuthzRequest};
use intelligence_rest_proxy::{EventSink, LlmGatewayEvent, SecretProviderStore};

pub struct AlwaysAllow;
impl intelligence_kernel::AuthzGate for AlwaysAllow {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

pub struct NoopSink;
impl EventSink for NoopSink {
    fn emit(&self, _: LlmGatewayEvent) {}
}

pub struct StubStore;
impl SecretProviderStore for StubStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _: &'a str,
    ) -> intelligence_rest_proxy::SecretProviderFuture<'a, String> {
        Box::pin(async { Ok("stub-token".to_string()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
    ) -> intelligence_rest_proxy::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}
