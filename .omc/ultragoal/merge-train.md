# Merge train — ultragoal PRs onto dev (manual pessimistic queue until Tide lands)

**Why this exists:** dev has a single required context (`oya-ci-required`) that runs per-PR against the PR head —
there is NO live projected-merge-state queue (ADR-0515 Tide is spec-only). Per-PR green does not prove post-merge
green. Until G011 lands the real queue, merges follow this manual Landcastle-style protocol (FRIC-007).

## Admission contract (Tide-survivable — the interface, not the choreography)
**Admit a change iff `oya-ci-required` is GREEN on the projected post-merge state.** That is the entire contract,
and it is exactly the ADR-0515 Tide invariant. The manual rebase choreography below is merely the CURRENT ADAPTER
that computes the projected state (rebased-onto-fresh-dev head ≈ projected merge state when the train is serial).
**Cutover assertion:** when ADR-0515 cloud-ci/oya-ci Tide lands, steps 1–3 below are deleted and the projected-state
computation moves to the queue; the admission contract is unchanged. (Litmus "would this interface change at
cutover?" → NO.)

## Protocol (every train position — the current adapter)
1. Wait for the previous train position to MERGE.
2. Rebase your branch onto fresh `origin/dev` (resolve Cargo.toml workspace-members / ADR-INVENTORY.tsv / oya-ci.toml
   appends per the orderings below).
3. `oya-ci-required` must re-run GREEN on the REBASED head (this approximates projected merge state).
4. Founder review/sign-off where flagged (G001 ADRs, #635 oya-ci.toml carve-outs) → squash-merge → next position.
   (2026-06-10 simplification: when a lane's base == current dev tip, steps 1–3 are already satisfied — green on
   head IS green on projected state; merge directly.)

## Concurrency bound (2026-06-10, ralplan amendment 4; two-tier per r3)
Parallel in-flight lanes are bounded by the TWO-TIER surface model checked at dispatch time against
`dispatch-ledger.jsonl` `expected_surfaces` (domains defined in TEAMMATE-PREAMBLE §3.1):
**HARD collisions** (in-place path overlap, ADR numbering, workflow/config registries, shared-lib edits) ⇒ the
later lane is sequenced, not dispatched — these are semantic conflicts no tool can merge. **SOFT collisions**
(Cargo.lock, generated faces) ⇒ parallel dispatch allowed; only the MERGE rides the train in order, paying one
mechanical rebase step (lock merge driver #661 + face settle tool #668 exist precisely to make this cheap). The
serial merge tail is therefore bounded by total face-touching lane count (cheap mechanical steps), while the
expensive serialization applies only to the hard-colliding subset (rare by construction when briefs are scoped
to disjoint trees).

## Train order (smallest blast radius → largest; canon-sensitive early)
| Pos | PR / lane | Surface | Notes |
|---|---|---|---|
| 1 | #635 plan rescue | docs/audit/** + oya-ci.toml carve-outs | GREEN already; founder eye on oya-ci.toml diff |
| 2 | G001 contract lock | authority docs + 2 Proposed ADRs + libs/ crates + Cargo.toml | claims ADR numbers FIRST (see reservation) |
| 3 | LANE-3 office | oya/office deltas (existing dirs) | rename reconcile; small new-tree risk |
| 4 | LANE-4 intelligence-sdk | cloud/cloud-intelligence/ new crates | founder-corrected destination |
| 5 | LANE-2 os | cloud/cloud-os/ NEW tree | total-accounting: needs ownership/justification rows |
| 6 | LANE-1 kernel | cloud/cloud-kernel/ NEW tree (largest) | same accounting requirement; last |

## Shared-surface orderings (conflict law)
- **ADR numbers:** G001 reserves the next two free (≥0536). NO other lane authors an ADR without leader assignment
  AFTER G001 lands. ADR-INVENTORY.tsv appends resolve trivially at rebase in train order.
- **Root Cargo.toml workspace members:** alphabetical insertion, one commit per lane, rebase resolves; later positions
  take the union.
- **oya-ci.toml:** #635 owns this train's only intended edit. Lanes needing accounting/vocab entries for NEW dirs add
  the minimal config-data rows at rebase time (positions 5–6), never predicate code.
- **buck2/reindeer:** lanes adding crates regenerate per the documented pattern at their rebase; conflicts take later
  position's regeneration.
- **`*.generated.json`:** never in any diff (CI materializes). No exceptions on any position.

## Standing rule
G02–G09 substrate lanes (after G001) inherit this protocol with leader-assigned positions; the train retires when
G011 ships the real pessimistic merge queue + projected-state CI.
