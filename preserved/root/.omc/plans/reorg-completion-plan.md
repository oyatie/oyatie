# Reorg Completion — Decision Request (rev 3)

**Status: PENDING FOUNDER DECISION.** Date: 2026-07-27.
rev 1 and rev 2 were both rejected on measurement. rev 3 does not select an option — it states the corrected cost and asks.

---

## Why this is a decision request, not a plan

Two revisions pre-selected "finish the migration" and both priced it wrong:

| revision | claimed | true |
|---|---|---|
| rev 1 | codemod cannot author catalog rows ⇒ build E1 | it *moves* rows; the real obligation is different (below) |
| rev 1 | no pre-merge graph proof exists ⇒ build E2 | `oracle::dry_run` (`oracle.rs:70`) already proves resolution without landing, CLI-wired, with RED + GREEN fixtures at `:288`/`:317`. I read `capture_snapshot` at `:50` and stopped 20 lines short **in the same file** |
| rev 1 | 6 move-plans ⇒ serial, does not converge | those 6 plans = **107 crate moves**; #1416 moved **41 crates on 2026-07-26** |
| rev 2 | net new catalog rows = **4** ⇒ delete E1 | **395** — off by ~100× (F1 below) |

Each time: one fact verified, the adjacent one inherited. rev 3 stops proposing.

---

## Corrected baseline (full clone, `origin/dev`, re-derived)

| Fact | Value |
|---|---|
| Packages | **926** (929 `Cargo.toml` − 3 workspace-only) |
| Legacy remainder | **466** — `oya/` 231, `libs/` 185, `tools/` 30, `cloud/` **20** |
| Destinations | `governance/` 128 · `app/` 110 · `intelligence` 109 · **unmapped 60** · `build/` 22 · `kernel/` 20 · `ci` 12 · `data` 5 |
| **Legacy crates already carrying a catalog row** | **395 of 466** — oya 205, libs 166, tools 24 |
| Fan-in | `tools/` **1** · `cloud/` **0** · `oya/` 281 · `libs/` 442 (max 128, `oya-data-boundary-kernel`) |
| Proven batch size | 41–46 crates/PR |

rev 2's 467 / cloud 21 / cloud fan-in 9 were wrong. `cloud/`'s 20 packages are **entirely** under `cloud/cloud-kernel/`, a nested excluded workspace.

---

## F1 — the finding that flips the decision

`catalog-parity` has **two** predicates (`ci/facade/service-catalog-parity/src/lib.rs`):

- **`catalog_live_crate_without_row`** (`:194`) — fires only for governed-root members. **This is the one rev 2 analysed**, giving "5 land in `data/`, 4 need rows."
- **`catalog_record_no_live_crate_unmarked`** (`:162`) — fires for **any** catalog row, governed or not, whose stem is no longer a live workspace package name.

Moves **de-brand the package name** (`os-move-plan.json`: `oya-cloud-os-apid-domain` → `os-apid-domain`). So every one of the **395** legacy crates that already has an `oya-*`-keyed row REDs on the second predicate the moment it moves — unless the row is renamed in the same change.

The codemod does not author these. They are **hand-enumerated `artifacts[]` entries in the move plan**, 1:1 with moves — verified on the landed `intelligence-move-plan.json`. `os-move-plan.json` needed none only because those 41 crates happened to have zero rows: the lucky subset, not the precedent.

**Per-batch hand-authored artifact rows: `tools/` 24 · `libs/` 166 · `oya/` 205.**

---

## Corrected cost of finishing

1. **~395 hand-authored catalog-rename artifact entries**, diff-invisible, one per crate that has a row.
2. **A root `Cargo.toml` members glob per new root, as a precondition.** #1416's own diff states it: *"This glob MUST be added BEFORE the codemod runs: without it the codemod appends literal-path members, which is `workspace_member_explicit_path` — a `frozen_empty` violation that cannot be baselined."* No `app/*/*`, no `build/*/*` today, and `governance/` is covered only by `governance/corpus/*`. (`allowed_root_dirs` **is** already done — #1404 — so rev 2's S2 was half-landed and named the wrong artifact.)
3. **`cloud/` is unprovable by the existing tooling.** `dry_run` checks `cargo metadata --no-deps` at the repo root (`oracle.rs:124-137`), which does not enumerate nested-workspace members. All 20 `cloud/` packages are nested, destination `kernel/` is also nested.
4. **60 crates have no destination** (`specs/capability-registry.json:585`, `burn_down_target: 0`, all `libs/`). 11 clear ≥3 consuming **capabilities**; 29 have zero cargo fan-in — but **≥6 of those are binaries**, for which zero fan-in is expected, not evidence of death.
5. **ADR-0627 already rejected this option and the rebuttal was never made.** Its Alternatives reject *"finish the migration first, then enforce"* because *"each batch actively removes crates from tier enforcement."* That specific mechanism **was fixed** — `owning_service()` became capability-aware in #1423 — and #1422 closed the authz scan-root blind spot a batch had opened. **That is the rebuttal, and neither rev made it.** It is now made.

**Precondition-free work is 17 crates** (`tools/oya-governance-*` → `governance/`), not the ~51 rev 2 claimed — and even those need a members glob first.

---

## The decision

**Option C — stop; bless dual state permanently.**
Cost: unmeasured, and the "dual state taxes product work" claim I used to justify finishing has **no citation anywhere** — verified: zero entries in `.omc/ultragoal/friction-ledger.jsonl` (215 rows) and zero in `friction-accounting-baseline.json`. ADR-0627 already records this as the standing decision, with `deciders: founder`. The closed registry resolves placement for 406 of 466 **pre-move**, so the "two homes" cost is paid at read time, not decide time.

**Option B — finish.**
Cost: ~395 hand-authored artifact rows, a non-baselineable members glob per root, `cloud/` outside what `dry_run` can prove, 60 architectural rulings, and ~11 batch PRs of 41–46 crates each.

**What I am not doing:** pre-selecting. Both revisions that did were wrong, and the corrected cost of B is roughly two orders of magnitude above what I first told you.

**If B:** the only work worth scoping first is **E1 reinstated** — automating the 395 catalog-rename artifact entries. That is the mechanical, repetitive, diff-invisible work automation is actually for, and rev 2 deleted it on a number that measured a different predicate.

**If C:** #1434 should be **withdrawn**, since it rewrites ADR-0627 to remove the pause that C keeps.

---

## What survives regardless of the ruling

**S1 — rule the 60 unmapped crates.** They are unresolvable today, block the `libs/` remainder under B, and are the largest single block of undecided architecture under C. 11 are `base/` candidates; 29 have zero fan-in of which ~6 are binaries and need a different test. **Note `base/` admission itself reverses ADR-0627** (*"`base/` is not created"*) and needs the same ruling.

This is the one item that pays off either way, and it is a decision, not machinery.

---

## Claims I could NOT verify
- **"~11 PRs"** — arithmetic (466 ÷ 41–46), not a measurement, and it prices in none of the above. Do not treat as an estimate.
- **"69 of 73 stale `absorbs_current_dirs` prefixes"** — carried from memory across three revisions and never checked. **Do not cite it.**
- Whether the 461 already-migrated crates all arrived via move-plans (6 plans account for 107).
