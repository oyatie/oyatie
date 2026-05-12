# Oyatie Runbook — Cloud Region Failover

> **Status:** Stub (P0 must-have for W-Foundation gate per [`RUNBOOKS-INDEX.md`](../../RUNBOOKS-INDEX.md))
> **Owner:** `axis-cloud + ops-sre-reliability`
> **Severity scope:** Sev 1
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored; full procedure landed at W-Foundation gate)

## Symptom
a region or AZ has degraded; tenant traffic must be cut over to the failover region/cell per ADR-0009 cell-architecture + ADR-0049 cross-region replication

## Detection
- Source signal: Region health dashboard; AZ-loss alert; cell-isolation evidence anomaly
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev 1 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Initiate cell-routing failover; freeze writes on the failing region; ensure data replication is up to the failover cell's replay anchor

## Diagnosis
Verify replication lag is within RPO; check tenant residency policy per ADR-0049; identify any tenants pinned to the failed region with no failover

## Recovery
Cut over traffic; verify SLO recovery; emit EVT-REGION-FAILOVER; notify affected tenants per RACI; schedule a DR drill if the failover surfaced a gap

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
