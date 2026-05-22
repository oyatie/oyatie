---
ip_id: IP-EMR-007
title: Workers — BCMA ingestion, vital streaming, results consumer, audit emitter, bulk export, legal hold
microservice: emr
status: planned
date: 2026-05-21
sequence: 7
depends_on: [IP-EMR-002, IP-EMR-003, IP-EMR-005, IP-EMR-006]
unblocks: [IP-EMR-010]
estimated_effort_hours: 60
owner: axis-emr
---

# IP-EMR-007: Workers

## Goal

Implement the 7 worker crates that handle async / long-running / fan-in / fan-out workflows.

## Deliverables

- `oya-emr-worker-bcma-ingest` — barcode med-administration scan ingestion.
- `oya-emr-worker-vital-stream` — high-frequency device-stream ingestion to TimescaleDB.
- `oya-emr-worker-results-consumer` — consume `diagnostics.result.recorded.v1` AsyncAPI; route to `result` BC.
- `oya-emr-worker-audit-emitter` — flush batched audit events to audit-chain µservice.
- `oya-emr-worker-bulk-export` — handle FHIR `$export` async kickoff + NDJSON output.
- `oya-emr-worker-legal-hold` — apply legal-hold flags + freeze retention.
- `oya-emr-worker-deidentify-projection` — emit HIPAA Safe Harbor deidentified projections for research-and-analytics tenants per consent.

## Acceptance criteria

- All 7 workers compile and respond to a simulated trigger.
- Workers run under `tokio::main` with graceful shutdown.
- Worker-level metrics exposed.
- BCMA worker integration-tests barcode-scan-validation against fixtures.
- Bulk-export worker tested with a fixture cohort of 1,000 patients.

## Out of scope

- Worker auto-scaling tuning (operations).
- Cell-level worker capacity planning (operations).
