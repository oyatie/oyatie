# Plan: cloud-iac-opentofu-plan-changeset-model

**Lane:** infra
**Crate:** `oya-cloud-iac-domain` (the ONLY crate this task may touch)
**Branch:** `feat/cd-cloud-iac-opentofu-plan-changeset-model`
**Base:** `origin/dev`

---

## Objective

Add a pure OpenTofu plan-changeset domain model to `oya-cloud-iac-domain` alongside
the existing GitOps-drift reconciler and IaC plan-diff surface.

This models **OpenTofu plan output** (what `tofu plan` would emit), distinct from:
- GitOps drift reconciliation (Argo CD sync state vs desired)
- IaC plan-diff (topology desired vs observed module-ref set)

No I/O is introduced. The crate contract (no filesystem, network, OpenTofu CLI,
Argo CD API, or Kubernetes client) is preserved.

---

## Requirements Analysis

### Core taxonomy

OpenTofu plan output classifies each resource address with one of five actions:

| Action | Meaning |
|--------|---------|
| `Create` | Resource will be created (was absent) |
| `Update` | Resource exists, in-place update possible |
| `Delete` | Resource will be removed |
| `Replace` | Resource must be destroyed and re-created (implies destructive) |
| `NoOp` | Resource exists and is in-state; no change |

`Replace` is the critical distinction from the existing `PlanAction` (which has
`Destroy` not `Replace`). A replace involves a destroy+create and is therefore
always destructive even if both sub-operations succeed.

### Destructive change rule

`has_destructive_changes()` returns `true` if any `ResourceChange` has action
`Delete` or `Replace`. This is the safety gate that upstream callers check before
applying a plan.

### Deterministic summarize

`summarize()` returns a `PlanChangesetSummary` with integer counts per action type
and a total. Counts are always exact; summary is computed without I/O or randomness.

### Validation errors

- Resource address must be non-empty and must not contain whitespace or control chars
- Plan ID must be a non-empty, slug-safe identifier
- PlanChangeset with zero entries is valid (empty apply is a no-op)
- Duplicate resource addresses within one changeset are rejected

### Edge cases

- All NoOp entries: `has_destructive_changes()` = false, summary totals correctly
- Mix of actions including Replace: `has_destructive_changes()` = true
- Empty changeset: valid, no destructive changes, all summary counts zero
- Duplicate resource address: error at construction time
- Resource address with embedded secrets / path-traversal chars: validation rejects

---

## Subtasks (ordered)

### [pcs-1] Write plan + spec documents
- `tasks/cloud-iac-opentofu-plan-changeset-model-plan.md` (this file)
- `docs/specs/task-cloud-iac-opentofu-plan-changeset-model.md`

### [pcs-2] Write red-phase tests
- `crates/oya-cloud-iac-domain/tests/opentofu_plan_changeset.rs`
- Cover: all action variants, `has_destructive_changes`, `summarize`, validation
  errors, duplicate address rejection, empty changeset, determinism

### [pcs-3] Implement domain types and function
- Extend `crates/oya-cloud-iac-domain/src/lib.rs` with:
  - `ResourceChangeAction` enum
  - `ResourceChange` value object
  - `PlanChangeset` aggregate
  - `PlanChangesetSummary` read model
  - New `CloudIacError` variants for changeset validation

### [pcs-4] Verify green: cargo check + nextest
- `cargo check -p oya-cloud-iac-domain --all-targets`
- `cargo nextest run -p oya-cloud-iac-domain`

### [pcs-5] Self-review + simplify
- Correctness, security, performance, cloud-native readiness
- Fix Critical/High issues; re-run nextest

---

## Acceptance criteria

1. All new tests in `opentofu_plan_changeset.rs` pass green.
2. No existing tests regress.
3. `has_destructive_changes()` returns `true` iff any `Delete` or `Replace` present.
4. `summarize()` counts match the changeset entries exactly.
5. Duplicate resource addresses are rejected with `DuplicateResourceAddress`.
6. Empty resource address is rejected with `InvalidResourceAddress`.
7. `cargo nextest run -p oya-cloud-iac-domain` exits 0.
8. `git diff --stat origin/dev` touches only `oya-cloud-iac-domain` + two lane docs.
