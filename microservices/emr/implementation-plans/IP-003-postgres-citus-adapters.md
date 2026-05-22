---
ip_id: IP-EMR-003
title: Postgres + Citus persistence adapters per BC
microservice: emr
status: planned
date: 2026-05-21
sequence: 3
depends_on: [IP-EMR-001]
unblocks: [IP-EMR-007, IP-EMR-008]
estimated_effort_hours: 120
owner: axis-emr
---

# IP-EMR-003: Postgres + Citus adapters

## Goal

Implement the per-BC Postgres + Citus tenant-sharded persistence adapters that bind the kernel ports. Schema management via `sqlx::migrate!`.

## Deliverables

- 14 crates `oya-emr-<bc>-adapter-postgres` (one per BC except `vital` which uses TimescaleDB, IP-EMR-008).
- Migration files under `microservices/emr/src/crates/oya-emr-<bc>-adapter-postgres/migrations/`.
- Citus distributed-table `SELECT create_distributed_table('<table>', 'tenant_id')` for every table.
- Tenant-shard-key index on every table.
- Index strategy per BC (e.g., MRN lookup, encounter by patient-id+date-range, medication active-by-patient-id, allergy by patient-id, note by patient-encounter, order by status, result by ordering-clinician).

## Acceptance criteria

- All adapter crates compile.
- Integration tests bring up a Citus testcontainer; CRUD operations pass.
- Schema migration files are versioned; rollback (`sqlx migrate revert`) works.
- Citus distribution column verified.
- `cargo nextest run --workspace --test-threads=8` exits 0.

## Out of scope

- Vital (TimescaleDB) — IP-EMR-008.
- Portal session (Valkey) — IP-EMR-009.
- Schema seeding for tenant onboarding — separate runbook.
