---
doc_class: Sdk Plan
microservice: marketplace
status: Accepted
date: 2026-05-20
owner_team: axis-marketplace
primary_adr: ADR-0314
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0249, ADR-0314]
companion_docs: [microservices/marketplace/README.md, docs/standards/documentation-rigor.md]
planned_enforcement_ref: oya-governance-marketplace-doc-set
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
---

# SDK Plan

## Purpose
Defines typed client generation from OpenAPI, AsyncAPI, and proto3 without exposing secrets.

## Scope
seller listing, buyer order, deal set acceptance, escrow reservation, revenue share, mediation, export, appointment commitment, and cross-border settlement evidence.

## Controls
Tenant scope, Cedar default-deny, audit-chain evidence, OpenBao secret references, and ADR-0314.

## Verification
Contract parsing, JSON parsing, YAML parsing, line-floor checks, artifact counts, and marker-token scans.

## Naming justifications: BNF v4 and 13-layer enum conformance

Every new artifact uses the BNF v4 grammar `<service>.<bounded_context>.<action>.<resource>` for actions and `oya-marketplace-<bounded-context>-<layer>` for crate and catalog names.
The ADR-0105/ADR-0106 canonical 13-layer enum used by this doc set is kernel, domain, usecase, app, adapter, infrastructure, rest, grpc, graphql, worker, cli, sdk, api.
The doc set keeps ADR-0105 compatibility by mapping the 12 deployable layers into the larger canonical enum without inventing a new layer name.
The service slug `marketplace` is retained because it is already the microservice directory name, policy prefix, catalog prefix, and endpoint namespace.
The primitive name `DealSet` is retained because it is the smallest stable object that lets the journey IP slices share one contract without leaking unrelated service ownership.
The secondary primitive `SettlementLedger` is retained because it names the audit-backed record that downstream services consume without taking direct table ownership.

