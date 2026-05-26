---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P16-IP-002
title: Mermaid + D2 + Graphviz emitters
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Emit diagrams in Mermaid + D2 + Graphviz formats.
---

# M01-P16-IP-002 — Mermaid + D2 + Graphviz emitters

## Purpose
Emit diagrams in Mermaid + D2 + Graphviz formats.

## Symbols-to-grit-claim
```
crates/oya-intelligence-architecture-map-kernel/src/emit/mermaid.rs::Emitter
crates/oya-intelligence-architecture-map-kernel/src/emit/d2.rs::Emitter
crates/oya-intelligence-architecture-map-kernel/src/emit/graphviz.rs::Emitter
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

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
icm store -t context-oyatie -c 'M01-P16-IP-002 Mermaid + D2 + Graphviz emitters shipped; acceptance commands green' -i high -k 'M01-P16-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- One `sanitize_id` helper used by Mermaid + Graphviz — three emitters don't redo the same `[^A-Za-z0-9_] → _` rule.
- D2 uses quoted identifiers, sidestepping the sanitization step entirely — node ids with slashes stay human-readable.
- Output ordering is deterministic (nodes sorted by id, edges insertion-order) — diffs of the rendered diagram remain reviewable.
- Edge-label strings come from a single `match EdgeKind` per emitter — adding an edge variant flags every emitter at compile time.
- Empty-input cases are explicit (Mermaid → just header line; D2 → empty string; Graphviz → opening + closing braces) — no panics, no surprise output.
