---
ip_id: IP-EMR-004
title: REST surface (FHIR R5 default + R4 bridge)
microservice: emr
status: planned
date: 2026-05-21
sequence: 4
depends_on: [IP-EMR-002, IP-EMR-003]
unblocks: [IP-EMR-005, IP-EMR-006, IP-EMR-010]
estimated_effort_hours: 90
owner: axis-emr
---

# IP-EMR-004: REST surface (FHIR R5 + R4)

## Goal

Build the REST surface per `contracts/openapi-emr-v1.yaml`. Default FHIR R5; R4 via Accept-Version (per ADR-EMR-MS-002).

## Deliverables

- 15 crates `oya-emr-<bc>-api` per BC.
- One collated `oya-emr-rest` crate that mounts all BC routes + auth middleware + Cedar middleware + tracing middleware.
- FHIR R5↔R4 bridge in `oya-emr-rest/src/fhir_bridge/r5_r4_map.rs`.
- Capability Statement at `/fhir/metadata` declaring both versions.
- OpenAPI 3.2.0 emission from code via `utoipa` (Rust crate).

## Acceptance criteria

- Every endpoint in `openapi-emr-v1.yaml` responds.
- FHIR R5 reference-validator passes for emitted resources.
- FHIR R4 reference-validator passes for downgraded resources.
- R5↔R4 round-trip unit tests pass.
- `cargo test -p oya-emr-rest --test fhir_r5_r4_roundtrip` exits 0.
- Cedar middleware invocation per `x-cedar-action` annotation in OpenAPI matches request handlers.
- Trace context propagated end-to-end.

## Out of scope

- AsyncAPI (IP-EMR-005).
- gRPC (IP-EMR-006).
- Mobile-app client (IP-EMR-010).

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/emr/implementation-plans/IP-004-rest-fhir-r5-r4-bridge.md:18` - Build the REST surface per `contracts/openapi-emr-v1.yaml`. Default FHIR R5; R4 via Accept-Version (per ADR-EMR-MS-002).; `microservices/emr/implementation-plans/IP-004-rest-fhir-r5-r4-bridge.md:30` - - Every endpoint in `openapi-emr-v1.yaml` responds..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/emr/implementation-plans/IP-004-rest-fhir-r5-r4-bridge.md:26` - - OpenAPI 3.2.0 emission from code via `utoipa` (Rust crate)..
