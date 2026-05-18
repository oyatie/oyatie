---
doc_class: AdrSpec
template_id: TPL-ADR
adr_id: ADR-TRANSLATE-0002
title: Translation Memory and leverage model
status: Accepted
deciders: council-architecture, axis-translate, ops-security
date: 2026-05-17
microservice: translate
supersedes: []
superseded_by: []
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/IP-005-translation-memory-stack.md
doc_status: published
---

# ADR-TRANSLATE-0002 — Translation Memory and leverage model

## Context

Translation Memory (TM) is the value-compounding asset of a TMS — every translated segment becomes a future leverage candidate, dropping cost and improving consistency. The `translate` µservice must implement TM at hyperscaler-grade with three properties simultaneously:

1. **Per-tenant tight isolation** (no cross-tenant or cross-pack match; HARD; FM-13).
2. **Sub-80 ms p99 leverage match** (per PRD §"Performance"; competitive with Phrase + Lokalise + Smartling).
3. **Industry-standard match kinds** — exact (100 %), in-context exact (ICE), fuzzy (75–99 %), no-match.

Industry references:

- **OmegaT** (open-source desktop CAT; `omegat.org/`) — canonical reference for TM leverage algorithm + fuzzy match scoring (token-edit-distance + minhash bucketing).
- **Memsource / Phrase TMS** — leverage model; ICE detection.
- **Trados Studio** — leverage model; perfect-match definitions.
- **MateCat** — fuzzy match scoring (open-source TMS).
- **LISA OSCAR TMX 1.4** (`www.gala-global.org/tmx-14b`) — TM exchange schema.
- **minhash + LSH** (Broder 1997; Indyk & Motwani 1998) — approximate near-neighbor search at sub-linear cost.

## Decision

### 1. Three-kind leverage match

Per ADR-TRANSLATE-0001, the TM leverage match returns one of:

- **`Exact100`** — source segment HMAC-hashed with per-tenant key matches an existing TM unit's HMAC byte-equally. Return target verbatim.
- **`Ice` (In-Context Exact)** — `Exact100` plus context match: previous-segment (and optionally next-segment) HMAC matches. Return target verbatim with `MatchKind::Ice` annotation.
- **`Fuzzy75to99`** — token-normalized minhash similarity ∈ [0.75, 1.00) via LSH bucketing. Return candidate + `similarity_pct` (0..=100).
- **None** — no candidate at threshold; caller falls through to engine.

### 2. Per-tenant HMAC keying for ground-truth isolation

Every per-tenant TM segment is hashed with HMAC-BLAKE3 using a tenant-specific key resolved from `openbao://<pack>/<tenant>/translate/tm-hash-key`. Tenant keys differ → identical source segments hash to different bytes across tenants → cross-tenant exact match becomes **structurally impossible**, not just policy-impossible.

### 3. Per-tenant Meilisearch index isolation

Each tenant gets a dedicated Meilisearch index (`tm-<tenant>-<project|global>`). Search is per-index, so even at the search-engine layer there is no cross-tenant query path. Defense-in-depth: index isolation + Postgres RLS + Cedar policy default-deny.

### 4. Minhash-LSH for fuzzy matching

- Tokenize per source-lang using Unicode tokenizer (per `unicode-segmentation` crate) + locale-aware normalization (per `unicode-normalization` crate; NFKC + lowercase + remove combining marks).
- Compute minhash signature (128 permutations) per token-set.
- LSH bucketing — 8 bands × 16 rows for similarity-threshold ≥ 0.75.
- Postgres stores canonical source + target + minhash signature.
- Meilisearch indexes signature + ngrams for sub-80 ms p99 lookup.

### 5. ICE context modeling

ICE detection requires prior-segment context (optionally next-segment). For paragraph-level translation, "context" is the immediately-preceding TU within the same document/project. Stored as `context_prev` / `context_next` per TU; HMAC-hashed alongside source. ICE match requires both source HMAC and context HMAC byte-equal.

### 6. Per-tenant + per-project scoping

TM units default to **per-project**; a tenant may opt into a **global tenant-wide TM** that spans all projects. Cross-project leverage forbidden by default; tenant admin opts in.

### 7. DSR cascade

When a tenant raises a DSR (right-to-erasure per GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)), the `oya-dsr-cascade-runner` walks Postgres `tm_units` matching the subject's identifiers, soft-deletes (30 d grace), then hard-deletes. Meilisearch index re-syncs from Postgres on each cycle. Audit-chain seals every cycle.

## Alternatives Considered

### Alternative A — No HMAC; rely solely on Postgres RLS + Cedar

- **Pros**: simpler; one fewer concern.
- **Cons**: RLS + Cedar are policy-layer; a misconfiguration could match cross-tenant; HMAC makes cross-tenant exact match **structurally impossible** (different keys → different bytes), defense-in-depth.
- **Verdict**: rejected. HMAC + RLS + Cedar together are belt + suspenders + structural impossibility.

### Alternative B — Single shared Meilisearch index with tenant filter

- **Pros**: simpler operations; one index per pack.
- **Cons**: a filter-bypass bug = cross-tenant match; per-tenant index is hard-isolation.
- **Verdict**: rejected.

### Alternative C — Use a hosted TM service (e.g., Smartling TM)

- **Pros**: zero ops.
- **Cons**: tenant data leaves oyatie; residency invariant broken; cost; vendor lock-in.
- **Verdict**: rejected per residency posture.

### Alternative D — Postgres-only (no Meilisearch); SQL similarity (trigram, levenshtein)

- **Pros**: one fewer system to operate.
- **Cons**: Postgres trigram/levenshtein cost-prohibitive at 80 ms p99 for ≥ 100k TM units per tenant; Meilisearch is purpose-built.
- **Verdict**: rejected.

### Alternative E — Hashes only (no token+minhash for fuzzy)

- **Pros**: simpler; faster.
- **Cons**: no fuzzy match; tenants lose 75–99 % match category which is where most leverage value lives (industry data: 30–60 % of leverage by volume).
- **Verdict**: rejected.

### Alternative F — Embedding-based semantic match (sentence-embedding cosine)

- **Pros**: catches paraphrastic matches missed by token-edit-distance.
- **Cons**: 10–100× cost; embedding-drift across model versions; harder to explain to LSPs; not industry-standard for TM leverage.
- **Verdict**: scheduled-for-distinct-tracked-work; tracked as "TM v2 semantic-leverage" future ADR. Token-edit-distance per OmegaT remains industry baseline for M01.

## Consequences

### positive

1. **Cross-tenant TM match is structurally impossible** — HMAC + per-tenant Meilisearch index + RLS + Cedar.
2. **Per-tenant TM value compounds** — every translated segment becomes a future leverage candidate; tenant cost-per-translation drops over time.
3. **Industry-standard match kinds** — interop with OmegaT / Memsource / Phrase / MateCat conventions.
4. **Sub-80 ms p99 leverage match** — Meilisearch + minhash-LSH achieve this for ≥ 100k TM units per tenant.

### negative

1. **HMAC key rotation cost** — per-tenant key rotation requires re-hashing all per-tenant TM units; expensive at scale (10 m–1 h per tenant); mitigated by quarterly cadence + per-tenant rotation.
2. **Per-tenant Meilisearch index per pack** — operational complexity scales with tenant count; HPA on Meilisearch sizing; eviction of inactive tenants beyond storage budget.
3. **Minhash-LSH false-positive rate at threshold** — ~ 1 % false-positive at 75 % threshold; mitigated by candidate-set verification with exact token-edit-distance before returning.

### neutral

1. **ICE context model bounded to single-prior-segment by default** — multi-segment context optional per tenant.
2. **DSR cascade SLA** is determined by the strictest pack overlap (e.g., LGPD 15 d for pack-br); not a problem but worth documenting.
3. **TM export (TMX 1.4)** is lossless; tenant ownership preserved.

## Validation

- `tests/integration/postgres_rls_enforces_tenant_id.rs` — RLS verified.
- `tests/integration/meilisearch_index_isolated.rs` — index isolation verified.
- `tests/integration/cross_tenant_lookup_returns_none.rs` — HMAC structural impossibility verified.
- `tests/load/tm_leverage_p99_under_80ms.rs` — performance bar.
- WMT-eval set rerun per-quarter to verify fuzzy-match thresholds still relevant.

## References

- OmegaT — `omegat.org/` (canonical reference for TM leverage algorithm + fuzzy match scoring).
- Memsource / Phrase TMS leverage model.
- Trados Studio leverage model.
- MateCat leverage scoring.
- LISA OSCAR TMX 1.4 — `www.gala-global.org/tmx-14b`.
- minhash: Broder, A. "On the resemblance and containment of documents" 1997.
- LSH: Indyk + Motwani 1998.
- Meilisearch docs.
- ADR-0028 — audit-chain (TmUpdated seal).
- ADR-0117 — pack residency (per-tenant index isolation).
- ADR-TRANSLATE-0001 — engine routing (TM leverage short-circuit).
- ADR-TRANSLATE-0004 — residency-bound inference (TM honors pack boundary).
- GDPR Art. 17; PIPA Art. 36; DPDPA §12; LGPD Art. 18(V) (right-to-erasure).
