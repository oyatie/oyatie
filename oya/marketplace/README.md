---
doc_class: Readme
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

# Marketplace

Marketplace is the universal deal-settlement substrate for seller listing, buyer order, deal set acceptance, escrow reservation, revenue share, mediation, export, appointment commitment, and cross-border settlement evidence.

## Start here
- Product requirements: PRD.md
- Architecture: ARCHITECTURE.md
- Compliance: compliance.md
- Contracts: contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/marketplace-v1.proto
- Policies: policies/*.cedar
- Operations: runbooks/*.md, dashboards/*.json, slos/*.openslo.yaml
- Implementation sequence: ip/IP-001-*.md through ip/IP-025-*.md

## Existing journey IP anchors
- j101: Deal Settlement Ledger -> IP-journey-j101-deal-settlement-ledger.md
- j102: Deal Settlement Ledger -> IP-journey-j102-deal-settlement-ledger.md
- j103: Deal Settlement Ledger -> IP-journey-j103-deal-settlement-ledger.md
- j107: Deal Settlement Ledger -> IP-journey-j107-deal-settlement-ledger.md
- j108: Deal Settlement Ledger -> IP-journey-j108-deal-settlement-ledger.md
- j112: Deal Settlement Ledger -> IP-journey-j112-deal-settlement-ledger.md
- j146: Seller Flow And Escrow -> IP-journey-j146-seller-flow-and-escrow.md
- j23: Seller Listing -> IP-journey-j23-seller-listing.md
- j24: Buyer Order -> IP-journey-j24-buyer-order.md
- j29: Sale Event Emitter -> IP-journey-j29-sale-event-emitter.md
- j52: Order Ledger -> IP-journey-j52-order-ledger.md
- j55: Seller Buyer Mediation -> IP-journey-j55-seller-buyer-mediation.md
- j65: Order Export -> IP-journey-j65-order-export.md
- j69: Appointment And Service Commitments -> IP-journey-j69-appointment-and-service-commitments.md
- j73: Revenue Share -> IP-journey-j73-revenue-share.md

## Naming justifications: BNF v4 and 13-layer enum conformance

Every new artifact uses the BNF v4 grammar `<service>.<bounded_context>.<action>.<resource>` for actions and `oya-marketplace-<bounded-context>-<layer>` for crate and catalog names.
The ADR-0105/ADR-0106 canonical 13-layer enum used by this doc set is kernel, domain, usecase, app, adapter, infrastructure, rest, grpc, graphql, worker, cli, sdk, api.
The doc set keeps ADR-0105 compatibility by mapping the 12 deployable layers into the larger canonical enum without inventing a new layer name.
The service slug `marketplace` is retained because it is already the microservice directory name, policy prefix, catalog prefix, and endpoint namespace.
The primitive name `DealSet` is retained because it is the smallest stable object that lets the journey IP slices share one contract without leaking unrelated service ownership.
The secondary primitive `SettlementLedger` is retained because it names the audit-backed record that downstream services consume without taking direct table ownership.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
