---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-foundation
phase: P01-notes-foundation
status: Active
entry_gate: |
  ADR-0135 (parallel Connect dual-context) + ADR-0131 + ADR-0132 accepted;
  observability µservice IP-001..IP-015 merged so notes can author OpenSLO manifests and pass promotion-readiness gate;
  drive µservice ready for attachment refs; tasks µservice ready to receive checklist-emitted events.
exit_gate: |
  All 15 IPs merged; all 111 crates compile + nextest green;
  oya gate validate per-microservice-layout --microservice notes exits 0;
  oya gate validate dual-context-isolation --microservice notes exits 0;
  oya gate validate e2e-ai-refusal --microservice notes exits 0;
  HG-NOTES gate registers green;
  end-to-end note-create + tag + `[[wikilink]]` + backlink + daily-note + share-link drill passes within performance budget;
  pack-kr overlay deployed to dedicated notes cluster; pack-eu overlay ready;
  AC-01..AC-16 in PRD.md green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: notes requires observability gate + tenancy + ontology + audit-chain + cedar
  - microservice: drive
    reason: embed BC references drive attachment blob refs
  - microservice: tasks
    reason: checklist BC emits ChecklistItemEmitted Workflow events to tasks
owner_team: axis-notes
related_adrs: [ADR-0008, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
doc_status: published
---

# P01-notes-foundation: Land the notes µservice end-to-end

## Purpose

This phase ships the full notes µservice per parallel ADR-0135 + ADR-0132: short-form personal-notes + knowledge-capture, dual-context-safe across personal (B2C) and professional (B2B), with E2E-default on the Personal tier.

It advances master-plan principles:
- Hyperscaler-grade in every practice (Apple Notes / Obsidian / Standard Notes-class parity + native Workflow + Ontology integration).
- Nothing scheduled-for-distinct-tracked-work (no FUTURE stubs; every NFR covered).
- No silent regression (production-tier change gated by observability ADR-0130).
- Per-microservice flat layout (ADR-0131 native authoring).
- Dual-context isolation by data model (NOT runtime flag) per parallel ADR-0135.
- AI-refusal on E2E content as a structural impossibility, not a setting (per ADR-NOTES-0005).

## Scope

### In-scope

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `notes` | `note-store`, `tag-graph`, `backlink-graph`, `daily-note`, `template-gallery`, `web-clipper-bridge`, `share-link`, `embed`, `checklist`, `version-history`, `search-index`, `graph-view-data`, `collab-edit`, `import-pipeline`, `export-pipeline`, `ai-assist`, `e2e-key-management` | 111 crates |

Plus cross-cutting:
- `.github/branch-protection.yaml` — add `release/notes/*` pattern protection.
- `/specs/hyperscaler-gates.json` — register HG-NOTES per ADR-0133.
- `Cargo.toml` (workspace) — register 111 crates.
- `microservices/notes/decisions/ADR-NOTES-0001..0006` — six service-scoped ADRs.

### Out-of-scope

- Native mobile-platform-extension capture (iOS Share Sheet beyond Web Share API; future IP).
- Federated note exchange with external Obsidian / Logseq vaults via Matrix (Open Question; future ADR).
- AI-driven auto-organisation T2 capability (capability declared but disabled at minimum-shippable-tier; opt-in tenant-admin).
- OCR-on-clipped-images (future capability; clipper captures raw image only at minimum-shippable-tier).

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac.md`](IP-001-iac.md) | Helm/Kustomize + Terraform for notes (Postgres + Redis + Meilisearch + S3 + Loro server-side + MLS key-bundle directory) | pending | axis-notes + ops-sre-reliability | observability IP-001 |
| [`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md) | Register 111 crates in workspace Cargo.toml | pending | axis-notes | — |
| [`IP-003-note-store-kernel-domain.md`](IP-003-note-store-kernel-domain.md) | `oya-notes-note-store-kernel` + `-domain` port traits + entities | pending | axis-notes | IP-002 |
| [`IP-004-tag-graph-kernel-domain.md`](IP-004-tag-graph-kernel-domain.md) | `oya-notes-tag-graph-{kernel,domain}` | pending | axis-notes | IP-003 |
| [`IP-005-backlink-graph-kernel-domain.md`](IP-005-backlink-graph-kernel-domain.md) | `oya-notes-backlink-graph-{kernel,domain}` | pending | axis-notes | IP-003 |
| [`IP-006-daily-note-template-gallery.md`](IP-006-daily-note-template-gallery.md) | `oya-notes-daily-note-*` + `oya-notes-template-gallery-*` | pending | axis-notes | IP-003 |
| [`IP-007-web-clipper-bridge.md`](IP-007-web-clipper-bridge.md) | `oya-notes-web-clipper-bridge-*` + Chrome MV3 + Firefox MV3 + Safari WebExt | pending | axis-notes + ops-security | IP-003 |
| [`IP-008-share-link-and-embed.md`](IP-008-share-link-and-embed.md) | `oya-notes-share-link-*` + `oya-notes-embed-*` (with drive client) | pending | axis-notes | IP-003 |
| [`IP-009-checklist-and-version-history.md`](IP-009-checklist-and-version-history.md) | `oya-notes-checklist-*` + `oya-notes-version-history-*` | pending | axis-notes | IP-003 |
| [`IP-010-search-and-graph-view.md`](IP-010-search-and-graph-view.md) | `oya-notes-search-index-*` (Meilisearch) + `oya-notes-graph-view-data-*` | pending | axis-notes | IP-005 |
| [`IP-011-collab-edit-loro.md`](IP-011-collab-edit-loro.md) | `oya-notes-collab-edit-*` (Loro 1.x; Professional-only) | pending | axis-notes | IP-003 |
| [`IP-012-import-export-pipelines.md`](IP-012-import-export-pipelines.md) | `oya-notes-import-pipeline-*` + `oya-notes-export-pipeline-*` (Apple Notes / ENEX / OneNote / Notion / Bear / Obsidian) | pending | axis-notes | IP-003 |
| [`IP-013-ai-assist-and-e2e-refusal.md`](IP-013-ai-assist-and-e2e-refusal.md) | `oya-notes-ai-assist-*` + e2e-ai-refusal Cedar policy + CI lane | pending | axis-notes + axis-foundry-runtime + council-privacy | IP-003 |
| [`IP-014-e2e-key-management.md`](IP-014-e2e-key-management.md) | `oya-notes-e2e-key-management-*` (openmls 0.6) + recovery seed UX | pending | axis-notes + council-privacy | IP-003 |
| [`IP-015-hg-notes-conformance.md`](IP-015-hg-notes-conformance.md) | HG-NOTES hyperscaler-grade conformance gate per ADR-0133 | pending | axis-notes + council-architecture | IP-003..IP-014 |

## Per-IP Test Coverage Threshold

| Class | Coverage line / branch | Test types required |
|---|---|---|
| kernel | 90 % / 80 % | per-port-trait + per-entity unit; sealed-trait smoke; data-class annotation check |
| domain | 90 % / 80 % | pure-math / pure-logic unit |
| usecase | 85 % / 75 % | orchestrator unit with port mocks; happy + error path |
| adapter | 80 % / 70 % | integration vs real backend (Postgres / Redis / S3 / Meilisearch / Loro / MLS) where feasible; otherwise contract-mock |
| rest | 85 % / 75 % | per-endpoint happy + 401 + 403 + 422 |
| worker | 85 % / 75 % | event-loop unit + integration |
| app | 75 % / 65 % | smoke startup |

E2E: ≥ 1 per AC-NN row in PRD.

## Open Questions Tracked

- PRD #1 — closed in ADR-NOTES-0004 (encrypted-inverted-index in IndexedDB with token-bloom-filters).
- PRD #2..#6 — open; scheduled-for-distinct-tracked-work to successor-IP IPs or capability ADRs.

## References

- `microservices/notes/PRD.md`.
- ADR-0135; ADR-0130; ADR-0131; ADR-0132; ADR-0133.
- ADR-NOTES-0001..0006.
