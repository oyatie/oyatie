---
ip_id: IP-EMR-008
title: TimescaleDB hypertable for vital signs
microservice: emr
status: planned
date: 2026-05-21
sequence: 8
depends_on: [IP-EMR-001, IP-EMR-002]
unblocks: [IP-EMR-007]
estimated_effort_hours: 30
owner: axis-emr
---

# IP-EMR-008: TimescaleDB vital hypertable

## Goal

Implement the `vital` BC's TimescaleDB adapter as a separate persistence stack from Postgres+Citus, optimized for high-frequency time-series.

## Deliverables

- Crate `oya-emr-vital-adapter-timescale`.
- TimescaleDB hypertable `vital_observation` chunked on `observed_at` (1-day chunks).
- Continuous aggregates for `hourly` and `daily` downsampled vitals.
- Compression after 7 days, retention chunk-drop after 7 years.
- Tenant-shard-key (`tenant_id`) on hypertable.

## Acceptance criteria

- Hypertable created via migration.
- Insert throughput ≥ 50k/sec sustained per cell.
- Continuous aggregate query latency ≤ 100ms p99.
- Retention policy verified.
- `cargo test -p oya-emr-vital-adapter-timescale` exits 0.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emr/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emr/implementation-plans/IP-008-timescale-vital-hypertable.md:32` - - Continuous aggregate query latency ≤ 100ms p99..
