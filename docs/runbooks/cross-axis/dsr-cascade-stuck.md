---
doc_status: published
---

# Oyatie Runbook — DSR Cascade Stuck

> **Status:** Stub (P0 must-have for W-Foundation gate per [`RUNBOOKS-INDEX.md`](../../RUNBOOKS-INDEX.md))
> **Owner:** `council-privacy + platform-eventing-og`
> **Severity scope:** Sev 2
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored; full procedure landed at W-Foundation gate)

## Symptom
a Data Subject Request cascade has not acknowledged completion across all owning axes within SLA

## Detection
- Source signal: DSR queue dashboard; per-pack regulator-watch alert; per-axis DsrCascadeAck timeout
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev 2 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Pause the originating DSR; capture the cascade trace; notify council-privacy

## Diagnosis
Walk the per-axis ack list; identify which axis kernel did not ack; check for backpressure on the eventing backbone or pillar-flow violation per PRIVACY-PROGRAM §2.2

## Recovery
Re-drive the failing axis ack; verify cascade completion; emit DSR completion receipt to subject; file MFL row if backpressure was structural

## Verify-recovery
- Confirm SLO error budget recovers within Sev 2 recovery SLO
- Confirm audit-chain integrity per ADR-0003
- Run the per-axis fitness lane that originally would have caught this; if it did not, file a prevention ticket per [`standards/prevention-doctrine.md`](../../standards/prevention-doctrine.md)

## Post-incident
- Author postmortem within Sev 2 SLA per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md)
- Add row to [`MISTAKES-LEDGER.md`](../../MISTAKES-LEDGER.md) with mechanical-prevention proposal
- Emit `EVT-PREVENTION-SHIPPED` per ADR-0003 once prevention lands

## Sources
[INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](../../SLO-CATALOG.md), [`standards/prevention-doctrine.md`](../../standards/prevention-doctrine.md), [`templates/runbook-template.md`](../../templates/runbook-template.md), ADR-0003.
