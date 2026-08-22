---
doc_class: AdrIndex
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
doc_status: published
---

# sheets service-scoped ADRs

This directory holds **service-scoped** Architecture Decision Records owned by the `sheets` µservice per ADR-0131 §"Canonical folder shape". Repo-wide ADRs continue to live at `/Users/jasonlee/oyatie/docs/decisions/` (e.g., ADR-0105, ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)).

Service-scoped ADRs are numbered `ADR-SHEETS-####` (four-digit, sequential within this directory). The `SHEETS` prefix prevents collision with the repo-wide `ADR-####` series and matches the convention adopted by sibling µservices.

## Index

| ADR | Title | Status | Closes PRD Open Question |
|---|---|---|---|
| [ADR-SHEETS-0001](ADR-SHEETS-0001-crdt-library-selection.md) | CRDT library selection — Loro 1.x aligned with workflow-studio ADR-WS-0001 + docs ADR-DOCS-0001; AC-06 never-silent-loss invariant defensible | Accepted | Q1 (CRDT library) |
| [ADR-SHEETS-0002](ADR-SHEETS-0002-formula-engine-conformance-target.md) | Formula-engine conformance — core-subset ≥ 400 functions with LibreOffice Calc reference corpus; AC-11 100% pass-rate enforced | Accepted | Q2 (formula-engine conformance) |
| [ADR-SHEETS-0003](ADR-SHEETS-0003-large-sheet-storage-substrate.md) | Large-sheet storage substrate — Postgres hot OLTP + Apache Arrow/Parquet cold analytical hybrid; 1M-cell ≤ 10s recalc defensible | Accepted | Q3 (large-sheet storage) |
| [ADR-SHEETS-0004](ADR-SHEETS-0004-recalc-engine-architecture.md) | Recalc-engine architecture — dependency-graph + topological + parallel-task-graph (rayon-backed); 100k-cell ≤ 1s + 1M-cell ≤ 10s defensible | Accepted | Q4 (recalc-engine architecture) |
| [ADR-SHEETS-0005](ADR-SHEETS-0005-ai-formula-and-smart-fill-bounds.md) | AI-formula + smart-fill capability tier bounds — T0/T1 intra-sheets by default; T2-cross gated by Cedar + ChangeSet review + 2-person rule | Accepted | Q5 (AI-formula scope) |
| [ADR-SHEETS-0006](ADR-SHEETS-0006-per-range-acl-granularity.md) | Per-range ACL granularity — named-range ACL via Cedar policy fragments; column/range-level enforcement at every read/write path | Accepted | Q6 (per-range ACL granularity) |
| [ADR-SHEETS-0007](ADR-SHEETS-0007-export-fidelity-policy.md) | XLSX export fidelity — best-effort fidelity at M03 with named-limit list (no VBA, no ActiveX, image downgrade); gVisor + ClamAV + OPSWAT sandboxed; strict-OOXML scheduled-for-distinct-tracked-work subsequent-to-M03-completion | Accepted | Q7 (XLSX export fidelity) |

## Cross-reference policy

- Every service-scoped ADR in this directory MUST reference the repo-wide ADRs it inherits from (e.g., ADR-0135 sheets net-new µservice, ADR-0131 layout, ADR-0140 Cedar, ADR-0105 layer enum).
- Repo-wide ADRs in `/Users/jasonlee/oyatie/docs/decisions/` MUST NOT depend on service-scoped ADRs in this directory; the dependency direction is one-way.
- Service-scoped ADRs may reference each other freely within this directory.
- **Cross-µservice ADRs** (e.g., ADR-SHEETS-0001 ↔ ADR-WS-0001 ↔ ADR-DOCS-0001 Loro alignment) reference each other as `related_external_adrs` in frontmatter. Coordinated upgrade contract per ADR-SHEETS-0001 §"Cross-µservice operational" applies.
- Supersession of a service-scoped ADR is recorded by adding `superseded_by:` to the old ADR's frontmatter and `supersedes:` to the new ADR's frontmatter; old ADRs are **never deleted**.

## Sibling µservice ADR directories

- `microservices/workflow-studio/decisions/` — workflow-studio service-scoped ADRs (Loro alignment per ADR-SHEETS-0001).
- `microservices/docs/decisions/` — docs service-scoped ADRs (Loro alignment per ADR-SHEETS-0001).
- `microservices/cell/decisions/` — cell µservice service-scoped ADRs (per-workbook cell substrate).
- (Other µservices acquire their own `decisions/` directory at the time they author their first service-scoped ADR.)

## Open Questions still tracked in PRD

After this batch of ADRs the PRD's `Open Questions` table is closed for entries 1–7.

| # | Question | Resolution status |
|---|---|---|
| 1 | CRDT library | **Closed** — ADR-SHEETS-0001 |
| 2 | Formula-engine conformance | **Closed** — ADR-SHEETS-0002 |
| 3 | Large-sheet storage substrate | **Closed** — ADR-SHEETS-0003 |
| 4 | Recalc-engine architecture | **Closed** — ADR-SHEETS-0004 |
| 5 | AI-formula scope | **Closed** — ADR-SHEETS-0005 |
| 6 | Per-range ACL granularity | **Closed** — ADR-SHEETS-0006 |
| 7 | XLSX export fidelity | **Closed** — ADR-SHEETS-0007 |

## Author + reviewer protocol

Per the documentation-and-adrs skill and ADR-0131:

1. Author a draft ADR under this directory using the structure: Status / Date / Context / Decision / Alternatives Considered (≥3 alternatives) / Consequences.
2. Decision must be concrete (no TODO comments; no deferral within scope).
3. Consequences must list ≥3 downstream impacts.
4. ADR must cross-reference (a) the repo-wide ADRs it inherits from, (b) named industry sources where applicable (RFCs, regulations, standards, OSS library docs).
5. ChangeSet review per ADR-0110 with reviewer-agent APPROVE before merge to `dev`.

## Coordinated upgrade contracts

For ADRs that align with sibling µservice ADRs:

- **Loro CRDT alignment** (ADR-SHEETS-0001 ↔ ADR-WS-0001 ↔ ADR-DOCS-0001): version bumps across all three µservices within the same calendar week; joint operational drill.
- **AI-formula / AI-copilot / AI-assist tier bounds** (ADR-SHEETS-0005 ↔ ADR-WS-0005 ↔ ADR-DOCS-0005): tier framework + EU AI Act posture aligned; per-µservice subcapabilities differ but the gating shape is identical.

Quarterly review by council-architecture verifies the coordinated upgrade contracts remain in sync.
