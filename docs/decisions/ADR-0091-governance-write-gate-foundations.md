---
id: ADR-0091
status: Accepted
doc_status: published
---

# ADR-0091: Foundry write-gate foundations (Phase 05 contract)

> **Status:** Accepted
> **Date:** 2026-05-14
> **Owner:** `council-foundry`
> **Supersedes:** —
> **Superseded-by:** —
> **Related:** ADR-0054, ADR-0090

---

## Status

Accepted (2026-05-14).

## Context

M02-P04 ships the transport-parity layer (REST / ~~GraphQL~~ [dropped per ADR-0565] / SSE / WebSocket)
for read paths. M02-P05 introduces the first write-capable transports
(gRPC / Webhook / Kafka). Before any write transport lands, Foundry needs:

1. A canonical write-gate state machine governing every mutation regardless
   of transport.
2. A default-deny posture: a mutation in flight is REFUSED until it
   transitions through Reviewed and Approved.
3. Separation of duties between the proposer, reviewer, approver, and
   executor principals — no single principal may carry a write end-to-end.

Without (1)–(3) up front, M02-P05 transports would diverge in their gating
semantics and audit-event shape, repeating the policy fragmentation that
ADR-0064/ADR-0090 retired for HTTP framing.

## Decision

The write-gate kernel owns the canonical write-gate state machine:

```
Proposed → Reviewed { reviewer } → Approved { approver } → Executed
       \                                                 /
        +----------> Rejected { reason } <--------------+
```

Linear forward path: `Proposed → Reviewed → Approved → Executed`. Any
non-terminal state may transition to `Rejected`. `Executed` and `Rejected`
are terminal.

### Default-deny

Every newly proposed gate starts in `Proposed`. Calls to `approve` or
`execute` on a fresh gate return `WriteGateError::Denied`. Skipping a stage
(`Proposed → Approved`, `Reviewed → Executed`) returns `Denied`. Terminal
states refuse all further transitions with `WriteGateError::Terminal`.

### Separation of duties

The reviewer MUST differ from the proposer. The approver MUST differ from
both the proposer and the reviewer. The executor MUST differ from all
three. Violations return `WriteGateError::SamePrincipal`.

### Contract surface for M02-P05

M02-P05 gRPC/Webhook/Kafka adapters MUST construct a `WriteGate` via
`WriteGate::propose(...)` and drive transitions through this kernel. No
M02-P05 transport may execute a mutation without observing
`WriteGateState::Approved`.

## Drivers

1. **No silent regression** — public write-path semantics are codified in
   one kernel before any transport binds them; ADR + version bump required
   to break compat (per global no-silent-regression directive).
2. **Hyperscaler-grade governance** — Stripe/AWS-style four-eyes principle
   on every write surface; separation of duties is non-negotiable.
3. **Single state machine** — exactly one source of truth across REST,
   ~~GraphQL~~ [dropped per ADR-0565], SSE, WebSocket, gRPC, Webhook, Kafka transports.

## Alternatives Considered

- **Per-transport gating** — diverges immediately; rejected.
- **Two-stage gate (Proposed → Executed)** — collapses reviewer/approver
  into one role; insufficient for the audit-chain emission SLO; rejected.
- **Implicit auto-approve for low-risk paths** — opaque to auditors,
  bypasses the separation-of-duties invariant; rejected.

## Consequences

### Positive

- Every write transport in M02-P05 inherits the same gating semantics for
  free; no per-transport policy drift.
- Audit-chain emission consumes one canonical envelope
  (`ApprovalEnvelope`) instead of N transport-specific shapes.
- Default-deny means new code paths fail closed if a developer forgets to
  drive a gate to `Approved`.

### Negative

- Synchronous flows MUST coordinate three distinct principals; for
  automation use cases this implies machine identities per role and
  associated key management.
- Adds latency to writes vs. a direct-execute path; mitigated by
  M02-P05 batch APIs.

## Follow-ups

1. M02-P05 IP-001: bind gRPC adapter to `WriteGate`.
2. M02-P05 IP-002: bind Webhook adapter to `WriteGate`.
3. M02-P05 IP-003: bind Kafka producer to `WriteGate`.
4. M03: persistent `ApprovalEnvelope` store + replay.
