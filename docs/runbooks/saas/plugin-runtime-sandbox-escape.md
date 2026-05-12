# Oyatie Runbook — Plugin Runtime Sandbox Escape

> **Status:** Stub (P0 must-have for W-Foundation gate per [`RUNBOOKS-INDEX.md`](../../RUNBOOKS-INDEX.md))
> **Owner:** `axis-saas + axis-foundry + ops-security`
> **Severity scope:** Sev 1
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored; full procedure landed at W-Foundation gate)

## Symptom
plugin sandbox escape detected per ADR-0036 + ADR-0023

## Detection
- Source signal: Plugin sandbox audit-emission anomaly; per-plugin syscall outside allowlist; resource-cap exceedance
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev 1 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Quarantine the offending plugin; revoke its install per affected tenant; preserve sandbox state

## Diagnosis
Identify escape vector; check trust-tier of plugin (verified-isv / community / experimental); audit Cosign signing chain

## Recovery
Patch sandbox; ship plugin-substrate update; if vendor: notify vendor of mandatory update; emit EVT-PLUGIN-SANDBOX-ESCAPE

## Verify-recovery
- Confirm SLO error budget recovers within Sev 1 recovery SLO
- Confirm audit-chain integrity per ADR-0003
- Run the per-axis fitness lane that originally would have caught this; if it did not, file a prevention ticket per [`standards/prevention-doctrine.md`](../../standards/prevention-doctrine.md)

## Post-incident
- Author postmortem within Sev 1 SLA per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md)
- Add row to [`MISTAKES-LEDGER.md`](../../MISTAKES-LEDGER.md) with mechanical-prevention proposal
- Emit `EVT-PREVENTION-SHIPPED` per ADR-0003 once prevention lands

## Sources
[INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](../../SLO-CATALOG.md), [`standards/prevention-doctrine.md`](../../standards/prevention-doctrine.md), [`templates/runbook-template.md`](../../templates/runbook-template.md), ADR-0003.
