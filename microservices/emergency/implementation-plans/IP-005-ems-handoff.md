# IP-005 — EMS Handoff (Prehospital + Bedside)

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight)
Sequence: 5 / 10
Depends-on: IP-001, IP-002

---

## Scope

Receive NEMSIS v3.5 / FHIR Encounter[prehospital] / AsyncAPI EMS reports. Surface pre-arrival on the tracking board with ETA. Lock bedside handoff with co-signed close. Support BYOK for third-party EMS uplinks.

## Deliverables

- `src/crates/emergency-emshandoff/` — handoff aggregate.
- NEMSIS payload validator.
- Pre-arrival board cell rendering.
- Bedside handoff co-sign workflow.
- BYOK credential resolution for third-party CAD vendors.
- `ed.ems.report.received`, `ed.ems.handoff.completed` events.
- Cedar `byok-credential-mode.cedar` enforced for vendor uplinks.

## Acceptance

- NEMSIS sample payload ingests successfully.
- Pre-arrival board update within 500 ms of EMS report.
- Co-signed handoff produces an attestable record.
