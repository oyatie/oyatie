---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-intelligence
microservice: intelligence
status: Accepted
sales_segment: shared-substrate
tier: internal
related_adrs: [ADR-0215, ADR-0219, ADR-0220, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
date: 2026-05-18
owner_team: axis-intelligence
doc_status: published
---

# PRD-intelligence: Consumer Intelligence Substrate

## Purpose

The `intelligence` microservice provides tenant-scoped assist-draft and context-aware retrieval surfaces for consumer and builder workflows. It is separate from internal Foundry automation and only returns advisory drafts or citations.

## Scope

In:
- Assist-draft suggestions for deterministic builders.
- Context-aware retrieval with consent, budget, and tenant policy checks.
- Policy refusal events and audit-chain evidence.

Out:
- Direct mutation of tenant configuration by AI output.
- Internal Foundry agent orchestration.
- Claims of model quality, production scale, or operational maturity.

## Acceptance

- Every request carries principal, context, and consent identifiers.
- Draft output is advisory and importable into deterministic builders.
- Refusals are explicit and auditable.
- Runtime quality and SLO achievement remain outside this design claim.

## Non-Functional Requirements

### DR posture (ADR-0343)

- Service target: RTO p99 ≤ 300s and RPO p99 ≤ 60s, matching `manifest.json` `rpo_rto` (`tier = hot`, `authority_adr = ADR-0152`).
- Compliance floors considered: HIPAA-2024 RTO 3600s/RPO 300s/multi-region true, EU-AI-ACT-2024-HIGH-RISK RTO 1800s/RPO 300s/multi-region true for Annex III refusal surfaces, and SOC2-T2 RTO 14400s/RPO 900s. The effective target remains 300s/60s because the manifest is stricter.
- Failover runbook reference: `runbooks/provider-outage-openai.md`, `runbooks/provider-rate-limit-saturation.md`, `runbooks/model-router-stall-investigation.md`, and `runbooks/byok-rotation-tenant-cascade.md`.
- Multi-region posture: active-active dispatch ingress with tenant home-cell context and BYOK material pinned to the regulated cell; model/provider failover must keep consent, residency, and refusal evidence intact.
- Tenant-visible behavior: builders receive an explicit refusal or advisory-degraded response during a provider outage instead of silent mutation, missing audit rows, or cross-region credential use.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.5 vCPU/1GiB dispatch capacity per 10 concurrent advisory streams, 1GiB retrieval/eval artifact storage where consent allows persistence, and four provider connections per tenant policy bundle.
- Scaling dimension: `dispatch_request`, `token_stream`, `retrieval_context_bytes`, `tool_call_count`, and `refusal_case` independently size router, retrieval, and audit-tap queues.
- Cell placement class: default Tier-1 shared cell and regulated Tier-3 cell exactly match `manifest.json` `cell_eligibility`; ADR-0338 runtime is Kata/Cloud-Hypervisor for regulated BYOK and prompt-bearing paths.
- Autoscaling boundaries: minimum two dispatch replicas per active cell, one audit-tap worker per home cell, maximum 20 dispatch workers and 8 retrieval workers per tenant before budget and abuse-defence admission gates shed work.
- Tenant load profile served: short advisory drafts, retrieval-heavy builder context, and refusal bursts from abuse-defence incidents remain isolated from internal Foundry automation.

### Sustainability + cost attribution (ADR-0344)

- Every `DispatchRequestReceived`, `DispatchCompleted`, `RefusalDecisionEmitted`, `EuAiActAnnexIiiRefusalEmitted`, BYOK rotation, prompt-injection, and audit-tap failure row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Carbon-aware provider routing: yes for offline evals, golden-set scoring, batch embeddings, and non-regulated advisory drafts; no for EU-AI-Act-Annex-III refusal paths, HIPAA emergency support, PCI-realtime-fraud, or abuse-defence bypass paths.
- Tenant cost transparency surface: the intelligence attribution ledger and FinOps portal show provider/model, cell, compliance_pack, token/request class, and refusal cost per tenant.
- Regulatory driver: CSRD, SB-253, and SEC climate-disclosure exports need per-provider emissions evidence aligned to the same audit rows that prove refusal and consent behavior for high-risk AI governance.

### API versioning posture (ADR-0342)

- Public API version model: tenant-facing REST, AsyncAPI, and proto contracts use the YYYY-MM-DD carrier triplet: `Oyatie-API-Version: <date>`, `/api/intelligence/<date>/...`, and proto3 `api_version` fields.
- SDK semver model: generated SDKs publish `major.minor.patch`; semver major only follows removal or breaking change of a supported date-versioned contract.
- Support window: last N=3 public contract dates are supported for at least 180 days.
- Per-tenant pinning: yes for tenant/admin APIs and advisory clients; internal provider-dispatch mesh is not tenant-pinned.
- Internal-mesh exemption: yes; internal Foundry/provider direct gRPC keeps ADR-0145 behavior while boundary carriers remain date-versioned.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `intelligence` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `intelligence` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_request` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
