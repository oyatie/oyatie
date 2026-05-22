---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: docs
deprecated_artifact: oya-connect-docs-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-DOCS accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-DOCS-0001, ADR-DOCS-0002, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
related_specs: [/specs/microservices/docs.json]
owner_team: axis-docs
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-connect-docs-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-docs-*` crate family under `microservices/docs/src/crates/` per ADR-0131. See **`microservices/docs/migration-from-connect.md`** for the full import-path map (~75 crate mappings), Hyrum's-Law-bound surface callouts (8 surfaces), configuration delta table, runbook continuity table (3 preserved + 4 net-new), and step-by-step migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-DOCS accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #3). Following the 6-month Strangler window in ADR-0134 (Phase 2 adapter soak + Phase 3 canary), the indicative advisory removal date is **2026-12-17**, gated on the SLO trigger.

## Reason

The legacy `oya-connect-docs-*` family was authored before the following ADRs crystallised; each ADR makes the legacy shape non-conforming:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle membership at the architecture layer; bundle membership is a brand-layer concept and must not appear in crate names.
2. **ADR-0139 — agentic SLO-gated promotion.** Docs needs independent SLO targets per surface (doc-open, save, collab-cursor-sync, export-pdf, search-within-doc, doc-list, crdt-no-silent-loss 100%, share-acl-enforcement-correctness 100%, pandoc-export-pipeline-availability); a `connect-*` umbrella SLO cannot serve them.
3. **ADR-0131 — per-µservice flat layout.** Docs's IaC, runbooks, threat-model, DPIA, compliance, capacity-model, cost-budget all need to live under one folder (`microservices/docs/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr, pack-eu, pack-us, pack-us-healthcare, pack-jp, pack-sg, pack-au, pack-in, pack-br, pack-ae, pack-ksa each need per-µservice overlay granularity.
5. **ADR-DOCS-0001 → ADR-DOCS-0006** — docs-specific decisions (CRDT library shared with workflow-studio, block-type system, export pipeline architecture, per-block ACL, AI writing-assist bounds, DOCX import fidelity) need to live at per-µservice ADR granularity.
6. **Cross-µservice CRDT consistency** with workflow-studio (per ADR-WS-0001 + ADR-DOCS-0001) is impossible at the legacy connect-suite granularity.

## Migration Guide pointer

→ **`microservices/docs/migration-from-connect.md`**

Includes: ~75-row 1:1 import-path map; net-new-boundary features (Loro CRDT, block-types BC, chromium opt-in PDF backend, per-block ACL, embed-resolver, T0/T1/T2 AI writing-assist, audit-chain emission, eIDAS PAdES); concrete `use` and `Cargo.toml` rewrites; configuration delta table; dual-context isolation invariant preservation + strengthening; Hyrum's-Law surface callouts (8 surfaces: CRDT ordering, DOCX byte stability, embed retry semantics, attachment URL TTL, block enumeration order, comment anchor stability, suggestion auto-acceptance, export sync→async); runbook continuity table (3 preserved + 4 net-new); 5-step migration recipe; 6-phase Strangler timeline; verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-connect-docs-*'` (2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-connect-docs-domain` | split per BC → `oya-docs-{document-store,collab-crdt,block-types,comments-and-suggestions,version-history,sharing-and-permissions,export-import,embed-resolver}-domain` |

Plus all `oya-connect-docs-{kernel,usecase,api,adapter*,rest,worker,sdk,app}-*` crates scaffolded during Phase 2 adapter authoring.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-docs-*` crates ship in parallel | 1 | No (additive) | — |
| New Loro CRDT engine (ADR-DOCS-0001) | 1 | **Behaviourally divergent** for concurrent edits — last-write-wins → conflict-surfaced | adapter does NOT mask divergence; documented in migration guide Hyrum #1 |
| New block-type schema (ADR-DOCS-0002) | 1 | **Schema-divergent** — untyped → strictly-typed blocks | adapter provides translation; consumers migrate types |
| New Pandoc 3.x export (ADR-DOCS-0003) | 1 | **Byte-level divergent** for DOCX (Pandoc 2.x → 3.x) | adapter does NOT mask; documented Hyrum #2 |
| New per-block ACL (ADR-DOCS-0004) | 1 | No (additive; whole-doc ACL preserved as default) | — |
| New T0/T1/T2 AI writing-assist (ADR-DOCS-0005) | 1 | No (net-new; no legacy counterpart) | — |
| New DOCX import fidelity matrix (ADR-DOCS-0006) | 1 | No (additive; surface unsupported features instead of silent loss) | — |
| New embed-resolver (cross-µservice embed) | 1 | No (net-new) | — |
| New embed-resolver retry semantics | 1 | **Behaviourally divergent** | adapter does NOT mask Hyrum #3 |
| Attachment URL TTL configurable | 1 | **Behaviourally divergent** | adapter does NOT mask Hyrum #4 |
| Block enumeration order (CRDT-tree-traversal default) | 1 | **Behaviourally divergent** | adapter provides additive `blocks_in_insertion_order()` Hyrum #5 |
| Comment anchor CRDT-aware | 1 | **Schema-divergent** | adapter provides anchor-migration utility Hyrum #6 |
| Suggestion no auto-acceptance | 1 | **Behaviourally divergent** | adapter does NOT mask Hyrum #7 |
| Export async (not sync) | 1 | **Behaviourally divergent** | adapter provides `awaitExport()` shim Hyrum #8 |
| `oya-connect-docs-migration-adapter` shim authored | 2 | No (preserves legacy symbol surface modulo Hyrum surfaces) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-connect-docs-*` crates removed from workspace** | **5** | **YES — breaking** | **7-mo advisory sunset from 2026-05-17** |
| `microservices/connect/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (renders the change loud + immediate + CI-detectable).
- **ADR-0134** (carries the migration policy decision).
- **ADR-DOCS-0001 + ADR-DOCS-0003 + ADR-DOCS-0006** (specifically document the CRDT + export + import behavioural strengthenings as deliberate, owner-authored design choices — NOT silent regressions).
- **Version bump.** The `Cargo.toml` of every consumer crate is bumped per semver when its legacy imports are removed (treating the `oya-connect-docs-*` re-export as the public contract).
- **Sunset schedule.** 7-month advisory window from this notice; concrete date 2026-12-17 contingent on the HG-DOCS SLO trigger.
- **Owning-axis migration ChangeSets.** axis-docs ships migration ChangeSets for every known internal consumer per the Churn Rule before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases — HG-DOCS gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples — `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4 commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5 commands.
- [ ] No references to the deprecated system remain — `rg "oya_connect_docs" --type rust` produces zero hits outside historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-DOCS-0001 (CRDT library — Loro 1.x; cross-µservice consistent with workflow-studio ADR-WS-0001).
- ADR-DOCS-0002 (block-type system).
- ADR-DOCS-0003 (export pipeline architecture — Pandoc 3.x + WeasyPrint default + Chromium opt-in inside gVisor).
- ADR-DOCS-0004 (ACL granularity — per-block).
- ADR-DOCS-0005 (AI writing-assist EU AI Act bounds).
- ADR-DOCS-0006 (DOCX import fidelity — best-effort fidelity with named edge-case matrix).
- ADR-WS-0001 — workflow-studio CRDT library selection (cross-µservice alignment).
- `microservices/docs/migration-from-connect.md` — full migration guide.
- `microservices/docs/PRD.md` — target-state product definition.
- `microservices/docs/runbooks/*.md` — 7 runbooks (3 preserved + 4 new).
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
- ECMA-376 — OOXML; ISO 19005 — PDF/A; CommonMark + GFM — Markdown; EPUB 3 — W3C; HTML5 — W3C; eIDAS 910/2014 — PAdES.
