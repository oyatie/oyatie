# Spec: pooling-openbao-secret-fetch

## Objective

Implement live OpenBao KV-v2 secret-fetch behind the `CredentialHandleIssuerPort`
(SecretResolution port) in `intelligence-credential-resolver-adapter`.
Replaces the metadata-only sidecar adapter (`CredentialResolverAdapter`) with a real
HTTP adapter (`OpenBaoKvAdapter`) that calls `GET /v1/secret/data/<seat-path>` and
resolves the returned credential material into a short-lived in-memory
`CredentialHandle`.

Satisfies: `Unimplemented::OpenBaoSecretResolution`.

## Crate Boundary

Crate: `intelligence-credential-resolver-adapter`
Location: `intelligence/credential-resolver-adapter/`

All changes are INSIDE this crate. No new workspace member. Root `Cargo.toml` is NEVER
touched.

## Contracts

### HTTP Wire Protocol

```
GET /v1/secret/data/<seat-path>
Headers:
  X-Vault-Token: <BAO_TOKEN>
  Content-Type: application/json

200 OK:
{
  "data": {
    "data": {
      "api_key": "<raw-key>"       // OR
      "oauth_access_token": "<token>"
    }
  }
}

401 → token-expired
403 → forbidden
404 → secret-not-found
503 → vault-sealed
other 4xx/5xx → unexpected-status
```

Seat-path is derived from `SecretReference::canonical_ref()` by stripping the
`openbao://` prefix. Example:
`openbao://secret/ten_a/intelligence/provider/openai`
→ seat-path = `secret/ten_a/intelligence/provider/openai`

### CredentialHandle contract

- `handle_id`: `handle://openbao/<seat-path>/gen-1` (generation always 1 for fresh fetch)
- `tenant_id`: from request
- `provider`: from request
- `audience`: from request
- `issued_at_epoch_seconds`: from `request.now_epoch_seconds`
- `expires_at_epoch_seconds`: `now + MAX_CREDENTIAL_HANDLE_TTL_SECONDS` (60s ceiling)
- `sidecar_signature_ref`: `openbao://kv/fetched`

The `CredentialHandle` contains NO raw credential material — only opaque refs.

## Mod Layout (flat-clean-arch)

```
src/
  lib.rs            — re-exports (existing sidecar adapter types + new OpenBaoKvAdapter)
  openbao_kv.rs     — NEW: OpenBaoKvAdapterConfig + OpenBaoKvAdapter + mod tests
```

## Implementation Details

### OpenBaoKvAdapterConfig

```rust
pub struct OpenBaoKvAdapterConfig {
    pub base_url: String,       // e.g. "http://openbao.svc:8200"
    pub vault_token: RedactedToken,  // BAO_TOKEN env var
}

impl OpenBaoKvAdapterConfig {
    pub fn from_env(base_url: impl Into<String>) -> Result<Self, OpenBaoKvConfigError>
}
```

### OpenBaoKvAdapter

```rust
pub struct OpenBaoKvAdapter {
    config: OpenBaoKvAdapterConfig,
    client: hyper_util::client::legacy::Client<...>,
}

impl CredentialHandleIssuerPort for OpenBaoKvAdapter {
    fn issue_handle(&mut self, request: CredentialHandleRequest)
        -> Result<CredentialHandle, CredentialHandleIssueFailure>
}
```

The `issue_handle` implementation:
1. Derives seat-path from `request.secret_reference.canonical_ref()`
2. Calls `GET /v1/secret/data/<seat-path>` with `X-Vault-Token`
3. Parses JSON response into `KvSecretData`
4. Extracts `api_key` or `oauth_access_token` (first non-empty wins)
5. Validates credential is non-empty
6. Issues `CredentialHandle` via `CredentialHandle::issue()`
7. Returns handle (raw material is never stored on the handle or in any returned type)

### Error Mapping

```
HTTP 401           → reason = "openbao:token-expired"
HTTP 403           → reason = "openbao:forbidden"
HTTP 404           → reason = "openbao:secret-not-found"
HTTP 503           → reason = "openbao:vault-sealed"
HTTP other         → reason = "openbao:unexpected-status"
Transport error    → reason = "openbao:transport-error"
JSON decode        → reason = "openbao:decode-error"
Empty credential   → reason = "openbao:empty-credential"
evidence_ref always = "openbao:kv:fetch"
```

Raw material NEVER appears in `reason` or `evidence_ref`.

## Testing Strategy

### Unit tests (in `src/openbao_kv.rs` `#[cfg(test)]`)

- JSON parse: `KvReadResponse` with `api_key` field
- JSON parse: `KvReadResponse` with `oauth_access_token` field
- JSON parse: missing both fields → empty-credential error
- Error mapping: `map_status_error()` for each HTTP code
- Seat-path extraction from canonical_ref

### Integration tests (in `src/openbao_kv.rs` `#[cfg(test)]` with tokio)

In-process mock server on `127.0.0.1:0` (port 0 = OS assigns free port):
- Happy-path api_key fetch → CredentialHandle issued
- Happy-path oauth_access_token fetch → CredentialHandle issued
- 401 response → correct failure reason
- 403 response → correct failure reason
- 404 response → secret-not-found
- 503 response → vault-sealed
- Debug output never contains raw token or credential material

All tests are hermetic — no external process, no real OpenBao.

## Observability

- `tracing::debug!` on KV path before HTTP call (path only, no token/credential)
- `tracing::warn!` on non-success status

## SLO

No new OpenSLO file required for adapter-only changes (no new microservice binary).
The existing `microservices/intelligence/` SLO governs the intelligence µservice.

## Security Properties

- `RedactedToken` wraps the raw vault token; `Debug` and `Display` print `<REDACTED>`
- Credential material (api_key, oauth_access_token) never stored beyond the local frame
- No raw material in `CredentialHandle`, `CredentialHandleIssueFailure`, or log output
- `BAO_TOKEN` sourced from env at construction time only
