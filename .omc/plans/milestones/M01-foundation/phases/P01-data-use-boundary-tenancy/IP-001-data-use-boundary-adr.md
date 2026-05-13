---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P01-IP-001
title: Data Use Boundary ADR-0008 authoring
status: stub
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
