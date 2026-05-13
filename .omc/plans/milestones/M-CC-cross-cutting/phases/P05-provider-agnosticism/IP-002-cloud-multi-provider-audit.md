---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P05-IP-002
title: Multi-provider adapter audit + remediation across Cloud kernels
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Audit Cloud kernels; ensure ≥2 adapters per kernel.
---

# M-CC-P05-IP-002 — Multi-provider adapter audit + remediation across Cloud kernels

## Purpose
Audit Cloud kernels; ensure ≥2 adapters per kernel.

## Symbols-to-grit-claim
```
crates/oya-cloud-*-adapter-{aws,oci,gcp,azure,naver,kt,kakao}-*::Adapter
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
icm store -t context-oyatie -c 'M-CC-P05-IP-002 Multi-provider adapter audit + remediation across Cloud kernels shipped; acceptance commands green' -i high -k 'M-CC-P05-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
