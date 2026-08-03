# Plan: pooling-openai-apikey-pool

## Objective

Replace the in-memory mock in `intelligence-openai-subscription-adapter` with a
live API-key pooling path for OpenAI. OpenAI API keys do NOT expire (unlike Anthropic OAuth
tokens) — the pooling concern is blacklist + jittered cooldown + success-restore + correct
`Authorization: Bearer` header injection.

## Requirements Analysis

### OpenAI API Key Characteristics
- API keys are long-lived (no expiry); they may be revoked by the provider at any time
- Authentication failures are detected via HTTP response codes, not token TTL
- Multiple keys should be pooled: round-robin selection, blacklist on failure, restore on
  success-restore

### Status Classification (NOT substring matching)
OpenAI error responses have a structured `error.type` or HTTP status:
- HTTP 401 (`invalid_api_key`, `invalid_authentication`) → terminal blacklist (key is bad)
- HTTP 403 (`insufficient_permissions`) → terminal blacklist
- HTTP 429 (`rate_limit_exceeded`) → transient, cooldown then retry
- HTTP 429 (`insufficient_quota`) → terminal for this key (quota exhausted)
- HTTP 5xx → transient, cooldown
- HTTP 200 → success, restore key if it was on cooldown

### Blacklist / Cooldown / Restore Logic
Lifted from gpt-load / one-api circuit-breaker pattern:
- `Active`: key is eligible for selection
- `Cooling`: failure count exceeded threshold; key skipped for `cooldown_until_epoch`
- `Blacklisted`: terminal error; key never selected again in this process lifetime

Failure count threshold: 3 consecutive transient failures → Cooling.
Jitter: cooldown base 60s + uniform random [0, 30s).
Terminal error → immediate Blacklisted (no cooldown).
On any success: if key was Cooling, restore to Active + reset failure count.

### Header Injection
OpenAI uses `Authorization: Bearer <api-key>` (NOT `x-api-key`).

### Key Selection
Round-robin over Active keys. If all keys are Cooling or Blacklisted: return
`AuthError::NetworkUnavailable`.

## Subtasks (ordered)

1. **Write plan doc** (this file) — DONE
2. **Write spec doc** `docs/specs/task-pooling-openai-apikey-pool.md`
3. **Extend kernel** — add `KeyStatus`, `KeyPoolError`, export `outbound_auth_headers` fn
   in the subscription kernel (or keep in adapter — prefer adapter since kernel must stay
   dependency-minimal)
4. **Implement modules in adapter** (flat mod layout):
   - `src/key_status.rs` — `KeyStatus` enum + `KeyEntry` struct
   - `src/classifier.rs` — `classify_response(http_status, error_type)` → `ResponseClass`
   - `src/key_pool.rs` — `KeyPool` round-robin with blacklist/cooldown/restore
   - `src/outbound_headers.rs` — `openai_auth_headers(api_key)` returns `Vec<(String,String)>`
   - `src/lib.rs` — update public surface, keep `ProviderAuthPort` impl updated
5. **Write tests (RED)**: unit tests in each mod + integration test with local HTTP mock server
6. **Implement (GREEN)**: make tests pass
7. **Self-review** (correctness/security/performance)
8. **Simplify** (cleanup, naming, dead code)

## Acceptance Criteria

- `cargo check -p intelligence-openai-subscription-adapter --all-targets` passes
- `cargo nextest run -p intelligence-openai-subscription-adapter` passes (all green)
- No real OpenAI calls in tests — all use 127.0.0.1:0 local mock server
- `Authorization: Bearer` header injected (NOT `x-api-key`)
- Status classification uses HTTP status + structured error type field (NOT substring match)
- Round-robin, blacklist on terminal, cooldown on transient, restore on success all exercised
- Jittered cooldown range confirmed in unit tests
- All keys blacklisted/cooling → `AuthError::NetworkUnavailable` returned
- Zero `cargo check` warnings
- Changes confined to `intelligence-openai-subscription-adapter` crate +
  plan/spec docs
