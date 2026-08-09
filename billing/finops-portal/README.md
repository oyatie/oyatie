# finops-portal µservice

Per-tenant FinOps presentation + chargeback portal. Per ADR-0199 §In-house
roadmap Phase 2 target.

**Status**: full-pack-ready (2026-05-18). All planning artifacts authored
to hyperscaler bar; ready for the implementation phase to build the crates.

## Scope

The `finops-portal` µservice sits **on top of** the FinOps data plane
(OpenCost + Mimir + FOCUS 1.3 exports per ADR-0199) and provides the
differentiated tenant-billing UX layer:

- Tenant-facing invoice presentation.
- Drill-down cost dashboards by `cost-center`, `workload-class`, time.
- Anomaly explanation (why did spend spike?).
- Cost-allocation policy editor (who pays for shared resources).
- FOCUS 1.3 export downloads.
- Regulator-evidence quarterly emit (per ADR-0174 + ADR-0162).
- Committed-use discount UX (per FOCUS 1.3 Contract Commitment dataset).
- Credit-ledger UX (customer-success negotiated credits).

## Tenant Class And Billing Components

Per ADR-0330, `finops-portal` uses `tenant_class` instead of customer
capability tiers. `demo_trial` tenants run inside capped cost and usage
envelopes, with OCI Always Free as the default free-profile target where
applicable. `paid` tenants receive the same capability surface and compose
commercial terms through `billing_components`: `revenue_share`, `per_seat`,
and `per_usage`.

The portal must not expose retired customer-tier columns, filters, or feature
tables. Cost visibility, forecast cadence, export limits, and chargeback
workflows are modeled as tenant_class caps, billing_component contract terms,
compliance_pack activation, or cell_topology constraints.

## What this µservice is NOT

- NOT the cost-aggregation engine — that's OpenCost (ADR-0199 D-3).
- NOT the cost-anomaly detector — that's Prometheus rules (ADR-0199 D-5).
- NOT the chargeback-formula owner — that's ADR-0174.

`finops-portal` is the UX + workflow layer on the in-house ladder; the
substrate underneath stays OSS-canonical.

## Layout (per ADR-0131 flat layout)

- `manifest.json` — µservice manifest (BC list + LTS pins + dependencies).
- `PRD.md` — product requirements doc.
- `PHASE-01-tenant-billing-presentation.md` — Phase P01 milestone doc.
- `implementation-plans/IP-001..IP-015.md` — 15 slice-sized IPs.
- `slos/*.openslo.yaml` — 9 OpenSLO declarations.
- `runbooks/*.md` — 8 operational runbooks.
- `dashboards/*.grafana.json` — 3 Grafana dashboards + spec.
- `catalog/bnf-v4.1.yaml` — BNF v4.1 named entities (21 crate names).
- `policy/cedar/*.cedar` — 4 Cedar policies + schema.
- `capabilities/*.capability.yaml` — 3 capability declarations (EU AI Act).
- `decisions/ADR-finops-portal-*.md` — 7 service-scoped ADRs.
- `iac/helm/finops-portal/` — Helm chart + 3 per-pack overlays + templates.
- `contracts/` — OpenAPI 3.1 + AsyncAPI 3.0 + proto3 contracts.
- `scorecards/adr-*.md` — 4 framework scorecards.
- `threat-model.md`, `dpia.md`, `compliance-matrix.md`,
  `cost-model.md`, `multi-region-strategy.md`,
  `incident-playbook.md`, `capacity-model.md`,
  `failure-modes.md`, `sdk-reference.md`,
  `competitor-parity.md`, `backfill-plan.md` —
  dual-purpose product-engineering docs.

## Promotion gate

This µservice enters dev → staging after:

- IP-001 through IP-015 are implemented (crates compile + tests green).
- All 4 Cedar policies pass `cedar validate` + unit tests.
- All 9 SLOs report green for 24 h on the metric series.
- The 4 framework scorecards remain green.
- `oya gate self-slo-promotion-gated` passes per ADR-0130.
- Multispectrum review v2.3.0 lane green (11+ facets).

See `PHASE-01-tenant-billing-presentation.md` for the P01 acceptance
criteria + `evidence/finops-portal-full-pack-expansion-report.json` for
the ledger of artifacts produced.

## Evidence

The expansion ledger is at
`evidence/finops-portal-full-pack-expansion-report.json`. It cross-
references every entry in `evidence/storage-batch-followup-scope.json`
with the produced artifact path.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
