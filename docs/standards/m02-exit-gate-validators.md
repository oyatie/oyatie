---
purpose: "M02b/P22 exit-gate — historical 14 quality/scalability lane inventory, reframed for Buck2/Prow evidence"
doc_status: published
change_id: claude-m02b-p22-doc-coverage-mv-1779009579
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
---

# M02b/P22 Exit-Gate Validators

> **Status (2026-06-05):** this document is compatibility provenance for the
> 14 quality/scalability validator lanes. Active merge evidence is produced by
> Rust kernels through Buck2 targets and Prow/Kubernetes-native `oya-ci-required`
> jobs. Retired local CLI bindings are not CI, merge, or protected-branch
> authority.

## What "wired" means now

A lane is **wired** when:

1. the lane has a real Rust kernel or app owned under `libs/oya-check-*`,
   `libs/oya-governance-*`, or a successor Rust/Buck2-owned package;
2. a Buck2 target, quality-lane registry entry, or ProwJob shard can execute the
   kernel without a retired local CLI dispatcher;
3. at least one Rust unit/integration/golden-fixture test proves the validator
   rejects a known-bad input and accepts the intended good path; and
4. PR evidence cites the Buck2 target output and the trusted
   Prow/Kubernetes-native `oya-ci-required` context once native CI owns the lane.

A lane is **not wired** when only prose, historical CLI dispatch, or an
unregistered local script demonstrates the intended behavior.

## The 14 lanes

The "14 quality/scalability lanes" tracked for M02b/P22 exit-gate provenance are
retained below so existing PR references remain understandable. The active check
surface is the Rust kernel plus Buck2/Prow evidence, not a local CLI command.

| # | Lane slug | Kernel capability | Kernel crate | Current evidence posture |
|---|---|---|---|---|
| 1 | `quality-statelessness` | `statelessness` | `oya-check-statelessness` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 2 | `quality-shardability` | `shardability` | `oya-check-shardability` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 3 | `quality-perf-budget` | `perf-budget` | `oya-check-perf-budget` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 4 | `quality-benchmark` | `benchmark` | `oya-check-benchmark` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 5 | `lean-a-active-artifact-contract` | `active-artifact-contract` | `oya-check-active-artifact-contract` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 6 | `lean-a-cedar-fragment-coverage` | `cedar-fragment-coverage` | `oya-check-cedar-fragment-coverage` | Historical partial wiring; follow-up must add Buck2/Prow target evidence and tests. |
| 7 | `lean-a-openapi-rest-route-parity` | `openapi-rest-route-parity` | `oya-check-openapi-rest-route-parity` | Buck2 target exists in `registry/quality/lanes.yaml`; verify with current Buck2/Prow output before promotion reliance. |
| 8 | `foundation-bypass` | `foundation-bypass` | `oya-check-foundation-bypass` | Historical test claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 9 | `audit-chain-replay` | `audit-chain-replay` | `oya-check-audit-chain-replay` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 10 | `foundry-capability-schema` | `foundry-capability-schema` | `oya-check-foundry-capability-schema` | Historical partial wiring; follow-up must add Buck2/Prow target evidence and tests. |
| 11 | `foundry-eval` | `foundry-eval` | `oya-check-foundry-eval` | Historical partial wiring; follow-up must add Buck2/Prow target evidence and tests. |
| 12 | `cross-tenant-access-fuzz` | `cross-tenant-access-fuzz` | `oya-check-cross-tenant-access-fuzz` | Historical partial wiring; follow-up must add Buck2/Prow target evidence and tests. |
| 13 | `lean-a4-semver` | `api-semver` | `oya-check-api-semver` | Historical wired+tested claim; must be re-proven by Buck2/Prow evidence before promotion reliance. |
| 14 | `lean-a5-documentation` | `documentation-system` | `oya-check-documentation-system` | Documentation-system kernel exists; verify with current Buck2/Prow output before promotion reliance. |

## Lanes outside this historical table

Do not add Cargo, pnpm, Node, or retired local CLI rows to this table. If a lane
needs durable evidence, register it as a Rust/Buck2 target and, when native CI is
ready, a ProwJob shard that reports into `oya-ci-required`. Local developer tools
may remain advisory only when explicitly documented by the relevant dual-build or
compatibility policy.

## BLOCKER workflow flip policy

The old BLOCKER workflow flip policy is retired as merge authority. Any future
blocking posture must be expressed as:

1. lane-owned Rust kernel or app;
2. Buck2 target and registry entry;
3. Prow/Kubernetes-native job or generated ProwJob shard;
4. protected-branch evidence through `oya-ci-required`; and
5. reviewer approval plus PR evidence.

## Sources

- `docs/standards/ci-lanes.md` — lane catalog provenance.
- `registry/quality/lanes.yaml` — current quality-lane registry entries.
- `libs/oya-check-*` and `libs/oya-governance-*` — Rust kernel/app ownership.
- `specs/repo-hygiene-automation.json` — active repo-hygiene and retired-tooling guardrails.
- `docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md` — Prow/Kubernetes-native CI direction.
