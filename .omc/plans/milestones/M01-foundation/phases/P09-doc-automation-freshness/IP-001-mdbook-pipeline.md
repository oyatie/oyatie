---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P09-IP-001
title: mdbook publishing pipeline kernel + source walkers
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship mdbook kernel that walks rustdoc + OpenAPI + ADR-INDEX + frontmatter.
---

# M01-P09-IP-001 — mdbook publishing pipeline kernel + source walkers

## Purpose
Ship mdbook kernel that walks rustdoc + OpenAPI + ADR-INDEX + frontmatter.

## Symbols-to-grit-claim
```
crates/oya-intelligence-mdbook-kernel/src/lib.rs::walk_sources
crates/oya-intelligence-mdbook-kernel/src/lib.rs::publish_site
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
icm store -t context-oyatie -c 'M01-P09-IP-001 mdbook publishing pipeline kernel + source walkers shipped; acceptance commands green' -i high -k 'M01-P09-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- One `SourceKind::chapter_prefix` predicate places each kind under a stable section — runners don't pick chapter paths.
- `walk_sources` validates path + title + uniqueness before producing chapters — a malformed runner cannot leak invalid entries into the published book.
- Chapters are sorted by `(kind, path)` — identical inputs always emit byte-identical `SUMMARY.md` (deterministic for diffs/CI).
- `publish_site` is a pure function over the chapter tree; renderers (mdbook serializers) live in runners.
