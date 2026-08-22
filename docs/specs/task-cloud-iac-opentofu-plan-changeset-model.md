# Spec: OpenTofu Plan-Changeset Domain Model

**Task slug**: `cloud-iac-opentofu-plan-changeset-model`
**Vertical**: infra
**Crate**: `cloud-iac-domain` (`crates/cloud-iac-domain`)
**Status**: SPEC

---

## Objective

Add a pure, deterministic OpenTofu plan-changeset domain model to
`cloud-iac-domain` alongside the existing GitOps-drift reconciler and IaC
plan-diff surface.

This slice models **OpenTofu plan output** — the five-action taxonomy that
`tofu plan` / `tofu show -json` emits per resource address.  It is a distinct
axis from:

- `reconcile_gitops_drift`: compares Argo CD sync/health state vs desired
- `compute_iac_plan_diff`: compares desired vs observed `CellTopologyPlan`
  module-ref sets

No I/O is introduced. The crate contract (no filesystem, network, provider SDK,
OpenTofu CLI, Argo CD API, or Kubernetes client) is preserved throughout.

---

## Scope

- Extends `crates/cloud-iac-domain/src/lib.rs`
- Adds `crates/cloud-iac-domain/tests/opentofu_plan_changeset.rs`
- No new workspace members; no new `[dependencies]`
- Plan file: `tasks/cloud-iac-opentofu-plan-changeset-model-plan.md`

---

## Flat clean-arch mod layout (ADR-0509)

All types live directly in `src/lib.rs` alongside existing types — no sub-modules.
The crate has no `src/` subdirectories; ADR-0509 mandates one flat crate with
mod-based subsystem separation via inline comments/sections.

---

## Domain model

### `ResourceChangeAction`

```rust
pub enum ResourceChangeAction {
    /// Resource will be created (was absent from state).
    Create,
    /// Resource exists; in-place attribute update is possible.
    Update,
    /// Resource will be removed from state and infrastructure.
    Delete,
    /// Resource must be destroyed and re-created (implies destructive).
    Replace,
    /// Resource is in-state and unchanged; no plan action required.
    NoOp,
}
```

Derives `Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd`.
Natural Ord (declaration order): `Create < Update < Delete < Replace < NoOp`.

**Destructive actions**: `Delete` and `Replace`.

### `ResourceChange`

```rust
pub struct ResourceChange {
    /// Fully-qualified resource address, e.g. `module.cell_vpc.aws_vpc.main`.
    resource_address: String,
    /// The planned action for this resource.
    action: ResourceChangeAction,
}
```

Derives `Clone, Debug, Eq, PartialEq, Ord, PartialOrd`.
Constructed via `ResourceChange::new(address, action) -> Result<Self, CloudIacError>`.

**Validation**:
- `resource_address` must be non-empty
- must not contain ASCII whitespace or control characters
- must not look secret-like (inherits crate-wide `looks_secret_like` guard)

### `PlanChangeset`

```rust
pub struct PlanChangeset {
    plan_id: String,
    changes: Vec<ResourceChange>,
}
```

Constructed via `PlanChangeset::new(plan_id, changes) -> Result<Self, CloudIacError>`.

**Validation**:
- `plan_id` must be a non-empty slug (lowercase alphanumeric + hyphens)
- duplicate `resource_address` values within `changes` are rejected

**Methods**:
- `plan_id(&self) -> &str`
- `changes(&self) -> &[ResourceChange]`
- `has_destructive_changes(&self) -> bool`
  Returns `true` iff any entry has action `Delete` or `Replace`.
- `summarize(&self) -> PlanChangesetSummary`
  Returns per-action counts and a total.

### `PlanChangesetSummary`

```rust
pub struct PlanChangesetSummary {
    pub create_count: usize,
    pub update_count: usize,
    pub delete_count: usize,
    pub replace_count: usize,
    pub no_op_count: usize,
    pub total: usize,
}
```

Derives `Clone, Copy, Debug, Eq, PartialEq`.
`total` == sum of all per-action counts.

---

## New `CloudIacError` variants

```rust
InvalidResourceAddress,
DuplicateResourceAddress,
InvalidPlanId,
```

---

## Testing strategy

File: `crates/cloud-iac-domain/tests/opentofu_plan_changeset.rs`

Coverage:
1. Happy-path: all five action variants accepted, correct summary counts
2. `has_destructive_changes` — false when only Create/Update/NoOp present
3. `has_destructive_changes` — true when Delete present
4. `has_destructive_changes` — true when Replace present
5. `has_destructive_changes` — true when both Delete and Replace present
6. Empty changeset: valid, `has_destructive_changes` = false, all summary counts zero
7. Duplicate resource address rejected with `DuplicateResourceAddress`
8. Empty resource address rejected with `InvalidResourceAddress`
9. Whitespace-only resource address rejected
10. Resource address with embedded tab/newline rejected
11. Invalid plan_id (empty) rejected with `InvalidPlanId`
12. Invalid plan_id (uppercase) rejected
13. Summarize counts are exact across a mixed-action changeset
14. `summarize().total` == sum of per-action counts
15. Determinism: two calls with identical inputs produce identical output

---

## Observability / SLO note (ADR-0130)

The `PlanChangeset` domain model is consumed by upstream use-case and adapter
layers that emit OTel spans. The flat field layout of `PlanChangesetSummary`
(one field per action type) is intentional so telemetry adapters can emit
histogram/counter metrics without re-iterating the changeset.

`has_destructive_changes()` is the boolean gate that feeds safety-check SLO
indicators (e.g., `iac-plan-destructive-gate-pass-rate`).

No OTel code lives in this domain crate — instrumentation belongs in the
infrastructure/adapter layer per the flat-clean-arch boundary.

---

## Crate boundary

`cloud-iac-domain` remains a pure domain crate:
- zero runtime dependencies beyond `std`
- `#![forbid(unsafe_code)]` inherited
- no I/O of any kind
- all types are `Clone + Debug + Eq + PartialEq`
