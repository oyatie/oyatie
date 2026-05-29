# pooling-429-retry-after-rotation — Task Plan

## Problem

The dispatch loop in `oya-intelligence-provider-pool-app/src/lib.rs` currently
treats `Ok(ProviderResponse { status: 429, … })` as a **success** — the transport
`dispatch` method returns `Ok(response)` for any HTTP response it can parse,
and the dispatch loop only distinguishes `Ok` (success path) from
`Err(TransportError::Retryable)` / `Err(TransportError::NonRetryable)`.

A 429 from an upstream provider signals that the **seat (account)** is
rate-limited and should be cooldown-quarantined so subsequent `pick_account`
calls skip it. Today this signal is silently swallowed.

## Acceptance criteria

1. When the transport returns `Ok(ProviderResponse { status: 429 })`:
   - Parse `Retry-After` (seconds integer) header first.
   - If absent, parse `Retry-After-Ms` (millis integer).
   - If absent, parse Anthropic rate-limit headers:
     `anthropic-ratelimit-requests-reset` / `anthropic-ratelimit-tokens-reset`
     (ISO 8601 durations or absolute timestamps — treat as seconds if integer).
   - If absent, parse OpenAI-style `x-ratelimit-reset-requests` /
     `x-ratelimit-reset-tokens` (seconds or millis integer).
   - Fall back to `CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429,
     consecutive_failures)` from the kernel.
2. Record a seat-level cooldown: insert `(account_id, now + cooldown_ms)` into
   a `QuarantineMap` that is threaded through the dispatch loop.
3. Call `health_store.record_failure` (increments consecutive_failures, may
   transition to Degraded/Unhealthy) — same as retryable 5xx path.
4. Advance to the next seat in `fallback_chain` — a 429 is retryable with
   cooldown, not a non-retryable short-circuit.
5. The `QuarantineMap` is plumbed into `pick_account_with_cooldown` (the
   kernel's cooldown-aware routing entry point) on subsequent picks within
   the same dispatch call (if the loop re-queries the kernel — not required
   today; the map is populated for external callers and future use).
6. Hermetic tests (no network):
   - `dispatch_429_rotates_to_next_seat_and_records_cooldown`: mock transport
     returning 429 + `Retry-After: 60` for seat A, 200 for seat B. Assert
     seat B served, seat A has cooldown in QuarantineMap, health store has
     failure recorded for A.
   - `dispatch_429_parses_retry_after_ms_header`: `Retry-After-Ms: 30000` →
     cooldown_ms = 30_000.
   - `dispatch_429_falls_back_to_kernel_cooldown_policy_when_no_header`:
     no header → cooldown from `CooldownPolicy::window_for(UpstreamRateLimit429, 1)`
     = 30_000 ms.
   - `dispatch_429_chain_exhaustion_all_seats_rate_limited`: all seats 429 →
     `DispatchError::AllProvidersExhausted`.
7. Existing tests remain GREEN.
8. `cargo check -p oya-intelligence-provider-pool-app --all-targets` clean.
9. `cargo nextest run -p oya-intelligence-provider-pool-app` green.

## Subtasks (ordered)

1. [x] Write plan (this file)
2. [ ] Write spec (`docs/specs/task-pooling-429-retry-after-rotation.md`)
3. [ ] Add `QuarantineMap` + `CooldownPolicy` + `FailureKind` + `populate_quarantine_from_changes`
       to the `pub use` re-exports in `src/lib.rs`.
4. [ ] Add `parse_retry_after_ms(headers)` helper function in `src/lib.rs`.
5. [ ] Extend `dispatch_to_pool` to detect `status == 429` in the `Ok` branch,
       call the helper, record cooldown in a local `QuarantineMap`, call
       `health_store.record_failure`, and continue the fallback chain.
6. [ ] Write RED tests (compile-fail or failing assertions) for the 4 acceptance
       scenarios above.
7. [ ] Verify tests fail with `cargo nextest run -p ... --no-run` (or run them).
8. [ ] Fix any compile issues; verify tests pass.
9. [ ] Self-review: correctness, security, performance, cloud-native-readiness.
10. [ ] Simplify: guard clauses, naming, dead code.
11. [ ] Final `cargo nextest run -p oya-intelligence-provider-pool-app` green.

## Edge cases

- `Retry-After` header may be an HTTP-date (RFC 7231) — ignore, fall back to kernel.
- `Retry-After-Ms` may be very large — cap at `u64::MAX / 2` to avoid overflow.
- Multiple rate-limit headers present — first match wins (priority order above).
- `now.0.saturating_add(cooldown_ms)` avoids overflow.
- A 429 with `retry_after_seconds: None` in the `ProviderResponse` struct should
  still populate the QuarantineMap via the kernel fallback.
- The `ProviderResponse.retry_after_seconds` field on a 429 response is not
  currently set by `InMemoryProviderInvocationTransport` — the test script must
  set `retry_after_seconds` in the returned `ProviderResponse`, but for the
  dispatch loop the headers vector is the authoritative source.

## Architecture invariants (ADR-0509 / flat-clean-arch)

- All changes stay in `oya-intelligence-provider-pool-app/src/lib.rs`.
- No new crate. No new workspace dependency.
- `QuarantineMap` is a `BTreeMap` alias already in the kernel; re-export only.
- `dispatch_to_pool` signature is **not** changed (backward compat); the
  `QuarantineMap` is a local variable inside the function.
