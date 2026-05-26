---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P16-IP-001
title: oya-intelligence-architecture-map-kernel source walkers
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Walk Cargo metadata + contracts/ + docs/products/ + ROADMAP + ADR-INDEX + milestone frontmatter.
---

# M01-P16-IP-001 — oya-intelligence-architecture-map-kernel source walkers

## Purpose
Walk Cargo metadata + contracts/ + docs/products/ + ROADMAP + ADR-INDEX + milestone frontmatter.

## Symbols-to-grit-claim
```
crates/oya-intelligence-architecture-map-kernel/src/lib.rs::walk_cargo_metadata
crates/oya-intelligence-architecture-map-kernel/src/lib.rs::walk_openapi
crates/oya-intelligence-architecture-map-kernel/src/lib.rs::walk_frontmatter
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

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
icm store -t context-oyatie -c 'M01-P16-IP-001 oya-intelligence-architecture-map-kernel source walkers shipped; acceptance commands green' -i high -k 'M01-P16-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- Walkers consume **pre-parsed** typed records (`CargoPackage`, `OpenApiContractMeta`, `FrontmatterRecord`) — file I/O / serde / YAML parsing stays in runners; the kernel is testable in pure-std without a filesystem.
- `ArchitectureMap::merge` accepts exact-duplicate nodes — idempotent re-walks don't fail; only label/owning-team conflicts surface as errors.
- `walk_cargo_metadata` filters dependencies to in-workspace names only — external crates don't pollute the map; self-loops are dropped.
- `walk_openapi` synthesizes placeholder BC + Cedar-fragment nodes so its edges have valid endpoints in isolation; merge with the richer `walk_frontmatter` output is documented to use matching labels.
- Symbols ship in `src/walk.rs` as a submodule (`oya_intelligence_architecture_map_kernel::walk::walk_*`) rather than directly in `lib.rs` — keeps the existing `ArchitectureMap` model surface uncluttered.
