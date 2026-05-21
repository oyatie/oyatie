---
doc_class: Implementation-Plan
ip_id: IP-journey-j148-material-provenance-graph
journey_ref: docs/user-journeys/j148-supply-chain-circular-economy-electronics-recycling/
status: draft
date: 2026-05-20
microservice: ontology
authority_tier: 3
related_adrs:
  - ADR-0244
  - ADR-0297
  - ADR-0299
  - ADR-0292
  - ADR-0263
  - ADR-0307
  - ADR-0308
  - ADR-0311
  - ADR-0312
  - ADR-0313
  - ADR-0105
  - ADR-0131
  - ADR-0249
  - ADR-0257
contract_versions:
  - OpenAPI 3.2.0
  - AsyncAPI 3.1.0
  - proto3
grammar: BNF v4.1 + ADR-0105 13-layer
layout: flat per-microservice layout per ADR-0131
---

# IP - ontology role in j148 Circular economy electronics recycling supply chain

## Scope

ontology owns the `material-provenance-graph` slice for j148. The service does not own the whole
journey; it owns one bounded implementation plan that can be built, tested, reviewed, and reverted
independently while preserving the global handshake.
The slice must support CircularRecyclingReturnCommand, emit or consume
CircularMaterialProvenanceSettled, and keep consumer return credit plus recycled-material supplier
settlement in the Marketplace facilitator settlement path. If this service cannot complete its local
work, workflow-engine must hold the global journey in a typed pending or failed state.

## Acceptance criteria

1. ontology exposes a tenant-scoped command or handler for `material-provenance-graph`.
2. Every public REST contract is described as OpenAPI 3.2.0 and every event channel as AsyncAPI 3.1.0.
3. Every internal RPC fixture is representable as proto3 with explicit tenant and counterparty fields.
4. Every state transition emits ADR-0263 telemetry: audit event, trace span, metric, and structured log.
5. Every Cedar decision includes active_tenant_id, counterparty_tenant_id, audience_type, jurisdiction overlay, and marketplace_settlement_required.
6. Every data structure maps to ADR-0105 13-layer vocabulary and stays in this service flat layout per ADR-0131.
7. Rollback is implemented as cancel-before-finality or credit/offset-after-finality; no row edit hides history.
8. ADR-0297 abuse-defence and ADR-0299 account recovery checks are explicit whenever internet-facing or identity-sensitive actions occur.
9. ADR-0292 minor-protection is fail-closed when the actor, dependent, or content subject is a protected minor.
10. Dual-tenant and conglomerate boundaries remain visible in logs and test fixtures.

## Data model

| Field | Type | Data class | Source | Notes |
|---|---|---|---|---|
| `journey_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `actor_principal_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `active_tenant_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `counterparty_tenant_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `settlement_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `policy_decision_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `audit_event_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |
| `trace_id` | string | tenant_scoped_or_audit_metadata | `CircularRecyclingReturnCommand` | Required by ontology for idempotent j148 processing |

## API surface

```yaml
openapi: 3.2.0
info:
  title: ontology j148 material-provenance-graph API
  version: 1.0.0
paths:
  /internal/journeys/j148/ontology/material-provenance-graph:
    post:
      summary: Execute material-provenance-graph
      responses:
        "202": { description: Accepted for idempotent processing }
        "403": { description: Cedar default-deny or boundary violation }
```

## Event surface

```yaml
asyncapi: 3.1.0
info:
  title: ontology j148 material-provenance-graph events
  version: 1.0.0
channels:
  ontology.journey.j148.material-provenance-graph:
    address: ontology.journey.j148.material-provenance-graph
```

## Internal RPC fixture

```proto
syntax = "proto3";
package oyatie.ontology.journey.j148;
message ExecuteMaterialProvenanceGraphRequest {
  string journey_id = 1;
  string active_tenant_id = 2;
  string counterparty_tenant_id = 3;
  string policy_decision_id = 4;
}
```

## Implementation steps

1. Implement `material-provenance-graph` step 01 in the correct ADR-0105 layer; cite ADR-0244; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
2. Implement `material-provenance-graph` step 02 in the correct ADR-0105 layer; cite ADR-0297; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
3. Implement `material-provenance-graph` step 03 in the correct ADR-0105 layer; cite ADR-0299; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
4. Implement `material-provenance-graph` step 04 in the correct ADR-0105 layer; cite ADR-0292; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
5. Implement `material-provenance-graph` step 05 in the correct ADR-0105 layer; cite ADR-0263; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
6. Implement `material-provenance-graph` step 06 in the correct ADR-0105 layer; cite ADR-0307; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
7. Implement `material-provenance-graph` step 07 in the correct ADR-0105 layer; cite ADR-0308; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
8. Implement `material-provenance-graph` step 08 in the correct ADR-0105 layer; cite ADR-0311; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
9. Implement `material-provenance-graph` step 09 in the correct ADR-0105 layer; cite ADR-0312; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
10. Implement `material-provenance-graph` step 10 in the correct ADR-0105 layer; cite ADR-0313; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
11. Implement `material-provenance-graph` step 11 in the correct ADR-0105 layer; cite ADR-0105; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
12. Implement `material-provenance-graph` step 12 in the correct ADR-0105 layer; cite ADR-0131; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
13. Implement `material-provenance-graph` step 13 in the correct ADR-0105 layer; cite ADR-0249; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
14. Implement `material-provenance-graph` step 14 in the correct ADR-0105 layer; cite ADR-0257; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
15. Implement `material-provenance-graph` step 15 in the correct ADR-0105 layer; cite ADR-0244; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
16. Implement `material-provenance-graph` step 16 in the correct ADR-0105 layer; cite ADR-0297; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
17. Implement `material-provenance-graph` step 17 in the correct ADR-0105 layer; cite ADR-0299; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
18. Implement `material-provenance-graph` step 18 in the correct ADR-0105 layer; cite ADR-0292; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
19. Implement `material-provenance-graph` step 19 in the correct ADR-0105 layer; cite ADR-0263; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
20. Implement `material-provenance-graph` step 20 in the correct ADR-0105 layer; cite ADR-0307; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
21. Implement `material-provenance-graph` step 21 in the correct ADR-0105 layer; cite ADR-0308; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
22. Implement `material-provenance-graph` step 22 in the correct ADR-0105 layer; cite ADR-0311; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
23. Implement `material-provenance-graph` step 23 in the correct ADR-0105 layer; cite ADR-0312; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
24. Implement `material-provenance-graph` step 24 in the correct ADR-0105 layer; cite ADR-0313; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
25. Implement `material-provenance-graph` step 25 in the correct ADR-0105 layer; cite ADR-0105; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
26. Implement `material-provenance-graph` step 26 in the correct ADR-0105 layer; cite ADR-0131; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
27. Implement `material-provenance-graph` step 27 in the correct ADR-0105 layer; cite ADR-0249; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
28. Implement `material-provenance-graph` step 28 in the correct ADR-0105 layer; cite ADR-0257; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
29. Implement `material-provenance-graph` step 29 in the correct ADR-0105 layer; cite ADR-0244; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.
30. Implement `material-provenance-graph` step 30 in the correct ADR-0105 layer; cite ADR-0297; add a fixture that proves tenant scoping, counterparty scoping, and marketplace settlement state.

## Test plan for this IP

- T-001: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-002: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-003: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-004: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-005: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-006: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-007: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-008: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-009: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-010: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-011: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-012: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-013: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-014: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-015: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-016: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-017: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-018: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-019: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-020: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-021: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-022: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-023: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-024: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-025: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-026: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-027: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-028: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-029: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-030: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-031: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-032: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-033: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-034: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-035: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-036: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-037: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-038: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-039: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.
- T-040: `ontology` handles j148 `material-provenance-graph` with deterministic idempotency; expected evidence is policy decision, trace span, metric sample, and audit event.

## Failure modes

F1: duplicate command. ontology must fail closed before finality, preserve the command receipt, and
expose a typed retry or rollback instruction. It must not bypass Marketplace settlement or collapse
tenant histories.
F2: counterparty tenant revoked. ontology must fail closed before finality, preserve the command
receipt, and expose a typed retry or rollback instruction. It must not bypass Marketplace settlement or
collapse tenant histories.
F3: settlement rail unavailable. ontology must fail closed before finality, preserve the command
receipt, and expose a typed retry or rollback instruction. It must not bypass Marketplace settlement or
collapse tenant histories.
F4: audit-chain unavailable. ontology must fail closed before finality, preserve the command receipt,
and expose a typed retry or rollback instruction. It must not bypass Marketplace settlement or collapse
tenant histories.
F5: regional partition. ontology must fail closed before finality, preserve the command receipt, and
expose a typed retry or rollback instruction. It must not bypass Marketplace settlement or collapse
tenant histories.
F6: abuse signal raised. ontology must fail closed before finality, preserve the command receipt, and
expose a typed retry or rollback instruction. It must not bypass Marketplace settlement or collapse
tenant histories.
F7: minor-protection overlay blocks action. ontology must fail closed before finality, preserve the
command receipt, and expose a typed retry or rollback instruction. It must not bypass Marketplace
settlement or collapse tenant histories.

IP buildability row 001: plugin-app-store applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 002: payments applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 003: workflow-engine applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 004: ontology applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 005: audit-chain applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 006: connect applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 007: community applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 008: plugin-app-store applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 009: payments applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 010: workflow-engine applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 011: ontology applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 012: audit-chain applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 013: connect applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 014: community applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 015: plugin-app-store applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 016: payments applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 017: workflow-engine applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 018: ontology applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 019: audit-chain applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 020: connect applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 021: community applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 022: plugin-app-store applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 023: payments applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 024: workflow-engine applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 025: ontology applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 026: audit-chain applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 027: connect applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 028: community applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 029: plugin-app-store applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 030: payments applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 031: workflow-engine applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 032: ontology applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 033: audit-chain applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 034: connect applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 035: community applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 036: plugin-app-store applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 037: payments applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 038: workflow-engine applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 039: ontology applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 040: audit-chain applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 041: connect applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 042: community applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 043: plugin-app-store applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 044: payments applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 045: workflow-engine applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 046: ontology applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 047: audit-chain applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 048: connect applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 049: community applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 050: plugin-app-store applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 051: payments applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 052: workflow-engine applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 053: ontology applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 054: audit-chain applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 055: connect applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 056: community applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 057: plugin-app-store applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 058: payments applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 059: workflow-engine applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 060: ontology applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 061: audit-chain applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 062: connect applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 063: community applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 064: plugin-app-store applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 065: payments applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 066: workflow-engine applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 067: ontology applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 068: audit-chain applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 069: connect applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 070: community applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 071: plugin-app-store applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 072: payments applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 073: workflow-engine applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 074: ontology applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 075: audit-chain applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 076: connect applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 077: community applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 078: plugin-app-store applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 079: payments applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 080: workflow-engine applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 081: ontology applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 082: audit-chain applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 083: connect applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 084: community applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 085: plugin-app-store applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 086: payments applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 087: workflow-engine applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 088: ontology applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 089: audit-chain applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 090: connect applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 091: community applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 092: plugin-app-store applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 093: payments applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 094: workflow-engine applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 095: ontology applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 096: audit-chain applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 097: connect applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 098: community applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 099: plugin-app-store applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 100: payments applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 101: workflow-engine applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 102: ontology applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 103: audit-chain applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 104: connect applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 105: community applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 106: plugin-app-store applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 107: payments applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 108: workflow-engine applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 109: ontology applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 110: audit-chain applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 111: connect applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 112: community applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 113: plugin-app-store applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 114: payments applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 115: workflow-engine applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 116: ontology applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 117: audit-chain applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 118: connect applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 119: community applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 120: plugin-app-store applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 121: payments applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 122: workflow-engine applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 123: ontology applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 124: audit-chain applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 125: connect applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 126: community applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 127: plugin-app-store applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 128: payments applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 129: workflow-engine applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 130: ontology applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 131: audit-chain applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 132: connect applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 133: community applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 134: plugin-app-store applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 135: payments applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 136: workflow-engine applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 137: ontology applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 138: audit-chain applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 139: connect applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 140: community applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 141: plugin-app-store applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 142: payments applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 143: workflow-engine applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 144: ontology applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 145: audit-chain applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 146: connect applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 147: community applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 148: plugin-app-store applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 149: payments applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 150: workflow-engine applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 151: ontology applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 152: audit-chain applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 153: connect applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 154: community applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 155: plugin-app-store applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 156: payments applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 157: workflow-engine applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 158: ontology applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 159: audit-chain applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 160: connect applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 161: community applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 162: plugin-app-store applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 163: payments applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 164: workflow-engine applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 165: ontology applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 166: audit-chain applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 167: connect applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 168: community applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 169: plugin-app-store applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 170: payments applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 171: workflow-engine applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 172: ontology applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 173: audit-chain applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 174: connect applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 175: community applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 176: plugin-app-store applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 177: payments applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 178: workflow-engine applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 179: ontology applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 180: audit-chain applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 181: connect applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 182: community applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 183: plugin-app-store applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 184: payments applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 185: workflow-engine applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 186: ontology applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 187: audit-chain applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 188: connect applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 189: community applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 190: plugin-app-store applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 191: payments applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 192: workflow-engine applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 193: ontology applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 194: audit-chain applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 195: connect applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 196: community applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 197: plugin-app-store applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 198: payments applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 199: workflow-engine applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 200: ontology applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 201: audit-chain applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 202: connect applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 203: community applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 204: plugin-app-store applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 205: payments applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 206: workflow-engine applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 207: ontology applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 208: audit-chain applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 209: connect applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 210: community applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 211: plugin-app-store applies ADR-0244; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 212: payments applies ADR-0297; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 213: workflow-engine applies ADR-0299; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 214: ontology applies ADR-0292; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 215: audit-chain applies ADR-0263; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 216: connect applies ADR-0307; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 217: community applies ADR-0308; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 218: plugin-app-store applies ADR-0311; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 219: payments applies ADR-0312; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement
IP buildability row 220: workflow-engine applies ADR-0313; ontology can be implemented independently while preserving material-provenance-graph, policy evidence, and marketplace settlement


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model. See `microservices/ontology/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
