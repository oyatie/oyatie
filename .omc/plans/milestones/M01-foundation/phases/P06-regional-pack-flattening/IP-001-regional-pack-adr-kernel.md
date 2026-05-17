---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P06-IP-001
title: Regional Pack ADR + kernel
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Author the regional-pack ADR + ship the kernel that pack contracts plug into.
---

# M01-P06-IP-001 — Regional Pack ADR + kernel

## Purpose
Author the regional-pack ADR + ship the kernel that pack contracts plug into.

## Symbols-to-grit-claim
```
docs/decisions/ADR-0010-regional-pack-architecture.md::Decision
crates/oya-regional-pack-domain/src/lib.rs::RegionalPack
crates/oya-regional-pack-domain/src/lib.rs::RegionalPackError
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test --locked -p oya-regional-pack-domain
cargo clippy --locked -p oya-regional-pack-domain --all-targets -- -D warnings
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
icm store -t context-oyatie -c 'M01-P06-IP-001 Regional Pack ADR + kernel shipped; acceptance commands green' -i high -k 'M01-P06-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
