---
id: ADR-0175
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - council-privacy
  - axis-workspace
supersedes: []
superseded_by: [ADR-700]
related:
  - ADR-0002-tenant-and-identity-kernel.md
  - ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md
  - ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md
  - ADR-0222-saga-compensation-portfolio-policy.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
doc_class: Architecture-Decision-Record
purpose: >
  Canonical six-state tenant lifecycle (Pending → Active → Suspended
  → Migrating → Offboarded → DeletionConfirmed). Every transition is
  a saga (per ADR-0222). Deletion confirmation requires proof-of-erasure
  fan-out from every data-class-bearing µservice (per ADR-0038).
enforcement_status: advisory-until-tenancy-microservice-implements
enforced_by: oya gate validate tenant-lifecycle
---

# ADR-0175: Tenant lifecycle workflow

## Status

Accepted — 2026-05-18. Enforcement is advisory until the tenancy
µservice (`microservices/tenancy/`) implements the six-state machine
and every data-class-bearing µservice wires its erasure-receipt
emitter.

## Context

ADR-0002 (tenant + identity kernel) establishes the Tenant entity but
is silent on the *workflow* that moves a tenant through onboard →
active → suspended → migrating → offboarded → deleted.

ADR-0038 (DSR cascade + proof of erasure) covers deletion narrowly: it
guarantees that when a tenant requests erasure, every µservice with
PII for that tenant must emit an erasure receipt. But there is no
binding ADR for the broader lifecycle: how does the tenant arrive in
"Active"? What does "Suspended" mean operationally? Who triggers
"Migrating"? When is "Offboarded" complete vs "DeletionConfirmed"?

Without an explicit lifecycle workflow, individual µservices invent
their own onboard + offboard paths, and the audit-chain can record
neither the full lifecycle nor a provable end-state.

## Decision

### D-1. Canonical six-state machine

```
Pending ──onboard_saga──▶ Active ──┬──suspend_saga──▶ Suspended ──unsuspend_saga──▶ Active
                                    │
                                    └──migrate_saga──▶ Migrating ──migrate_completion──▶ Active (in target cell)
                                                                                │
                                                                                └──offboard_saga──▶ Offboarded
                                                                                                       │
                                                                                              delete_saga
                                                                                                       │
                                                                                                       ▼
                                                                                          DeletionConfirmed
```

State semantics:

| State | Meaning | Allowed transitions |
| --- | --- | --- |
| `Pending` | Tenant record created; cell-assignment pending | → Active (onboard_saga success); → Cancelled (terminal) |
| `Active` | Tenant operating normally | → Suspended; → Migrating; → Offboarded |
| `Suspended` | Tenant operations frozen (non-payment / regulatory hold / abuse investigation) | → Active (unsuspend); → Offboarded |
| `Migrating` | Tenant data in flight between cells (per ADR-0009 cell architecture) | → Active (target cell); → Suspended (rollback) |
| `Offboarded` | Tenant tools turned off; data retained per retention policy | → DeletionConfirmed (after retention window + erasure cascade) |
| `DeletionConfirmed` | All erasure receipts collected; tenant record marked TOMBSTONE | TERMINAL |

Cancelled (from Pending) is a terminal cousin state for never-activated
tenants.

### D-2. Every transition is a saga

Every state transition is a saga (per ADR-0222). The saga catalogue:

| Saga | Trigger | Steps (high level) |
| --- | --- | --- |
| `onboard_saga` | Tenant signup or admin provision | (1) reserve cell; (2) create identity entities; (3) provision µservice records per opted-in µservice; (4) emit `tenant.onboarded` event; compensation = reverse each step |
| `suspend_saga` | Non-payment / regulatory / abuse | (1) revoke session tokens; (2) freeze write paths per µservice; (3) emit `tenant.suspended` event; compensation = restore each |
| `migrate_saga` | Cell capacity / sovereignty change | (1) snapshot source cell; (2) replicate to target cell; (3) cut DNS; (4) drain source; compensation = abort cutover, restore source |
| `offboard_saga` | Tenant request or contract end | (1) suspend writes; (2) export data per retention policy; (3) mark each µservice "offboarded"; (4) emit `tenant.offboarded`; compensation = restore writes |
| `delete_saga` | After retention window elapses on Offboarded | (1) trigger DSR cascade (ADR-0038) per µservice; (2) collect erasure receipts; (3) verify all received; (4) mark tenant TOMBSTONE; (5) emit `tenant.deletion-confirmed`; compensation = NOT permitted (terminal) |

### D-3. Per-µservice acknowledgment

Each µservice that touches tenant data declares an `acknowledgment_kind`
per saga in its manifest:

```yaml
tenancy_acknowledgments:
  onboard_saga:
    required: true
    timeout_ms: 30000
  suspend_saga:
    required: true
    timeout_ms: 10000
  migrate_saga:
    required: optional   # only for stateful µservices
  offboard_saga:
    required: true
    timeout_ms: 60000
  delete_saga:
    required: true       # the erasure receipt is mandatory
    timeout_ms: 120000
```

The tenancy µservice fan-outs the saga step to each declared µservice;
non-acknowledgment within `timeout_ms` triggers the saga's compensation
chain.

### D-4. Audit chain integration

Every state transition emits an audit row of class `TenantLifecycle`
(forward) + per-µservice rows of class `TenantLifecycleAck` (per fan-out
acknowledgment). Compensation emits `TenantLifecycleCompensate`. The
combined row sequence is the audit-evidence the council-privacy team
references for regulator demands.

### D-5. Erasure proof binding

`delete_saga` cannot complete unless every µservice with
`acknowledgment_kind: required: true` has emitted its erasure receipt
to the audit chain. The proof binding is computed at step 3 of the
saga (per D-2) and is the gate that promotes the tenant to
DeletionConfirmed. This satisfies ADR-0038 + GDPR Art. 17 +
KR PIPA Art. 21 + CCPA + LGPD Art. 18.

### D-6. Pending state guards

A tenant remains Pending until `onboard_saga` completes. If the saga
fails non-recoverably, the tenant transitions to Cancelled (terminal).
Pending tenants cannot be billed (per ADR-0174 chargeback formula —
billing scope excludes Pending + Cancelled + Offboarded + DeletionConfirmed).

## Alternatives considered

### Alt-1. Free-form per-µservice lifecycle

Each µservice owns its own tenant lifecycle. **Rejected.** Makes the
audit-evidence requirement (D-4) impossible; regulators have no single
artifact to point at. Also makes per-µservice onboarding race-prone
(workspace ready before cell capacity reserved).

### Alt-2. Three-state machine (Active / Suspended / Deleted)

Collapse the lifecycle to a coarse three-state. **Rejected.** Hides the
Migrating state, which is operationally distinct (data in flight, dual
write); hides the Pending state, which has different billing rules;
hides the Offboarded vs DeletionConfirmed distinction, which the
retention-window jurisprudence (GDPR Art. 17 §3 exceptions) requires.

### Alt-3. Event-driven lifecycle (no central state machine)

Publish events; let each µservice derive the tenant state from the
event stream. **Rejected.** Choreography defeats the audit-evidence
requirement and makes "current state of tenant X" a query over an
event log, not a primary attribute. Operationally undebuggable.

## Consequences

### C-1. Positive

- **Lifecycle is provable.** Audit chain reconstructs every state
  transition.
- **Erasure is provable.** Every regulator demand for tenant erasure
  proof references a single artifact (the `delete_saga` audit rows).
- **Onboarding race-conditions eliminated.** Cell capacity is reserved
  before any µservice provisions.
- **Migration is a first-class operation** (matches Google's tenant-move
  pattern + AWS Organizations move-account workflow).
- **Hyperscaler-grade.** Matches Stripe onboarding state
  machine + AWS Organizations lifecycle.

### C-2. Negative

- **Every data-class-bearing µservice MUST implement the
  acknowledgment_kind block.** Mitigation: the validator catches missing
  declarations; manifest schema enforces.
- **Saga timeout tuning is per-µservice.** Mitigation: defaults in
  the canonical schema; per-µservice overrides allowed.
- **Cross-cell migration is a heavyweight saga.** Mitigation: matches
  the operational reality (data physically moves between cells).

### C-3. Sustainability

- The lifecycle is the substrate for tenant offboarding metrics:
  time-to-offboard, time-to-delete, erasure-receipt-collection latency.
- Per-quarter offboarding report joins this lifecycle's audit rows
  with ADR-0174's chargeback rows for the council-privacy team.

## Implementation surface

- `specs/tenant-lifecycle.json` — canonical state machine +
  acknowledgment schema.
- `microservices/tenancy/policy/lifecycle.md` — full policy doc.
- `microservices/tenancy/specs/saga-onboard.json` and friends
  (one per saga) — saga shape per ADR-0222.
- Validator: lane `tenant-lifecycle` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).

## References

- Stripe onboarding — *Account state machine* (public docs
  2024).
- AWS Organizations — *Account creation lifecycle* (AWS docs).
- Google Workspace Admin SDK — *Domain lifecycle states*.
- GDPR Art. 17 — *Right to erasure*.
- KR PIPA Art. 21 — *Data subject rights*.
- ADR-0002 (this portfolio) — tenant + identity kernel.
- ADR-0038 (this portfolio) — DSR cascade + proof of erasure.
- ADR-0222 (this portfolio) — saga + compensation portfolio policy.
