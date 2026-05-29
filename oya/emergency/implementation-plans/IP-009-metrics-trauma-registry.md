# IP-009 — Metrics Projection + Trauma Registry Feed

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight) | ADR-0251 (ACS Trauma Verification pack)
Sequence: 9 / 10
Depends-on: IP-001..IP-008

---

## Scope

Continuous door-to-X metric projection. ACS TQIP / NTDB-conformant trauma registry feed with signed export.

## Deliverables

- `src/crates/emergency-metrics/` — metrics projection.
- `src/crates/emergency-traumaregistry/` — trauma registry record.
- Door-to-doctor, door-to-CT, door-to-needle, door-to-balloon, door-to-disposition, LOS, LWBS, boarding 4h/24h metrics.
- TQIP conformant export job (signed via `audit-chain`).
- All 12 OpenSLO objects wired.
- `ed.metrics.snapshot`, `ed.trauma.registry.exported` events.
- gRPC `MetricsSubscribe` RPC.

## Acceptance

- Metrics snapshot lag ≤ 5 s end-to-end.
- TQIP export passes ACS-conformance sample.
- Signed export verifiable via `audit-chain`.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emergency/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emergency/implementation-plans/IP-009-metrics-trauma-registry.md:21` - - All 12 OpenSLO objects wired..
