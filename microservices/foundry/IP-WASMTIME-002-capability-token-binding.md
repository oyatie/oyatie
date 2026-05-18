# IP-WASMTIME-002 — Capability-token binding for foundry-tool sandbox

> ADR anchor: ADR-0200, ADR-0136.
> Owner: `oya-foundry`.
> Estimate: 3 days.

## Goal

Generate, rotate, and verify per-(tenant, sandbox-class,
call-site) capability tokens for the `foundry-tool` sandbox
class. The kernel rejects invocations whose token does not
verify.

## Why this IP

Per ADR-0200 §"Security model — no ambient authority", every
WASM invocation carries a capability token. Foundry's tool
runner must mint, attach, and verify those tokens.

## Pre-conditions

- IP-WASMTIME-001 lands.
- OpenBao mount for capability tokens (per ADR-0173).

## Tasks

### 1. Token minting

- On tenant onboarding to Foundry: mint a per-(tenant,
  sandbox-class) root token.
- On tool registration: mint a per-(tenant, sandbox-class,
  call-site) derived token bound to the root.

### 2. Token rotation

- Per-tenant key cycle = 90 days.
- Overlap window: 14 days.

### 3. Token verification

- Kernel verifies token at invocation time by checking the
  HMAC-SHA256 of (tenant_id, sandbox_class, call_site)
  against the OpenBao-stored secret for the tenant's current
  cycle.

### 4. Audit

- Token mint, rotate, revoke events emit into ADR-0145 audit
  chain.

### 5. Tests

- Unit tests for the HMAC verification path.
- Integration tests for rotation overlap window.
- Negative test: token from a prior cycle rejected after
  overlap window closes.

## Failure modes

- OpenBao unavailable: token verification fails; tool runner
  rejects all WASM invocations until OpenBao recovers.

## Acceptance criteria

- 100% of `foundry-tool` invocations verify a token.
- Audit chain captures mint / rotate / revoke events.

## References

- ADR-0200 §"Capability tokens".
- ADR-0173 secrets storage.
