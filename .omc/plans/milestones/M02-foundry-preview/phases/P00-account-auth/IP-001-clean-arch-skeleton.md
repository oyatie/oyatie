---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M02-P00-IP-001
title: Clean Architecture skeleton + 7-crate scaffold-claim (ADR-0054 path)
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Scaffold the 7 oya-foundry-account-* crates per P00-01.
---

# M02-P00-IP-001 — Clean Architecture skeleton + 7-crate scaffold-claim (ADR-0054 path)

## Acceptance Criteria

- **AC-001**: All 7 `oya-foundry-account-*` crates scaffold successfully and compile with zero errors.
  - test_id: `cargo check -p oya-intelligence-account-kernel -p oya-intelligence-account-domain -p oya-foundry-account-app -p oya-foundry-account-adapter-codex-cli -p oya-foundry-account-adapter-claude-code -p oya-foundry-account-adapter-gemini-cli -p oya-foundry-account-adapter-openbao -p oya-foundry-account-runtime`
  - verification_command: `cargo check --workspace --all-features`
- **AC-002**: Cohesion fitness lane passes for all 7 new crates.
  - test_id: `oya gate validate cohesion`
  - verification_command: `cargo run -p oya-dev-cli -- gate validate cohesion`
- **AC-003**: No provider-specific deps outside adapter crates (Directive 4 compliance).
  - test_id: `oya gate validate architecture-boundaries`
  - verification_command: `cargo run -p oya-dev-cli -- gate validate architecture-boundaries`
- **AC-004**: All direct deps are current LTS or carry an ADR-tracked exception (Directive 8).
  - test_id: `oya gate validate supply-chain`
  - verification_command: `cargo run -p oya-dev-cli -- gate validate supply-chain`
- **AC-005**: `cargo test -p <each-new-crate> --all-features` returns 0 for all 7 crates.
  - test_id: `cargo nextest run -p oya-intelligence-account-kernel --all-features`
  - verification_command: `cargo nextest run --workspace --all-features`
- **AC-006**: Naming justification BNF v4 comment present in each new crate's `lib.rs`.
  - test_id: `oya gate validate cargo-prefix`
  - verification_command: `cargo run -p oya-dev-cli -- gate validate cargo-prefix`
  - status: pending-spec-author

## Purpose
Scaffold the 7 oya-foundry-account-* crates per P00-01.

## Symbols-to-grit-claim
```
crates/oya-intelligence-account-kernel/src/lib.rs::placeholder
crates/oya-intelligence-account-domain/src/lib.rs::placeholder
crates/oya-foundry-account-app/src/lib.rs::placeholder
crates/oya-foundry-account-adapter-codex-cli/src/lib.rs::placeholder
crates/oya-foundry-account-adapter-claude-code/src/lib.rs::placeholder
crates/oya-foundry-account-adapter-gemini-cli/src/lib.rs::placeholder
crates/oya-foundry-account-adapter-openbao/src/lib.rs::placeholder
crates/oya-foundry-account-runtime/src/lib.rs::placeholder
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
icm store -t context-oyatie -c 'M02-P00-IP-001 Clean Architecture skeleton + 7-crate scaffold-claim (ADR-0054 path) shipped; acceptance commands green' -i high -k 'M02-P00-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
