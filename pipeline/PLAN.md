---
doc_class: Owner-PLAN
owner: pipeline
status: Active
date: 2026-08-28
---

# Pipeline declaration-integrity sequence

<baseline>

## What has landed

Pipeline currently has protected repository-layout admission, a draft Git read
port, path/layout/occupancy rules, and `presubmit` fan-in. It has not landed a
versioned SCM-neutral snapshot or owner-fact contract, a Build declaration
invocation, ChangeSet application, qualification runtime, repair proposal, or
declaration enforcement.

This owner law depends on merged ADR-0719 D17a. It creates no code, Cargo or
lockfile change, workflow, Build contract, parser, native-query harness, or
generated artifact.

</baseline>

<sequence>

## 1. Establish the owner boundary

Class: documentation authority only, completed by this four-file PR after D17a.

- Keep the write envelope to `pipeline/{ADR,PRD,SPEC,PLAN}.md`.
- Record Pipeline's fact, protected-invocation, ChangeSet, and orchestration
  boundaries without describing Build semantics or current behavior as landed.
- Success: all four documents agree; hostile review cannot derive a second
  workflow, parser, native-query execution path, or implementation claim.
- Failure: a root/Build/workflow/code edit, Build semantic prescription, or
  apparent present-tense implementation authority.
- Rollback: revert these four documents only; no other owner is changed.

## 2. Establish prerequisites before parser work

Class: separate Build and Pipeline structural lanes.

- Finish third-party reconciliation and prove deterministic safe regeneration
  of `third-party/BUCK`.
- Amend the six-package execution map and Pipeline package admission before a
  parser port, parser adapter, root manifest, or lockfile change.
- Success: third-party output is qualified and structural admission authorizes
  the future packages without inventing first-party semantics.
- Failure: hand-edited generated BUCK, parser dependency before the prerequisite,
  or a shared structural/semantic lane.
- Rollback: withdraw the incomplete prerequisite lane; this owner law remains.

## 3. Implement facts and ChangeSets separately

Class: future test-driven Pipeline port/core/adapter lanes after their exact
packages and shared contracts are independently approved.

- Add immutable SCM-neutral repository facts and caller-resolved owner facts.
- Add the canonical ChangeSet wrapper that preserves each source RepairSet
  identity, whole-set digest, and exact owner-group identity; reject incomplete,
  duplicate, ambiguous, or overlapping owner-group partitions. Then add full
  fact recheck, disjoint-successor decision, atomic successor construction, and
  protected repair-PR publication.
- Verify immutable-object, lossless-delta, owner ambiguity, semantic mismatch,
  disjoint successor, retry, and every compare/publish fault boundary.
- Rollback: retain existing layout admission and withdraw an unrouted port or
  nonapplying repair result; never fall back through partial mutation.

## 4. Integrate and qualify the Build contract

Class: future protected facade lane after Build's versioned check-only contract
and complete-head extractor qualification exist.

- Invoke one protected Build result through the existing layout-admission seam;
  preserve opaque output and one `presubmit` fan-in.
- Consume separate protected differential-harness evidence, repair current drift
  through one owner canary then disjoint shards, and replay clean complete HEAD.
- Activate only after adversarial qualification and zero detected drift. Prove a
  Cargo-only or BUCK-only declaration change refuses existing `presubmit`
  without native query or a second candidate compile.
- Rollback: before activation, withdraw the unrouted invocation. After a future
  activation, refuse affected changes until a separately qualified replacement
  is available; do not silently return to shadow-only admission.

</sequence>

<verification>

## Verification for this owner-law PR

- Confirm exactly the four owner-law paths changed and each is at most 300
  physical lines.
- Run `git diff --check`, `cargo fmt --all --check`, and the focused protected
  execution-context and workflow tests in `pipeline-admission`.
- Obtain independent Pipeline and Build-boundary review; green CI is evidence,
  never approval or merge authority.

</verification>
