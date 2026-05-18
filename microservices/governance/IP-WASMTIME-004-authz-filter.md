# IP-WASMTIME-004 — Authz filter (Cedar evaluation at gateway)

> ADR anchor: ADR-0200, ADR-0183.
> Owner: `oya-governance`.
> Estimate: 3 days.

## Goal

Evaluate Cedar policies at the gateway via a WASM filter so
per-request authz decisions happen before the request reaches
the target µservice.

## Why this IP

Per ADR-0183 Cedar is the application-tier authz engine. Doing
the Cedar evaluation at the gateway saves a hop per request
and gives a single place to attach evaluation traces for
audit.

## Tasks

### 1. Cedar engine in WASM

- Compile Cedar (Rust upstream) to component-model bytecode
  with the `oya:envoy/*` host imports.

### 2. Policy fetch

- Per-tenant policies fetched at request time from the
  policy bundle service; cached for 30 s with cache-bust on
  policy update.

### 3. Tests

- Allow: known-good request reaches the upstream µservice.
- Forbid: known-forbidden request returns 403 at the gateway
  without reaching the upstream.
- Performance: p99 added latency ≤ 5 ms.

## Failure modes

- Policy fetch unavailable: fail-closed (return 503); never
  default-allow.
- Cedar engine fuel exhaustion: return 503; alert.

## Acceptance criteria

- 100% of inter-µservice traffic transits the authz filter.
- Audit chain captures every allow/deny.

## References

- ADR-0200, ADR-0183.
- Cedar upstream.
