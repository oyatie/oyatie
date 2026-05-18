---
doc_class: PhasePlan
template_id: TPL-PHASE-PLAN
microservice: docs
phase_id: PHASE-01
phase_title: Docs Foundation — document-store + collab-crdt + block-types + comments + version-history + sharing + export-import + embed-resolver
status: Accepted
date: 2026-05-17
owner_team: axis-docs
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-WS-0001]
doc_status: published
---

# PHASE-01 — Docs Foundation

## Intent

Stand up the eight bounded contexts (document-store, collab-crdt, block-types, comments-and-suggestions, version-history, sharing-and-permissions, export-import, embed-resolver) with full Layer-A + Layer-B substrate, Bominal ADR-0208 + ADR-0215 inheritance, Loro CRDT alignment with workflow-studio per ADR-WS-0001 + ADR-DOCS-0001, dual-context isolation, audit-chain emission, gVisor-sandboxed export workers, WCAG 2.2 AA conformance, and SLO-gated promotion. Phase exit = AC-01 through AC-16 in `PRD.md` green.

## Phase scope

In-scope:
- 65 crates per the layer mapping table.
- Postgres document-metadata schema + per-tenant RLS + tenant-DEK envelope encryption.
- S3 content-blob storage + per-tenant prefix + Object Lock for legal-hold.
- Valkey collab-presence + CRDT op fan-out + per-doc cache; cluster mode.
- Loro 1.x CRDT engine adoption with shared library alignment to workflow-studio (per ADR-DOCS-0001 + ADR-WS-0001).
- Block-type schema + sanitisation (paragraph, heading, list, table, image, embed, code, math, callout).
- Comments + suggestions BC with anchor stability across edits.
- Per-doc + per-block ACL (Notion-style) per ADR-DOCS-0004.
- Export pipeline (Pandoc 3.x + WeasyPrint default; Chromium-headless opt-in) inside gVisor sandbox per ADR-DOCS-0003.
- Import pipeline (DOCX / Markdown / HTML / Google-Docs format) with sanitisation per ADR-DOCS-0006.
- Embed-resolver for workflow-studio + sheets cells + slides decks with policy-bounded refresh.
- Workflow events produced + consumed per `PRD.md`.
- Ontology writes + reads per `PRD.md`.
- HG-DOCS hyperscaler-maturity claim registered per ADR-0123 + ADR-0133.

Out-of-scope (scheduled-for-distinct-tracked-work):
- External Google Docs / Word source-of-truth federation — migration-only at GA (subsequent-to-M04-completion).
- Public-read URL publishing — share-link only at GA (subsequent-to-M04-completion).
- ML-based smart summarisation post-publish — subsequent-to-M03-completion+1.
- Cross-document semantic search over corpus — subsequent-to-M04-completion (ontology integration).

## Phase outputs

| Output | Path | Owner |
|---|---|---|
| 65 crates | `microservices/docs/src/crates/oya-docs-*` | axis-docs |
| Postgres schema migrations | `microservices/docs/iac/helm/postgres/migrations/` | axis-docs |
| Helm charts | `microservices/docs/iac/helm/{postgres,redis,s3,clamav,gvisor-pool}` | ops-sre-reliability |
| Kustomize overlays | `microservices/docs/iac/kustomize/{base,overlays/pack-kr,pack-eu}` | ops-sre-reliability |
| OpenAPI / AsyncAPI / Proto contracts | `microservices/docs/contracts/` | axis-docs |
| Cedar policies | `microservices/docs/policy/*.cedar` | ops-security |
| Runbooks | `microservices/docs/runbooks/*.md` | ops-sre-reliability |
| Dashboards | `microservices/docs/dashboards/*.json` | axis-observability |
| SLOs (9 OpenSLO manifests) | `microservices/docs/slos/*.openslo.yaml` | axis-docs |
| HG-DOCS claim entry | `registry/hyperscaler-maturity-claims.json` | axis-docs |

## Phase milestones (ChangeSets, per ADR-0110)

| CS | Title | DAG-position | Slice |
|---|---|---|---|
| CS-01 | document-store kernel + domain + usecase + api | Layer-B base | A |
| CS-02 | document-store -adapter-postgres + -adapter-s3 + RLS schema + tenant-DEK envelope | depends CS-01 | A |
| CS-03 | document-store rest + worker + sdk + app | depends CS-02 | A |
| CS-04 | block-types kernel..app (7 crates) | depends CS-01 | A |
| CS-05 | collab-crdt kernel..adapter (Loro wrapping; ADR-DOCS-0001) | depends CS-01 + CS-04 | B |
| CS-06 | collab-crdt -adapter-redis + worker + sdk + app | depends CS-05 | B |
| CS-07 | comments-and-suggestions kernel..rest + worker + app (9 crates) | depends CS-01 + CS-04 | B |
| CS-08 | version-history kernel..worker + app (8 crates) | depends CS-01 + CS-05 | B |
| CS-09 | sharing-and-permissions kernel..rest + app (8 crates) | depends CS-01 | C |
| CS-10 | export-import kernel..adapter-pandoc + adapter-weasyprint + rest + worker + app (11 crates) | depends CS-01 + CS-04 | C |
| CS-11 | export-import -adapter-chromium (high-fidelity opt-in) | depends CS-10 | C |
| CS-12 | embed-resolver kernel..rest + worker + app (8 crates) | depends CS-01 + CS-09 | C |
| CS-13 | Cedar policy + DPIA + threat-model sign-off | depends CS-01..CS-12 | D |
| CS-14 | OpenAPI + AsyncAPI + Proto contracts + capabilities | depends CS-01..CS-12 | D |
| CS-15 | Helm + Kustomize + dashboards + runbooks | depends CS-01..CS-12 | D |
| CS-16 | HG-DOCS maturity-claim entry + SLO manifests + canary cohort weighting | depends all | D |

## Phase gate

Phase-exit gate (per ADR-0139): all 16 AC-IDs green; SLO eligibility verdict `eligible` for `docs` µservice over `dev → staging` window; reviewer-agent APPROVE on each ChangeSet; per-changeset evidence committed at `microservices/docs/evidence/multispectrum/*.json`.

## Risks + mitigations

| Risk | Mitigation |
|---|---|
| CRDT library divergence with workflow-studio (different versions / different port-trait shape) | Loro version pin + cross-µservice CI lane `oya-governance-crdt-cross-microservice-consistency` validates the port trait shape at every PR (paired with ADR-WS-0001) |
| OOXML round-trip fidelity below 95% | Establish best-effort tier with named edge-case test matrix per ADR-DOCS-0006; surface unsupported features to user before persisting |
| PDF export pipeline escape from gVisor sandbox | Pre-deployment escape-attempt test; tmpfs only; no network egress from sandbox; mandatory CI lane `oya-governance-export-sandbox-conformance` |
| Per-block ACL performance regression on large documents | Pre-compute block-ACL projection cache per query; LEAN check on ACL-coverage |
| Embed-resolver cross-µservice tight coupling | Resolver carries a refresh-on-source-change contract via Workflow event subscription only; never direct call into source µservice |
| Loro upstream supply-chain compromise | Pinned version + RustSec subscription + Ed25519-signed advisory feed monitoring per ADR-DOCS-0001 |
| WCAG 2.2 AA regression across export formats | per-export axe-core + Pa11y validation in `oya-governance-wcag-22-aa-conformance` lane |
| Attachment-storage malware bypass | ClamAV + (pack-us-healthcare) OPSWAT MetaDefender pre-persistence scan; per-extension allowlist; archive bombs refused |
