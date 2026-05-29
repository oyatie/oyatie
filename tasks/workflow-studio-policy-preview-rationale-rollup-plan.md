# Plan: workflow-studio-policy-preview-rationale-rollup

## Objective
Extend `oya-workflow-studio-policy-preview-domain` with a deterministic severity-rollup and decision-rationale projection.

## Scope
Crate: `crates/oya-workflow-studio-policy-preview-domain`
No new workspace member, no new dependency.

## Changes

### 1. Add `WorkflowPolicyPreviewRollup` value object
- Fields: `info_count: usize`, `warning_count: usize`, `blocker_count: usize`
- Derived fields: `highest_severity: Option<WorkflowPolicyFindingSeverity>`, `blocking_finding_kinds: Vec<WorkflowPolicyFindingKind>` (sorted, deduped, only Blocker-severity kinds), `decision: WorkflowPolicyPreviewDecision`

### 2. Add `WorkflowPolicyPreviewReport::rollup()` method
- Pure projection over `self.findings` and `self.decision`
- Decision reconciliation:
  - Any Blocker finding => `Blocked`
  - Any Warning finding (and no Blocker) => `RequiresHumanReview`
  - No Warning or Blocker => `AllowPreview`
- `blocking_finding_kinds`: collect kinds where `severity == Blocker`, sort (BTreeSet deduplication), convert to Vec
- `highest_severity`: max severity present, or None if no findings

### 3. Tests (>=8 hermetic unit tests)
1. Empty findings => AllowPreview, zero counts, no highest_severity, empty blocking_finding_kinds
2. Only Info findings => AllowPreview, info_count > 0, warning/blocker = 0
3. Only Warning findings => RequiresHumanReview, blocking_finding_kinds empty
4. Only Blocker findings => Blocked, blocking_finding_kinds non-empty and sorted+deduped
5. Mixed Info+Warning => RequiresHumanReview
6. Mixed Info+Warning+Blocker => Blocked, blocking_finding_kinds sorted+deduped
7. LLM-draft + high-risk review path (report has warning findings => RequiresHumanReview)
8. Multiple Blocker findings of same kind => blocking_finding_kinds has no duplicates
9. Multiple Blocker findings of different kinds => blocking_finding_kinds sorted deterministically

## Acceptance criteria (from spec)
- rollup is consistent with report.decision for all three decisions
- blocking_finding_kinds is deterministically ordered and contains no duplicates
- empty-findings report rolls up to AllowPreview with zero counts
- >=8 hermetic unit tests covering each severity mix and the high-risk/llm-draft review paths
- cargo nextest passes for the crate with no new workspace member and no new dependency
