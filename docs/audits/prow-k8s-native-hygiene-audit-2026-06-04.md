---
doc_class: Audit
shape: lane-handoff
length_cap: 900
owner_team: platform-scm-ci-cd + council-architecture
status: Accepted
created_at: 2026-06-04
purpose: |
  P00 audit/backlog for moving parallel development to a Sapling-inspired native SCM
  service, Prow/Kubernetes-native oya-ci, release-conveyor-like CD, Buck2 authority,
  and GitHub PR/publication adapters without reviving retired CLI or substrate surfaces.
doc_status: published
---

# Prow/Kubernetes-native SCM/CI/CD Hygiene Audit — 2026-06-04

## Position

- **Agree with the direction:** the right fit is a Sapling-inspired, Rust-biased native SCM service/control plane feeding Prow/Kubernetes-native `oya-ci`, with Buck2 as build/test/check authority and GitHub only as PR/publication adapter.
- **Do not revive CLI authority:** `oya vcs`, `oya git`, `oya gate`, and `oya verify` must not return as sanctioned CI/merge/VCS command surfaces. Reusable logic should survive as Rust libraries, Buck2 targets, and Prow jobs.
- **CD shape:** use a release-conveyor-like native CD seam: signed build/evidence/release ledger -> progressive delivery -> rollback/policy/audit. GitHub Actions CD remains dry-run/shadow evidence only.

## Anti-pattern guardrails

- **No candidate-owned merge truth:** PR bytes, shell scripts, or GitHub Actions jobs must not mint `oya-ci-required`; that context belongs to the trusted Prow/Kubernetes controller path.
- **No shared mutable bottlenecks:** root docs, branch-protection files, and workflow manifests stay pointer-thin or generated from lane-owned shards.
- **No retired substrate bridges:** retired external SCM/CI/CD names remain historical/tombstone context only; active plans use native SCM, Prow/Kubernetes-native CI, and release-conveyor seams.
- **No blind Kubernetes mutation:** workload shutdown means controller-owned scale/pause/drain with rollback evidence, not ad-hoc pod deletion.
- **No ad-hoc automation sprawl:** new gate authority is Rust + Buck2 first; Python/shell remains bootstrap-only with deletion criteria.

## Fit map

| Layer | Owns | Consumes / publishes | Conflict-avoidance effect |
|---|---|---|---|
| Native SCM service | worktree leases, stacked changes, semantic conflict metadata, virtual merge heads, Git/GitHub publication adapters | exposes trusted candidate/base snapshots to CI | agents work in disjoint lanes without shared branch/file churn |
| oya-ci | ProwJob-style Buck2 workloads, status reporting, Tide-like merge pools, trusted/untrusted job split | consumes SCM/controller state; publishes `oya-ci-required` | one required context; candidate PR bytes cannot weaken gates |
| Release-conveyor-like CD | release ledger, progressive rollout, rollback, policy, audit | consumes signed CI/build evidence | product/infra/cloud deployment lanes stay decoupled from CI internals |
| GitHub adapter | PRs, public/private mirrors, checks/status display | mirrors selected native SCM state | unlocks collaboration without making GitHub durable SCM/CI authority |

## Prow capability mapping authority

Use [`/specs/oya-ci-prow-capability-parity.json`](../../specs/oya-ci-prow-capability-parity.json) as the feature/parity map. Required target areas include hook/plugin routing, ProwJob config, presubmit/postsubmit/periodic/batch jobs, plank/controller-manager, crier, deck, tide, sinker, horologium, branch protection, pod utilities/artifacts, trusted/untrusted job separation, config validation, retest/lgtm, status reconciliation, and metrics.

Official/source references recorded in repo specs include:

- https://docs.prow.k8s.io/docs/
- https://docs.prow.k8s.io/docs/overview/architecture/
- https://docs.prow.k8s.io/docs/components/core/tide/
- https://docs.prow.k8s.io/docs/jobs/
- https://architecture.cncf.io/
- https://sapling-scm.com/docs/introduction/
- https://sapling-scm.com/docs/scale/overview/

## Backlog before broad parallel fan-out

1. **P00 native SCM/CI/CD fit propagation** — keep root docs, root hub, masterplan, canonical primitives, ADR-0513, ADR-0516, and repo-hygiene automation aligned on native SCM service + `oya-ci-required` + release-conveyor-like CD.
2. **P00 active exact-name guard** — keep active root/spec/ADR guidance free of retired substrate names; repo-wide residue remains large in historical ADRs, generated evidence, and retired code, so archive/delete tranches must be isolated PRs.
3. **P00 Prow job registry generation** — create Rust/Buck2 generator for desired ProwJob graph so vertical lanes edit disjoint registry shards instead of one shared workflow.
4. **P00 retire CLI authority** — convert remaining retired CLI authority rows to Rust libraries, Buck2 targets, and Prow jobs; keep any CLI invocations as historical/local migration inputs only.
5. **P00 TypeScript/pnpm retirement review** — audit stale frontend surfaces against current Rust/Leptos/product decisions before deletion; likely candidates include app-shell and workspace-studio remnants plus pnpm quality rows.
6. **P00 Python/shell migration** — rewrite active Python/shell gate/tool surfaces in Rust with Buck2 targets. Keep only one-time bootstrap or host-prelude glue with deletion criteria.
7. **P00 shared-surface reduction** — keep root docs/workflows pointer-thin; move lane detail into registry-owned shards and generated consolidation outputs.
8. **P01 stale documentation/archive pass** — use Rust stale-doc inventory for update/archive/delete PRs; do not bulk-delete historical ADR provenance.
9. **P01 security hardening lanes** — implement the validated zero-trust, ABAC, workload identity, runtime isolation, CI secret isolation, default-deny network, audit, honeytoken, and multi-account guardrail backlog from `/specs/repo-hygiene-automation.json`.

Fresh residue scan on 2026-06-04 found 11,038 repo-wide retired exact-name references outside `.git/`, `buck-out/`, and `node_modules/`; this is a cleanup backlog measurement, not a completion claim. The active guidance guard is intentionally narrower and enforced by `//:repo-hygiene-automation-check`.

## Safe PR order

1. Authority propagation and check updates.
2. Prow job registry/generator seed.
3. CLI-governance authority retirement tranche.
4. TypeScript/pnpm retirement inventory and deletion PRs.
5. Python/shell-to-Rust/Buck2 migration tranches.
6. Doc catalog/index/root-hub stale-pointer repair.

## Non-claims

This audit does not claim full Prow parity, live branch-protection mutation, live Kubernetes workload mutation, full native SCM implementation, or production CD readiness.
