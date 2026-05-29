# Plan: wf-engine-state-machine-domain-ordered-batch-fold

## Objective

Add `evaluate_domain_transition_batch` to `oya-workflow-engine-state-machine-domain`: a
pure deterministic fold over an ordered `Vec<WorkflowStateMachineDomainRequest>` that
threads each `Applied` checkpoint as the `current_checkpoint` of the next request, halts
on the first `Denied` with a batch-position-indexed denial, and returns an aggregate
receipt containing the terminal checkpoint and accumulated sorted-unique audit_refs.

## Subtasks (ordered)

1. **Write the plan document** (this file).
2. **Write the spec document** `docs/specs/task-wf-engine-state-machine-domain-ordered-batch-fold.md`.
3. **Write failing tests (red phase)** covering all five acceptance criteria.
4. **Implement `evaluate_domain_transition_batch`** (green phase) — minimum viable
   addition inside `src/lib.rs` reusing existing helpers with no new abstractions.
5. **Self-review** — correctness / architecture / security / performance.
6. **Simplify** — guard clauses, dead-code removal, naming; re-run nextest.
7. **Commit** only the allowed paths and push.

## Edge cases

- Empty batch → return initial checkpoint wrapped in `Applied` with no new audit_refs.
- First element fails → `Denied` at batch index 0.
- Denial must carry `workflow-state-machine-domain:batch-index:N` audit_ref.
- Later elements MUST NOT execute after first denial.
- Determinism: applying elements one-by-one must equal calling `evaluate_domain_transition_batch`.
- Kernel mid-batch denial (e.g. terminal-state-refuses-event) surfaces as `KernelDenied`
  with batch index.
- Sequence drift mid-batch is preserved by delegating to `evaluate_domain_transition`.

## Acceptance criteria

1. Happy-path start→step-started→step-completed→completed batch yields one aggregate
   `Applied` receipt with terminal `WorkflowRunStatus` and deduped sorted `audit_refs`.
2. Batch whose 3rd element fails scope/evidence/unsafe-metadata returns `Denied` carrying
   batch index 2 and the underlying `DomainTransitionDenialKind`; no later elements apply.
3. Sequence-drift / terminal-refusal mid-batch is preserved as `KernelDenied` with the
   kernel reason and batch index.
4. `evaluate_domain_transition_batch` is byte-identical-deterministic across repeated
   runs and equals applying `evaluate_domain_transition` one-by-one.
5. Empty batch returns unchanged initial checkpoint as no-op `Applied`.

## Architecture notes

- One function in `src/lib.rs` (flat-clean-arch, single mod).
- New struct: `BatchDomainTransitionReceipt` — `checkpoint`, `origin` (from last applied),
  `audit_refs` (accumulated sorted-unique across all applied elements).
- New enum variant not needed — `DomainTransitionDecision` re-used as return type via a
  new `BatchDomainTransitionDecision` wrapper.
- Actually: return `DomainTransitionDecision` directly for simplicity — on success an
  `Applied(DomainTransitionReceipt)` with merged audit_refs; on failure a `Denied` with
  batch-index audit_ref injected.
- No I/O, no clock, no random.
