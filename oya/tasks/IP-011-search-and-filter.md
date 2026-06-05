---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-011-search-and-filter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, oya-governance-amendment-3-backend-qualified-adapter, search-degraded-fallback]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: search-index BC — Meilisearch 0.10.0 LTS + rebuildable fallback

## Intent

Ship the `search-index` BC end-to-end backed by Meilisearch 0.10.0 LTS.
Per-tenant Meilisearch index (per-tenant index key + per-tenant master
key envelope-encrypted via OpenBao). Indexed fields: `title`,
`description` (truncated to 4KB), `labels`, `status`, `priority`,
`assignees`, `due_at`, `project_id`. Per-tenant master keys mean
Meilisearch cluster admins cannot read tenant content (Bominal ADR-
0111 envelope-encryption pattern extended to search).

Degraded fallback path per PRD AC-09: a full Meilisearch index loss
degrades the search-rest crate to direct-Postgres `tsvector` lookups
within ≤ 1 min (GIN index from IP-004). Rebuild completes in ≤ 30 min
for 10M tasks via the `search-index-worker` crate.

## ChangeSet boundary

8 search-index crates (kernel/domain/usecase/api/adapter/adapter-
meilisearch/worker/app). Per ADR-0105 Amendment 3:
`-adapter-meilisearch` is the backend-qualified adapter.

## Crate Naming

`oya-tasks-search-index-*` per ADR-0056 v4.1; `-adapter-meilisearch`
per ADR-0105 Amendment 3 backend-qualification.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-search-index-{kernel,domain,usecase,api,adapter,adapter-meilisearch,worker,app}/src/lib.rs` | created/replaced | 8-crate stack |
| `microservices/tasks/src/oya-tasks-search-index-domain/tests/filter_dsl.rs` | created | parser tests |
| `microservices/tasks/src/oya-tasks-search-index-worker/tests/rebuild_bench.rs` | created | 10M-task rebuild bench |
| `microservices/tasks/tests/integration/search-degraded-fallback.rs` | created | E2E fallback |
| `microservices/tasks/catalog/oya-tasks-search-index-*.yaml` | created | catalog entries |

## Acceptance Gates

```bash
cargo test -p oya-tasks-search-index-domain
cargo test -p oya-tasks-search-index-adapter-meilisearch
cargo bench -p oya-tasks-search-index-worker rebuild
buck2 build //:quality-lane-registry-authority-check # lane=search-degraded-fallback --microservice tasks
buck2 build //:quality-lane-registry-authority-check # lane=amendment-3-backend-qualified-adapter --crate oya-tasks-search-index-adapter-meilisearch
```

## Test Plan

- Cross-project Meilisearch search p95 ≤ 300ms.
- Per-tenant key isolation: tenant A cannot read tenant B's index even
  with cluster admin access (envelope encryption).
- Degraded fallback: kill Meilisearch container; search-rest serves
  direct-Postgres results within 60s with degraded-mode header set.
- Rebuild benchmark: 10M synthetic tasks rebuild ≤ 30 min on standard
  CI runner.

## Halt Conditions

- Cross-tenant index leakage detected — refuse to ship; P0.
- Rebuild bench exceeds 30 min — investigate; do not relax the AC.

## Next IP

[`IP-012-bulk-edit-pipeline.md`](IP-012-bulk-edit-pipeline.md)

## References

- ADR-TASKS-0004 (search backend choice — aligned with view-engine's
  search-degraded fallback).
- Meilisearch 0.10 LTS — `docs.meilisearch.com`.
- Bominal ADR-0111 (envelope encryption).
