---
doc_class: ImplementationPlan
microservice: marketplace
status: Accepted
date: 2026-05-20
owner_team: axis-marketplace
primary_adr: ADR-0314
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0249, ADR-0314]
companion_docs: [microservices/marketplace/README.md, docs/standards/documentation-rigor.md]
planned_enforcement_ref: oya-governance-marketplace-doc-set
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
impl_plan_id: IP-015-async-settlement-events
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-015: Async Settlement Events

## Intent
Deliver the async-settlement-events slice for Marketplace while preserving DealSet, tenant scope, Cedar default-deny, audit-chain evidence, and ADR-0314.

## Existing journey anchor
This IP is additive to `microservices/marketplace/IP-journey-j73-revenue-share.md` and weaves Revenue Share into the build sequence.

## Boundary
- Owns: microservices/marketplace/ code, docs, contracts, policy, SLO, dashboard, catalog, and IaC for this slice.
- Consumes: payments, treasury, finops-portal, ontology, workflow-engine, connect, identity, audit-chain, global-trade through typed contracts only.
- Does not own: adjacent service tables, provider credentials, or global ADR doctrine.

## Deliverables
1. Kernel/domain invariant for DealSet.
2. Usecase command and idempotency behavior.
3. REST or worker binding with OpenAPI 3.2.0, AsyncAPI 3.1.0, or proto3 as applicable.
4. Cedar action marketplace.async-settlement-events.execute using BNF v4.
5. Audit event MarketplaceDealOffered.
6. Dashboard and OpenSLO evidence.
7. Runbook branch for failure and rollback.

## Acceptance criteria
- Contract parses and declares exact required version.
- Policy denies cross-tenant access without explicit grant.
- Audit evidence includes tenant_id, sub_scope_path, principal_hash, cell_id, region, and evidence_ref.
- Tests cover positive, denial, idempotency, replay, and compensation paths.
- No unresolved marker tokens remain.

## Naming justifications: BNF v4 and 13-layer enum conformance

Every new artifact uses the BNF v4 grammar `<service>.<bounded_context>.<action>.<resource>` for actions and `oya-marketplace-<bounded-context>-<layer>` for crate and catalog names.
The ADR-0105/ADR-0106 canonical 13-layer enum used by this doc set is kernel, domain, usecase, app, adapter, infrastructure, rest, grpc, graphql, worker, cli, sdk, api.
The doc set keeps ADR-0105 compatibility by mapping the 12 deployable layers into the larger canonical enum without inventing a new layer name.
The service slug `marketplace` is retained because it is already the microservice directory name, policy prefix, catalog prefix, and endpoint namespace.
The primitive name `DealSet` is retained because it is the smallest stable object that lets the journey IP slices share one contract without leaking unrelated service ownership.
The secondary primitive `SettlementLedger` is retained because it names the audit-backed record that downstream services consume without taking direct table ownership.

