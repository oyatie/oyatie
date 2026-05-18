# SVC-ADR-WASM-001 — Envoy WASM canonical for governance filters

- Status: Accepted
- Date: 2026-05-18
- Scope: `governance` µservice only
- ADR anchors: ADR-0200, ADR-0182

## Context

Per ADR-0200 + ADR-0182, governance filter chain uses the
Envoy WASM substrate. This service-scoped ADR records the
binding to the `envoy-filter` sandbox class + the WIT contract.

## Decision

- Wasmtime via `oya-shared-wasm-runtime-kernel`.
- Sandbox class `envoy-filter` with ADR-0200's invariants.
- Coraza WAF + Cedar authz + regulatory response shapers
  packaged as component-model bytecode.

## Alternatives considered

- Lua filters: rejected — older, less secure, no sandbox
  parity with WASM.
- ExtAuthz HTTP callouts: rejected — per-request RPC tax.

## Consequences

- All governance filters compile to component-model.
- IP-WASMTIME-001..004 implement the integration.

## References

- ADR-0200, ADR-0182.
