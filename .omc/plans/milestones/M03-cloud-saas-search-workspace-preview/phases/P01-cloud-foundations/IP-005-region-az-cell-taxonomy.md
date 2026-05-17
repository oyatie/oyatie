---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P01-IP-005
title: Cloud region + AZ + cell taxonomy
status: stub
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Stable cloud.region.list + cloud.az.list with cell-isolation evidence per cell.
---

# M03-P01-IP-005 — Cloud region + AZ + cell taxonomy

## Purpose
Stable cloud.region.list + cloud.az.list with cell-isolation evidence per cell.

## Symbols-to-grit-claim
```
crates/oya-cloud-region-api/src/lib.rs::list_regions
crates/oya-cloud-region-api/src/lib.rs::list_azs
crates/oya-cloud-region-app/src/lib.rs::cell_isolation_evidence
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
icm store -t context-oyatie -c 'M03-P01-IP-005 Cloud region + AZ + cell taxonomy shipped; acceptance commands green' -i high -k 'M03-P01-IP-005,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
