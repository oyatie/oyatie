# SVC-ADR-WASM-001 — Wasmtime canonical for Foundry tools

- Status: Accepted
- Date: 2026-05-18
- Scope: `foundry` µservice only
- ADR anchors: ADR-0200, ADR-0136

## Context

Foundry tool sandboxing needs a canonical runtime. ADR-0200
makes Wasmtime canonical at the substrate; this service-scoped
ADR records Foundry's binding to the sandbox class
`foundry-tool` and the WIT contract.

## Decision

- Wasmtime via `oya-shared-wasm-runtime-kernel`.
- Sandbox class `foundry-tool` with ADR-0200's invariants
  (500M fuel, 512 MiB memory, 60s wall-clock, `oya:foundry/*`
  imports).
- WIT contract authoritative; tools failing WIT validation
  rejected at registration.

## Alternatives considered

- Per-µservice Wasmtime embedding: rejected by ADR-0200
  discipline.
- Sub-process (gVisor) per tool: rejected — too heavy for the
  expected tool QPS.

## Consequences

- Foundry tool runner depends on the kernel.
- IP-WASMTIME-001..004 implement the integration.

## Open

- Cross-tenant token chaining (a tool calls another tool):
  deferred.
