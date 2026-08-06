---
id: ADR-0523
title: "Zero-shell posture + the closed irreducible-glue ledger (minimal not zero; pinned; reproducible); refines ADR-0515 D3 no-shell-bar-a-narrow-exception"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0515, ADR-0522]
amends: []
related: [ADR-0515, ADR-0516, ADR-0522, ADR-0524, ADR-0525, ADR-0526]
related_specs:
  - /specs/masterplan.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

# ADR-0523: Zero-shell posture + the closed irreducible-glue ledger

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Refines ADR-0515 D3 ("no shell bar a documented, justified, narrowly-scoped exception") by defining
precisely WHAT minimal residue is permitted and why. Detail under Component 1 of ADR-0516.

## Context

ADR-0515 D3 forbids shell "bar a documented, justified, narrowly-scoped exception" but does not
enumerate the exception set, so a zero-shell dogma could either delete a load-bearing pinned bootstrap
or, conversely, let shell sprawl back in under "as-needed." ADR-0522 makes every real-work shell target
a retirement target; this ADR closes the set of what may legitimately remain.

## Decision

The lifecycle target is ZERO shell/CLI glue that does real work — every `build.sh` / `cargo` / Makefile
orchestrator is a defect to retire into a buck2 target — EXCEPT a closed, authoritative
**IRREDUCIBLE-GLUE LEDGER** of six items that genuinely cannot be a pure in-graph buck2 action:

1. **Toolchain bootstrap** — the buck2 binary itself + the first rustc/QEMU/musl downloads (a build
   tool cannot build itself in-graph). Pin buck2 by release tag; move the CI `curl` into a
   sha256-pinned `download_file` in the toolchains cell; pin rustc via `rust-toolchain.toml`.
2. **The scm-facts emitter** — git is inherently ambient and CANNOT be a pure function of declared
   inputs inside a hermetic action; resolution = ONE emitter step at the graph edge whose committed,
   content-addressed output every in-graph action is pure over. (The rename + the `ScmFactsSource`
   trait is OWNED by ADR-0526 and not re-litigated here; the hermetic-boundary itself is ADR-0525.)
3. **CI checkout** — `actions/checkout fetch-depth:0` (adapter-level; required so the emitter can
   derive last-touch; shallow collapses ages → false-green).
4. **Hardware/endpoint CD bring-up** — `gen-media.sh`, CAPI `init.sh`, bare-metal `up.sh` ("boot media
   on a physical node" is inherently side-effecting / one-shot); keep as documented runbook steps with
   pinned `talosctl` / `clusterctl` / `tofu`.
5. **reindeer buckify** — the standard out-of-graph third-party BUCK generator; pin reindeer, fold the
   `*.patch` hand-edits into reindeer fixups or a checked-in `select()` overlay, wrap as
   `buck2 run //tools:buckify`.
6. **Agent-hook exec shims** — Claude/Codex project hook APIs execute configured commands, so a hook
   may retain a minimal shell shim only to locate and `exec` a repo-local Rust binary under
   `tools/hooks/bin/`, with no policy logic and fail-open warning behavior when the binary is absent.
   First authorized row: `tools/hooks/main-checkout-guard.sh` for FRIC-022 / FRIC-1781062867; the
   policy implementation is `tools/oya-checkout-guard-app`.

RULE: everything NOT in this ledger is a retirement target; any PR that touches a ledger item is a
one-way door requiring re-justification.

## Drivers

- The directive says "0 to minimal" — the ledger IS the minimal.
- The pre-mortem: a zero-shell dogma must not delete a pinned bootstrap or the `fetch-depth:0`
  checkout.

## Alternatives considered

- **Literal zero-shell** — rejected (breaks hermeticity by removing the pinned bootstrap and the
  `fetch-depth:0` checkout).
- **Unbounded shell-as-needed** — rejected (the sprawl this posture eliminates).

## Consequences

Named retirement set: the cargo CI lanes, the per-program `build.sh` × N, `run-qemu-*.sh`,
`conformance-probe.sh`, `diff-oracle*.sh`, the kernel verify-gate scripts, `build-carriers.sh`-as-
orchestrator, the tracked `out/*.elf` blobs, `.envrc` + `bin/oya` manual PATH.

CI-011 records the protected closeout inventory for this retirement set at
`evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json` and
`evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md`. Those artifacts are
point-in-time evidence for downstream retirement work, not new shell/CLI authority.

**KEY TENSION (Consensus Addendum):** reproducibility OUTRANKS shell-count — "minimal shell" may
legitimately mean adopting a minimal Nix flake for QEMU/talosctl if `download_file` pinning proves
non-reproducible across host classes, which the founder must accept as MORE external machinery, not
less (carried as OQ-10 in ADR-0521; ADR-0524 may be deferred on this basis). This ADR
complements/operationalizes ADR-0515's "no shell, declarative gitops" firewall posture by defining
precisely what minimal residue is permitted and why. door:one-way.

---

## Addendum 2026-06-23 — verified-dead `.sh` retirement set (deletion step 1)

**Founder-directed shell-retirement execution, step 1.** Determination `wsk1vi3j1` set-reconciled the
49 tracked `.sh` to exactly **28 DELETE-as-dead + 11 MIGRATE-to-non-CLI + 10 IRREDUCIBLE-GLUE** (the
10 = the ADR-0523 ledger of this document). The DELETE set is dead cruft — verified to have **no live
invoker** — so per the founder directive "don't migrate stale, deprecated, no longer valid shell
scripts," the correct outcome is deletion, not migration.

This PR retires the verified-dead subset. Each script was re-verified to have NO live invoker before
deletion: greps of the canonical CI (the GitHub-workflows gate-fleet + the check-substrates lanes), the
Claude and Codex project hook wiring files, the root Makefile and BUCK, and all surviving live scripts.
A script was deleted ONLY when its sole remaining references were the script itself, the generated
born-accounting faces (accounting-registry, scm-facts, gate-baseline), the shrink-only
rust-first-automation hygiene baseline, debt-inventory/audit docs, the retirement-marked
retired dev-cli gate-list surfaces (the quality-lanes registry, the marketplace dev-cli facade, the
governance gate-catalog domain lib — local bridge feedback, never merge authority per
`cli_surface_policy`), or other dead scripts in the same delete set.

**Deleted (26)** — verified-dead, no live invoker:

- Superseded PR-queue automation (ADR-0363/0515): `scripts/trigger-next-queue-automerge.sh`,
  `scripts/repair-sequential-pr-queue.sh`, `scripts/check-sequential-pr-merge-conflicts.sh`.
- Skip-everything no-op hook: `scripts/hooks/pre-push.sh`.
- Shims over RETIRED `oya` commands (Rust ports complete): `scripts/onprem-bring-up.sh`,
  `scripts/onprem-host-decommission.sh`, `scripts/supply-chain-adr0039.sh`,
  `scripts/install-trivy-ci.sh`.
- Dead `reject-*` duplicates of shipped Rust kernels: `scripts/reject-retired-grouping-wording.sh`,
  `scripts/reject-public-dev-domains.sh`, `scripts/reject-placeholder-digests.sh`.
- Dead CI bridges over the retired `./bin/oya`: `scripts/ci/oya-ci-post.sh`,
  `scripts/github-actions-required-secrets-check.sh`, `scripts/evidence-secret-scan.sh`,
  `scripts/pr-review-workflow-pr-head-check.sh` (targets a nonexistent workflow).
- Six orphan `scripts/tests/*.test.sh` harnesses (no runner; die with their dead targets):
  `github-actions-required-secrets-check.test.sh`, `pr-review-workflow-pr-head-check.test.sh`,
  `reject-placeholder-digests.test.sh`, `reject-public-dev-domains.test.sh`,
  `reject-retired-grouping-wording.test.sh`, `trigger-next-queue-automerge-required-contexts.test.sh`.
- Born-advisory / structurally-broken / cargo-era residue: `tools/governance/adr-0221-governance-gates.sh`
  (cited wiring absent), `oya/intelligence/iac/cedar/guardrails-build.sh` (ADR-0140-retired),
  `scripts/agent-pre-push-validate.sh` (cargo-era), `scripts/branch-protection-apply.sh`,
  `scripts/build/build-and-push-cloud-intelligence.sh` (targets the retired `microservices/` tree, 0
  files).

**Excluded — kept because a live invoker was found (2, reported for founder review):** the two
session-start-context-inject and userprompt-canonical-primer compat-no-op hook stubs under tools/hooks/.
The determination classed both as dead unwired stubs, and they ARE unwired in the Claude/Codex hook
files. BUT the canonical, matrix-registered, born-blocking `oya-cloud-ci-enforcement-liveness-app` gate
carries a hardcoded live-corpus census assertion — `hook_rows == 12`, `stub_rows == 2` — over the actual
tracked top-level hook-script tree (these two ARE the 2 `stub_marked` rows). Deleting them would drop
the census to `hook_rows == 10` / `stub_rows == 0` and turn that canonical gate RED. Per the safety
rule (a script with an unexpected live invoker is excluded, kept, and reported), these two are retained;
their retirement requires a paired update to the enforcement-liveness census, sequenced separately.

**Faces + baselines re-regenerated** (via the cloud-ci faces materialize command): the deleted paths
drop out of accounting-registry, scm-facts, and gate-baseline (rows 18485 → 18459, −26; gate-baseline
0 keys added / 76 removed) and the 25 corresponding rust-first-automation hygiene exceptions are removed
(the gate's `exception_stale` shrinkage rule). Every baseline SHRANK; none grew. Committed == regenerated.

door:one-way (touches the IRREDUCIBLE-GLUE ledger context only by deleting NON-ledger residue; no
ledger item is modified).

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md (RATIFY-TO-ADR). Refines ADR-0515 D3. Item (2)
supplied by ADR-0525. Addendum 2026-06-23: verified-dead `.sh` retirement step 1.*
