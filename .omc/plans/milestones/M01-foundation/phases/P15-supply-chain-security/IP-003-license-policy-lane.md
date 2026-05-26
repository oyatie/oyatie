---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P15-IP-003
title: License-policy lane (AGPL/GPL/SSPL/BUSL/RSAL hard-deny)
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Lane hard-denies AGPL/GPL/SSPL/BUSL/RSAL in product code.
---

# M01-P15-IP-003 — License-policy lane (AGPL/GPL/SSPL/BUSL/RSAL hard-deny)

## Purpose
Lane hard-denies AGPL/GPL/SSPL/BUSL/RSAL in product code.

## Symbols-to-grit-claim
```
crates/oya-governance-license-policy-kernel/src/lib.rs::check
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
icm store -t context-oyatie -c 'M01-P15-IP-003 License-policy lane (AGPL/GPL/SSPL/BUSL/RSAL hard-deny) shipped; acceptance commands green' -i high -k 'M01-P15-IP-003,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- `denied` is checked before `allowed` in the OR branch — a license in both lists is still blocked (deny wins), removing a "which takes precedence" ambiguity.
- Compound SPDX (`A OR B`, `A AND B`) is parsed once into one of three operators (`Single`/`Or`/`And`) — downstream logic is a single uniform iteration.
- Mixed `OR` + `AND` without parens is `Unparseable` — refuses to guess precedence, so a malformed manifest fails closed.
- Empty/whitespace license string is an explicit `EmptyLicense` violation — a missing field never reads as "anonymous OK".
- Empty allowlist is `Err`, not `0 violations` — a misconfigured policy cannot silently let everything through.
