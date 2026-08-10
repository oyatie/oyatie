---
doc_class: Reference
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0245]
companion_docs:
  - app/notes/PRD.md
  - app/notes/ARCHITECTURE.md
  - app/notes/manifest.json
inbound_citations:
  - docs/README.md
---

# Notes µservice — README

## What this µservice does

Notes is oyatie's personal + work notes product. Hyperscaler precedent: Notion + Obsidian + Apple Notes + Bear + Roam Research + Logseq + Craft + Reflect + Mem.ai. Personal workspaces default to E2E encryption (per ADR-NOTES-0001); work workspaces support server-side search + intelligence with tenant-admin gating.

## Quick links

- Product requirements: `PRD.md`
- Architecture: `ARCHITECTURE.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance: `compliance.md`
- Capacity / cost / failure modes / multi-region / incident-response / backfill: `*.md`
- Competitor parity: `competitor-parity-matrix.md`
- SDK plan: `sdk-plan.md`
- Contracts: `contracts/{openapi,asyncapi,proto}/`
- Cedar fragments: `policy/*.cedar`
- Runbooks: `runbooks/*.md`
- IPs: `IP-*.md`
- Dashboards: `dashboards/*`
- SLOs: `slos/*.openslo.yaml`
- Catalog: `catalog/*.yaml`
- IaC: `iac/**`

## How to consume

- Native + web clients: sync surface over HTTP/3 + QUIC.
- Backups + export: portable Markdown + JSON archive per ADR-0276.
- Collab-edit: MLS Group + Loro CRDT.

## Status

Product, ga. Adopts the ADR-0330 `tenant_class` model: `demo_trial` tenants use capped OCI Always Free profiles where applicable, and `paid` tenants use composable `billing_components`. HIPAA + PCI pack overlays are `compliance_pack` gated.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
