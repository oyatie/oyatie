---
doc_class: ImplementationPlan
impl_plan_id: IP-013-ai-assist-and-e2e-refusal
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + axis-foundry-runtime + council-privacy
acceptance_lanes: [cargo-check, cargo-test, oya-governance-e2e-ai-refusal, oya-governance-dual-context-isolation]
---

# IP-013: ai-assist + E2E refusal CI lane

## Intent

Land `oya-notes-ai-assist-{kernel,domain,usecase,api,adapter,worker,sdk,app}` (T0 + T1 capabilities; T2 stub-but-disabled). Author the `oya-check-e2e-ai-refusal` CI lane and register it BLOCKER on `dev` per ADR-NOTES-0005.

## Type-System Invariant

```rust
pub trait AssistInvoker {
    fn invoke(&self, note: ProfessionalNoteRef, request: AssistRequest) -> Result<AssistResult, AssistError>;
    // No method accepting PersonalNoteRef. Period.
}
```

## Cedar Policy

`policy/tenant-scope.cedar` already carries:

```cedar
forbid (principal, action == Action::"invoke_ai_assist", resource in Note::?m)
when { resource has context_kind && resource.context_kind == "Personal" };
```

## CI Lane

`crates/oya-check-e2e-ai-refusal/`: AST + control-flow analysis verifying no path from `PersonalNoteRef` → `AssistInvoker::invoke`.

## Regression Suite

`tests/regression/e2e-ai-refusal/`:
- type-system: `compile_fail` test attempting to construct `PersonalNoteRef → AssistInvoker::invoke`.
- runtime: Cedar evaluation returns deny on Personal resource.
- CI lane: lane exit 0 with no findings.
- runtime metric: `oya_notes_ai_call_blocked_e2e_total` increments on attempted call.

## Acceptance Gates

```bash
cargo check -p oya-notes-ai-assist-kernel
cargo test --test e2e-ai-refusal
cargo run -p oya-dev-cli -- gate validate e2e-ai-refusal --microservice notes
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice notes
```

## Halt Conditions

- e2e-ai-refusal lane returns any finding — BLOCK PR.
- runtime metric increments in any non-test environment — Sev-1.

## Next IP

[`IP-014-e2e-key-management.md`](IP-014-e2e-key-management.md)
