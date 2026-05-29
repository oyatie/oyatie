# Spec: workflow-studio-policy-preview-rationale-rollup

## Purpose
Provide a deterministic severity-rollup and decision-rationale projection from a
`WorkflowPolicyPreviewReport` without re-running policy evaluation or any I/O.

## New Types

### `WorkflowPolicyPreviewRollup`
```
pub struct WorkflowPolicyPreviewRollup {
    pub info_count: usize,
    pub warning_count: usize,
    pub blocker_count: usize,
    pub highest_severity: Option<WorkflowPolicyFindingSeverity>,
    pub blocking_finding_kinds: Vec<WorkflowPolicyFindingKind>,
    pub derived_decision: WorkflowPolicyPreviewDecision,
}
```

- `info_count`: count of findings with `severity == Info`
- `warning_count`: count of findings with `severity == Warning`
- `blocker_count`: count of findings with `severity == Blocker`
- `highest_severity`: `None` iff `findings` is empty; otherwise the maximum severity
  (Blocker > Warning > Info)
- `blocking_finding_kinds`: sorted (by `WorkflowPolicyFindingKind` natural Ord), deduplicated
  list of `kind` values from findings whose `severity == Blocker`. Empty if no Blockers.
- `derived_decision`:
  - `Blocked` if `blocker_count > 0`
  - `RequiresHumanReview` if `blocker_count == 0 && warning_count > 0`
  - `AllowPreview` if `blocker_count == 0 && warning_count == 0`

## New Method

```
impl WorkflowPolicyPreviewReport {
    pub fn rollup(&self) -> WorkflowPolicyPreviewRollup { ... }
}
```

Pure computation over `self.findings`. No I/O, no policy evaluation.

## Acceptance Criteria

1. Rollup derived_decision is consistent with report.decision for all three decision outcomes.
2. `blocking_finding_kinds` is deterministically ordered (BTreeSet dedup + sort) and contains no duplicates.
3. Empty-findings report rolls up to `AllowPreview` with zero counts and `highest_severity == None`.
4. At least 8 hermetic unit tests covering:
   - Empty findings -> AllowPreview, zero counts, None severity
   - Info-only findings -> AllowPreview, correct info_count
   - Warning-only findings -> RequiresHumanReview, correct warning_count
   - Blocker findings -> Blocked, correct blocker_count, blocking_finding_kinds populated
   - Mixed Info+Warning -> RequiresHumanReview
   - Mixed Warning+Blocker -> Blocked
   - LLM-draft review path (derived from preview_workflow_policy output)
   - High-risk review path (derived from preview_workflow_policy output)
   - Consistency: rollup().derived_decision == report.decision for produced reports

## Constraints
- Pure: no I/O, no external calls, no new dependencies
- No new workspace member
- Flat clean-arch (ADR-0509): all code in `src/lib.rs` (single-concern crate)
