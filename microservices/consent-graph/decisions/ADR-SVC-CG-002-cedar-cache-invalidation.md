# ADR-SVC-CG-002: Cedar policy compilation cache invalidation contract

- Status: Accepted
- Scope: service
- Date: 2026-05-18
- Authority: ADR-0214 §2.4 (real-time revocation), IP-004, IP-006.

## Context

Cedar policies are compiled at agreement-acceptance and cached in `enforcement-app` per pod. Revocation
must invalidate this cache within the 1s propagation SLO; otherwise stale Permit decisions leak.

## Decision

Three-tier invalidation contract with explicit time bounds:

| Tier | Path | Bound |
|------|------|-------|
| 1. In-pod | Pulsar revocation subscriber → DashMap removal | ≤10ms from message consumption |
| 2. Cross-pod | Pulsar shared subscription per region | ≤500ms p99 from publish |
| 3. Cross-region | Pulsar georeplication | ≤1s p99 from publish |

Additional safety: **200ms freshness check** at evaluation time. Every `EnforcementRequest` carries
`prior_revocation_check_ms_ago`; if > 200ms, request fails closed (`Deny{StaleRevocationCheck}`).
Callers piggy-back a freshness ping on every request (cheap — Pulsar tail-read).

## Alternatives

- TTL-based eviction (rejected: 1s TTL = ≤1s stale window guaranteed; revocation-driven gives ≤500ms
  typical).
- Per-eval freshness check via Postgres (rejected: 50ms RTT × every eval = 100K req/s × 50ms = 5K Postgres
  conn-needed — unscalable).
- No cache (rejected: Cedar compile is 100ms p99; can't be hot path).

## Consequences

- Pod-local cache adds memory cost (~5KB per agreement × 100K agreements/pod = 500MB).
- Cross-pod consistency is eventual within ≤500ms — accepted per ADR-0214 §2.4 SLO.
- Cross-region consistency within ≤1s — accepted; partition condition = fail closed.

## Verification

- Synthetic test: revoke + immediately read → stale window ≤500ms.
- Chaos: kill subscriber pod → fail-closed during reconnect (≤30s).
- Burn alert: 1h fast burn at 14.4× → P1 page.
