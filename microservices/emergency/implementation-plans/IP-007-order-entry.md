# IP-007 — Rapid CPOE + Verbal Order Countersign + Protocol Order Sets

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight)
Sequence: 7 / 10
Depends-on: IP-001, IP-003

---

## Scope

Rapid CPOE tuned for ED tempo. Protocol-driven order sets (e.g., Sepsis Bundle = lactate + cultures + antibiotic + crystalloid). Verbal-order entry with 24h countersign window. Routing to `pharmacy`, `lab`, `imaging` µservices over AsyncAPI.

## Deliverables

- `src/crates/emergency-orderentry/` — order aggregate.
- Protocol order set catalog.
- Verbal-order bridge with countersign timer.
- Drug-interaction check via `intelligence` clinical decision support (300 ms timeout, advisory-skipped fallback).
- Cedar `verbal-order-bridge.cedar` enforced.
- `ed.order.placed`, `ed.order.signed`, `ed.order.unverified.window-exceeded` events.

## Acceptance

- p95 order entry round-trip ≤ 400 ms.
- Protocol order set drops in one click.
- Verbal-order backlog block triggers at 5+ unsigned.
