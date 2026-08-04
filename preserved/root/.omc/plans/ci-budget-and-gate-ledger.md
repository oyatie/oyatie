# CI budget + gate firing ledger — the damping term

Design only. `prevention-doctrine.md` §1 makes every mistake mint a gate and attaches no
price to one. 55 jobs, 42 of which have never fired in 1,966 executions. This adds the
missing cost side as data + one gate, not as a rule anyone has to remember.

## 0. What this optimizes — and what it does not

It optimizes **billable runner-minutes**, not wall-clock. The 42 quiet jobs are parallel
matrix legs of `gate`; deleting all of them changes the critical path by roughly zero. The
critical path is `producer-regen → gate-affected-target-set` (`timeout-minutes: 120`), and
nothing in this proposal touches it. Any claim that this "speeds up CI" is false. It makes
verification cost *visible and finite* so that adding gate #56 forces a trade.

Second-order effect, stated but not sold: at 55 concurrent legs the repo is near GitHub's
per-account concurrency ceiling, so minutes and queueing do couple. Unquantified — do not put
it in a justification. A `critical_path_minutes` limit is carried too, but it governs exactly
two jobs, since only `affected-target-set` / `buck2` can move that number.

## 1. The budget — `specs/ci-budget-policy.json`

Cross-cutting policy consumed by more than one crate → `specs/`, matching
`specs/cache-warmth-policy.json`. Single-gate policy stays beside its crate
(`ci/facade/*/‑policy.json`). Same idiom: `_comment`, `policy_id`, `schema_version`, `adr`,
a fail-closed `default_for_unlisted`, a `product_contract` block, zero oyatie literals in
the engine.

```json
{
  "policy_id": "ci-budget-policy",
  "schema_version": "1.0.0",
  "adr": "ADR-06NN",
  "budgets": {
    "presubmit_runner_minutes": { "limit": 260, "adr": "ADR-06NN" },
    "presubmit_critical_path_minutes": { "limit": 60, "adr": "ADR-06NN" }
  },
  "default_for_unlisted_gate": { "allowance_minutes": 0,
    "reason": "fail-closed: registering a gate without an allowance row is RED" },
  "gate_allowances": {
    "cloud-ci-canonical-json": { "allowance_minutes": 3, "critical_path": false, "reason": "..." },
    "cloud-ci-affected-set":   { "allowance_minutes": 45, "critical_path": true,  "reason": "..." }
  }
}
```

`allowance_minutes` is a **declared** budget, not an observed count — it changes only when
someone intends it to. That is deliberate: an auto-derived table would be
`AP-A23 Manual Count Chasing` wearing a generator, and committing volatile measurements
re-opens the de-committed-faces problem.

## 2. Enforcement — `ci/facade/ci-budget/`

New crate `ci-budget` (lib `ci_budget`), BUCK targets `//ci/facade/ci-budget:{ci-budget,
ci-budget-unittest, ci-budget-gate}`, one matrix `include` line
`{ crate: ci-budget, label: "gate · ci-budget (conservation law over verification cost)" }`.
Same shape as every sibling: `load_policy` / `collect_observed` / `evaluate(&policy,
&observed) -> Report{Verdict, BTreeSet<Finding>}`. `collect_observed` does not re-parse the
workflow — it reuses `ci_gate_self_conformance::collect_observed_gates`, which already
enumerates registered gates from `ci/facade/**` plus `oya-ci-required.yml`.

Three predicates, all pure, no network, ~2 seconds:

1. **Coverage** — every registered gate has an allowance row (else `default_for_unlisted` → 0 → RED).
2. **Conservation** — `Σ allowance_minutes ≤ budgets.presubmit_runner_minutes.limit`.
3. **Door** — raising a `limit` requires the sibling `adr` field to name an ADR that exists in
   `docs/decisions/` and cites this policy. Lowering needs no door (shrink is always allowed),
   mirroring the shrink-only baseline idiom already used by license-policy and brand-residue.

This is the damping term: a PR adding gate #56 goes RED unless it fits the headroom, **retires
a gate in the same PR**, or opens an ADR. Retirement becomes the cheapest path to green. No
new job, no new workflow, no CLI surface, no runtime cost.

## 3. The firing ledger

**Source.** GitHub Actions REST API — `actions: read` is already granted at
`.github/workflows/oya-ci-required.yml:55`. Per-job `conclusion` + `started_at`/`completed_at`
+ `run_attempt` over a rolling window is exactly the sample that produced this diagnosis.
`oya-cloud-ci-step-telemetry.rs` is a heartbeat wrapper with no persistence; not the source.

**Producer.** A `ledger` mode on the `ci-budget` crate, one step inside the existing
`oya-ci-required` fan-in job, guarded `if: github.event_name == 'push'`. PR runs pay nothing;
the fan-in on `push` is otherwise idle. No new workflow.

**Shape.** Reuse `specs/cloud-ci-run-observability-packet.schema.json` per run (already
specified, already validated by `ci/facade/baseline-ratchet/src/run_observability_packet.rs`,
currently produced by nothing); add `specs/gate-firing-ledger.schema.json` for the rolling
aggregate: per gate, `red_runs`, `green_runs`, `p50_minutes`, `total_minutes`, `first_seen`,
`class`. Stored as a run artifact, never committed — volatile derivation, which
`generated-output-diff-policy` exists to keep off merge surfaces.

**Consumers.** (a) Humans/agents, for sunset decisions. (b) On `push` only, the budget gate
REDs when a gate's measured `p50_minutes` exceeds its declared allowance — the declaration is
then load-bearing in both directions. Missing or stale ledger on `push` is RED, not skipped.

## 4. The sunset criterion

A gate that never fires may be succeeding by deterrence. Several are explicitly
"born-blocking" or ratchet-vs-frozen-baseline, where a flat green line is the *designed*
steady state. Red-count alone cannot separate those from dead weight, and the CI red-count is
worse than neutral — it is a **systematically biased undercount**, because a gate doing its
job best fires in the author's local `buck2 test` and never reaches a pushed run at all.

So: never sunset on red-count. Classify on four signals, three of which are mechanical.

- **Fixture liveness.** Every gate already ships RED fixtures asserting each violation class
  fails closed (`ci-canonical-json`'s BUCK comment states this as the norm). Count them. A
  predicate that cannot be made to fire by an injected violation is not deterring — it is inert.
- **Governed population.** Every gate's `collect_observed` already knows the corpus it
  polices. Population 0 deters nothing.
- **Proximity.** Population ∩ changed paths, per run. **Deterrence requires that authors were
  actually near the tripwire.** If no PR in the window touched the governed corpus, quiet is not
  deterrence — it is irrelevance to the change stream.
- **Baseline drain.** For frozen-baseline ratchets, a shrink *is* a firing. Count
  `baseline_delta` as green-side work; a ratchet that never fires red and never shrinks is
  frozen debt, not a live tripwire.

Three classes:

- **INERT** — zero RED fixtures, or fixtures fail to fire, or population 0. Provable dead
  weight, no judgment. The budget gate REDs on it directly: fix or delete.
- **UNEXERCISED** — fires in fixture, but proximity 0 across the window and no baseline drain.
  Mechanically classified, **not auto-deleted**. The ledger emits a review row naming the gate's
  motivating ADR. Sunset is an ADR amendment.
- **LIVE** — everything else, including all 13 known-firing infra gates.

**The honest part.** For a violation class that is extinct *because* the gate deters it, no
measurement distinguishes deterrence from irrelevance. That is a counterfactual, not a missing
metric, and no design here or elsewhere resolves it. The budget is what makes it survivable:
we never have to prove a gate useless. We only have to rank it against the *other* uses of the
same fixed pool of minutes. The budget converts an unanswerable epistemic question into an
answerable allocation question — that, not the ledger, is the load-bearing idea.

One narrow empirical probe: an UNEXERCISED non-security gate may be demoted to advisory for N
weeks; if reds appear, it was deterring. Buys evidence, **not minutes** — advisory legs still
run. Never for security/authz gates.

## 5. What this does NOT solve

Wall-clock (§0). The 120-minute `affected-target-set` timeout. Flake-vs-real reds — an
infra-flaky gate looks LIVE, and `run_attempt` disambiguation is partial. The counterfactual
(§4). Population/proximity/fixture census needs per-gate emission (slice 3); until then the
sunset lane has only red-counts, which §4 shows are not sufficient.

## 6. Smallest first slice

**Slice 1 — declaration side only.** `specs/ci-budget-policy.json` + `ci/facade/ci-budget/`
with predicates 1–3, allowances seeded from the sampled p50s, `limit` seeded at today's
measured total (so the PR is behaviour-neutral by construction, the same safety property
`oya-ci.toml`'s `profile = 'oyatie'` uses). No API, no artifact, no ledger, no census.

That alone ships the damping term: from that commit, every new gate must fit or free minutes.
Slices 2 (ledger + measured-vs-declared) and 3 (per-gate census → INERT/UNEXERCISED) make the
numbers honest and the sunset conversation data-driven; each is independently valuable.
