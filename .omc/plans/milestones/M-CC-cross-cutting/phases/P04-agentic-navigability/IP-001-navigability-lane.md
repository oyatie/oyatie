---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P04-IP-001
title: Agentic-navigability lane kernel + parent-pointer validator
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Lane CI-blocks missing INDEX.md / missing parent-pointer / undeclared symbols / undeclared purpose.
---

# M-CC-P04-IP-001 — Agentic-navigability lane kernel + parent-pointer validator

## Purpose
Lane CI-blocks missing INDEX.md / missing parent-pointer / undeclared symbols / undeclared purpose.

## Symbols-to-grit-claim
```
crates/oya-foundry-fitness-agentic-navigability-kernel/src/lib.rs::check
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged (except for IPs IN M-CC-P01 itself).

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M-CC-P04-IP-001 Agentic-navigability lane kernel + parent-pointer validator shipped; acceptance commands green' -i high -k 'M-CC-P04-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- `PlanNodeKind::requires_*` predicates replace N×M "if kind == X and field Y is missing" branches — the kernel walks one uniform loop.
- `NavigabilityViolationKind` enumerates all violation kinds in one place — adding a rule means one variant + one match arm.
- Empty/None purpose and empty/None parent-pointer are treated identically — runners cannot quietly skip the check by writing an empty string.
- Duplicate path is an error, not a silent dedupe — a bad runner cannot hide repeated nodes that mask broken parent pointers.
- The kernel is I/O-free; tree walkers, frontmatter parsers, and disk readers stay in runners.
