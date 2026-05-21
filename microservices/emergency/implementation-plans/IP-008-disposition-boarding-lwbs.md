# IP-008 — Disposition + Boarding + LWBS + EMTALA Transfer

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight) | ADR-0251 (EMTALA pack)
Sequence: 8 / 10
Depends-on: IP-001, IP-002, IP-006

---

## Scope

Disposition (admit / transfer / discharge / AMA / expired), boarding tracking with threshold alerts, LWBS detection + outreach, EMTALA-compliant transfer documentation.

## Deliverables

- `src/crates/emergency-disposition/` — disposition aggregate.
- `src/crates/emergency-boarding/` — boarding hold tracking.
- `src/crates/emergency-lwbs/` — LWBS / LBTC / LBR detection.
- EMTALA transfer form integration.
- AMA flow with patient acknowledgement.
- Expired flow → chaplaincy + decedent-affairs notification.
- Cedar `ed-only-disposition.cedar` + `ama-disposition.cedar` enforced.
- `ed.disposition.set`, `ed.boarding.threshold`, `ed.lwbs.recorded`, `ed.expired.notify` events.
- OpenSLO `boarding-burden.openslo.yaml` + `lwbs-rate.openslo.yaml` wired.

## Acceptance

- Disposition close < 2 s.
- Boarding threshold events fire at exact 2h/4h/8h/12h/24h marks.
- LWBS auto-flag triggers per pack threshold.
- EMTALA transfer requires complete documentation.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emergency/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emergency/implementation-plans/IP-008-disposition-boarding-lwbs.md:25` - - OpenSLO `boarding-burden.openslo.yaml` + `lwbs-rate.openslo.yaml` wired..
