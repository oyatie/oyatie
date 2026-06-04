# ADR-0516: GitHub Actions Interim Lane Unlocker

Status: Accepted for temporary bridge use on 2026-06-03; amended 2026-06-04 as **shadow compatibility only** for active CI authority. ADR-0513 owns the Rust/Prow/Kubernetes-native `oya-ci-required` direction. This ADR remains useful for GitHub PR/publication adapter evidence and legacy GitHub Actions compatibility, but it is not P0.0 green and is not durable CI/CD authority.

## Context

We need product, infra, and cloud lanes to move concurrently while native SCM,
CI/CD, cloud workspace, and release-conveyor-like seams continue to mature. The current
sequential workflow slows product development when infra/cloud substrate work is
still in progress. GitHub/GitHub Actions was the available bridge to unlock dev
work immediately; after the 2026-06-04 amendment it remains PR/publication and
shadow evidence, while `oya-ci-required` is the desired CI authority context.

This is a temporary lane-unlocker: no retired external SCM/CI/CD substrates are
interim authorities for SCM, CI, or CD. Exact tombstones for those retired names live in `/specs/retired-external-substrate-registry.json` so active guidance can stay generic. GitHub/GitHub Actions is also not the
permanent destination. The permanent destination remains a cloud native,
Kubernetes-native, hyperscaler native Oyatie developer substrate.

## Decision

Use GitHub as temporary PR/publication surface and GitHub Actions as shadow
compatibility evidence for dev-lane unlock, while Buck2 remains the
build/test/check authority.
The trusted required context is `oya-ci-required`. The GitHub Actions aggregate `github-lane-unlocker-required` is retained only as shadow compatibility and evidence visibility; it cannot impersonate the Prow/Kubernetes-native oya-ci producer. The temporary workflow runs on GitHub-hosted `ubuntu-24.04-arm` runners to
match the current Buck2 `aarch64-unknown-linux-gnu` Rust toolchain, pins
`BUCK2_RELEASE=2026-06-01`, opts GitHub JavaScript actions into the Node 24
runtime with `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true`, uses
`actions/checkout@v6` (stable v6 major; latest verified release v6.0.3 on
2026-06-04), and runs
`scripts/ci/github-actions-lane-unlocker-bootstrap.sh` before any Buck2 fanout.
Node 26 is not used as the JavaScript action runtime because GitHub's
action-runtime migration target is Node 24; Node 26 remains available only for
explicit `actions/setup-node` application jobs, which this Buck2/Rust bridge
does not need. That bootstrap serializes rustup setup, pins Rust through
`rust-toolchain.toml`,
installs `llvm-tools-preview`, installs the Linux targets Buck2 probes, then
installs Buck2 if the runner image lacks it. GitHub-hosted and self-hosted
runners both fail closed on the same Rust/Buck2 authority instead of lazily
installing toolchains inside concurrent Buck2 actions.

The native SCM direction is a pure-Rust Sapling-compatible native SCM service/control-plane that adopts
best-of-existing hyperscaler patterns and is not a wholesale reimplementation of
Prow, Sapling, Piper, CitC, GitHub Actions, or any single upstream system. Prow,
Sapling, Piper, CitC, Buck2, Kubernetes, and GitHub Actions are reference inputs
for patterns such as stacked changes, cloud workspaces, merge pools, required
status rollups, affected builds, Kubernetes-native job execution, and source
based coverage.

Do not revive an `oya vcs` CLI. The SCM seam owns durable change graph,
workspace leases, stacked changes, virtual merge heads, semantic conflict
metadata, Git protocol, and GitHub publication adapters through service APIs and
adapters. The Prow/Kubernetes-native `oya-ci` seam consumes trusted SCM/controller
state and publishes `oya-ci-required`. The CD seam is release-conveyor-like:
progressive delivery, rollback, policy, and audit consume signed CI evidence and
release-ledger records rather than CLI state.

Rust coverage remains Buck2-driven LLVM source-based coverage. The CI bootstrap
preinstalls the Rust `llvm-tools-preview` component so `llvm-profdata` and
`llvm-cov` come from the pinned rustup sysroot. Tarpaulin is not
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

- GitHub/GitHub Actions unlocks dev concurrency through PR/publication and shadow
  compatibility evidence, but it is not permanent SCM, CI, or CD authority.
- Buck2 remains the only build/test/check authority for the bridge.
- The native path continues toward cloud native, Kubernetes-native, hyperscaler
  native SCM, Prow-shaped CI, release-conveyor-like CD, and cloud workspace seams.
- This ADR is not P0.0 green, not Phase 0 complete, and not proof that cloud-ci
  or oya-ci live authority is ready.
- Legacy bridge documents may retain historical commands, but their interim
  authority is superseded by this ADR until native cutover evidence lands.

## Verification

The local contract is enforced by:

- `buck2 build //:github-lane-unlocker-bridge-check`
- `buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check`
- `buck2 build //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check`

Live branch-protection mutation and live Kubernetes scale-down are intentionally
outside this ADR's local static evidence.


## Manual bridge avoidance

During the GitHub shadow bridge, target `dev` branch protection belongs to the trusted Prow/Kubernetes-native `oya-ci-required` producer. Live GitHub may still observe the automated `github-lane-unlocker-required` aggregate for compatibility, but agents must not post manual `oya-ci-required` success statuses and must not treat GitHub Actions shadow evidence as merge authority.
