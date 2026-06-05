---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-007-search-bc
status: pending
owner: axis-recordings
acceptance_lanes: [shardability, statelessness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: Search BC — Meilisearch adapter + transcript indexing

## Intent

Land cross-recording + cross-transcript search via Meilisearch 0.10.0 LTS,
per-tenant index sharded; Cedar-policy server-side filter.

## Concrete crates

`oya-recordings-search-{kernel,domain,usecase,api,adapter-meilisearch,rest,sdk,app}`.

## Acceptance Gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=shardability --microservice recordings
buck2 build //:quality-lane-registry-authority-check # lane=statelessness --microservice recordings
```

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-007-search-bc
depends_on_changesets: [CS-RECORDINGS-IP-004-recording-bc, CS-RECORDINGS-IP-006-transcript-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-008-redaction-bc, CS-RECORDINGS-IP-009-chapter-summary-bcs]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Per-tenant Meilisearch index isolated by namespace `tenant_<id>` | `cargo nextest run -p oya-recordings-search-adapter-meilisearch -- per_tenant_index` |
| AC-02 | Cross-tenant query refused at adapter layer | `cargo nextest run -p oya-recordings-search-adapter-meilisearch -- cross_tenant_refused` |
| AC-03 | Cedar server-side filter prunes results to permitted recordings | `cargo nextest run -p oya-recordings-search-domain -- cedar_filter` |
| AC-04 | 1k-hour archive query p99 ≤ 300ms (PRD §Tenant Outcome 3) | `cargo nextest run -p oya-recordings-search-adapter-meilisearch -- query_1k_hours` |
| AC-05 | `oya gate validate shardability + statelessness --microservice recordings` green | ADR-0131 |

## Build Sequence

1. Kernel: `SearchIndex`, `QueryScope`, `Reindexer` ports.
2. Domain: `IndexableTranscript`, `Hit`, `Facet`.
3. Usecase: `IndexTranscript`, `Query`.
4. Adapter: `-adapter-meilisearch` pinned 0.10.0 LTS.
5. `cargo nextest run -p oya-recordings-search-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-05 (search across recordings + transcripts) |
| PRD-recordings NFR | NFR perf — search p99 ≤ 300ms |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Cross-tenant leak via Meilisearch master key | Per-tenant master key + namespace |
| Reindex storm under bulk transcript finalisation | Coalesce events; batch size 500 |
| Cedar policy bypass via raw Meilisearch call | Cedar filter enforced at usecase layer; lane refuses adapter-direct query |

## References

- Meilisearch documentation (`meilisearch.com/docs`).
- Elasticsearch multi-tenancy reference (Elastic Docs).
- Otter.ai search infrastructure overview (Otter.ai engineering blog).
- ADR-0140 (retired per ADR-0145) (Cedar policy).

## Next IP

[`IP-008-redaction-bc.md`](IP-008-redaction-bc.md)
