---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry
deciders: axis-foundry, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/contracts/asyncapi/provider-events.yaml
  - microservices/intelligence-providers/PRD.md
  - microservices/intelligence-providers/policy/credential-isolation.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (foundry-providers µservice)

## Purpose

Specify how foundry-providers handles two scenarios:

1. **Backfill** — historical `ProviderInvoked` events need re-emission (e.g., after a corrupted audit-chain segment, or after a new downstream consumer requires events the µservice has already emitted but missed).
2. **Replay** — a recorded request needs to be re-invoked against the upstream provider (e.g., to reproduce a bug, to verify an adapter version change, to validate parity for an in-house model).

These scenarios are deliberately constrained because both touch sensitive material (tenant prompts, credentials, costs).

## Backfill

### Contract

Backfill of `ProviderInvoked` events is allowed only under strict conditions:

1. **Source**: the events must exist in the foundry-evidence audit-chain (which is the authoritative store); foundry-providers does not store ProviderInvoked events itself beyond an emission buffer.
2. **Trigger**: an authorised operator runs `cargo run -p oya-dev-cli -- providers backfill --from <ts> --to <ts> --tenant <t> --consumer <consumer-id> --reason "<id>" --approver <p1> --approver <p2>` (2-person signed).
3. **Path**: foundry-providers reads the historical events from foundry-evidence, re-publishes them to the requested consumer's NATS subject with a `backfilled=true` label.
4. **Bound**: per-tenant per-day backfill bandwidth is capped (default 10⁵ events / day) to prevent abuse.

### Constraints

- Backfill events carry `backfilled=true` so downstream consumers can distinguish from live events.
- Backfill does NOT re-invoke upstream providers; it only re-emits the historical event record.
- Backfill does NOT include prompt or response bytes (only hashes); the original prompt/response bytes are owned by the tenant + vendor and not stored by oyatie beyond the audit-chain hash.
- Backfill events are subject to the same Cedar policy as live events (tenant-scope).

### Verification

- `tests/integration/backfill_authorised.rs` exercises the 2-person approval gate.
- `tests/integration/backfill_rate_limited.rs` verifies the per-tenant bandwidth cap.

## Replay (re-invocation)

Replay is the more sensitive scenario: it sends a recorded prompt back to the upstream vendor.

### Contract

Replay is permitted only when:

1. **2-person rule + tenant operator consent**. Operator + ops-security approval AND the affected tenant operator's explicit consent (recorded as a `ReplayApproved` event before the replay).
2. **Reason category** must be one of `bug-reproduction | adapter-version-validation | parity-validation | regulatory-audit`. Other categories are denied.
3. **Replay credentials**. A FRESH credential is used (not the original); rotation runbook may be invoked first.
4. **Replay scope**. Single request OR small bounded batch (≤ 100 requests per replay session).
5. **Replay isolation**. Replay invocations are tagged `replayed=true` in the ProviderInvoked event; downstream consumers (workflows, evidence) treat replayed events as out-of-band.

### Constraints

- Replay does NOT execute tool calls from the replayed response (per `threat-model.md` T-07).
- Replay does NOT affect tenant cost ledger (replay cost is charged to oyatie ops account, not tenant).
- Replay sandbox: a replay-only adapter pod (separate from production adapter pods) is used; isolation prevents cross-contamination.
- Replay history is itself audited (`ReplayInvoked` event); auditors can verify what was replayed and why.

### Verification

- `tests/integration/replay_requires_consent.rs` verifies the tenant-consent requirement.
- `tests/integration/replay_isolated.rs` verifies replay events do not affect tenant cost ledger.

## Forbidden Operations

| Operation | Status |
|---|---|
| Bulk replay of all tenant invocations | FORBIDDEN |
| Replay of credentials (replaying captured credentials) | FORBIDDEN; impossible by design (we never store credentials outside OpenBao) |
| Backfill into a foreign tenant's stream | FORBIDDEN; Cedar deny |
| Replay outside the bounded categories | FORBIDDEN; deterministic deny |

## References

- `microservices/intelligence-providers/contracts/asyncapi/provider-events.yaml`.
- `microservices/intelligence-providers/policy/credential-isolation.md`.
- ADR-0028 audit-chain seal posture.
- `microservices/intelligence-providers/runbooks/credential-rotation.md`.
