# Plan: pooling-hyper-client-transport

**Lane:** pooling | **Priority:** high | **Effort:** L

## Objective

Replace the `Unimplemented::OpenBaoSecretResolution` stub body in
`HyperProviderInvocationTransport::dispatch` (and `dispatch_stream`) with a real
hyper-backed HTTP proxy implementation. Non-streaming only for this slice.

## Acceptance Criteria

1. `cargo check -p oya-intelligence-provider-pool-app --all-targets` passes with zero errors.
2. `cargo nextest run -p oya-intelligence-provider-pool-app` passes all tests (green).
3. `HyperProviderInvocationTransport::dispatch` forwards a POST to the upstream provider
   (Anthropic `/v1/messages` or OpenAI `/v1/chat/completions`) using the credential passed
   via `ProviderCredential` resolved before calling `dispatch`.
4. RFC-7230 hop-by-hop headers are stripped from both request and response directions.
5. Auth headers are injected per-provider:
   - `ProviderFamily::Claude` → `x-api-key: <token>` + `anthropic-version: 2023-06-01`
   - `ProviderFamily::OpenAiOrCodex` (and variants) → `Authorization: Bearer <token>`
6. HTTP 5xx responses and network errors → `TransportError::Retryable`.
7. HTTP 4xx responses → `TransportError::NonRetryable`.
8. HTTP 2xx responses → `Ok(ProviderResponse)`.
9. `retry_after_seconds` is parsed from the `Retry-After` response header when present (integer seconds).
10. All tests are hermetic — no real network calls. The hyper client is tested against an
    in-process `tokio::net::TcpListener` bound to `127.0.0.1:0`.

## Ordered Subtasks

- [x] Write plan (`tasks/pooling-hyper-client-transport-plan.md`)
- [x] Write spec (`docs/specs/task-pooling-hyper-client-transport.md`)
- [x] Add `hyper`, `hyper-util`, `hyper-rustls`, `http-body-util` to crate `[dependencies]`
- [x] Add pure unit tests for `filter_hop_by_hop`, `classify_status` (red phase)
- [x] Add hermetic integration tests against in-process hyper test server (red phase)
- [x] Implement `mod transport` in `src/lib.rs` with:
    - `HopByHopFilter`
    - `classify_status` helper
    - Real `dispatch` body on `HyperProviderInvocationTransport`
- [x] Verify green: `cargo check --all-targets` + `cargo nextest run`
- [ ] Self-review: correctness / security / cloud-native / architecture
- [ ] Simplify: guard clauses, dead-code, naming

## Edge Cases

- `content-length` must be managed by hyper (do not forward from caller headers)
- `host` header must be derived from the upstream URL, not forwarded
- `authorization` must never be forwarded from caller headers (injected from credential only)
- Empty `Retry-After` or non-integer value → `retry_after_seconds: None`
- `connection`-nominated tokens must be stripped (RFC 7230 §6.1 dynamic removal)
- Network timeout / connect error → `TransportError::Retryable`

## Contract Implications

- `ProviderInvocationTransport::dispatch` signature unchanged — credential comes in via
  the separate `SecretResolution` port resolution done in `dispatch_to_pool` before calling
  the transport. The transport receives the raw `body: Bytes` only. The **credential is
  passed in the `account_id` or resolved separately** — since the current interface does NOT
  pass a `ProviderCredential` to `dispatch` (only to `dispatch_stream`), the credential bytes
  must come from the `HyperProviderInvocationTransport`'s internal `SecretResolution` port,
  OR we wire the resolved credential into the struct state for this call. Looking at the existing
  interface: `dispatch(account_id, provider, body)` — no credential argument. The real
  implementation must therefore hold a `SecretResolution` adapter internally.

  **Decision**: `HyperProviderInvocationTransport` will hold an `Arc<dyn SecretResolution>`
  (defaulting to `OpenBaoSecretResolver` which returns `Unimplemented`). When credential
  resolution fails with `Unimplemented`, the honest-claims boundary is maintained by
  returning `TransportError::NonRetryable` referencing the debt ID. When a real
  `InMemorySecretResolver` is injected (tests), the transport performs real HTTP dispatch.

  This keeps the existing test (`hyper_transport_surfaces_typed_unimplemented_boundary`)
  passing without change while enabling hermetic integration tests to inject credentials.

## K8s / Cloud-Native Notes

- `HyperProviderInvocationTransport` holds one process-wide hyper `Client` (connection pool).
- TLS via `hyper-rustls` on `aws-lc-rs` backend (ADR-0506) + `webpki-tokio` trust roots.
- No native-certs; fully self-hostable in a distroless/hardened container.
