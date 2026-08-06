---
id: ADR-0130
status: Superseded
superseded_by: [ADR-335, ADR-562]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0130: Deprecate `registry/knowledge-graph-semantic.json` and Migrate to Ontology Type System

**Status:** Accepted
**Date:** 2026-05-17
**Owner:** council-architecture + ontology-team
**Supersedes:** N/A
**References:** ADR-0122 (ontology rename), D-09 product-level Ontology+KG merge, OP-11 (no compat seams), [[glossary-ontology-not-object-graph]] memory note

---

## Context

`registry/knowledge-graph-semantic.json` was the SEMANTIC layer of the 3-layer KG split (semantic + kinetic + dynamic per Palantir Foundry pattern). It held:
- 36 node type definitions
- 27 edge type definitions
- 19 graph invariants
- 11 read-side query examples

On 2026-05-16, the user picked D-09: Ontology and Knowledge Graph are **one product**. ADR-0122 completed the crate rename from `oya-object-graph-*` to `oya-ontology-*`. Memory note [[glossary-ontology-not-object-graph]] made this permanent.

This left an infra-level drift: the Ontology product PRD (`specs/products/ontology.json`) referenced the KG semantic registry file as an external dependency, rather than owning the type-system definitions directly. This is the exact Palantir pattern inversion — Palantir Foundry's Ontology owns its Object Types and Link Types directly; it does not reference a separate registry file.

The file was last substantively updated in the PR #44 sequence (KG acceptance CI, changeset provenance, agent decision provenance chains). No further standalone evolution is expected — the type system evolves with the Ontology product.

## Decision

**COMPULSORY deprecation.** `registry/knowledge-graph-semantic.json` is deleted. Its entire content (all 36 node types, 27 edge types, 19 invariants, 11 read-side query examples) is migrated into `specs/products/ontology.json#type_system`.

The kinetic and dynamic layers (`registry/knowledge-graph-kinetic.json`, `registry/knowledge-graph-dynamic.json`) are **not** affected — they are write-side action types and live-state telemetry respectively, and do not duplicate Ontology's type-system concern.

All consumers updated:
- `crates/oya-dev-cli/tests/knowledge_graph_semantic.rs` — `semantic_graph()` now loads from `specs/products/ontology.json#type_system`
- `registry/knowledge-graph-kinetic.json` — sibling_layers pointer updated
- `registry/knowledge-graph-dynamic.json` — sibling_layers pointer updated
- `registry/kg-audit/index.json` — all refs updated to `specs/products/ontology.json#type_system`
- `specs/root-hub-pointers.json` — `knowledge_graph_semantic` entry updated
- `registry/milestone-audit/index.json` — write_scope ref updated; `architectural_changes` row added

Evidence files under `evidence/` are historical provenance records and intentionally retain original path citations.

## Alternatives Considered

**Keep both files in parallel**
Rejected. Two homes for the same type-system content creates drift. Every schema change would require dual updates. The D-09 product merge already decided Ontology owns this concern.

**Advisory deprecation with tombstone file**
Rejected. OP-11 (no compat seams) prohibits tombstone files for internal infrastructure. All consumers are in this repo and were migrated atomically in this PR.

**New file `specs/ontology-type-system.json`**
Rejected. The `specs/products/ontology.json` PRD is the canonical Ontology product spec. Adding a `type_system` section directly matches the Palantir pattern of owning Object Types inline in the ontology definition, and avoids a third pointer hop. JSON-with-pointers topology (ADR-0119) supports inline sections.

## Consequences

**Positive:**
- Single source of truth for semantic type definitions — no pointer hop from Ontology PRD to external registry
- Eliminates infra-level naming confusion between "ontology" and "knowledge graph"
- `specs/products/ontology.json` is now self-contained for type-system consumers
- Tests load from the canonical location and will catch type-system regressions in the right file

**Negative / Trade-offs:**
- `specs/products/ontology.json` grows in size (type_system section adds ~450 lines). Acceptable per ADR-0119 flat topology — size is a quality of the document, not a reason to fragment it.
- Historical evidence files retain old path citations. This is correct — they are immutable provenance records.

## Migration Verification

```bash
# Canonical home exists and is valid JSON
python3 -m json.tool specs/products/ontology.json > /dev/null

# Original file is gone
! test -f registry/knowledge-graph-semantic.json

# No live references remain (evidence/ excluded — historical)
grep -r "knowledge-graph-semantic" \
  --include="*.json" --include="*.rs" --include="*.md" --include="*.yaml" \
  --exclude-dir=evidence . | grep -v "ADR-0130\|ontology.json\|migration_note\|migrated_from\|_provenance\|deprecated_path" \
  | wc -l  # should be 0

# Tests still pass
cargo nextest run -p oya-dev-cli --test knowledge_graph_semantic
```

## Node/Edge Migration Summary

| Category | Count | Now at |
|---|---|---|
| node_types | 36 | `specs/products/ontology.json#type_system/node_types` |
| edge_types | 27 | `specs/products/ontology.json#type_system/edge_types` |
| invariants | 19 | `specs/products/ontology.json#type_system/invariants` |
| read_side_query_examples | 11 | `specs/products/ontology.json#type_system/read_side_query_examples` |
