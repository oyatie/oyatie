# Oyatie Runbook — Subscription Token Expired (Provider Subscription Auth)

> **Status:** Stub (P0 must-have for W-Foundation gate per [`RUNBOOKS-INDEX.md`](../../RUNBOOKS-INDEX.md))
> **Owner:** `axis-foundry`
> **Severity scope:** Sev 2
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored; full procedure landed at W-Foundation gate)

## Symptom
a subscription-mode session token (Claude / OpenAI / Gemini) has expired or been invalidated; subscription-auth invocations are failing

## Detection
- Source signal: Per-provider subscription health probe; PTY warm-pool exhaustion; capability error rate spike on subscription path
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev 2 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Failover the affected tenant traffic to API-mode auth per ADR-0020; pause subscription-only capabilities

## Diagnosis
Check the per-provider auth flow; verify the provider has not changed its subscription contract; verify the PTY warm-pool refresh schedule

## Recovery
Refresh the subscription token (idle-refresh path); restart the PTY warm pool if needed; if provider auth flow changed, ship a per-provider contract-test update + adapter patch

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
