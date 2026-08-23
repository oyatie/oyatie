# Spec: task-pooling-openai-apikey-pool

## Objective

Live API-key pool for the OpenAI subscription adapter inside
`intelligence-openai-subscription-adapter`. Replaces the in-memory mock with a
real circuit-breaker-style pool: failure-count blacklist, jittered cooldown, success-restore,
and correct `Authorization: Bearer` header injection.

## Crate Boundary

ONLY `intelligence/adapter-openai-subscription-adapter`
is modified. The kernel (`intelligence-adapter-openai-subscription-kernel`) is NOT changed.

## Mod Layout (flat clean-arch per ADR-0509)

```
src/
  lib.rs               — re-exports public surface; updated ProviderAuthPort impl
  classifier.rs        — classify_response(http_status, error_type) → ResponseClass
  key_status.rs        — KeyStatus enum, KeyEntry struct
  key_pool.rs          — KeyPool: round-robin + blacklist/cooldown/restore
  outbound_headers.rs  — openai_auth_headers(key: &str) → Vec<(String,String)>
tests/
  key_pool_integration.rs  — in-process mock HTTP server (127.0.0.1:0) tests
```

## Contracts

### Status Classification

Input: `(http_status: u16, error_type: Option<&str>)`

| HTTP Status | error_type                                            | ResponseClass         |
|-------------|-------------------------------------------------------|-----------------------|
| 200         | any                                                   | Success               |
| 401         | any                                                   | TerminalKeyInvalid    |
| 403         | any                                                   | TerminalKeyInvalid    |
| 429         | `"insufficient_quota"` or `"quota_exceeded"`          | TerminalQuotaExhausted|
| 429         | other / None                                          | TransientRateLimit    |
| 5xx         | any                                                   | TransientServer       |
| other       | any                                                   | TransientUnknown      |

### KeyStatus

```
Active                — eligible for selection
Cooling { until_epoch_secs, failure_count }  — skip until cooldown expires
Blacklisted           — never selected again
```

### KeyPool::select(now_epoch_secs) → Option<key_index>

Round-robin over keys whose status is `Active` or `Cooling` with `until_epoch_secs <= now`.
Returns `None` if all keys are `Blacklisted` or still cooling.

### KeyPool::record_result(key_index, class, now_epoch_secs, jitter_secs)

- Success → if Cooling: restore to Active; reset failure_count
- TransientRateLimit / TransientServer / TransientUnknown:
  - increment failure_count
  - if failure_count >= 3: transition to Cooling { until = now + 60 + jitter, failure_count }
- TerminalKeyInvalid / TerminalQuotaExhausted → Blacklisted

### Header Injection

`openai_auth_headers(key: &str)` returns:
```
[("authorization", "Bearer <key>")]
```
No `x-api-key`. No Anthropic-style versioning headers.

## Testing Strategy

### Unit Tests (in-module)
- `classifier.rs`: test every (status, error_type) combination from the table above
- `key_status.rs`: round-trip KeyStatus fields
- `key_pool.rs`: round-robin selection; blacklist on terminal; cooldown on 3 transients;
  restore on success; all-cooling → None; jitter range [60, 90)
- `outbound_headers.rs`: Bearer scheme; no x-api-key

### Integration Tests (tests/key_pool_integration.rs)
- Mock server returns 401 → key blacklisted; second key selected on retry
- Mock server returns 429 rate-limit × 3 → key enters cooling; `authenticate` returns Err
- Mock server returns 200 on first call, verifies Bearer header in received request
- All keys blacklisted → `AuthError::NetworkUnavailable`

All tests use `127.0.0.1:0` (OS-assigned port) with hyper HTTP/1.1 server.

## Observability

- `tracing::warn!` on key blacklisted (no key material in log)
- `tracing::debug!` on key selected (index only, no key material)
- `tracing::info!` on key restored from cooling

## SLO / OpenSLO

No new SLO file required for this adapter-level crate (the pool SLO is in the app layer).
Existing `providers-pool-seat-availability.openslo.yaml` covers pool availability.

## Dependency Additions (Cargo.toml)

```toml
tokio = { workspace = true, features = ["sync", "time", "net"] }
hyper = { workspace = true }
hyper-util = { workspace = true }
http-body-util = { workspace = true }
bytes = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "time", "net", "sync"] }
```
