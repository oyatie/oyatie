# Spec: IaC Plan-Diff Domain Model

**Task slug**: `cloud-iac-plan-diff-domain`  
**Vertical**: infra  
**Crate**: `cloud-iac-domain` (`crates/cloud-iac-domain`)  
**Status**: SPEC

---

## Objective

Add a pure, deterministic IaC plan-diff surface to `cloud-iac-domain` alongside
the existing `reconcile_gitops_drift` function.  Given a *desired* topology (what the
declarative pipeline intends) and an *observed* topology (what was last successfully
applied), produce a per-resource diff keyed by `(module_ref, cell_id)` and an
aggregate verdict.

There is no I/O: no filesystem, network, provider SDK, OpenTofu CLI, Argo CD API,
or Kubernetes client.  The function is a pure Rust `fn` with no side effects.

---

## Scope

- Extends `crates/cloud-iac-domain/src/lib.rs`
- Adds `crates/cloud-iac-domain/tests/iac_plan_diff.rs`
- No new workspace members; no new `[dependencies]`

---

## Domain model

### `PlanAction`

```
pub enum PlanAction {
    NoChange,
    Create,
    Update,
    Destroy,
}
```

Derives `Clone, Debug, Eq, PartialEq, Ord, PartialOrd`.
Ord rank: `NoChange < Create < Update < Destroy` (natural declaration order).

### `PlanDiffEntry`

```
pub struct PlanDiffEntry {
    pub module_ref: OpenTofuModuleRef,
    pub cell_id: String,
    pub action: PlanAction,
}
```

Derives `Clone, Debug, Eq, PartialEq, Ord, PartialOrd`.
Natural sort key: `(module_ref, cell_id, action)` — fully deterministic.

### `IacPlanDiffVerdict`

```
pub enum IacPlanDiffVerdict {
    /// All entries are NoChange — topologies are fully converged.
    Converged,
    /// At least one Create / Update / Destroy entry exists.
    HasChanges,
    /// The top-level identity (topology_id, region) differs between desired and
    /// observed, OR a shared cell_id carries mismatched tenant_id values.
    /// Fail-closed: never silently treated as NoChange.
    IdentityMismatch,
}
```

Derives `Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd`.

### `IacPlanDiffReport`

```
pub struct IacPlanDiffReport {
    pub verdict: IacPlanDiffVerdict,
    pub entries: Vec<PlanDiffEntry>,
}
```

`entries` is sorted by `PlanDiffEntry`'s natural `Ord` (module_ref → cell_id → action).
When `verdict == IdentityMismatch` the `entries` vec is empty.

---

## `compute_iac_plan_diff` contract

```
pub fn compute_iac_plan_diff(
    desired: &CellTopologyPlan,
    observed: &CellTopologyPlan,
) -> IacPlanDiffReport
```

### Identity check (fail-closed, checked first)

If `desired.topology_id() != observed.topology_id()` **or**
any cell's `tenant_id` differs **or** `desired.region() != observed.region()`,
return `IacPlanDiffReport { verdict: IacPlanDiffVerdict::IdentityMismatch, entries: vec![] }`.

`CellTopologyPlan` carries a single `region` field; cells within the same plan
share that region (validated at construction time).  The tenant comparison checks
that every cell in `desired` has a matching cell in `observed` with the same
`tenant_id`.

### Diff algorithm

For each `(module_ref, cell_id)` pair appearing in either topology:

| Desired | Observed | Action |
|---------|----------|--------|
| present | absent   | Create |
| absent  | present  | Destroy |
| present | present, same ref (namespace/name/system/version) | NoChange |
| present | present, different ref (version or name changed) | Update |

A "module_ref present in a cell" means the cell's `module_refs()` slice contains
that ref.  The key is the pair `(module_ref, cell_id)`.

### Determinism guarantee

- No randomness, no I/O, no `SystemTime`, no `HashMap` (use `BTreeMap` / sorted
  vecs internally).
- `entries` is sorted via `Vec::sort()` on the natural `Ord` of `PlanDiffEntry`.
- Identical inputs always produce identical output (byte-level).

### Aggregate verdict

After collecting all entries:
- Any entry with action `Create | Update | Destroy` → `HasChanges`
- All entries `NoChange` (including empty entries vec) → `Converged`
- Identity mismatch (from check above) → `IdentityMismatch`

---

## Mod layout (flat clean-arch, `src/lib.rs`)

The additions live directly in `src/lib.rs` (single-file crate, no sub-mods yet).
If the file grows large enough to warrant splitting, introduce `mod plan_diff;` in
a follow-on task — outside this slice's scope.

---

## Testing strategy

File: `tests/iac_plan_diff.rs`

| Test name | Scenario | Expected verdict |
|-----------|----------|-----------------|
| `all_converged` | desired == observed | `Converged`, all `NoChange` |
| `desired_only_module` | extra module in desired | `HasChanges`, one `Create` |
| `observed_only_module` | extra module in observed | `HasChanges`, one `Destroy` |
| `version_update` | same ref, different version | `HasChanges`, one `Update` |
| `identity_mismatch` | topology_id differs | `IdentityMismatch`, empty entries |
| `determinism` | same inputs called twice | byte-identical `entries` + verdict |

Pre-existing tests (`cloud_iac_foundation.rs`, `gitops_drift_reconciliation.rs`)
must remain green.

---

## Boundaries

- **No new deps**: `[dependencies]` in `Cargo.toml` stays empty; std-only.
- **No new workspace member**: root `Cargo.toml` is not touched.
- **No new crates**: this is an extension of the existing flat crate.
- **No I/O**: the function is a pure domain computation.
- **Reuse existing types**: `OpenTofuModuleRef`, `CellDefinition`, `CellTopologyPlan`
  are reused verbatim; no duplicate identity structs.
