---
id: ADR-0359
status: Superseded
planning_impact: true
date: 2026-05-25
owners:
  - council-architecture
supersedes: []
superseded_by: [ADR-0515]
amends:
  - ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0359: cloud-ci completely replaces GitHub Actions as the CI orchestrator (superseded by ADR-0515)

## Status

Superseded by ADR-0515 — 2026-06-06 (the anti-GHA-SPOF verdict is retained in ADR-0515).

## Date

2026-05-25

## Context

ADR-0349 established cloud-ci + ArgoCD as the canonical self-hostable CI/CD substrate but scoped cloud-ci to *augment* GitHub Actions, retaining GitHub Actions as the primary CI for the hosted-on-GitHub PR review surface. Operating experience invalidates the augment stance:

- **Hard dependency on a metered third party.** PR #180's entire check matrix (37 jobs) failed in 2-3 seconds with the annotation "the job was not started because an Actions budget is preventing further use." A billing/quota limit on the GitHub account blocked the *entire* merge-gate surface — a single point of failure outside our control.
- **Multi-context reality (ADR-0215/0164).** Air-gap, on-prem, colo, and oyatie-as-cloud-provider deployment contexts have no GitHub-hosted runners at all; an augment model leaves those contexts permanently uncovered.
- **Parity-drift maintenance tax.** Maintaining two CI surfaces (`.github/workflows/*` + `microservices/<ms>/ci/` cloud-ci pipelines) in lockstep is ongoing overhead with no benefit once cloud-ci is sufficient on its own.

Research grounding (2026-05): GitHub branch protection can require any CI provider's reported status check, not only GitHub Actions jobs; cloud-ci reports PR commit statuses to GitHub via a GitHub App + pipeline integration; cloud-ci best practice is declarative pipelines + config-as-code + ephemeral Kubernetes agents + strict RBAC/credentials — all already specified in `specs/ci-farm-substrate-canonical.json`. The local `./bin/oya verify --ci-required` mirror (ADR-0346) already provides green-parity evidence independent of any hosted runner, so verification is never blocked by an Actions budget.

## Decision

1. **cloud-ci is the sole CI orchestrator; GitHub Actions is completely removed.** This amends ADR-0349: replace "cloud-ci augments GitHub Actions / GitHub Actions retained as primary" with "cloud-ci replaces GitHub Actions." ArgoCD remains the deploy substrate.
2. **PR gating moves to cloud-ci-reported statuses.** A cloud-ci GitHub App publishes commit statuses; GitHub branch-protection required checks switch from `.github/workflows` job names to cloud-ci-reported status contexts. The `infra/branch-protection/dev.json` + `.github/branch-protection.yaml` required-check sets are rewritten accordingly.
3. **`.github/workflows/*` (36 workflows) are retired** once the equivalent cloud-ci pipeline stages exist (the per-lane mapping already enumerated by the parity contract). The cloud-ci canonical gate lane replaces the parity lane (refuses any new `.github/workflows` CI definition).
4. **Migration is sequenced and reversible** (see masterplan `ideal_production_roadmap.P-TOOLCHAIN`): stand up cloud-ci controller + config-as-code + ephemeral K8s agents; port the lanes; wire the GitHub App + switch required checks; retire `.github/workflows`; then layer the remote cache (sccache->SeaweedFS) and Bazel affected-targets (ADR-0358). The local `oya verify` mirror is the gate during the transition.

## Consequences

Positive: removes the GitHub-Actions-budget single point of failure (the PR #180 block); one self-hostable CI surface covering every deployment context including air-gap; retires the parity-drift maintenance tax. Negative/cost: a cloud-ci controller + ephemeral-agent cluster must be operated (config-as-code + RBAC + GitHub App credentials in OpenBao); the migration period must keep the merge-gate trustworthy (local `oya verify` mirror covers it); contributors lose the zero-setup hosted-runner convenience. Neutral: ArgoCD, cosign image promotion (ADR-0181), and the deployment model are unchanged; this ADR is doctrine + sequencing, not the migration execution itself.
