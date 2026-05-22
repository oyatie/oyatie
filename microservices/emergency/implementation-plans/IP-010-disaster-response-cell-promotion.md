# IP-010 — Disaster Response + Drill Mode + Cell-Tier Promotion

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight) | ADR-0248 (cellular architecture)
Sequence: 10 / 10
Depends-on: IP-001..IP-009

---

## Scope

ICS / HICS activation + facility status state machine (green/yellow/red/black + MCI flag). Drill mode with parallel metrics. Cell-tier promotion gates wired.

## Deliverables

- `src/crates/emergency-disasterresponse/` — ICS activation + facility status.
- Surge coordination event fanout to `incident-management`, `ops-dashboard-control-center`, supply-chain / staffing µservices.
- Drill mode end-to-end (already partly wired in IP-004; finalized here).
- Cell-tier promotion playbook + automated gate checks:
  - 14-day SLO green.
  - All Cedar policies signed at the current revision.
  - HIPAA + SOC2 + EMTALA packs attested.
  - Trauma registry export sample passes ACS-conformance.
- `ed.disaster.activated`, `ed.disaster.deactivated` events.

## Acceptance

- ICS activation publishes within 1 s.
- Drill mode runs concurrently with production without contamination.
- Promotion gates run from `dev` → `Tier-1` → `Tier-2` cleanly in staging.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emergency/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emergency/implementation-plans/IP-010-disaster-response-cell-promotion.md:21` - - 14-day SLO green..
