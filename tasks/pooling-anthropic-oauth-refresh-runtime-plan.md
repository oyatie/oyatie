# pooling-anthropic-oauth-refresh-runtime — Implementation Plan

## Summary
Replace the in-memory mock `AnthropicSubscriptionAdapter` with a live Anthropic OAuth runtime adapter
that keeps pooled subscription seats authenticated via token refresh. Zero real network calls in tests.

## Edge Cases / Acceptance

1. **SINGLEFLIGHT**: Concurrent calls for the same seat coalesce into one refresh HTTP request; others
   wait for the winner and receive the same `AuthToken` without a second network round-trip.
2. **OpenBao persist-before-mutate**: New access_token is persisted to the `CredentialStorePort` BEFORE
   the in-memory state is updated; a crash after persist but before memory update is recoverable on
   restart.
3. **Terminal errors** (`refresh_token_expired`, `reused`, `invalidated`): 24-hour backoff +
   `OperatorAlertSignal` emitted; no auto-retry.
4. **Transient errors** (network, 5xx): exponential backoff with jitter up to `MAX_TRANSIENT_RETRIES`.
5. **ExpiresLead BinaryHeap**: background ticker refreshes seats `EXPIRES_LEAD_SECS` before expiry so
   in-flight requests never see an expired token.
6. **Authorization: Bearer** (not `x-api-key`) + `anthropic-version` + `anthropic-beta` injected on
   outbound proxy calls via the `InjectedHeaders` value type.
7. **PKCE enrollment**: `enroll_seat()` wires the oauth-subscription-kernel PKCE primitives — either
   localhost-callback or manual-paste path — returning a `SecretReference` stored via
   `CredentialStorePort`.
8. **Test hermetic**: local in-process mock OAuth token server bound to `127.0.0.1:0` (Tokio listener);
   no real Anthropic calls.

## Ordered Subtasks

- [x] S1: Write `tasks/pooling-anthropic-oauth-refresh-runtime-plan.md` (this file)
- [x] S2: Write `docs/specs/task-pooling-anthropic-oauth-refresh-runtime.md`
- [ ] S3 (TEST RED): Add new tests to the crate covering: mock-server token exchange, singleflight
      coalescing, expires-lead scheduling, terminal vs transient classification. Confirm `cargo check
      --all-targets` compiles (or fails due to missing impl) and `--no-run` nextest reports test collection.
- [ ] S4 (BUILD GREEN): Add mods: `oauth_client`, `token_state`, `refresh_policy`, `singleflight`,
      `enrollment`, and the live `AnthropicOAuthAdapter` impl. Update `Cargo.toml` with `tokio`, `hyper`,
      `hyper-rustls`, `http-body-util`, `serde`, `serde_json`, `tracing`, `bytes`. Wire everything in
      `src/lib.rs`. Confirm `cargo check -p oya-intelligence-adapter-anthropic-subscription-adapter
      --all-targets` + nextest green.
- [ ] S5 (REVIEW): Correctness, security (no raw token in logs), performance (pool-safe Arc/Mutex),
      cloud-native readiness. Fix any Critical/High findings.
- [ ] S6 (SIMPLIFY): Dead-code removal, naming pass, guard-clause cleanup. Re-run nextest.

## Acceptance Criteria

- `cargo nextest run -p oya-intelligence-adapter-anthropic-subscription-adapter` exits 0
- `cargo check -p oya-intelligence-adapter-anthropic-subscription-adapter --all-targets` exits 0
- `git diff --stat origin/dev` touches ONLY `microservices/intelligence/crates/oya-intelligence-adapter-anthropic-subscription-adapter/`, `docs/specs/task-pooling-anthropic-oauth-refresh-runtime.md`, `tasks/pooling-anthropic-oauth-refresh-runtime-plan.md`
- No real Anthropic network calls; all tests use in-process mock server on `127.0.0.1:0`
- Token `Debug` output contains `[REDACTED]`; no raw bearer value in any log
- `OperatorAlertSignal` emitted on terminal refresh error
- `CredentialStorePort` persisted before in-memory state mutated (persist-before-mutate)
