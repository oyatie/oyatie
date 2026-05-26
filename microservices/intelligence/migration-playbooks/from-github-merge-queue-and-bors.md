# Migration playbook — GitHub Merge Queue or Bors → Oyatie `foundry`

Audience: a platform/SRE team operating one of:
- GitHub Merge Queue on GitHub Enterprise Cloud or Server
- Bors-NG or a Bors fork
- a hand-rolled shell-script-based merge serialization layer

…who wants to migrate to a Foundry-shaped pipeline. Foundry is internal-only (ADR-0136-amendment), so this is for organizations
building their own Foundry-style tooling on top of the Oyatie patterns, not consuming Foundry as a service.

> Phase budget: 90 days for ≤ 50 engineer org; 180 days for ≥ 200 engineer org.

## Phase 0 — Inventory (Day 0…14)

1. **Repo + workspace map.** Document every shared crate / shared package / shared library. These are the cascade risk surfaces.
   ```bash
   cargo tree --workspace --depth 1 | sort -u > shared-crates.txt
   ```
2. **PR throughput baseline.** Capture PR/day, merge p50/p95, validator pass rate, time-to-merge by PR size bucket. You'll measure
   against this throughout the migration.
3. **Existing validators.** Enumerate every required check that GitHub Merge Queue / Bors gates on.
4. **Current reviewer model.** Document who reviews what; for AI-assisted reviewers, capture the model + prompt.
5. **Audit retention.** Capture current audit-trail retention + integrity (most orgs have append-only logs; no chain).

## Phase 1 — Foundation infrastructure (Day 14…45)

You need 4 substrate pieces before Foundry is operable:

1. **Cedar evaluator** — adopt the Cedar policy library + evaluator (the Rust crate `cedar-policy 4.x`).
2. **Audit chain** — implement a BLAKE3-256-chained append-only event store (the Oyatie `audit-chain` µservice is open-license but
   typically you'll fork or re-implement against your own datastore).
3. **Webhook receiver** — per ADR-0112, you need a service that accepts GitHub webhooks + emits structured `foundry.pr_event.*`.
4. **`oya vcs` CLI surface** — implement `claim/work/done/verify/promote` as protocol verbs your agents/humans call. The CLI is the
   only legitimate way to interact with the pipeline outside the GitHub UI.

These four pieces are non-trivial; allocate ≥ 60 % of total migration budget here.

## Phase 2 — Coordinator + merge queue (Day 45…75)

Implement the merge queue with projected-state semantics per ADR-0111:
- Rebase the PR onto current target in-memory.
- Run validators on the projection.
- Acquire shared-crate locks for any crate the PR touches.
- Admit only if projection passes + locks are held.
- On target advance, recompute projections for all in-flight PRs that touch advance-affected crates.

The coordinator state machine is the hardest part; budget 30 days for a small team. The Oyatie reference implementation lives at
`crates/oya-foundry-coordinator/` if you have permission to read the source.

## Phase 3 — Reviewer-agent multispectrum (Day 75…120)

Adopt the multispectrum v2.4.0 doctrine:
- 11 facets (F1-F9 critique + M1+M2 meta + A1-A7 adherence where applicable).
- Each facet evaluated by a separate subagent session with that facet as its lens.
- Consensus among model results on high-risk PRs.

Start with 3 facets (F1 correctness + F4 security + A3 structural-adherence) and expand. Run reviewer in shadow mode for 30 days
before making verdicts gating.

## Phase 4 — Shadow-mode + cut-over (Day 120…150)

For 30 days, run Foundry in shadow alongside GitHub Merge Queue / Bors:
- Both systems receive the PR.
- GitHub Merge Queue / Bors gates merge.
- Foundry runs its full pipeline in shadow + emits verdicts.

Compare daily:
```bash
./bin/oya migrate compare --source github-merge-queue --target foundry --window 24h
```

When verdict-parity is ≥ 95 %, cut over: disable GitHub Merge Queue, enable Foundry as the gate.

## Phase 5 — Optimization (Day 150+)

- Add the lean-a* lanes incrementally (start with `lean-a3-tenant-trace` + `lean-a5-doc-coverage`; add others as policies stabilize).
- Add per-pack overlays as you adopt compliance regimes.
- Add the cosign-signing webhook chain.

## Rollback strategy

The substrate pieces (Cedar, audit-chain, webhook receiver, CLI) are usually not destructive to roll back from; they sit alongside
existing tools. The destructive operation is **disabling the existing merge queue**. If Foundry fails after cutover:
1. Re-enable GitHub Merge Queue / Bors (your old config is preserved).
2. Foundry continues running in shadow mode.
3. Diagnose the failure; re-cutover when ready.

After 60 days clean on Foundry, decommission the old queue:
- GitHub Merge Queue: configure branch protection to no longer require the queue check.
- Bors: shut down the Bors process; remove the `bors.toml`.

## What you gain

- Cascade-clog prevention via the coordinator + projected merge state.
- Multispectrum reviewer with model consensus.
- Tamper-evident BLAKE3 audit chain.
- Cedar at the reviewer + at the protocol verbs.
- Self-modification governance.

## What you give up

- "Plug-and-play" — Foundry is substantial infrastructure to operate.
- GitHub native UI integration — your PRs will have additional check runs from your `oya vcs` surface.
- The vendor's bug-fix cadence — you own the queue logic now.

## When NOT to do this migration

- Org < 50 engineers.
- Single-repo, no shared-crate cascade risk.
- No compliance pressure for tamper-evident audit.
- No engineering capacity for ≥ 3 dedicated pipeline engineers.

In those cases, stay on GitHub Merge Queue or Aviator ShipIt.
