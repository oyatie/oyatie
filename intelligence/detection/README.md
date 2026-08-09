---
doc_class: Operational-Doc
shape: Reference
status: Proposed
date: 2026-05-21
owner_team: axis-detection
microservice: detection
related_adrs:
  - ADR-0307-detection-substrate-streaming-batch
  - ADR-0308-ml-model-lifecycle-ai-act-compliance
  - ADR-0309-detection-fairness-audit-civil-rights
  - ADR-0310-investigation-case-management
  - ADR-0263-observability-emission-contract
  - ADR-0105-13-layer-enum-and-check-family-patterns
  - ADR-0131-per-microservice-flat-layout
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0703-cas-cache-live-apex.md
planned_enforcement_ref: oya-governance-detection-baseline
bnf_version: v4.1
layer_enum: layer_5_shared_substrate
---

# Detection Microservice README

## Purpose
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Scope
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Inputs
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Procedure
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Metrics
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Failure modes
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Rollback
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## References
- This section binds Detection Microservice README to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Tenant Class Model
- Detection follows ADR-0330.
- `tenant_class` is `demo_trial` or `paid`.
- Paid commercial shape is carried by `billing_components`: `revenue_share`, `per_seat`, and `per_usage`.
- Model serving, graph investigation, fairness audit, and replay availability are not customer ladder features; operational differences belong to `cell_topology`, `compliance_pack`, and explicit demo caps.
- Canonical model: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.

Detection Microservice README buildability note 1: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 2: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 3: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 4: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 5: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 6: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 7: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 8: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Microservice README buildability note 9: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
