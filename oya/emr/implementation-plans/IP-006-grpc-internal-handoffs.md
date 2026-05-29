---
ip_id: IP-EMR-006
title: gRPC inter-µservice handoff implementations (8 services)
microservice: emr
status: planned
date: 2026-05-21
sequence: 6
depends_on: [IP-EMR-002, IP-EMR-004]
unblocks: [IP-EMR-010]
estimated_effort_hours: 80
owner: axis-emr
---

# IP-EMR-006: gRPC handoffs

## Goal

Implement the gRPC services per `contracts/proto/emr.proto` and the peer-µservice client adapters that EMR uses (per ADR-0145 direct-gRPC discipline).

## Deliverables

- Server-side: 8 gRPC services compiled from `emr.proto` (PatientService, EncounterService, MedicationService, OrderService, NoteService, VitalService, ResultService, PortalSessionService).
- Client-side adapters in `oya-emr-adapter-client-*` (14 crates):
  - `oya-emr-adapter-client-pharmacy`
  - `oya-emr-adapter-client-diagnostics`
  - `oya-emr-adapter-client-clinical-decision-support`
  - `oya-emr-adapter-client-care-management`
  - `oya-emr-adapter-client-healthcare-integration`
  - `oya-emr-adapter-client-audit-chain`
  - `oya-emr-adapter-client-policy-engine`
  - `oya-emr-adapter-client-consent-graph`
  - `oya-emr-adapter-client-workflow-engine`
  - `oya-emr-adapter-client-cloud-billing`
  - `oya-emr-adapter-client-cloud-iam`
  - `oya-emr-adapter-client-cloud-kms`
  - `oya-emr-adapter-client-cloud-storage`
  - `oya-emr-adapter-client-observability`
- Retries + circuit breaker + deadline propagation per ADR-0145.
- mTLS over QUIC (HTTP/3) per ADR-0253.

## Acceptance criteria

- All gRPC RPCs respond.
- Client adapters implement the kernel ports for outbound calls.
- mTLS handshake verified.
- W3C TraceContext propagated.
- Deadline propagation verified by simulating slow peer.
- Circuit-breaker integration tested.

## Out of scope

- Service-mesh sidecar wiring (operations).

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/emr/implementation-plans/IP-006-grpc-internal-handoffs.md:18` - Implement the gRPC services per `contracts/proto/emr.proto` and the peer-µservice client adapters that EMR uses (per ADR-0145 direct-gRPC discipline).; `microservices/emr/implementation-plans/IP-006-grpc-internal-handoffs.md:22` - - Server-side: 8 gRPC services compiled from `emr.proto` (PatientService, EncounterService, MedicationService, OrderService, NoteService, VitalService, ResultService,....
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.
