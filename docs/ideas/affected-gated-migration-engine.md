# Affected-Gated Migration Engine ("Sweep")

## Problem Statement
How might we apply a transform across hundreds of crates, land every change that passes the
unchanged build+test gate, auto-quarantine the few that fail, and auto-merge the green
majority — with no human in the merge loop and no weakening of the gate?

## Recommended Direction
One reusable Workflow (`agent()`/`parallel()`/`pipeline()`) parameterized by
`(transform, unit-discovery, verify-cmd, risk-class)`. It unifies the four organs that this
session's bottlenecks demanded as one engine:
- **A. bulk rollout** — the body (pipeline over work-units),
- **B. adversarial-verify** — the *risk gate* (only for non-mechanical transforms),
- **C. parallel worktree lanes** — *how* A fans out without file collisions,
- **D. gate-failure auto-triage** — the feedback loop that makes A's quarantine automatic.

The enforced buck2 affected gate stays the **single quality invariant**; the engine only feeds
it faster and closes the red→quarantine→re-push→green loop *itself*, then auto-merges. The
`#84` `rust_test` rollout is the first user; later reused for buck2-cutover sweeps, dependency
bumps, and lint fixes.

This is the hyperscaler eng-productivity pattern scoped to one cluster: TAP-style affected
targets (already in the gate), **auto-quarantine** of broken/flaky targets (the `KNOWN_FAILING`
skip-list), **presubmit (cheap, local-darwin) vs postsubmit (authoritative gate-Linux)**, and
**merge-queue auto-merge on green**.

## Key Assumptions to Validate
- [ ] Auto-merge-on-green is safe for MECHANICAL transforms — test: run on `#84` (additive test
      emission), audit one merged batch for a false-green.
- [ ] The gate-RED status-summary reliably names the failing target(s) — depends on the v2
      observability (status description carries the failing target); confirm on the libs failure.
- [ ] Quarantine converges (failures are a minority per unit — libs was 3/164) — test: a libs
      run quarantines a handful, lands the rest.
- [ ] Concurrent subsystem PRs don't conflict at merge — mitigate by serializing the merge step
      / rebase-on-merge.

## MVP Scope
**In:** `pipeline` over remaining `#84` subsystems → append-emit
(`//tools/oya-buck-test-wiring-app:oya-buck-test-wiring --apply`, the owned-Rust ADR-0540
generator that replaced `scripts/emit_rust_tests.py`; `--check` is its coverage-gate mode)
→ local `buck2 test //unit/...` (darwin presubmit) → land-loop (open PR → poll `oya-ci-gate` →
on RED, parse the status-summary, quarantine the named crate via `KNOWN_FAILING` + revert +
follow-up, re-push, ≤K rounds → **auto-merge** on green) → report (landed/quarantined counts,
follow-up list, nothing silently dropped). `risk-class = mechanical` for `#84` (no adversarial
gate). **Out:** fixing quarantined crates; semantic transforms; a bespoke speculative merge-queue.

## Not Doing (and why)
- **Auto-merge of infra/toolchain/security transforms without adversarial review** — the gate
  can't catch semantic/deadlock risk (cf. the 3-lens Jenkinsfile review that prevented a gate
  deadlock).
- **Per-crate PRs** — 716 PRs is noise; batch by subsystem, isolate failures at crate granularity.
- **A bespoke speculative merge-queue** — GitHub merge API + serialized merge suffices now;
  ADR-0111 projected-state is later.
- **Touching the gate's quality logic** — the engine FEEDS the gate, never weakens it.

## Open Questions
- Batch size: whole subsystem vs cap N crates/PR (gate-latency vs PR-count trade-off)?
- Round cap K before quarantining a whole unit (start K=3)?
- Does auto-merge serialize on dev, or rebase-and-retry on merge conflict?

## Concrete workflow shape (authoring guide)
- **meta.phases:** `Discover` → `Transform+Verify` → `Risk-gate` → `Land` → `Report`.
- **Transform+Verify:** `pipeline(units, transform→localVerify)`, `isolation:'worktree'` per unit
  (parallel transforms don't collide). Returns `UNIT_RESULT{unit, green[], quarantined[{target,reason}]}`.
- **Risk-gate:** for `risk-class != mechanical`, `parallel([...lenses])` skeptic-reviewers; land
  only if it survives (the deadlock-review pattern, generalized).
- **Land-loop** (per unit): an agent opens the PR + polls the gate; on RED a triage agent
  (`schema=TRIAGE{action: quarantine|retry, targets[], class}`) parses the status-summary →
  quarantine (mutate skip-list + revert + re-push) or retry-once (flake); `while round<K && !green`;
  on green → auto-merge. The **merge step is serialized** (sequential reduce or a barrier) to
  avoid dev races.
- **Backstops:** `log()` every quarantine (no silent caps); `budget`-guard the loop; cap rounds K.

## How it compounds
Throughput comes from parallel transform+verify, auto-merge (no human in loop), and batching.
Quality is preserved because every landing still passes the unchanged build+test gate, risky
transforms get adversarial review, and failures are isolated/quarantined — never force-landed.
The engine is **gate-latency-bound** (libs closure ~4 min cold), which makes the case for the
[[nativelink-remote-cache-first]] cache concrete and measurable: warm cache → faster affected
builds → higher engine throughput. See [[parallel-swarm-model]], [[ci-gate-pipeline-state]].
