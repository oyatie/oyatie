---
id: ADR-0055
status: Superseded
superseded_by: [ADR-0709]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0055: Object Graph renamed to Ontology

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0006, ADR-0059, ADR-0018

---

## Context

Oyatie's typed-entity information layer was previously called "Object Graph" (OG). Bominal ADR-0106 describes this layer as "Replaces Palantir 'Ontology' terminology." Per session decision 2026-05-13, oyatie reverses this: we align directly with the Palantir term **Ontology**, because:

1. Ontology is the established term in the data-layer space.
2. "Object Graph" overlaps with graph-database terminology and is more generic.
3. Oyatie's data layer matches Palantir Ontology semantics 1:1 (typed entities + links + actions + functions with audit-chain, RLS, jurisdiction overlays per Bominal ADR-0106).
4. User instruction 2026-05-13: "lets rename object graph to ontology" + "object graph = ontology at palantir."

This is an Oyatie-specific override of Bominal ADR-0106 per `[[feedback-bominal-inheritance-precedence]]` override #2.

**Naming justification:** "Ontology" is the Palantir-established industry term for this concept; aligns with the data-layer space vocabulary. BNF v4.1 (ADR-0056) slot2 = `ontology` (registered µservice name).

---

## Decision

All "Object Graph" terminology is renamed to "Ontology" in all oyatie artifacts.

### Scope of rename

| Was | Now | Location |
|---|---|---|
| "Object Graph" | "Ontology" | All ADRs, plans, docs, code |
| `oya-*-object-graph-*` | `oya-ontology-*` | All crates (Shard 1 atomic rename) |
| `oya-platform-object-graph-kernel` | `oya-ontology-entity-kernel` | Primary kernel crate |
| `oya-shared-object-graph-*` | `oya-ontology-*` (no `shared` prefix; BNF v4.1) | All substrate crates |
| "Object Graph (OG)" in glossary | "Ontology" | ADR-0018, GLOSSARY.md |
| "OG" abbreviation | Retired | Everywhere |
| ADR-0006 title/content | Rewritten to "Ontology" | ADR-0006 |
| Bominal ADR-0106 cross-references | Cited as "ADR-0106 (Object Graph = Ontology in oyatie glossary)" | All ADRs that reference it |

### Canonical Ontology crate layout post-rename (BNF v4.1)

```
oya-ontology-entity-kernel       — typed entity types + port traits
oya-ontology-entity-domain       — entity business logic
oya-ontology-entity-adapter      — Postgres + RLS impl
oya-ontology-link-kernel         — typed link types + port traits
oya-ontology-link-domain
oya-ontology-action-kernel       — typed action types + port traits
oya-ontology-action-domain
oya-ontology-function-kernel     — Ontology Function types
oya-ontology-agent-gateway-kernel — LLM tool-call ingress (per Bominal ADR-0107)
oya-ontology-agent-gateway-adapter
oya-ontology-audit-chain-adapter — chains to oya-audit-chain-kernel (ADR-0003)
oya-ontology-pillar-kernel       — org-pillar + person-pillar types (per Bominal ADR-0132)
oya-ontology-pillar-domain
```

All crates registered under `ontology` in `[workspace.metadata.oya.microservices]`.

The Shard 1 atomic rename TSV (`/tmp/rename-map.tsv`) includes rows for every `oya-*-object-graph-*` → `oya-ontology-*` mapping before Shard 1 dispatches.

### Reading Bominal docs

When reading Bominal docs/ADRs, translate "Object Graph" → "Ontology" at the point of reading, but cite the Bominal ADR by its actual title with a parenthetical: e.g., "Bominal ADR-0106 (Object Graph architecture = Ontology in oyatie glossary)."

---

## Consequences

### Quality / Performance / Scalability (per ADR-0062)

- Rename is cosmetic at the API level; no performance impact.
- `oya-check-glossary` CI lane (ADR-0018) hard-fails on "Object Graph" tokens post-rename.
- Shard 1 atomic rename handles all crate name changes; no partial state.

### Positive

- Terminology aligns with the established industry term; onboarding cost reduced.
- Eliminates confusion with graph-database terminology.

### Negative

- Shard 1 rename includes all `oya-*-object-graph-*` rows; adds to rename count.

---

## Related

- ADR-0006 (Ontology typed-entity layer — rewritten to use Ontology terminology)
- ADR-0018 (Glossary — "Object Graph" added to forbidden tokens)
- ADR-0059 (Workflow + Ontology = ecosystem adapter layer)
- ADR-0060 (Bominal-inheritance precedence — override #2)
- `[[feedback-glossary-ontology-not-object-graph]]` — session decision 2026-05-13
- Bominal ADR-0106 (Object Graph architecture = Ontology in oyatie glossary)
- Bominal ADR-0107 (Ontology agent gateway, inherited)
- Bominal ADR-0132 (org/person pillar, inherited)
