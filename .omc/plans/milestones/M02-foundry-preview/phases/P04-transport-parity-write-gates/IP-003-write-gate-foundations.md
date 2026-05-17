---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M02-P04-IP-003
title: Write-gate foundations ADR + state machine
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Author ADR-XXXX Foundry write-gate foundations (Phase 05 contract).
---

# M02-P04-IP-003 — Write-gate foundations ADR + state machine

## Purpose
Author ADR-XXXX Foundry write-gate foundations (Phase 05 contract).

## Symbols-to-grit-claim
```
docs/decisions/ADR-XXXX-foundry-write-gate-foundations.md::Decision
crates/oya-foundry-write-gate-kernel/src/lib.rs::WriteGate
crates/oya-foundry-write-gate-kernel/src/lib.rs::ApprovalEnvelope
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

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
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M02-P04-IP-003 Write-gate foundations ADR + state machine shipped; acceptance commands green' -i high -k 'M02-P04-IP-003,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
