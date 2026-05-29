---
ip_id: IP-EMR-009
title: Portal session adapter on Valkey Cluster
microservice: emr
status: planned
date: 2026-05-21
sequence: 9
depends_on: [IP-EMR-001, IP-EMR-002]
unblocks: [IP-EMR-010]
estimated_effort_hours: 30
owner: axis-emr
---

# IP-EMR-009: Portal session adapter

## Goal

Implement the portal-session BC's Valkey Cluster adapter for hot session state + periodic flush to Postgres event log for audit.

## Deliverables

- Crate `oya-emr-portal-session-adapter-valkey`.
- Valkey Cluster connection pool (6-node).
- Session namespace `emr:session:<session_id>` with TTL 30min default, sliding.
- Flush worker writes session-events to Postgres `portal_session_log` table tenant-sharded.
- Failover-resilient.

## Acceptance criteria

- Adapter compiles.
- Valkey Cluster integration test passes (3 master + 3 replica).
- Session create / read / extend / destroy.
- Postgres flush verified.
- `cargo test -p oya-emr-portal-session-adapter-valkey` exits 0.
