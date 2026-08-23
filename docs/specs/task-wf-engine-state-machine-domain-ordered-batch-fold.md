# Spec: wf-engine-state-machine-domain-ordered-batch-fold

## Objective

Extend `workflow-engine-state-machine-domain` with a pure deterministic
`evaluate_domain_transition_batch` function that folds an ordered sequence of
`WorkflowStateMachineDomainRequest` values, threading state between steps, and
returns a single `DomainTransitionDecision` representing the aggregate outcome.

## Contracts

- **No external standards** (no auth, crypto, email, k8s API, OpenAPI, proto, SLO)
  are touched by this slice — it is a pure in-memory domain fold.
- **Inputs**: `initial_checkpoint: Option<StateCheckpoint>`,
  `requests: Vec<WorkflowStateMachineDomainRequest>`
- **Output**: `DomainTransitionDecision`
  - `Applied(DomainTransitionReceipt)` — terminal checkpoint, accumulated
    sorted-unique `audit_refs` across all applied steps, `origin` from the last applied
    element.
  - `Denied(DomainTransitionDenial)` — denial from the first failing element, with
    `workflow-state-machine-domain:batch-index:N` injected into `audit_refs`.

## Mod layout (flat-clean-arch)

All logic lives in `crates/workflow-engine-state-machine-domain/src/lib.rs`.
No new modules, no new files, no new crates.

## Implementation contract

```
pub fn evaluate_domain_transition_batch(
    initial_checkpoint: Option<StateCheckpoint>,
    requests: Vec<WorkflowStateMachineDomainRequest>,
) -> DomainTransitionDecision
```

Algorithm:
1. If `requests` is empty → return `Applied(DomainTransitionReceipt { checkpoint: initial_checkpoint.unwrap_or_default(), ... })`.
   - Since `StateCheckpoint` has no `Default`, use a sentinel: return the initial checkpoint
     from the caller, but as there is no checkpoint when the batch is empty, this needs
     careful handling. Per acceptance criterion 5: "empty batch returns the unchanged
     initial checkpoint as a no-op Applied". The function will require
     `initial_checkpoint` to be `Some` for the empty case to return a meaningful receipt,
     OR return an `Applied` with a synthetic identity receipt. **Decision**: for empty
     batch, return `Applied` with the initial checkpoint forwarded; since
     `DomainTransitionReceipt` requires a `StateCheckpoint`, the function signature will
     accept `initial_checkpoint: Option<StateCheckpoint>` and for the empty batch case
     return an `Applied` receipt synthesised from the initial checkpoint. If
     `initial_checkpoint` is `None` and the batch is empty, that is a no-op `Applied`
     with a minimal synthetic receipt (the acceptance test will provide a checkpoint).
2. For each `(index, request)` in `requests`:
   a. Override `request.current_checkpoint` with the threaded checkpoint.
   b. Call `evaluate_domain_transition(request)`.
   c. If `Applied` → accumulate `audit_refs`, update threaded checkpoint, continue.
   d. If `Denied` → inject `workflow-state-machine-domain:batch-index:{index}` into
      `denial.audit_refs` and return `Denied(denial)` immediately.
3. Merge all accumulated `audit_refs` with `sorted_unique`.
4. Return `Applied(DomainTransitionReceipt { checkpoint, origin, audit_refs })`.

## Testing strategy

Unit tests only, hermetic, no I/O, no clock, no random.

### Test cases

1. `batch_happy_path_start_step_started_step_completed_completed` — full lifecycle
   batch; assert terminal `WorkflowRunStatus::Completed`, sorted-unique `audit_refs`.
2. `batch_denial_at_third_element_halts_and_carries_batch_index` — 3-element batch
   where 3rd element has scope mismatch; assert `Denied` at index 2 with correct
   `DomainTransitionDenialKind::ScopeMismatch` and `batch-index:2` audit_ref.
3. `batch_kernel_denial_mid_batch_is_preserved_with_batch_index` — apply start, then
   attempt a step-started on a terminal checkpoint mid-batch; assert `KernelDenied`
   with kernel reason + batch index.
4. `batch_equals_sequential_evaluate_domain_transition` — manually fold
   `evaluate_domain_transition` one-by-one and compare to batch result.
5. `batch_empty_returns_initial_checkpoint_as_applied` — empty `Vec`, some initial
   checkpoint, assert `Applied` with that checkpoint.

## Observability / SLO

No new SLO instrumentation is introduced in this pure domain slice (no I/O surface).
Existing `audit_refs` accumulation provides the auditability surface.

## Crate boundary

Changes are confined to:
- `crates/workflow-engine-state-machine-domain/src/lib.rs`

No other crates are modified.
