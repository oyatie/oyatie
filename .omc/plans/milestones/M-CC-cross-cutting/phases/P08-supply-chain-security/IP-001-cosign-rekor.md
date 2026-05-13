---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P08-IP-001
title: Cosign + Rekor signing pipeline
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Ship Cosign signing + Rekor anchoring for every artifact.
---

# M-CC-P08-IP-001 — Cosign + Rekor signing pipeline

## Purpose
Ship Cosign signing + Rekor anchoring for every artifact.

## Symbols-to-grit-claim
```
.github/workflows/cosign.yml::Workflow
crates/oya-foundry-fitness-supply-chain-kernel/src/lib.rs::check_signed
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
icm store -t context-oyatie -c 'M-CC-P08-IP-001 Cosign + Rekor signing pipeline shipped; acceptance commands green' -i high -k 'M-CC-P08-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
