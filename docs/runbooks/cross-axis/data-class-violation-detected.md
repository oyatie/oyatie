---
doc_status: published
---

# Oyatie Runbook — Data-Class Violation Detected

> **Status:** Stub (P0 must-have for W-Foundation gate per [`RUNBOOKS-INDEX.md`](../../RUNBOOKS-INDEX.md))
> **Owner:** `council-privacy + axis-foundry`
> **Severity scope:** Sev 2
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored; full procedure landed at W-Foundation gate)

## Symptom
a record with a forbidden data_class crossed an axis boundary it should not have crossed

## Detection
- Source signal: `governance-data-class` runtime guard; per-class compile-time annotation diff; auction-side schema rejection log
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev 2 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Block the offending caller; quarantine the affected destination index/store; capture the call trace + the schema diff

## Diagnosis
Walk the call chain to find the source service; verify the source service is in the singleton allowlist (platform-ads-gate / platform-analytics-router); check for missing schema annotation

## Recovery
Add the missing schema annotation OR remove the offending call site; verify with replay; emit MFL row + ship a fitness lane catch

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
