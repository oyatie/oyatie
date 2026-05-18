---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-005-translation-memory-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, tenant-isolation-rls]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: Translation Memory stack (`oya-translate-tm-*`)

## Intent

Per-tenant per-project TM with leverage-match scoring (exact 100 % + ICE + fuzzy via minhash-LSH per ADR-TRANSLATE-0002). Postgres = ground truth; Meilisearch = leverage index.

## ChangeSet boundary

10 new Rust crates per ADR-0105 13-value layer enum:

- `oya-translate-tm-kernel` — ports + entities (`TmUnit`, `LeverageMatch`, `Project`, `Segment`)
- `oya-translate-tm-domain` — minhash-LSH similarity bucketing; ICE detection; HMAC per-tenant hashing
- `oya-translate-tm-usecase` — orchestration
- `oya-translate-tm-api` — DTO surface
- `oya-translate-tm-adapter-postgres` — RLS-isolated repo
- `oya-translate-tm-adapter-meilisearch` — leverage index
- `oya-translate-tm-rest` — REST surface
- `oya-translate-tm-worker` — minhash recompute job; Meilisearch sync
- `oya-translate-tm-sdk` — client
- `oya-translate-tm-app` — composition root

## Key Algorithm — Leverage Match

Per ADR-TRANSLATE-0002:

1. **Exact (100 %)**: BLAKE3(per-tenant HMAC key, source_segment) match → return target.
2. **ICE (In-Context Exact)**: Exact + previous-segment context match (joint hash over prior 1–2 segments) → return target with `MatchKind::Ice`.
3. **Fuzzy (75–99 %)**: Minhash-LSH over (tokenized source); cosine similarity threshold ≥ 0.75; bucket per percent → return top candidate with `similarity_pct`.
4. **None**: no match → caller falls through to engine.

Per-tenant HMAC keys live in OpenBao at `openbao://<pack>/<tenant>/translate/tm-hash-key`; pre-fetched on TM-rest startup; rotated quarterly per `runbooks/tm-corruption-restore.md`.

Cross-tenant matches are **structurally impossible**: HMAC key differs per tenant → identical source segment hashes differently → Meilisearch index isolated per tenant → Postgres RLS double-binds.

## Postgres Schema (Excerpt)

```sql
CREATE TABLE tm_units (
  id UUID PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT,
  source_lang TEXT NOT NULL,         -- BCP 47
  target_lang TEXT NOT NULL,         -- BCP 47
  source_segment TEXT NOT NULL,
  source_segment_hmac BYTEA NOT NULL,   -- HMAC-BLAKE3 per-tenant
  target_segment TEXT NOT NULL,
  context_prev TEXT,                 -- previous segment for ICE
  context_next TEXT,
  metadata JSONB,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  created_by TEXT NOT NULL,          -- principal
  origin TEXT NOT NULL               -- 'human' | 'mt' | 'post-edit'
);
CREATE INDEX idx_tm_units_tenant_hmac ON tm_units (tenant_id, source_segment_hmac);
CREATE INDEX idx_tm_units_tenant_lang_pair ON tm_units (tenant_id, source_lang, target_lang);
ALTER TABLE tm_units ENABLE ROW LEVEL SECURITY;
CREATE POLICY tm_units_tenant_isolation ON tm_units
  USING (tenant_id = current_setting('oya.tenant_id'));
```

## Meilisearch Index per Tenant

Index name: `tm-<tenant>-<project|global>`. Searchable: `source_segment` + `source_lang` + `target_lang`. Filterable: `tenant_id` (defense-in-depth even though per-tenant index already enforces).

## Test Plan

| Test | Verifies |
|---|---|
| `test_exact_match_uses_per_tenant_hmac` | T-04 / FM-13 prevented |
| `test_cross_tenant_lookup_returns_none` | RLS + Cedar deny |
| `test_ice_match_requires_context_match` | ICE invariant |
| `test_fuzzy_minhash_lsh_threshold_075` | bucket boundary |
| `test_tm_update_seals_audit_chain` | `TmUpdated` event |
| `test_meilisearch_reindex_from_postgres` | Scenario B replay |
| `test_dsr_erasure_propagates_to_meilisearch` | DSR cascade |
| `tests/integration/postgres_rls_enforces_tenant_id.rs` | RLS verified at session level |
| `tests/integration/meilisearch_index_isolated.rs` | per-tenant index isolation |

## Halt Conditions

- Cross-tenant TM match observed (P0; FM-13).
- HMAC per-tenant key reuse across tenants.
- DSR cascade fails to propagate to Meilisearch.

## Next IP

[`IP-006-termbase-and-glossary-stack.md`](IP-006-termbase-and-glossary-stack.md)
