# IP-WASMTIME-001 — Envoy WASM filter substrate

> ADR anchor: ADR-0200, ADR-0182, ADR-0174.
> Owner: `oya-governance`.
> Estimate: 6 days.

## Goal

Stand up the Envoy north-south WASM filter substrate via
`oya-shared-wasm-runtime-kernel`. Filters target the
`envoy-filter` sandbox class with the fixed fuel + memory +
wall-clock invariants.

## Why this IP

Per ADR-0182 the API gateway hosts WAF (Coraza), regulatory
response shaping, bespoke authz / observability filters. Each
filter must be hot-loadable per tenant pack without recompiling
Envoy. Wasmtime + WASI Preview 2 component model is the
canonical extensibility point per ADR-0200.

## Pre-conditions

- `crates/oya-shared-wasm-runtime-kernel` lands.
- ADR-0200 + ADR-0182 ratified.

## Tasks

### 1. Filter registry

- Each Envoy WASM filter declares its sandbox class
  (`envoy-filter`), its WIT contract surface
  (`oya:envoy/*` imports), and its target gateway routes.
- Registry validates the bytecode before the filter is
  deployed.

### 2. Hot reload

- Envoy WASM modules reload via xDS without restarting the
  gateway pod.
- The reload path is gated through the governance lane
  runtime kernel.

### 3. Per-tenant pack overlays

- Pack-overlay filters (ADR-0064) stack on top of canonical
  filters; the order is declarative and audited.

### 4. Tests

- Sample Coraza WAF filter compiled to component-model and
  loaded via the registry.
- Sample regulatory response-shape filter (e.g. ADR-0174
  EU AI Act Annex III refusal text injection).

## Failure modes

- Filter rejects at instantiation (allowlist violation): the
  gateway falls back to the prior generation; alert fires.
- Filter fuel exhaustion: the request fails with 503; metric
  emitted; alert on rate spike.

## Acceptance criteria

- An end-to-end request through the gateway transits a
  Coraza-class WASM filter with the full sandbox posture.
- Hot reload < 5 s without a connection drop.

## References

- ADR-0200, ADR-0182.
- Envoy WASM filter chain (upstream).
