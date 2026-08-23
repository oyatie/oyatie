# Spec: pooling-anthropic-oauth-refresh-runtime

**Crate**: `intelligence-anthropic-subscription-adapter`
**ADRs**: ADR-0083 (panic-free), ADR-0090 (hyper HTTP stack), ADR-0506 (aws-lc-rs crypto),
          ADR-0509 (flat clean arch), ADR-0130 (SLO), ADR-0043 (no raw tokens)

---

## Objective

Replace the in-memory mock `AnthropicSubscriptionAdapter` (which synthesizes fake tokens from a
`SecretReference` opaque string) with a live Anthropic OAuth runtime adapter that:

1. Exchanges and refreshes OAuth tokens via `POST https://console.anthropic.com/v1/oauth/token`
   (`grant_type=refresh_token`).
2. Keeps pooled subscription seats continuously authenticated with a `RefreshPolicy::ExpiresLead`
   background ticker (BinaryHeap of `next_due` epochs).
3. Coalesces concurrent refresh requests for the same seat via per-seat SINGLEFLIGHT lock.
4. Persists refreshed tokens through `CredentialStorePort` (OpenBao seam) BEFORE mutating
   in-memory state (persist-before-mutate invariant).
5. Classifies terminal vs transient refresh errors and emits `OperatorAlertSignal` on terminal.
6. Injects `Authorization: Bearer` (not `x-api-key`) + `anthropic-version` + `anthropic-beta`
   on outbound proxy calls.
7. Wires PKCE enrollment (localhost-callback + manual-paste) from the oauth-subscription-kernel.

---

## Contracts

- **Port**: `ProviderAuthPort` (from `intelligence-adapter-anthropic-subscription-kernel`)
- **Credential port**: `CredentialStorePort` (new port in this crate) — `store(sref, token_bytes)`,
  `load(sref) -> Option<TokenBytes>`, `delete(sref)`
- **Alert port**: `OperatorAlertPort` (new port in this crate) — `alert(SeatId, AlertKind)`
- **HTTP**: `hyper` + `hyper-rustls` (aws-lc-rs crypto, webpki-tokio roots); no reqwest in this crate
- **Async runtime**: `tokio`
- **Wire format**: `application/x-www-form-urlencoded` POST body; JSON response

---

## Mod Layout (flat clean-arch per ADR-0509)

```
src/
  lib.rs                   — public re-exports, AnthropicOAuthAdapter struct + ProviderAuthPort impl
  oauth_client.rs          — hyper-based POST oauth/token (exchange + refresh), returns OAuthTokenResponse
  token_state.rs           — SeatTokenState (access_token, refresh_token, expires_at, issued_at), TerminalError
  refresh_policy.rs        — RefreshPolicy enum, BinaryHeap<RefreshEntry> scheduler, background ticker fn
  singleflight.rs          — per-seat Mutex<Option<Shared<BoxFuture>>> singleflight coalescer
  enrollment.rs            — enroll_seat(): wires PkceVerifier/Challenge from oauth-subscription-kernel,
                             localhost-callback or manual-paste path, calls oauth_client.exchange()
  ports.rs                 — CredentialStorePort + OperatorAlertPort trait definitions
  inmemory_store.rs        — InMemoryCredentialStore (tests), InMemoryAlertPort (tests)
```

---

## Testing Strategy

All tests are hermetic. No real Anthropic calls. In-process mock OAuth token server bound to
`127.0.0.1:0` using Tokio `TcpListener` + hand-rolled `hyper` responder.

### Test cases:
1. **mock_server_token_exchange** — successful exchange returns `access_token` + `expires_in`
2. **mock_server_refresh** — successful refresh updates seat state; old token replaced
3. **singleflight_coalescing** — 10 concurrent `authenticate()` calls for same seat yield exactly
   1 HTTP request to the mock server; all 10 callers get the same `AuthToken`
4. **expires_lead_scheduling** — seat with `expires_at = now + 30s` and `lead = 60s` is scheduled
   immediately (already past lead window); BinaryHeap ordering verified
5. **terminal_classification** — `error=refresh_token_expired` → `TerminalError`, alert emitted,
   24h backoff in seat state
6. **transient_classification** — HTTP 503 → `TransientError`, backoff but no alert
7. **persist_before_mutate** — if `CredentialStorePort::store` succeeds but crash-sim before
   memory write: old in-memory state is preserved; new token is in store
8. **debug_redaction** — `AuthToken` Debug output contains `[REDACTED]`; no raw bearer value
9. **bearer_header_injection** — `InjectedHeaders::for_outbound()` returns map with
   `Authorization: Bearer <token>`, `anthropic-version`, `anthropic-beta`

---

## Observability / SLO

- `tracing` spans on every refresh attempt with `seat_id` (not token value)
- `tracing::warn!` on transient retry; `tracing::error!` on terminal
- SLO: `microservices/intelligence/slos/anthropic-subscription-refresh.openslo.yaml` (companion file,
  mandatory per ADR-0130 before promotion past dev — deferred to gate step; file created but empty
  targets until SLO data available)

---

## Crate Boundary

Only `intelligence-anthropic-subscription-adapter` is modified. No new workspace members.
`Cargo.toml` gains:
- `tokio` (workspace, features: sync, time, net)
- `hyper` (workspace)
- `hyper-rustls` (workspace)
- `hyper-util` (workspace)
- `http-body-util` (workspace)
- `serde` (workspace)
- `serde_json` (workspace)
- `tracing` (workspace)
- `bytes` (workspace)
- `intelligence-oauth-subscription-kernel` (path dep, for PKCE types)

Dev-deps gain:
- `tokio` (rt-multi-thread, macros, time, net, sync)
- `futures-util` 0.3

---

## Security Notes

- `Authorization: Bearer` value never appears in `tracing` output (secret stays in `SeatTokenState`)
- `SeatTokenState` has no `Display`; `Debug` is `[REDACTED]`
- `CredentialStorePort` receives encrypted bytes; raw token never stored to disk in plaintext
- Terminal errors trigger `OperatorAlertPort` before the seat is marked quarantined
