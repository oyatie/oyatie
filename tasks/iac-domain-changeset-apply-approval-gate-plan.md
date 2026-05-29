# Plan: iac-domain-changeset-apply-approval-gate

## Objective

Add a pure tiered apply-approval gating kernel to `oya-cloud-iac-domain`.

Introduce `ApplyApprovalVerdict` and `PlanChangeset::approval_gate(&self) -> ApplyApprovalVerdict`
that escalates from existing `has_destructive_changes`/`summarize` signals.

## Requirements Analysis

### Core logic

| Changeset profile | Verdict |
|---|---|
| empty / no-op only (all NoOp or zero changes) | `AutoApprove` |
| creates + updates only, blast-radius < threshold | `AutoApprove` |
| creates + updates only, blast-radius >= threshold | `RequiresReview { required_approvals: 1 }` |
| any Delete or Replace | `RequiresReview { required_approvals }` scaled on destructive blast radius |
| never explicitly blocked in spec; `Blocked` variant reserved for future policy hooks | `Blocked` |

### Blast-radius thresholds (deterministic, no clocks/I/O)

- `destructive_count = delete_count + replace_count` from `PlanChangesetSummary`
- `required_approvals` scaling (monotonic in destructive count):
  - 1–5 destructive → 1 approval
  - 6–20 destructive → 2 approvals
  - 21+ destructive → 3 approvals
- Non-destructive threshold: `create_count + update_count >= 50` triggers `RequiresReview { required_approvals: 1 }`

### Constraints

- Pure function, deterministic, no clocks/I/O
- Must NOT alter `compute_iac_plan_diff`, `summarize`, or `has_destructive_changes`
- Implemented as a method on `PlanChangeset` (same file, flat mod)

## Acceptance Criteria

- (a) empty/no-op changeset → `AutoApprove`
- (b) creates-only under threshold → `AutoApprove`
- (c) single delete → `RequiresReview { required_approvals >= 1 }`
- (d) large delete+replace blast radius → higher `required_approvals` (monotonic)
- (e) determinism: same input → same output (pure function)

## Subtasks

1. [x] Write plan doc (this file)
2. [ ] Write spec doc (`docs/specs/task-iac-domain-changeset-apply-approval-gate.md`)
3. [ ] Add `ApplyApprovalVerdict` enum and `approval_gate` method (red: tests written first, cargo check --no-run fails)
4. [ ] Implement `approval_gate` (green: tests pass)
5. [ ] Self-review + simplify
6. [ ] Commit and push PR
