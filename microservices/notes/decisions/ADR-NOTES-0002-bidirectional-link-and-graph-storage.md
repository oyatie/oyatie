---
id: ADR-NOTES-0002
status: Accepted
date: 2026-05-17
microservice: notes
deciders: axis-notes, council-architecture
owner: axis-notes
supersedes: []
superseded_by: []
related:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-NOTES-0001
  - ADR-NOTES-0004
related_artifacts:
  - microservices/notes/PRD.md (FR-04, FR-14)
  - microservices/notes/catalog/oya-notes-backlink-graph-kernel.yaml
  - microservices/notes/catalog/oya-notes-graph-view-data-kernel.yaml
purpose: Choose the storage model for Obsidian/Roam-style bidirectional `[[wikilinks]]` and the rendering split between server-side adjacency assembly and client-side force-directed graph view.
---

# ADR-NOTES-0002: Postgres adjacency + materialised backlink table on server; client-side force-directed graph render via WebGL

## Status

Accepted — 2026-05-17.

## Context

The PRD (FR-04 + FR-14) requires Obsidian/Roam-style `[[wikilink]]` bidirectionality + 5k-note graph render p95 ≤ 1s. Three subordinate questions follow:

1. **Storage model for the bidirectional adjacency** — pure-derived (parse on every read) vs. materialised adjacency table.
2. **Resolution semantics** — resolve `[[Foo]]` to a single canonical note vs. ambiguity-allowed.
3. **Graph render locus** — server-side rendered PNG/SVG vs. server-side JSON + client-side WebGL force-directed.

Performance budget (p95 ≤ 1s for 5k-note vault) rules out pure-derived; it would require parsing every note's body on every graph open. Materialised adjacency table is the established pattern (Obsidian uses sqlite + in-memory cache; Roam uses Datomic; Logseq uses local Datascript).

Rendering locus: server-side rendering of force-directed layouts is bandwidth-expensive (full PNG per pan/zoom). Client-side WebGL force-directed (cf. d3-force, fcose, sigma.js) is the industry standard; oyatie can supply nodes + edges as compact JSON and let the client do layout + interaction.

Privacy: Personal-tier notes are E2E-encrypted per ADR-NOTES-0001; the server has no plaintext to parse for `[[wikilinks]]`. Therefore Personal-tier backlink-graph must be *client-side derived* (the client SDK parses local plaintext and stores adjacency in IndexedDB or SQLite). Professional-tier backlink-graph is *server-side materialised* via the `backlink-graph` BC's worker.

## Decision

oyatie notes adopts a **tier-shaped + materialised + client-rendered** storage + render model:

1. **Professional-tier wikilink parsing is server-side.** The `oya-notes-backlink-graph-worker` subscribes to `NoteCreated` + `NoteEdited` events, parses the body via shared `pulldown-cmark` + custom wikilink extension, and writes triplets `(tenant_id, from_note_id, to_note_id, kind)` to a Postgres `backlink` table.

2. **Personal-tier wikilink parsing is client-side.** The TS / Swift / Kotlin / Rust SDKs parse the user's local plaintext and store adjacency in IndexedDB (web) or SQLite (mobile / desktop). The server never sees plaintext.

3. **Wikilink resolution** — `[[Foo]]` resolves to **the most recently accessed note titled "Foo" within the user's vault** (deterministic + UX-discoverable). If no such note exists, the link is *dangling* (rendered as a creation-affordance). If multiple notes share the title, the picker UX disambiguates; the server stores the resolved `to_note_id` on selection.

4. **Backlink table schema** (per-tenant + per-user partition):

   ```sql
   CREATE TABLE backlink (
       tenant_id      TEXT NOT NULL,
       user_id        TEXT NOT NULL,
       from_note_id   ULID NOT NULL,
       to_note_id     ULID NOT NULL,
       kind           backlink_kind NOT NULL,  -- 'explicit' | 'tag' | 'embed'
       resolved_at    TIMESTAMPTZ NOT NULL,
       PRIMARY KEY (tenant_id, from_note_id, to_note_id, kind),
       FOREIGN KEY (tenant_id, from_note_id) REFERENCES note(tenant_id, note_id),
       FOREIGN KEY (tenant_id, to_note_id) REFERENCES note(tenant_id, note_id)
   ) PARTITION BY HASH (tenant_id);

   CREATE INDEX backlink_to_idx ON backlink (tenant_id, to_note_id);
   ```

5. **Graph-view data BC (`oya-notes-graph-view-data-*`)** emits a compact JSON snapshot:

   ```json
   {
     "snapshot_id": "01HGZX...",
     "nodes": [{"id": "01HGZA...", "title": "Foo", "tag_count": 3, "size_class": 2}, ...],
     "edges": [{"from": "01HGZA...", "to": "01HGZB...", "kind": "explicit"}, ...],
     "stats": {"node_count": 5000, "edge_count": 12000}
   }
   ```

   The server-side cap is 50k nodes; beyond that, the snapshot is paginated by tag-cluster.

6. **Client-side force-directed render** uses `sigma.js` + `graphology-layout-forceatlas2` (web) or `wgpu` (Rust desktop) or platform-native (Metal / Vulkan on mobile). Personal-tier graph data is assembled entirely on the client from IndexedDB / SQLite.

7. **Backlink fan-in cap = 50,000** per note. Beyond cap, link write rejected with structured error; UX surfaces "this note is over-linked; consider tagging instead."

8. **Tag-as-pseudo-link**: tags also contribute to the graph (`kind=tag`) but with a different visual style; this gives Obsidian-class graph density without separate tag-graph + backlink-graph views.

## Alternatives Considered

### A. Pure-derived (no materialised adjacency; parse on read)
- Pros: zero storage overhead; always consistent.
- Cons: graph-open p95 well above 1s for any non-trivial vault; cannot meet PRD §FR-14 budget.
- Rejected.

### B. Server-side force-directed render (rasterised PNG / SVG)
- Pros: no client compute; instant on slow devices.
- Cons: kills interactivity (zoom + pan + node-drag); bandwidth expensive; non-accessible (screen reader cannot navigate); rejected by every modern incumbent.
- Rejected.

### C. Graph-DB (Neo4j / DGraph / Memgraph) for adjacency
- Pros: native graph queries; rich traversal.
- Cons: operational overhead (new substrate; LTS version-pinning; replication); per-tenant isolation harder; oyatie does not currently operate a graph-DB substrate.
- Rejected at minimum-shippable-tier; could revisit if traversal queries become dominant.

### D. Tier-shaped + materialised + client-rendered (this ADR's choice)
- Pros: respects ADR-NOTES-0001 E2E posture (Personal client-side); meets perf budget; matches incumbent patterns (Obsidian + Roam + Logseq); leverages existing Postgres substrate.
- Accepted.

### E. Resolve `[[Foo]]` to exact-match-only (no fuzzy + no recency-rank)
- Pros: deterministic.
- Cons: poor UX when multiple notes share title; user expectation per Obsidian/Roam is recency-rank.
- Rejected: recency-rank with picker disambiguation is the right UX.

### F. Block-level references (Roam-style `((block-uuid))`)
- Pros: finer-grained.
- Cons: significant scope expansion; oyatie does not adopt block-model at minimum-shippable-tier (the notes µservice is note-level not block-level); revisit when block-level becomes a user-demand signal.
- Rejected at minimum-shippable-tier; tracked as PRD Open Q successor-IP.

## Consequences

### Positive

- Performance budget achievable for 5k-note vaults; observability via `oya_notes_graph_render_seconds`.
- Personal-tier privacy posture preserved (client-side derivation for Personal).
- Obsidian-class UX matched; matches user expectation from incumbents.
- Tag + backlink + dangling-link surfaced uniformly.
- Backlink fan-in cap prevents pathological vaults from killing the graph view.

### Negative

- Two adjacency-derivation paths (server worker + client SDK). Mitigated by sharing the `pulldown-cmark` wikilink extension across both as a single Cargo crate (used by `oya-notes-backlink-graph-domain` on server and via Rust→Wasm bindings on TS SDK).
- Storage growth: backlink table grows ≈ 3-5× the note count (average 4 links/note); covered in `capacity-model.md`.
- Graph-view JSON snapshot can grow large for vaults > 10k notes; pagination + tag-cluster fallback engaged.

### Operational

- Crate `oya-notes-backlink-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk,app}` enumerated.
- Crate `oya-notes-graph-view-data-{kernel,domain,usecase,api,adapter,sdk,app}` enumerated.
- Worker SLO: `BacklinkResolved` event handled within 5s p99 of `NoteEdited`.
- Backfill / rebuild pipeline per `backfill-replay.md`.
- Backlink fan-in cap enforced at API + DB (CHECK constraint).

## Future

Block-level references (`((block-uuid))`) per PRD Open Q successor-IP; would extend backlink table with `from_block_id` + `to_block_id`. Out-of-scope at minimum-shippable-tier.

## References

- Obsidian Help — graph view + backlinks.
- Roam Research Help — block references.
- Logseq Documentation — graph + page-references.
- sigma.js documentation.
- graphology-layout-forceatlas2.
- `pulldown-cmark` — Rust CommonMark parser.
- ADR-NOTES-0001.
- ADR-NOTES-0004.
- `microservices/notes/PRD.md` FR-04 + FR-14.
- `microservices/notes/capacity-model.md`.
