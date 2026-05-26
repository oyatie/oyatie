---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P16-IP-003
title: mdbook publishing integration
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Wire architecture-map output into mdbook publishing pipeline.
---

# M01-P16-IP-003 — mdbook publishing integration

## Purpose
Wire architecture-map output into mdbook publishing pipeline.

## Symbols-to-grit-claim
```
crates/oya-intelligence-mdbook-kernel/src/lib.rs::wire_architecture_map
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

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

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M01-P16-IP-003 mdbook publishing integration shipped; acceptance commands green' -i high -k 'M01-P16-IP-003,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- `wire_architecture_map` returns a new site (consume + replace) — the published-site value cannot be partially mutated; either both chapters list AND kind counts update, or neither.
- Empty `architecture_map_path` is `Err`, not "silently appends an empty chapter" — a misconfigured runner fails loudly.
- Reuses the same `(kind, path)` sort key as `walk_sources` so wiring the map doesn't disturb the published order beyond inserting the new chapter.
