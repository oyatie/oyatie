# Spec: task-pooling-hyper-client-transport

**Crate:** `intelligence-provider-pool-app`
**ADR refs:** ADR-0083 (panic-free Tier-3), ADR-0090 (hyper preferred), ADR-0105 (composition root),
ADR-0506 (aws-lc-rs crypto), ADR-0509 (flat clean-arch)

## Objective

Implement the real `ProviderInvocationTransport` in the production
`HyperProviderInvocationTransport` adapter. Replace the
`Unimplemented::OpenBaoSecretResolution` stub with a hyper-backed non-streaming
HTTP proxy that forwards upstream POST requests to Anthropic and OpenAI endpoints.

## Contracts

- **Interface preserved**: `ProviderInvocationTransport::dispatch(account_id, provider, body)` is unchanged.
- **Credential seam**: `HyperProviderInvocationTransport` holds an `Arc<dyn SecretResolution + Send + Sync>`.
  Default = `OpenBaoSecretResolver` (honest-boundary). Tests inject `InMemorySecretResolver`.
- **HTTP method**: always `POST`.
- **Upstream URLs per provider family**:
  - `ProviderFamily::Claude` → `https://api.anthropic.com/v1/messages`
  - `ProviderFamily::OpenAiOrCodex` and related → `https://api.openai.com/v1/chat/completions`
  - Unknown families → `TransportError::NonRetryable { detail: "unsupported provider family" }`
- **Auth header injection per provider**:
  - Anthropic: `x-api-key: <credential>` + `anthropic-version: 2023-06-01`
  - OpenAI: `Authorization: Bearer <credential>`
- **Hop-by-hop filter** (RFC 7230 §6.1): strip from both request and response:
  `connection`, `keep-alive`, `proxy-authenticate`, `proxy-authorization`,
  `te`, `trailers`, `transfer-encoding`, `upgrade`, plus any tokens named in `connection:` value.
- **Headers NOT forwarded** (beyond hop-by-hop): `authorization`, `host`, `content-length`.
- **Status mapping**:
  - 2xx → `Ok(ProviderResponse { status, headers, body, retry_after_seconds, provider_account_id })`
  - 5xx + network errors → `TransportError::Retryable`
  - 4xx → `TransportError::NonRetryable`
  - Other (1xx, 3xx) → `TransportError::NonRetryable`
- **`retry_after_seconds`**: parse `Retry-After` integer header from response; `None` if absent or non-integer.

## Mod Layout (flat clean-arch, ADR-0509)

All changes are `mod transport` additions within `src/lib.rs` — no new files needed for
this single-crate slice. The hyper client construction lives in a private helper inside the
transport implementation. The hop-by-hop filter and status classifier are private fns
exposed only for unit tests via `#[cfg(test)]` visibility or `pub(crate)`.

```
src/lib.rs
  ├── // existing ports, adapters, dispatch use-case (unchanged)
  └── // transport mod block additions:
      ├── HOP_BY_HOP_HEADERS: &[&str] (const)
      ├── filter_hop_by_hop(headers: &[(String,String)], extra: &HashSet<String>) -> Vec<(String,String)>
      ├── classify_status(status: u16) -> StatusClass { Success, Retryable, NonRetryable }
      └── HyperProviderInvocationTransport (enhanced):
          ├── secret_resolver: Arc<dyn SecretResolution + Send + Sync>
          ├── client: OnceCell<Client<hyper_rustls::HttpsConnector<...>, Full<Bytes>>>
          ├── new(upstream_base_url) — retains existing signature, uses OpenBaoSecretResolver
          ├── with_secret_resolver(resolver) — builder for tests
          └── dispatch() — real HTTP forward implementation
```

## Testing Strategy

### Unit tests (hermetic, in-process)

1. `filter_hop_by_hop_strips_standard_headers` — pure fn, no I/O.
2. `filter_hop_by_hop_strips_connection_nominated_tokens` — connection header dynamic removal.
3. `filter_hop_by_hop_passes_safe_headers` — non-hop-by-hop headers survive.
4. `classify_status_maps_2xx_to_success` — 200, 201, 206.
5. `classify_status_maps_5xx_to_retryable` — 500, 502, 503.
6. `classify_status_maps_4xx_to_non_retryable` — 400, 401, 422, 429.

### Integration tests against in-process server (hermetic, `127.0.0.1:0`)

7. `hyper_transport_forwards_post_to_upstream_and_returns_200` — full round-trip via tokio test server.
8. `hyper_transport_maps_5xx_to_retryable` — server returns 500, verify `TransportError::Retryable`.
9. `hyper_transport_maps_4xx_to_non_retryable` — server returns 429, verify `TransportError::NonRetryable`.
10. `hyper_transport_injects_anthropic_auth_headers` — server echoes received headers; verify `x-api-key` + `anthropic-version`.
11. `hyper_transport_injects_openai_auth_header` — server echoes headers; verify `Authorization: Bearer`.
12. `hyper_transport_strips_hop_by_hop_from_response` — server returns hop-by-hop; verify stripped from `ProviderResponse`.
13. `hyper_transport_network_error_returns_retryable` — connect to port that refuses; verify `TransportError::Retryable`.

### Existing tests (must remain green)

- `hyper_transport_surfaces_typed_unimplemented_boundary` (in `lib.rs` inline tests) — continues to work
  because default constructor still uses `OpenBaoSecretResolver`.
- `hyper_transport_surfaces_unimplemented_via_dispatch_error` (in `tests/acceptance.rs`) — same.
- `hyper_transport_round_trips_upstream_base_url` — same.
- All other acceptance + unit tests — unaffected (in-memory transport path unchanged).

## Observability / SLO

- No new OTel spans in this slice (the existing `MetricsSink` port covers dispatch-level metrics).
- SLO: covered by existing `microservices/intelligence/slos/` (if present); no new SLO required.

## Crate Boundary

Only `intelligence-provider-pool-app/Cargo.toml` and `src/lib.rs` are modified.
No other crate is touched.

## Security Notes

- `ProviderCredential` bytes are passed to `x-api-key` / `Authorization` header — they MUST
  NOT appear in any `tracing` span or error `detail` field (only opaque "transport error" surfaces).
- Credential is sourced from `SecretResolution::resolve` and converted to a UTF-8 header value;
  invalid UTF-8 → `TransportError::NonRetryable { detail: "credential encoding" }` (no raw bytes in detail).
- TLS uses `hyper-rustls` with `webpki-tokio` trust roots; no OS certificate store dependency.
