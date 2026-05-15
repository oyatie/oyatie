---
purpose: Block non-distroless bases + shells/package-managers + oversized images.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P06-IP-001
title: Distroless base + image-discipline lane
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Block non-distroless bases + shells/package-managers + oversized images.
---

# M-CC-P06-IP-001 — Distroless base + image-discipline lane

## Purpose
Block non-distroless bases + shells/package-managers + oversized images.

## Symbols-to-grit-claim
```
crates/oya-foundry-fitness-image-discipline-kernel/src/lib.rs::check
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
icm store -t context-oyatie -c 'M-CC-P06-IP-001 Distroless base + image-discipline lane shipped; acceptance commands green' -i high -k 'M-CC-P06-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- `DISTROLESS_PREFIXES` is a single `const` array — adding a new sanctioned base (e.g., another Chainguard root) is a one-line change.
- `base_ref` strips `:tag` and `@digest` once at the top, so the prefix-match loop never has to think about tag variations.
- Missing budget is a violation (`MissingBudget`), not a silent skip — a forgotten size budget cannot let an image pass unnoticed.
- Forbidden final-layer paths use exact-match (not substring) — a future `/opt/legitimate/apt-config` won't false-positive as `apt`.
- Duplicate budgets are `Err`, not last-write-wins — a stale or conflicting policy cannot quietly raise the ceiling.
