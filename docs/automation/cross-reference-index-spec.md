---
doc_class: DisciplineSpec
shape: discipline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Auto-generate `docs/CROSS-REFERENCE-INDEX.md` listing every doc-class entry
  with: path | purpose | doc-class | owner (RACI) | lifecycle (DOC-CATALOG) |
  consumer-fitness-lane | last-verified. The single navigation surface for
  agents + humans across the entire canonical doc tree.
planned_enforcement_ref: governance-cross-reference-index
extends_crates:
  - governance-doc-catalog-kernel
  - governance-raci-team-coverage-kernel
  - governance-readme-doc-coverage-kernel
companion_docs:
  - INDEX.md
  - orphan-detection-discipline.md
  - doc-freshness-discipline.md
  - ../../docs/DOC-CATALOG.md
doc_status: published
---

# Discipline: cross-reference index

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

`docs/DOC-CATALOG.md` defines the per-doc lifecycle (update triggers, cadence, validation checks). `docs/RACI-OWNERSHIP.md` defines per-doc ownership. `doc-freshness-discipline.md` tracks last-verified dates. `orphan-detection-discipline.md` tracks `purpose:`. None of the four alone is enough; engineers and agents need a single table joining all four. `docs/CROSS-REFERENCE-INDEX.md` is that join.

## 2. Inputs

- `docs/machine-readable/catalog.json` (from `governance-doc-catalog-kernel`).
- `docs/machine-readable/raci.json` (from `governance-raci-team-coverage-kernel`).
- `docs/machine-readable/doc-freshness.json` (from `doc-freshness-discipline.md`).
- `docs/machine-readable/orphans.json` `purpose:` lookup (from `orphan-detection-discipline.md`).
- The fitness-lane registry (per `fitness-lane-reports-pipeline.md`).

## 3. Output schema (the `docs/CROSS-REFERENCE-INDEX.md` table)

```markdown
| path | purpose | doc-class | owner | lifecycle | consumer-governance-lanes | last_verified |
|---|---|---|---|---|---|---|
| docs/CONSTITUTION.md | Mission, decision rights, prohibitions, amendments | Constitutional | council-architecture | quarterly + on-amendment | authority-cohesion, adr-citation | 2026-05-12 |
| docs/DOC-CATALOG.md | Per-doc lifecycle protocol and trigger taxonomy | Operating-Contract | council-architecture | quarterly | doc-catalog, doc-freshness | 2026-05-09 |
| docs/runbooks/cloud/region-failover.md | Sev-1 regional-failover procedure | Reference (Sev-1 runbook) | ops-sre-reliability | per-incident + 90d | runbook-freshness, runbook-discoverability | 2026-04-22 |
```

Ordering: alphabetical by `path`. Multi-table sections by `doc-class`: Constitutional → Operating-Contract → Reference → Decision-Record → Working-Draft.

## 4. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching any input source | Re-generate; PR fails on drift. |
| Nightly | Full re-generation; archive snapshot. |
| On any new doc | The doc's row is auto-added on next regeneration; missing row at PR-time → HIGH. |

## 5. Validation gates (`governance-cross-reference-index`)

1. **Generated drift.** Committed `docs/CROSS-REFERENCE-INDEX.md` differs from re-generated (BLOCKER).
2. **Coverage.** Every doc in `docs/machine-readable/catalog.json` has a row (BLOCKER).
3. **Cross-input consistency.** A doc's `owner` in this index matches `docs/RACI-OWNERSHIP.md`; mismatch → BLOCKER.
4. **Lane-link integrity.** Every `consumer-fitness-lane` resolves to a registered lane (HIGH).
5. **Last-verified parity.** `last_verified` value matches `doc-freshness.json` for the doc (BLOCKER).
6. **Manual-edit lockout.** First line of the file is `<!-- generated-by: cross-reference-index-spec; do not edit -->`. Hand edits → BLOCKER.

## 6. Per-doc reverse map

For each doc, the pipeline also emits a `reverse-references` annotation: which other docs cite this one. Surfaces as a small `### Cited by` section appended to the doc's own page during mdbook publish (non-destructive; HTML-only). Lets agents pivot from "who depends on this" without re-scanning the corpus.

## 7. Per-lane forward map

For each fitness lane in the registry, the pipeline emits its consumed docs in the same JSON sidecar (`docs/machine-readable/lane-doc-map.json`). Lane authors use this to confirm "my lane reads these docs" — supporting rapid impact analysis when a doc changes.

## 8. mdbook integration

The mdbook chapter `docs/site/src/reference/cross-reference-index.md` is the human-facing rendered view. The mdbook-kernel validates the source; the lane validates the data.

## 9. Cross-references

- `orphan-detection-discipline.md`: ensures every file appears in this index.
- `doc-freshness-discipline.md`: provides the `last_verified` column.
- `adr-index-pipeline.md`: feeds Decision-Record class rows.
- `runbook-freshness-pipeline.md`: feeds Reference (runbook) class rows.

## 10. Out-of-scope

- Per-section anchor-level cross-references (granular; future enhancement).
- Search-engine indexing (handled by mdbook's built-in search + downstream Search axis).
- Per-region localized index (future per `governance-glossary-localization-kernel`).
