---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P06-IP-002
title: Flat-crates guard ratchet
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship the flat-crates guard per ADR-0015.
---

# M01-P06-IP-002 — Flat-crates guard ratchet

## Purpose
Ship the flat-crates guard per ADR-0015.

## Symbols-to-grit-claim
```
crates/oya-intelligence-cargo-prefix-domain/src/lib.rs::validate_cargo_prefix
crates/oya-dev-cli/src/commands/gate.rs::cargo-prefix
crates/oya-shared-architecture-check-cli/src/main.rs::naming-collision
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test --locked -p oya-intelligence-cargo-prefix-domain
cargo run --quiet -p oya-dev-cli -- gate validate cargo-prefix --workspace Cargo.toml --prefix oya-
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
icm store -t context-oyatie -c 'M01-P06-IP-002 Flat-crates guard ratchet shipped; acceptance commands green' -i high -k 'M01-P06-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
