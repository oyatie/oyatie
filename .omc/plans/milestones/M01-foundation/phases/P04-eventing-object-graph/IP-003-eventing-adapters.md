---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P04-IP-003
title: Provider adapter matrix + file-adapter foundation smoke
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Keep the provider-adapter matrix explicit while M01 proves the provider-neutral eventing boundary through the live file adapter.
---

# M01-P04-IP-003 — Provider adapter matrix + file-adapter foundation smoke

## Purpose
Keep the provider-adapter matrix explicit while M01 proves the provider-neutral eventing boundary through the live file adapter.

## Symbols-to-grit-claim
```
crates/oya-eventing-file-adapter/src/lib.rs::FileOutboxStore
crates/oya-eventing-domain/src/lib.rs::OutboxRecord
crates/oya-eventing-application/tests/eventing_outbox_publish.rs::outbox_publish_records_once_and_replays_same_idempotent_result
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test --locked -p oya-eventing-file-adapter
cargo test --locked -p oya-eventing-application --test eventing_outbox_publish
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
icm store -t context-oyatie -c 'M01-P04-IP-003 provider-adapter matrix tracked; live file-adapter foundation smoke green' -i high -k 'M01-P04-IP-003,eventing,file-adapter'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
