---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P08-IP-012
title: Authoritative-tracked repo-walk audit (P10)
status: complete
migration_status: cleanup
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship authoritative-tracked lane that walks docs/AGENTS.md authoritative list and verifies all are tracked.
---

# M01-P08-IP-012 — Authoritative-tracked repo-walk audit (P10)

## Purpose
Ship authoritative-tracked lane that walks docs/AGENTS.md authoritative list and verifies all are tracked.

## Symbols-to-grit-claim
```
crates/oya-governance-authoritative-tracked-kernel/src/lib.rs::check
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-governance-cohesion -- <owning-crate-glob>
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
icm store -t context-oyatie -c 'M01-P08-IP-012 Authoritative-tracked repo-walk audit (P10) shipped; acceptance commands green' -i high -k 'M01-P08-IP-012,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: authoritative tracking is checked from one typed path list parsed from `docs/AGENTS.md` rather than ad hoc per-doc exceptions; directories pass only when at least one tracked child exists.
