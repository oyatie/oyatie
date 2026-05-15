---
purpose: Auto-backfilled purpose for IP-003-network-api-adapters.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P01-IP-003
title: Cloud Network VPC + LB + DNS + CDN + interconnect API
status: stub
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Bring cloud.network.* surfaces to stable; ≥2 provider adapters per surface.
---

# M03-P01-IP-003 — Cloud Network VPC + LB + DNS + CDN + interconnect API

## Purpose
Bring cloud.network.* surfaces to stable; ≥2 provider adapters per surface.

## Symbols-to-grit-claim
```
crates/oya-cloud-network-vpc-api/src/lib.rs::create
crates/oya-cloud-network-lb-api/src/lib.rs::create
crates/oya-cloud-network-dns-api/src/lib.rs::create_zone
crates/oya-cloud-network-cdn-api/src/lib.rs::create_distribution
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
icm store -t context-oyatie -c 'M03-P01-IP-003 Cloud Network VPC + LB + DNS + CDN + interconnect API shipped; acceptance commands green' -i high -k 'M03-P01-IP-003,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
