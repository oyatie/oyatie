# Task: cloud-iac-plan-diff-domain

**Vertical**: infra  
**Crate (sole target)**: `oya-cloud-iac-domain` (`crates/oya-cloud-iac-domain`)  
**Branch**: `feat/task-cloud-iac-plan-diff-domain-2026-05-28`

---

## Objective

Extend the pure domain crate with a deterministic IaC plan-diff model: given a
desired `CellTopologyPlan` / module-ref set and an observed/current applied set,
compute a per-resource action (Create / Update / Destroy / NoChange), an aggregate
verdict, and a stable sorted summary.  No new workspace member, no new deps (std-only).

---

## Subtasks

### ct-1 — Plan-diff value objects

Add `PlanAction` and `PlanDiffEntry` to `src/lib.rs`.

- `PlanAction` enum: `NoChange | Create | Update | Destroy`
  derives `Clone, Debug, Eq, PartialEq, Ord, PartialOrd`
- `PlanDiffEntry` struct keyed by `(OpenTofuModuleRef, cell_id: String)` carrying the action
  derives `Clone, Debug, Eq, PartialEq, Ord, PartialOrd`
- Reuse existing `OpenTofuModuleRef` / `CellDefinition` types; no duplicate identity struct

**Acceptance**: `cargo check -p oya-cloud-iac-domain --all-targets` green;
`PlanAction` / `PlanDiffEntry` are `pub`, std-only, derive the documented traits,
and reference the existing `OpenTofuModuleRef` type.

---

### ct-2 — Pure diff function `compute_iac_plan_diff`

Add `pub fn compute_iac_plan_diff(desired: &CellTopologyPlan, observed: &CellTopologyPlan) -> IacPlanDiffReport`.

- Returns a `IacPlanDiffReport` containing a sorted `Vec<PlanDiffEntry>` and an
  aggregate `IacPlanDiffVerdict`.
- Verdict variants: `Converged` (all NoChange) | `HasChanges` | `IdentityMismatch`
  (topology_id / tenant / region differ — fail-closed, never silently NoChange).
- Deterministic: stable `Ord`-based sort, no I/O, no clocks, no randomness.
- Identity mismatch check precedes all diff logic.

**Acceptance**: Pure `fn` returning a struct with no filesystem/network/time
dependency; identity mismatch yields `IdentityMismatch`, not `HasChanges` or
`Converged`; same inputs always produce identical output.

---

### ct-3 — Integration tests + spec doc

Add `tests/iac_plan_diff.rs` covering:

1. All-converged => `Converged` verdict, all entries `NoChange`
2. Desired-only module-ref => `Create` entry, `HasChanges` verdict
3. Observed-only module-ref => `Destroy` entry, `HasChanges` verdict
4. Same ref differing version/source => `Update` entry, `HasChanges` verdict
5. Identity mismatch (topology_id differs) => `IdentityMismatch` verdict
6. Determinism assertion: two calls with identical inputs produce byte-identical reports

Add `docs/specs/task-cloud-iac-plan-diff-domain.md` describing the diff contract.

**Acceptance**: `cargo nextest run -p oya-cloud-iac-domain` passes all new and
pre-existing tests; spec doc is committed.

---

## Acceptance summary (all subtasks)

| Check | Command |
|-------|---------|
| Compile (lib + tests) | `cargo check -p oya-cloud-iac-domain --all-targets` |
| All tests pass | `cargo nextest run -p oya-cloud-iac-domain` |
| No new deps | `Cargo.toml` `[dependencies]` section unchanged |
| No new workspace member | root `Cargo.toml` untouched |
