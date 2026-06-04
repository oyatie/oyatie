# GitHub Actions Lane Unlocker Procedure

This procedure implements the ADR-0516 GitHub/GitHub Actions temporary
lane-unlocker. It is not P0.0 green and it does not make GitHub permanent.

## Operating boundary

Exact tombstones for retired external substrate names live in `/specs/retired-external-substrate-registry.json` so active guidance can stay generic.


- Use GitHub/GitHub Actions as the temporary lane-unlocker for dev.
- Use no retired external SCM/CI/CD substrates as interim SCM/CI/CD authorities.
- Keep Buck2 as build/test/check authority.
- Keep native cutover separate: cloud native, Kubernetes-native, hyperscaler native SCM/CI/CD and cloud workspace seams remain the destination.
- Use the pure-Rust Sapling-compatible native SCM direction as the durable SCM
  seam, adopting best-of-existing hyperscaler patterns and not a wholesale reimplementation of any single upstream system.

## Parallel lane model

1. Split product, infra, cloud, SCM/CI/CD, and governance into separate worktree
   lanes whenever their write sets are disjoint.
2. Product lanes consume stable APIs/events/resources and must not depend on
   GitHub, GitHub Actions, cloud workspace internals, or native CI internals.
3. Infra/cloud lanes publish stable resource contracts and Kubernetes-native
   execution seams without blocking product feature PRs.
4. Shared files (`BUCK`, root hub, master plan, evidence chain) are single-writer
   integration points; queue those edits or use small handoff PRs.

## CI shape

The workflow `.github/workflows/github-lane-unlocker-ci-cd.yml` fans out with a
matrix and `max-parallel`, cancels stale runs with `concurrency`, and aggregates a
single required check named `github-lane-unlocker-required`. Every job runs on `ubuntu-24.04-arm` because the current Buck2 Rust toolchain
defaults to `aarch64-unknown-linux-gnu`; this keeps the temporary bridge native
arm64 instead of cross-linking on x64. Every job opts GitHub JavaScript actions into the Node 24 runtime with
`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` and uses `actions/checkout@v6`
(the stable v6 major; latest verified release v6.0.3 on 2026-06-04). Node 26
is not used as the JavaScript action runtime because GitHub's migration target
for actions is Node 24; Node 26 remains available only for explicit
`actions/setup-node` application jobs, which this Buck2/Rust bridge does not
need. Every job first runs
`scripts/ci/github-actions-lane-unlocker-bootstrap.sh`, which serializes rustup
setup before Buck2 fanout, pins Rust through `rust-toolchain.toml`, installs
`llvm-tools-preview`, installs the Linux targets Buck2 probes, then installs
Buck2 from `BUCK2_RELEASE=2026-06-01` when the runner image does not already
provide `buck2`. The lane fails closed if rustup, LLVM tools, Buck2, or
`buck2 --version` fail.

Required local commands:

```bash
python3 scripts/ci/assert-github-lane-unlocker-bridge.py --json
buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check
buck2 build //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check
infra/ci/buck2-affected-gate.sh origin/dev HEAD
```

Rust coverage remains LLVM source-based coverage through Buck2 targets. The
serialized bootstrap installs `llvm-tools-preview`, so `llvm-profdata` and
`llvm-cov` are resolved from the pinned rustup sysroot instead of ambient runner
state. Tarpaulin is not required CI evidence. Cargo mutation testing is advisory
only for dual Cargo+Buck2 setups.

## Kubernetes and microservices

Follow CNCF principles: loosely coupled microservices are the default and
service boundaries are explicit. The target architecture is secure, resilient, manageable, sustainable, and observable. The CNCF Cloud Native Reference
Architecture lens adds distributable, observable, portable, interoperable, and available as durable properties.

To put pods down, scale the owning Deployment/StatefulSet/controller to zero or
use a workload-scoped drain/pause. Do not blindly delete pods; controllers can
recreate them and hide the real desired-state change.

## Auth/shared substrate split

Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now. During the lane-unlocker period there is no shared contract or shared surface between these two auth/shared domains. Cloud lanes own Cloud IdP, workspace identity, and resource-kernel identity contracts; product lanes own current Oyatie product auth/session/shared contracts. This should allow higher concurrency and conflict avoidance because product, infra, and cloud agents do not edit the same auth schema or runtime surface.

After the Cloud IdP stabilizes, create a deliberate migration lane to rewrite and rewire Oyatie products to consume the Cloud IdP. Do not silently merge the two surfaces before that migration evidence exists.

### Work-saving checklist

- Keep separate contract roots for Cloud auth/shared and Oyatie product auth/shared.
- Maintain a concept-mapping inventory, not a shared schema, during P00/P01.
- Add future Buck2 equivalence tests before rewiring products to Cloud IdP.
- Rewire through adapters/facades so product services do not absorb Cloud IdP internals.
- Retire product-local auth/shared only after rollback, audit, and migration evidence passes.


## Hygiene automation

The bridge runs repo hygiene as local/static evidence:

```bash
python3 scripts/ci/assert-repo-hygiene-automation.py --json
buck2 build //:repo-hygiene-automation-check
```

This covers git/worktree, branch/merge, repository publication, disk/workspace, Kubernetes workload, and documentation-sprawl hygiene. It inventories by default and does not delete tracked files, mutate live branch protection, or scale Kubernetes workloads.

## Cutover

Cut over from GitHub/GitHub Actions only after the native SCM/CI/CD/cloud
workspace/release conveyor path proves the required context and operational
contracts. Until then, this procedure is the temporary lane-unlocker and is not
P0.0 green.


## Manual bridge avoidance

During the temporary GitHub bridge, `dev` branch protection uses the automated `github-lane-unlocker-required` aggregate check. Agents must not post manual `oya-ci-required` success statuses to merge bridge PRs. `oya-ci-required` remains the native cutover context for the trusted cloud-ci/oya-ci producer only.
