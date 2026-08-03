---
doc_class: Reference
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0245, ADR-0292, ADR-0297]
companion_docs:
  - specs/microservices/social.json
  - oya/social/manifest.json
  - oya/social/contracts/openapi/social.yaml
inbound_citations:
  - docs/README.md
---

# Social µservice — README

## What this µservice does

Social is oyatie's broadcast-shape social product. Distinct from `community` (forum-shape). Hyperscaler precedent: Twitter/X + Facebook + Instagram + LinkedIn + Threads + Bluesky + Mastodon. Federation via ActivityPub.

## Quick links

- Source-authority PRD/source map: `../../specs/microservices/social.json` (Draft retained; Plan/Spec and RED fixture/contract planning only)
- Inventory/provenance manifest: `manifest.json` (not runtime/product-readiness authority)
- Legacy path note: `microservices/social/manifest.json` is absent historical provenance and must not be restored; use `oya/social/**` plus the Draft PRD source map.
- Product requirements: legacy `PRD.md` references must be reconciled through the Draft PRD/source map before RED/Build fanout
- Architecture: legacy `ARCHITECTURE.md` references must be reconciled through current `oya/social/**` planning paths before RED/Build fanout
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

- Native + web clients: REST + WebSocket over HTTP/3.
- Federation: ActivityPub inbound/outbound (opt-in per-tenant + per-post).
- Backups + export: ActivityPub JSON archive per ADR-0276.

## Critical operational notes

- Social is the **highest-targeted surface** for bots / scrapers / spoofers / sock-puppets in the platform. Read `ARCHITECTURE.md §abuse-defence` end-to-end before any change.
- Minor-protection per ADR-0292 is **non-negotiable**; COPPA <13 refusal and KOSA 14-17 strict-defaults are hard-coded into `policy/minor-protection.cedar`.
- UX-floor is critical: friction kills adoption on social surfaces. Default path is friction-free; challenges only on confirmed-suspicion.

## Status

Source-map locked by `t_df502234` with Draft PRD status retained. This README is inventory/provenance for the `oya/social/**` service root; it does not by itself assert product GA, runtime readiness, cell readiness, or implementation authority without Plan/Spec/RED/Build/Review evidence.

## Tenant Class Model

Social follows ADR-0330: tenant eligibility is expressed as `tenant_class`
(`demo_trial` or `paid`) plus paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Customer-facing capability
ladders are retired; product behavior is gated by tenant class, compliance
packs, cell topology, and abuse controls instead of commercial tiers.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
