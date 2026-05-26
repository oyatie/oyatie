---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P09-IP-002
title: Doc-freshness CI lane
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Block PRs that change source-of-truth without regenerating dependent docs.
---

# M01-P09-IP-002 — Doc-freshness CI lane

## Purpose
Block PRs that change source-of-truth without regenerating dependent docs.

## Symbols-to-grit-claim
```
crates/oya-governance-doc-freshness-kernel/src/lib.rs::check
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
icm store -t context-oyatie -c 'M01-P09-IP-002 Doc-freshness CI lane shipped; acceptance commands green' -i high -k 'M01-P09-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- One `SourceDependency { source, dependent }` row models the rule — adding a new source-of-truth → doc mapping is one append, not a scattered `match`.
- Self-dependencies are an explicit `Err`, not silently dropped — a misconfigured rule fails the build instead of hiding.
- `sources_changed` count is a `BTreeSet` so duplicate rules over the same source don't inflate the metric.
- Stale-doc output is sorted — diff-stable across runs.
