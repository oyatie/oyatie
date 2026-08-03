---
doc_class: MicroserviceREADME
microservice: treasury
status: preview-metadata-only
date: 2026-05-21
owner_team: axis-treasury + axis-erp-parity
authority_ref: ../../specs/microservices/treasury.json
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0253, ADR-0297, ADR-0314, ADR-0315]
planned_enforcement_ref: oya-governance-treasury-doc-set
---

# Treasury

## Source of authority

The governing product authority for this directory is `specs/microservices/treasury.json`, `PRD-MICROSERVICE-TREASURY`, status `preview`. The current scope is metadata-only: bank-account approval metadata, cash-position metadata, deterministic liquidity-forecast metadata, and cash-transfer proposal metadata.

Current non-claims from the PRD:

- No live bank connectivity, bank API call, SWIFT/host-to-host transport, or payment execution.
- No durable persistence, Postgres/RLS, Workflow execution, accounting ledger mutation, statutory filing, runtime audit-chain emission, or cloud deployment.
- No SAP, Oracle, or treasury workstation exhaustive parity claim.

## Purpose

Treasury is a flat cash-management microservice preview for ERP-grade metadata coverage. SAP TRM, Oracle Fusion Cash Management, Workday Financial Management, NetSuite Cash Management, and Microsoft Dynamics 365 Finance Cash and Bank are reference benchmarks only; this README does not claim exhaustive vendor parity or runtime readiness.

## Bounded contexts

- cash-position: metadata for bank-statement/exposure-flow evidence and computed cash-position fields.
- liquidity-forecast: deterministic liquidity projection metadata.
- bank-account: approval metadata for bank-account master references and control evidence.
- debt-instrument: planned catalog metadata only.
- fx-exposure: planned catalog metadata only.
- hedge-designation: planned catalog metadata only.

## Contracts

- REST contract shape: `contracts/openapi-v1.yaml`, OpenAPI 3.2.0.
- Event payload shape: `contracts/asyncapi-v1.yaml`, AsyncAPI 3.1.0.
- gRPC/protobuf shape: `contracts/treasury-v1.proto`, proto3.
- Naming: BNF v4.1 and ADR-0105 layers api, rest, application, usecase, domain, kernel, adapter, worker, and governance.

The contract files are API-first metadata surfaces. They are not evidence that an edge route, event broker, deployed service, payment rail, live bank integration, persistence layer, Workflow execution path, runtime audit-chain emitter, cloud deployment, SLO, or DR posture exists.

## Inventory posture

Files under `capabilities/`, `catalog/`, `policy/`, and `cedar/` are planning and metadata evidence for the preview PRD ceiling. Runtime-looking artifacts under `iac/`, `slos/`, `runbooks/`, `dashboards/`, and `dpia/` remain non-authoritative planning inventory unless and until a later PRD or evidence gate promotes them.

## Doctrine references

- [ADR-0105](../../docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md): naming and layer vocabulary.
- [ADR-0131](../../docs/decisions/ADR-0131-per-microservice-flat-layout.md) and [ADR-0132](../../docs/decisions/ADR-0132-product-platform-and-bundle-dissolution.md): flat single-concern microservice shape.
- [ADR-0244](../../docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md): tenant-as-scope security model.
- [ADR-0314](../../docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md): marketplace settlement boundary references.
