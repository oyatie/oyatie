use intelligence_rest_proxy::SecretProviderStore;

pub struct StubSecretStore;
impl SecretProviderStore for StubSecretStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _handle: &'a str,
    ) -> intelligence_rest_proxy::SecretProviderFuture<'a, String> {
        Box::pin(async { Ok("stub-refresh-token".to_string()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _handle: &'a str,
        _plaintext: &'a str,
    ) -> intelligence_rest_proxy::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}
