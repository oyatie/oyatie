---
doc_status: published
last_audited: 2026-05-20
---
# ADR Navigation Guide

`docs/decisions/` is the portfolio-wide decision ledger. Open `docs/ADR-INDEX.md` first; it lists every live `ADR-*.md` file and the generated machine-readable mirror in `docs/machine-readable/decisions.json`.

## How to Read the ADRs

1. Read `ADR-0001` through `ADR-0011` to understand the product thesis, tenant/identity/audit substrates, data boundary, cell shape, and contract registry.
2. Read `ADR-0056`, `ADR-0062`, `ADR-0063`, `ADR-0069`, and `ADR-0212` before changing architecture, documentation, or buildability surfaces.
3. Read `ADR-0242` through `ADR-0258` before touching tenant scope, Cedar, policy-engine, substrate/product layering, deployment topology, intelligence, ontology, or API versioning.
4. Read `ADR-0297` through `ADR-0321` before changing abuse defence, emergency bypass, account recovery, vulnerable-user flows, investigation/detection, personal/work boundary, marketplace, ERP coverage, or B2B SaaS scope.
5. If the change is service-specific, check `microservices/<service>/decisions/ADR-MS-*.md` after the portfolio ADRs.

## Status Semantics

`Proposed` means the decision is under review or advisory until its enforcement gates promote. `Accepted` means new work follows it. `Superseded`, `Amended`, and `Deprecated` preserve history; do not delete or rewrite them to make the ledger look clean. Amendments may share the same numeric ADR ID as their base decision; use the filename and title to distinguish them.

## Authoring Rules

Every new ADR needs `id`, `status`, `date`, `owners`, `related`, and a one-paragraph `purpose` in frontmatter. The body should explain context, decision, alternatives rejected, consequences, enforcement, observability, rollback, versioning, and related specs. Follow `docs/standards/documentation-rigor.md` section 1.1: name precedent, failure modes, capacity/performance implications, observability hooks, rollback, multi-region/sovereign-cell behavior, and versioning/deprecation where applicable.

## Tooling Notes

The canonical generator path is Rust/Buck2-owned: use `buck2 build //tools/oya-adr-index-regenerator-app:adr-index-regenerator-unit-tests //:adr-index-regeneration-check` to verify committed output, and use the regenerator app's `--write` mode only inside a dedicated ADR-index regeneration lane. Do not patch individual ADRs during index gardening unless that is the explicit task; normalize legacy metadata when the ADR itself is touched.
