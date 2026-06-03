# ADR-0516: GitHub Actions Interim Lane Unlocker

Status: Accepted for temporary bridge use on 2026-06-03; not P0.0 green.

## Context

We need product, infra, and cloud lanes to move concurrently while native SCM,
CI/CD, cloud workspace, and release-conveyor seams continue to mature. The current
sequential workflow slows product development when infra/cloud substrate work is
still in progress. The interim path must be GitHub/GitHub Actions because that is
the available bridge that can unlock dev work now.

This is a temporary lane-unlocker: no Jenkins, no Forgejo, and no ArgoCD are
interim authorities for SCM, CI, or CD. GitHub/GitHub Actions is also not the
permanent destination. The permanent destination remains a cloud native,
Kubernetes-native, hyperscaler native Oyatie developer substrate.

## Decision

Use GitHub as temporary SCM/merge surface and GitHub Actions as temporary CI/CD
bridge for dev-lane unlock, while Buck2 remains the build/test/check authority.
The required temporary context is `github-lane-unlocker-required`; the native
cutover context remains separate as `oya-ci-required` and cannot be impersonated
by the GitHub bridge. The temporary workflow pins `BUCK2_RELEASE=2026-06-01` and
installs Buck2 if the runner image lacks it, so GitHub-hosted and self-hosted
runners both fail closed on the same Buck2 authority.

The native SCM direction is a pure-Rust Sapling-compatible native SCM that adopts
best-of-existing hyperscaler patterns and is not a wholesale reimplementation of
Prow, Sapling, Piper, CitC, GitHub Actions, or any single upstream system. Prow,
Sapling, Piper, CitC, Buck2, Kubernetes, and GitHub Actions are reference inputs
for patterns such as stacked changes, cloud workspaces, merge pools, required
status rollups, affected builds, Kubernetes-native job execution, and source
based coverage.

Rust coverage remains Buck2-driven LLVM source-based coverage. Tarpaulin is not
the monorepo coverage authority. Cargo can remain advisory for local dual
Cargo+Buck2 mutation testing where Reindeer-like generation keeps Cargo metadata
and BUCK graphs aligned, but Cargo is not CI merge authority.

## Cloud-native fit

The bridge and the native destination must preserve loosely coupled microservices. Product, infra, cloud, SCM/CI/CD, and governance lanes can each
fan out into multiple worktrees/PRs as long as service contracts remain stable
and shared files have a single writer. This follows CNCF principles for systems
that are secure, resilient, manageable, sustainable, and observable, and the CNCF
Cloud Native Reference Architecture properties that systems are distributable, observable, portable, interoperable, and available.

Kubernetes operations must be controller-oriented. If pods need to be put down,
prefer scaling the owning controller to zero, rollout pause/drain, or another
explicit workload-scoped controller action; do not blindly delete pods.

## Auth and shared-substrate decoupling

Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now. This deliberately creates no shared contract or shared surface between the Cloud identity/resource substrate and the current Oyatie product auth/shared substrate while product, infra, and cloud lanes are being parallelized. The expected benefit is higher concurrency and conflict avoidance because each lane owns its own contract files, schemas, runtime surfaces, and tests.

Integration is postponed until the Cloud IdP and resource substrate stabilize. At that point we will rewrite and rewire Oyatie products to consume the Cloud IdP, then retire the product-local auth/shared substrate only after equivalence, migration, rollback, and evidence gates pass.

### Work-saving plan

This split is planned to save future work, not create permanent divergence. P00-A separates contracts now. P00-B records a thin compatibility inventory of equivalent identity concepts without sharing implementation. P01 stabilizes the Cloud IdP/resource substrate. P02 rewrites and rewires Oyatie products to consume the Cloud IdP through adapters and equivalence tests, then retires product-local auth/shared surfaces after rollback and audit evidence exists.

Anti-rework rule: name equivalent concepts early, but do not share code, schemas, or runtime surfaces until the Cloud IdP is stable enough to avoid repeated product rewrites.

## Consequences

- GitHub/GitHub Actions unlocks dev concurrency now, but it is not permanent SCM,
  CI, or CD authority.
- Buck2 remains the only build/test/check authority for the bridge.
- The native path continues toward cloud native, Kubernetes-native, hyperscaler
  native SCM/CI/CD and cloud workspace seams.
- This ADR is not P0.0 green, not Phase 0 complete, and not proof that cloud-ci
  or oya-ci live authority is ready.
- Legacy bridge documents may retain historical commands, but their interim
  authority is superseded by this ADR until native cutover evidence lands.

## Verification

The local contract is enforced by:

- `python3 scripts/ci/assert-github-lane-unlocker-bridge.py --json`
- `buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check`
- `buck2 build //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check`

Live branch-protection mutation and live Kubernetes scale-down are intentionally
outside this ADR's local static evidence.
