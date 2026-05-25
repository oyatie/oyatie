---
id: ADR-0359
status: Proposed
planning_impact: true
date: 2026-05-25
owners:
  - council-architecture
supersedes: []
amends:
  - ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md
---
# ADR-0359: Jenkins completely replaces GitHub Actions as the CI orchestrator

## Status

Proposed — 2026-05-25.

## Date

2026-05-25

## Context

ADR-0349 established Jenkins (LTS) + ArgoCD as the canonical self-hostable CI/CD substrate but scoped Jenkins to *augment* GitHub Actions, retaining GitHub Actions as the primary CI for the hosted-on-GitHub PR review surface. Operating experience invalidates the augment stance:

- **Hard dependency on a metered third party.** PR #180's entire check matrix (37 jobs) failed in 2-3 seconds with the annotation "the job was not started because an Actions budget is preventing further use." A billing/quota limit on the GitHub account blocked the *entire* merge-gate surface — a single point of failure outside our control.
- **Multi-context reality (ADR-0215/0164).** Air-gap, on-prem, colo, and oyatie-as-cloud-provider deployment contexts have no GitHub-hosted runners at all; an augment model leaves those contexts permanently uncovered.
- **Parity-drift maintenance tax.** Maintaining two CI surfaces (`.github/workflows/*` + `microservices/<ms>/ci/Jenkinsfile`) in lockstep (the `oya-governance-jenkins-github-actions-parity` lane) is ongoing overhead with no benefit once Jenkins is sufficient on its own.

Research grounding (2026-05): GitHub branch protection can require any CI provider's reported status check, not only GitHub Actions jobs; Jenkins reports PR commit statuses to GitHub via a Jenkins GitHub App + multi-branch pipelines; Jenkins best practice is declarative pipelines + shared library + JCasC (config-as-code) + ephemeral Kubernetes agents + strict RBAC/credentials — all already specified in `specs/ci-farm-substrate-canonical.json`. The local `./bin/oya verify --ci-required` mirror (ADR-0346) already provides green-parity evidence independent of any hosted runner, so verification is never blocked by an Actions budget.

## Decision

1. **Jenkins is the sole CI orchestrator; GitHub Actions is completely removed.** This amends ADR-0349: replace "Jenkins augments GitHub Actions / GitHub Actions retained as primary" with "Jenkins replaces GitHub Actions." ArgoCD remains the deploy substrate.
2. **PR gating moves to Jenkins-reported statuses.** A Jenkins GitHub App publishes commit statuses; GitHub branch-protection required checks switch from `.github/workflows` job names to Jenkins-reported status contexts. The `infra/branch-protection/dev.json` + `.github/branch-protection.yaml` required-check sets are rewritten accordingly.
3. **`.github/workflows/*` (36 workflows) are retired** once the equivalent Jenkins pipeline stages exist (the per-lane mapping already enumerated by the parity contract). The `oya-governance-jenkins-github-actions-parity` lane is replaced by `oya-governance-jenkins-canonical-no-gha-residue` (refuses any new `.github/workflows` CI definition).
4. **Migration is sequenced and reversible** (see masterplan `ideal_production_roadmap.P-TOOLCHAIN`): stand up Jenkins controller + JCasC + ephemeral K8s agents; port the lanes; wire the GitHub App + switch required checks; retire `.github/workflows`; then layer the remote cache (sccache->SeaweedFS) and Bazel affected-targets (ADR-0358). The local `oya verify` mirror is the gate during the transition.

## Consequences

Positive: removes the GitHub-Actions-budget single point of failure (the PR #180 block); one self-hostable CI surface covering every deployment context including air-gap; retires the parity-drift maintenance tax. Negative/cost: a Jenkins controller + ephemeral-agent cluster must be operated (JCasC + RBAC + GitHub App credentials in OpenBao); the migration period must keep the merge-gate trustworthy (local `oya verify` mirror covers it); contributors lose the zero-setup hosted-runner convenience. Neutral: ArgoCD, cosign image promotion (ADR-0181), and the deployment model are unchanged; this ADR is doctrine + sequencing, not the migration execution itself.
