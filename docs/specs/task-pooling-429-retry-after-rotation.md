# Spec: pooling-429-retry-after-rotation

## Objective

Close the 429-rotation loop in `dispatch_to_pool` so that an upstream
HTTP 429 / rate-limit response is treated as a retryable seat-level failure
rather than a success, with cooldown expiry computed from provider-supplied
`Retry-After*` headers (priority) or the kernel's `CooldownPolicy` table
(fallback).

## Crate boundary

`intelligence-provider-pool-app` only. No changes to any other crate.

## Mod layout (flat-clean-arch, ADR-0509)

All changes stay in `src/lib.rs`. New internal helper:

```
pub(crate) fn parse_retry_after_ms(headers: &[(String, String)], now: UnixMillis, consecutive_failures: u32) -> u64
```

Returns the cooldown duration in milliseconds derived from the response
headers, falling back to the kernel's `CooldownPolicy::window_for` table.

## Contracts

### HTTP header parsing (priority order)

| Priority | Header name               | Semantics                                         |
|----------|---------------------------|---------------------------------------------------|
| 1        | `retry-after`             | Integer seconds (HTTP-date values are ignored)    |
| 2        | `retry-after-ms`          | Integer milliseconds                              |
| 3        | `anthropic-ratelimit-requests-reset` | Integer seconds or ISO 8601 (ignored if non-integer) |
| 4        | `anthropic-ratelimit-tokens-reset`   | Same                                  |
| 5        | `x-ratelimit-reset-requests`         | Integer seconds                       |
| 6        | `x-ratelimit-reset-tokens`           | Integer seconds                       |
| fallback | (none matched)            | `CooldownPolicy::window_for(UpstreamRateLimit429, consecutive_failures).0` |

Headers are matched case-insensitively (already lowercased by the transport).

### Dispatch loop change

In `dispatch_to_pool`, the `Ok(mut response)` arm gains a 429-check branch:

```
Ok(mut response) if response.status == 429 => {
    // rate-limited: compute cooldown, record failure, walk chain
}
Ok(mut response) => {
    // success path (unchanged)
}
```

When `status == 429`:
1. Compute `cooldown_ms = parse_retry_after_ms(&response.headers, now, consecutive_failures_for_account)`.
2. Insert `(account_id.clone(), UnixMillis(now.0.saturating_add(cooldown_ms)))` into local `quarantine_map: QuarantineMap`.
3. Call `health_store.record_failure(tenant_id, pool_id, &account_id)`.
4. Emit `MetricEvent::Failure { retryable: true }` via metrics sink.
5. Optionally emit `MetricEvent::QuarantineTransition` if health state changed.
6. Increment `failover_depth`, set `prev_failed`, continue fallback chain.

The same change is applied symmetrically to `dispatch_to_pool_stream`'s
first-byte-retryable arm (the stream transport already returns
`TransportError::Retryable` for 429 there; no structural change needed).

### QuarantineMap threading

A local `quarantine_map: QuarantineMap = QuarantineMap::new()` is declared at
the top of `dispatch_to_pool`. It is populated as seats are rate-limited.
It is **not** threaded as a new function parameter (backward compatibility).
It is available to callers who need post-dispatch inspection through
`DispatchOutcome` — but adding it to `DispatchOutcome` is **out of scope** for
this slice (tracked under placeholder debt). For now it is purely local.

## Testing strategy

All tests are hermetic (`InMemoryProviderInvocationTransport`). No network.

### New tests (in `tests/acceptance.rs`)

1. `dispatch_429_rotates_to_next_seat_and_records_cooldown`
   - Pool: `["seat_a", "seat_b"]`, RoundRobin.
   - Script: `seat_a` → `Ok(ProviderResponse { status: 429, headers: [("retry-after", "60")], … })`.
   - Script: `seat_b` → `Ok(ProviderResponse { status: 200, … })`.
   - Assert: outcome served by `seat_b`; health store has `consecutive_failures >= 1` for `seat_a`.

2. `dispatch_429_parses_retry_after_ms_header`
   - Pool: `["seat_a", "seat_b"]`.
   - Script: `seat_a` → 429 with `retry-after-ms: 30000`.
   - Assert: `seat_b` serves; test verifies `seat_a` recorded as failed.

3. `dispatch_429_falls_back_to_kernel_cooldown_when_no_header`
   - Pool: `["seat_a", "seat_b"]`.
   - Script: `seat_a` → 429 with no rate-limit headers.
   - Assert: `seat_b` serves; `seat_a` has failure recorded.

4. `dispatch_429_chain_exhaustion_all_seats_rate_limited`
   - Pool: `["seat_a", "seat_b"]`.
   - All seats → 429.
   - Assert: `DispatchError::AllProvidersExhausted`.

### Changed tests

`classify_status_maps_4xx_to_non_retryable` — 429 is in the 4xx NonRetryable
bucket of `classify_status`. This is the **transport-layer** classification
(the `HyperProviderInvocationTransport` turns 429 into
`TransportError::NonRetryable`). The NEW dispatch loop change only applies to
the `Ok(response)` path from `InMemoryProviderInvocationTransport` (which
returns `Ok` for any status, including 429). The existing test must be updated
to **remove** 429 from the NonRetryable assertion OR the `classify_status`
function must be updated to classify 429 as `RateLimited` (a new class).

**Decision**: introduce `StatusClass::RateLimited` so 429 has a distinct
classification. Update `classify_status` + the existing test + the hyper
transport to map `RateLimited` → `Ok(ProviderResponse)` (returning the
response body + headers so the dispatch loop can see the `Retry-After`
headers). This matches the CLIProxyAPI / one-api `MarkResult` pattern where
the structured response is returned with all metadata intact.

## Observability / SLO

No new SLO file required for this slice (the crate has no `.openslo.yaml`
yet; SLO authoring is a future slice per ADR-0130). Metrics are emitted
via the existing `MetricsSink` (`Failure { retryable: true }` + optional
`QuarantineTransition`).

## Security

- `Retry-After` header values are parsed as integers only; non-integer /
  HTTP-date values are silently ignored (fall through to next priority).
- Cooldown values are capped by `saturating_add` to avoid u64 overflow.
- Header names are already lowercased by the transport; no case normalization
  needed in the helper.
- No credential or request-body content is included in any log or error.
