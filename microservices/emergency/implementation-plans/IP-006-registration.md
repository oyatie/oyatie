# IP-006 — Registration (Quick-Reg + Pre-Arrival + Walk-In) + Identity Reconciliation

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight) | ADR-0244 (tenant scoping)
Sequence: 6 / 10
Depends-on: IP-001, IP-005

---

## Scope

Three registration modes (quick-reg, pre-arrival from EMS, walk-in). Identity reconciliation workflow merges quick-reg placeholders into canonical Patient records once identity lands. Registration never blocks triage.

## Deliverables

- `src/crates/emergency-registration/` — registration aggregate.
- Quick-reg form (≤ 30 s) with placeholder name.
- Pre-arrival projection from EMS handoff.
- Walk-in registration with insurance fields.
- Identity reconciler workflow.
- Cedar `registration-can-quick-reg.cedar` enforced.
- `ed.patient.registered` event.

## Acceptance

- Quick-reg completes in under 30 s on standard hardware.
- Pre-arrival projection auto-populates on EMS report.
- Reconciliation preserves placeholder history.
