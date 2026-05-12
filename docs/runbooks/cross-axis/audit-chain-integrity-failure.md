# Oyatie Runbook — Audit-Chain Integrity Failure

> **Status:** Stub (P0 must-have for W-Foundation gate per [`RUNBOOKS-INDEX.md`](../../RUNBOOKS-INDEX.md))
> **Owner:** `platform-audit-evidence + ops-sre-reliability`
> **Severity scope:** Sev 1
> **Authored from:** [`templates/runbook-template.md`](../../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored; full procedure landed at W-Foundation gate)

## Symptom
audit-chain hash-link mismatch detected on a per-tenant or global shard

## Detection
- Source signal: ADR-0003 chain-integrity check exit code; `oya-foundry-fitness-audit-emit` block; chain-replay drill anomaly
- Page who: per [INCIDENT-MANAGEMENT.md](../../INCIDENT-MANAGEMENT.md) Sev 1 ladder

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"
5. Notify owner team's on-call rotation per RACI

## Containment
Freeze writes on the affected shard; switch tenant traffic to read-only; preserve current chain segment as evidence

## Diagnosis
Compare last-known-good Merkle root with current; identify the first divergent block; check for clock skew, KMS rotation race, or store-tier drift

## Recovery
Restore from last-known-good Merkle root; replay events from outbox; emit EVT-AUDIT-CHAIN-RECOVERED; re-enable writes after independent verifier signs

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
