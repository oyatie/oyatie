---
purpose: Author and ratify ADR-0008 (Data Use Boundary) — the P0 prereq per PRD §6 constraint 8.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P01-IP-001
title: Data Use Boundary ADR-0008 authoring
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Author and ratify ADR-0008 (Data Use Boundary) — the P0 prereq per PRD §6 constraint 8.
---

# M01-P01-IP-001 — Data Use Boundary ADR-0008 authoring

## Purpose
Author and ratify ADR-0008 (Data Use Boundary) — the P0 prereq per PRD §6 constraint 8.

## Symbols-to-grit-claim
```
docs/decisions/ADR-0008-data-use-boundary.md::Decision
docs/decisions/ADR-0008-data-use-boundary.md::Consequences
docs/PRIVACY-PROGRAM.md::§2.2.2-consent-tier-mapping
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
M-CC-P01 (agentic-pipeline cutover) ≥ P5 merged.

## Acceptance-test-commands
```
node scripts/validate-adr-shape.mjs docs/decisions/ADR-0008-data-use-boundary.md
cargo run -p oya-foundry-fitness-adr-shape
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P01-IP-002 (tenant kernel)

## Icm-store-payload
```
icm store -t decisions-oyatie -c 'ADR-0008 Data Use Boundary Accepted; per-consent-tier data-class mapping published' -i critical -k 'M01,P01,IP-001,adr-0008,data-use-boundary,accepted'
```

## Decision-log (Linus good-taste row)
Eliminates special-case 'per-axis re-implementation of consent boundary' — single consent-tier mapping table replaces N axis-specific boundary checks.


## Completion evidence

Completed on 2026-05-14. Evidence bundle: [`../../../../../../evidence/foundation/m01-p01-ip-001-data-use-boundary-adr.json`](../../../../../../evidence/foundation/m01-p01-ip-001-data-use-boundary-adr.json).

Fresh gates:
- `python3 -m json.tool docs/machine-readable/decisions.json` → pass.
- `node scripts/validate-adr-shape.mjs docs/decisions/ADR-0008-data-use-boundary.md` → pass.
- `node --check scripts/validate-adr-shape.mjs` → pass.
- `rustfmt --check crates/oya-foundry-fitness-adr-shape-kernel/src/lib.rs tools/oya-foundry-fitness-adr-shape/src/main.rs` → pass via `rustfmt.toml` (`edition = "2024"`, `style_edition = "2024"`).
- `cargo check -p oya-foundry-fitness-adr-shape-kernel` and `cargo check -p oya-foundry-fitness-adr-shape` → pass.
- `cargo test -p oya-foundry-fitness-adr-shape-kernel` → 6 passed.
- `cargo run -p oya-foundry-fitness-adr-shape` → `adr-shape ok: adrs_checked=67`.
- `cargo metadata --no-deps --format-version 1` → `packages=162 workspace_members=162`.
- Content assertions → `decisions.json` total 67, status counts Accepted 31 / Proposed 36, next ADR-0091, privacy token `FINANCIAL_KR`, rustfmt 2024 pinned.

`scripts/check.sh` remains blocked by the pre-existing missing `scripts/check-stage0-application-shell-prereqs.py`; this IP's targeted ADR-shape, mirror, Rust, and workspace metadata gates are executable and green.
