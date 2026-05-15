---
purpose: Auto-backfilled purpose for IP-002-multi-az-failover.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M05-P01-IP-002
title: Multi-AZ failover automation + quarterly drill
status: stub
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Multi-AZ failover automation with quarterly drill runbook.
---

# M05-P01-IP-002 — Multi-AZ failover automation + quarterly drill

## Purpose
Multi-AZ failover automation with quarterly drill runbook.

## Symbols-to-grit-claim
```
crates/oya-cloud-multi-az-failover-app/src/lib.rs::failover
docs/runbooks/multi-az-failover-drill.md::Procedure
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged.

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
icm store -t context-oyatie -c 'M05-P01-IP-002 Multi-AZ failover automation + quarterly drill shipped; acceptance commands green' -i high -k 'M05-P01-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
