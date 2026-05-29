---
ip_id: IP-EMR-005
title: AsyncAPI event surface + Kafka publisher / consumer
microservice: emr
status: planned
date: 2026-05-21
sequence: 5
depends_on: [IP-EMR-002]
unblocks: [IP-EMR-010]
estimated_effort_hours: 70
owner: axis-emr
---

# IP-EMR-005: AsyncAPI events

## Goal

Implement the AsyncAPI publisher + consumer per `contracts/asyncapi-emr-v1.yaml`. Kafka topics per per-cell deployment.

## Deliverables

- 15 crates `oya-emr-<bc>-events`.
- Each publisher emits to its declared `emr.*` topic.
- Each consumer subscribes to the upstream peer-µservice topics (diagnostics, pharmacy, care-management) and routes events to the appropriate `oya-emr-worker-*` crate.
- Idempotency by `idempotency_key` on every event.
- CloudEvents envelope per ADR-0263.

## Acceptance criteria

- All 26 channels (24 publish + 3 consume from peers) wired.
- Schema-registry-compatible (Avro / Protobuf or JSON-schema).
- Integration tests bring up Kafka testcontainer; publish/consume roundtrip works.
- Audit lag SLO (slos/audit-emission-lag.openslo.yaml) measurable.

## Out of scope

- gRPC sync (IP-EMR-006).
- Audit-chain emission via direct gRPC (IP-EMR-006).

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/emr/implementation-plans/IP-005-asyncapi-events.md:18` - Implement the AsyncAPI publisher + consumer per `contracts/asyncapi-emr-v1.yaml`. Kafka topics per per-cell deployment..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emr/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emr/implementation-plans/IP-005-asyncapi-events.md:33` - - Audit lag SLO (slos/audit-emission-lag.openslo.yaml) measurable..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/emr/implementation-plans/IP-005-asyncapi-events.md:33` - - Audit lag SLO (slos/audit-emission-lag.openslo.yaml) measurable.; `microservices/emr/implementation-plans/IP-005-asyncapi-events.md:38` - - Audit-chain emission via direct gRPC (IP-EMR-006)..
