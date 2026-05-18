---
id: ADR-NOTES-0004
status: Accepted
date: 2026-05-17
microservice: notes
deciders: axis-notes, council-privacy, council-architecture
owner: axis-notes
supersedes: []
superseded_by: []
related:
  - ADR-0105
  - ADR-0131
  - ADR-NOTES-0001
  - ADR-NOTES-0005
related_artifacts:
  - microservices/notes/PRD.md (FR-13; Open Question #1)
  - microservices/notes/catalog/oya-notes-search-index-kernel.yaml
  - microservices/notes/catalog/oya-notes-search-index-adapter-meilisearch.yaml
purpose: Resolve PRD Open Question #1 — pick a primary client-side encrypted-search design for Personal-tier notes, and lock the Professional-tier search backend.
---

# ADR-NOTES-0004: Meilisearch 0.10.0 LTS is the Professional-tier server-side search backend; Personal-tier search is client-side encrypted-inverted-index in IndexedDB / SQLite with per-note token-bloom-filters

## Status

Accepted — 2026-05-17. Closes PRD Open Question #1.

## Context

The PRD §FR-13 requires full-text search across notes (tag-faceted + cross-note) with p95 latencies ≤ 100ms for tag-search and ≤ 200ms for full-text-search.

The E2E posture (ADR-NOTES-0001) creates a hard constraint: **the server has no plaintext access to Personal-tier note bodies**, so the server cannot build a traditional inverted index for them. Two architectures must coexist:

- **Professional-tier search** — server-side traditional inverted index over plaintext (server has tenant-DEK envelope decryption authority for Cedar-scoped operations).
- **Personal-tier search** — client-side encrypted index, with the server holding only opaque encrypted state.

### Professional-tier backend candidates

| Candidate | License | Strengths | Weaknesses |
|---|---|---|---|
| Meilisearch 0.10.0 | MIT | mature; LTS; per-tenant index; faceted; fast cold; small ops footprint | rust-only embedding |
| Elasticsearch / OpenSearch | dual-licence / Apache | rich; battle-tested at scale | heavyweight; JVM ops; license complexity |
| Tantivy 0.22 | MIT | Rust-native embeddable; same as messenger | requires server-side embedding only (no separate search node) |
| Typesense | GPL | fast; HTTP-first | small adoption; fewer features |
| Quickwit | AGPL | log-shaped; S3-backed | mismatch with notes shape (low-volume vs. log-stream) |

For notes-shaped workloads (tens-of-millions of small docs per tenant + low cardinality of facets) Meilisearch hits the sweet spot. Sibling-µservice docs is in flight on Meilisearch alignment; messenger uses Meilisearch + Tantivy; the messenger ADR-MSGR-0003 selects Meilisearch as primary with Tantivy fallback for single-cell. For notes the per-tenant index model is the right grain.

### Personal-tier client-side encrypted-search candidates

Three families of approaches:

1. **Encrypted-inverted-index on the client** (token-bloom-filters per note + posting list in IndexedDB / SQLite). Pros: simple; well-understood; query is "tokenise → bloom-filter membership per note → fetch matching candidates → exact-grep on plaintext locally." Cons: bloom-filter false-positive rate; query has linear scan over `O(notes)` per query worst-case (mitigated by tenant-side partitioning by daily-note).
2. **Searchable symmetric encryption (SSE)** — encrypted index with trapdoor permutations (Curtmola / Cash / Jarecki literature). Pros: theoretically constant-time queries. Cons: complex implementation; high storage overhead; trapdoor leakage on repeated queries (cf. "leakage abuse attacks"); brittle.
3. **Order-preserving / order-revealing encryption (OPE/ORE)** — supports range queries. Pros: range queries possible. Cons: significant cryptographic leakage; widely deprecated for sensitive content (cf. Naveed et al. 2015 attacks).

The encrypted-inverted-index with per-note token-bloom-filters is the **most pragmatic + most-cited** approach in current literature for E2E notes (cf. Standard Notes search architecture, Signal's encrypted-attachment-name search, Tutanota's search). It accepts a small false-positive rate (resolved by client-side exact-grep on candidates) in exchange for implementation simplicity + no novel cryptography.

## Decision

oyatie notes adopts **Meilisearch 0.10.0 LTS for Professional-tier server-side search + client-side encrypted-inverted-index for Personal-tier search**:

1. **Professional-tier server-side search**: Meilisearch 0.10.0 LTS per-tenant namespace.
   - Per-tenant index name `tenant_<id>`.
   - On `NoteCreated` / `NoteEdited` (Professional-tier only), `oya-notes-search-index-worker` decrypts tenant-DEK envelope, tokenises, and submits to Meilisearch.
   - On `NoteDeleted`, the worker purges the doc.
   - Cedar-scoped server-side filter: search results filtered post-Meilisearch by `oya-notes-note-store-usecase` re-evaluating per-doc Cedar; no client-side trust.
   - Faceted by tag + notebook + created_at.

2. **Personal-tier client-side encrypted-search** (closes PRD Open Q #1):
   - Per-note encrypted index entries: `(note_id, bloom_filter_bitmap)` where `bloom_filter` is a 4096-bit bloom over normalized tokens of the note body. Bitmap encrypted with the user's per-vault key (derived from MLS group key).
   - Index storage: web → IndexedDB; iOS / macOS → CoreData; Android → SQLite; desktop Rust → SQLite.
   - Server stores **only** the encrypted bitmap blob (never the plaintext bitmap).
   - Query flow:
     1. Client tokenises query.
     2. For each query token, compute bloom-set membership across local `note_id → bitmap` map.
     3. Candidate notes (bloom match) fetched in encrypted form from server, decrypted locally, exact-grep verified.
     4. Results returned to user.
   - False-positive rate: ≤ 1 % at 4096-bit bloom with ≤ 1024 tokens/note (acceptable; verified by client-side exact-grep).
   - Tag search on Personal-tier notes is also client-side: the client maintains a local `(tag → note_id[])` map decrypted from server-stored per-tag encrypted bitmaps.
   - Performance target: tag-search p95 ≤ 100ms; full-text-search p95 ≤ 250ms on 5,000-note vault on commodity device (matches PRD §NFR Performance).

3. **Cross-tier search refused**: a single query cannot return both Personal-tier and Professional-tier results. The Cedar evaluator + UX makes this explicit — users switch persona to switch search scope.

4. **Meilisearch deployment**: per-pack Meilisearch cluster; per-tenant namespace; pinned version 0.10.0 LTS; `oya gate validate version-pinning-conformance` enforces.

5. **AI-search (semantic embedding search) refused on E2E content** per ADR-NOTES-0005. Professional-tier semantic search MAY be added as a follow-on capability (T1) opt-in.

## Alternatives Considered

### A. Server-side search for both tiers (Meilisearch over decrypted plaintext for Personal too)
- Pros: uniform pipeline; trivial to implement.
- Cons: violates ADR-NOTES-0001 E2E posture; server holds plaintext index = decryption oracle; rejected.
- Rejected.

### B. Client-side search for both tiers (no server-side index even for Professional)
- Pros: uniform pipeline; minimum server compute.
- Cons: large tenant search on 100M-note Professional vault is impractical client-side; bandwidth + sync cost prohibitive; Cedar-scoped filtering still required server-side anyway.
- Rejected.

### C. Tier-split with Meilisearch for Professional + encrypted-inverted-index in IndexedDB for Personal (this ADR's choice)
- Pros: respects ADR-NOTES-0001; performance budgets met; matches Standard Notes / Signal / Tutanota precedent.
- Accepted.

### D. Tier-split with Meilisearch for Professional + searchable-symmetric-encryption (SSE) for Personal
- Pros: theoretically faster client-side query.
- Cons: SSE leakage-abuse attacks (Cash + Grubbs + Perry + Ristenpart 2015); high storage overhead; complex implementation; brittle to tokenisation changes.
- Rejected.

### E. Tier-split with Meilisearch for Professional + order-preserving / order-revealing encryption (OPE/ORE) for Personal
- Pros: supports range queries.
- Cons: catastrophic leakage per Naveed et al. 2015 + Durak et al. 2016; widely deprecated; no responsible engineer ships OPE/ORE on user-content.
- Rejected.

### F. Elasticsearch / OpenSearch for Professional
- Pros: rich; battle-tested.
- Cons: JVM ops overhead; over-featured for notes workload; sibling alignment (messenger + docs use Meilisearch) prefers Meilisearch.
- Rejected for ops cost + sibling alignment.

### G. Tantivy embedded inside oya-notes-search-index-worker (no separate search node)
- Pros: minimal infra; matches messenger fallback.
- Cons: per-tenant index management harder; replica architecture harder; messenger uses Tantivy only as single-cell fallback (per ADR-MSGR-0003); notes-µservice expects multi-cell from M02.
- Rejected as primary; could revisit as single-cell fallback later.

## Consequences

### Positive

- ADR-NOTES-0001 E2E posture preserved; server has no plaintext access to Personal-tier content (no index over plaintext).
- Performance budgets met for both tiers.
- Sibling alignment with messenger + docs (Meilisearch).
- Bloom-filter false-positive rate documented + measurable; client-side exact-grep covers correctness.

### Negative

- Two code paths for search (server worker + client SDK). Mitigated by sharing the tokeniser / normaliser as a Rust crate + Wasm bindings.
- Personal-tier search index lives client-side; if user loses devices + seed, index is also destroyed (consistent with note content destruction per ADR-NOTES-0001).
- Client storage cost: ≈ 4KB encrypted bitmap per note in IndexedDB; for a 10k-note vault that's ~ 40MB per device. Acceptable.

### Operational

- Crate `oya-notes-search-index-{kernel,domain,usecase,api,adapter-meilisearch,worker,sdk,app}` enumerated.
- Meilisearch per-pack cluster in Helm chart.
- Backfill via `backfill-replay.md` "Search Index Rebuild" procedure.
- Personal-tier client-side bloom-filter rebuild logic in TS / Swift / Kotlin / Rust SDK.
- SLO `notes-full-text-search-latency.openslo.yaml` (under `slos/`).
- Periodic Meilisearch index-perm audit (security check; per `threat-model.md` T-I-09b).

## Future (Semantic / Vector Search)

Vector search (embedding-based semantic search) is intentionally out-of-scope at MVP. If added, it ships as:

- T1 capability declared in `capabilities/T1-assist.yaml`.
- Professional-tier-only with tenant-admin opt-in.
- E2E refusal invariant applies (per ADR-NOTES-0005).
- Vector backend candidates: Qdrant 1.x (preferred for Rust ecosystem); or Meilisearch built-in vector support (Meilisearch ≥ 1.6 supports vectors; pin gate engages).

## References

- Curtmola et al. — "Searchable Symmetric Encryption" (2006).
- Cash et al. — "Leakage-Abuse Attacks Against Searchable Encryption" (CCS 2015).
- Naveed et al. — "Inference Attacks on Property-Preserving Encrypted Databases" (CCS 2015).
- Durak et al. — "What Else is Revealed by Order-Revealing Encryption?" (CCS 2016).
- Standard Notes Search Architecture (publicly documented).
- Signal Search of Encrypted Attachments (developer docs).
- Tutanota Search Architecture.
- Meilisearch 0.10.0 LTS documentation.
- ADR-MSGR-0003 (sibling search backend selection).
- ADR-NOTES-0001 (E2E posture).
- ADR-NOTES-0005 (AI bounds).
- `microservices/notes/PRD.md` Open Question #1.
