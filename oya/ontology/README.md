---
doc_class: Reference
shape: Explanation
length_cap: 400
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0006, ADR-0257]
companion_docs:
  - microservices/ontology/PRD.md
  - microservices/ontology/ARCHITECTURE.md
  - microservices/ontology/manifest.json
inbound_citations:
  - docs/README.md
  - docs/DOC-CATALOG.md
---

# Ontology µservice — README

## What this µservice does

The Ontology µservice is the Palantir-Foundry-Ontology-equivalent layer for oyatie. It defines the canonical object-type registry (Person, Document, Recording, plus tenant extensions), serves typed entity reads with library-first dispatch per ADR-0257 amendment, gates writes through Cedar fragments per ADR-0243, and projects derived views to ClickHouse for analytical workloads. It is **substrate**, not a product surface, and is consumed by every product µservice in the platform.

## Quick links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance pack roster: `compliance.md`
- Capacity model: `capacity-model.md`
- Cost budget: `cost-budget.md`
- Failure modes: `failure-modes.md`
- Multi-region: `multi-region.md`
- Incident response: `incident-response.md`
- Backfill / replay: `backfill-replay.md`
- Competitor parity: `competitor-parity-matrix.md`
- SDK plan: `sdk-plan.md`
- Contracts: `contracts/openapi/ontology.yaml`, `contracts/asyncapi/ontology-events.yaml`, `contracts/proto/ontology.proto`
- Cedar fragments: `policy/*.cedar`
- Runbooks: `runbooks/*.md`
- IPs: `IP-*.md`
- Dashboards: `dashboards/*.{json,md}`
- SLOs: `slos/*.openslo.yaml`
- Catalog records: `catalog/*.yaml`
- IaC: `iac/**`
- Audit findings: `AUDIT-FINDINGS-*.json`

## How to operate

1. Read `ARCHITECTURE.md` end-to-end before making any change.
2. For any read-path change, cross-reference ADR-0257 + amendment.
3. For any Cedar fragment change, follow the 60s soak per ADR-0294 and the runbook at `runbooks/cedar-fragment-rollback.md`.
4. For any type-registry change, follow the deprecation playbook at `runbooks/object-type-deprecation.md`.

## How to consume

Library-first: depend on `oya-ontology-read-path-library` in your µservice's `Cargo.toml`; never network-call the ontology µservice for reads. Writes go through the REST contract at `contracts/openapi/ontology.yaml`.

## Status

Substrate, ga. Eligible for Tier-0/1/2 cells; Tier-3 edge cells use the library-only read path with eventual consistency.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
