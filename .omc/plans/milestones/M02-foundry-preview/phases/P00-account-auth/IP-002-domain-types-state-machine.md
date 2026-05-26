---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M02-P00-IP-002
title: Domain types + state machine + 40+ unit tests (P00-02)
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship ProviderAccount, AuthSession, UsageWindow, SecretReference, ProviderFamily allowlist with state-machine Draft→Verified→Active→Degraded→Disabled→Revoked.
---

# M02-P00-IP-002 — Domain types + state machine + 40+ unit tests (P00-02)

## Purpose
Ship ProviderAccount, AuthSession, UsageWindow, SecretReference, ProviderFamily allowlist with state-machine Draft→Verified→Active→Degraded→Disabled→Revoked.

## Symbols-to-grit-claim
```
crates/oya-intelligence-account-domain/src/lib.rs::ProviderAccount
crates/oya-intelligence-account-domain/src/lib.rs::AuthSession
crates/oya-intelligence-account-domain/src/lib.rs::UsageWindow
crates/oya-intelligence-account-domain/src/lib.rs::SecretReference
crates/oya-intelligence-account-domain/src/lib.rs::ProviderFamily
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

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
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M02-P00-IP-002 Domain types + state machine + 40+ unit tests (P00-02) shipped; acceptance commands green' -i high -k 'M02-P00-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- One `transition` method on each state owner replaces N×N per-edge guard checks — forbidden edges return a single `InvalidTransition { from, to }` carrying both ends.
- `Revoked` is modeled as terminal in the type itself (`is_terminal`), not by scattered "if state == Revoked" checks across the codebase.
- `Degraded { reason }` and `Disabled { reason }` carry their reason in the variant — callers can't lose the cause across boundaries.
- `check_silent_switch` is the single chokepoint for the silent-account-switch invariant — any new caller is forced through one audit point.
- `ProviderFamily::try_from` rejects unknown families with the original string in the error, so misconfigured registry entries surface their bad value.
