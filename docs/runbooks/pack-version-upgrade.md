---
purpose: Auto-backfilled purpose for pack-version-upgrade.md
---

# Oyatie Runbook — Pack Version Upgrade

> **Status:** Stub (deferred to W-Foundation gate per [`RUNBOOKS-INDEX.md`](../RUNBOOKS-INDEX.md))
> **Owner:** TBD per RACI
> **Authored from:** [`templates/runbook-template.md`](../templates/runbook-template.md)
> **Last verified:** 2026-05-09 (stub authored to satisfy doc-link integrity; full procedure lands at W-Foundation gate)

## Symptom
TODO — fill at W-Foundation authoring pass.

## Detection
TODO — fill at W-Foundation authoring pass. Source signals + paging policy per [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md).

## First-response checklist
1. Acknowledge page; declare incident in #incident-bridge
2. Open the SLO dashboard for the affected surface
3. Capture the audit-chain segment for the impact window per ADR-0003
4. Apply the immediate stop-bleeding step listed in §"Containment"

## Containment
TODO — fill at W-Foundation authoring pass.

## Diagnosis
TODO — fill at W-Foundation authoring pass.

## Recovery
TODO — fill at W-Foundation authoring pass.

## Verify-recovery
- Confirm SLO error budget recovers within recovery SLO
- Confirm audit-chain integrity per ADR-0003
- File MFL row + ship a fitness lane catch if structural

## Sources
[INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](../SLO-CATALOG.md), [`standards/prevention-doctrine.md`](../standards/prevention-doctrine.md), [`templates/runbook-template.md`](../templates/runbook-template.md), ADR-0003.
