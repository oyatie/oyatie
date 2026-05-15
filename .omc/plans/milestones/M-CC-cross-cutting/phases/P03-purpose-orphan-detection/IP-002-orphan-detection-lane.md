---
purpose: Auto-backfilled purpose for IP-002-orphan-detection-lane.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-002
title: Orphan-detection lane kernel
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Lane CI-blocks orphan artifacts (no inbound reference; not in known-orphan allowlist).
---

# M-CC-P03-IP-002 — Orphan-detection lane kernel

## Purpose
Lane CI-blocks orphan artifacts (no inbound reference; not in known-orphan allowlist).

## Symbols-to-grit-claim
```
crates/oya-foundry-fitness-orphan-detection-kernel/src/lib.rs::detect_orphans
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
icm store -t context-oyatie -c 'M-CC-P03-IP-002 Orphan-detection lane kernel shipped; acceptance commands green' -i high -k 'M-CC-P03-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- A single BFS from declared roots replaces N separate "is this node reachable" checks — cycles, diamonds, multi-roots all flow through one loop.
- Dangling references are surfaced as `Err`, not silently dropped — a broken pointer cannot create false-negative orphans.
- Duplicate-node detection happens at insert time — a stale `find` walker generating dup paths cannot mask a real orphan.
- No-roots case is explicit `Err`, not "everything is an orphan" — runners cannot lose their root config and get a green report.
- Orphan output is sorted — diff-stable across runs, no flaky CI failures from ordering.
