---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P09-IP-004
title: oya-foundry-fitness-architecture-map-freshness lane
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Lane blocks PRs that change source-of-truth without regenerating visualizations.
---

# M-CC-P09-IP-004 — oya-foundry-fitness-architecture-map-freshness lane

## Purpose
Lane blocks PRs that change source-of-truth without regenerating visualizations.

## Symbols-to-grit-claim
```
crates/oya-foundry-fitness-architecture-map-freshness-kernel/src/lib.rs::check
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged (except for IPs IN M-CC-P01 itself).

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
icm store -t context-oyatie -c 'M-CC-P09-IP-004 oya-foundry-fitness-architecture-map-freshness lane shipped; acceptance commands green' -i high -k 'M-CC-P09-IP-004,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
