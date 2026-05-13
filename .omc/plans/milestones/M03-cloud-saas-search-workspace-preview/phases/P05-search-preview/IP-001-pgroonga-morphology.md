---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P05-IP-001
title: pgroonga inverted index + KR/JP/EN morphology
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: pgroonga day-1 with mecab-ko / khaiii / JP / EN tokenizer pipelines.
---

# M03-P05-IP-001 — pgroonga inverted index + KR/JP/EN morphology

## Purpose
pgroonga day-1 with mecab-ko / khaiii / JP / EN tokenizer pipelines.

## Symbols-to-grit-claim
```
crates/oya-search-index-inverted-kernel/src/lib.rs::PgroongaIndex
crates/oya-search-parser-kernel/src/lib.rs::MorphologyPipeline
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
icm store -t context-oyatie -c 'M03-P05-IP-001 pgroonga inverted index + KR/JP/EN morphology shipped; acceptance commands green' -i high -k 'M03-P05-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
