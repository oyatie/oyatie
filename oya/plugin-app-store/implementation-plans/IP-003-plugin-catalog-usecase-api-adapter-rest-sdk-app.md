---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app: plugin-catalog remaining layers (usecase + api + adapter + adapter-postgres + rest + sdk + app)

## Intent

Wire the plugin-catalog BC end-to-end: usecase orchestrators + typed API + Postgres-backed search adapter (with tsvector full-text index) + REST routes + SDK client + composition-root app. This is the read-heavy serving path; needs Cilium L4 cache integration.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-plugin-catalog-usecase`
- `oya-plugin-app-store-plugin-catalog-api`
- `oya-plugin-app-store-plugin-catalog-adapter`
- `oya-plugin-app-store-plugin-catalog-adapter-postgres`
- `oya-plugin-app-store-plugin-catalog-rest`
- `oya-plugin-app-store-plugin-catalog-sdk`
- `oya-plugin-app-store-plugin-catalog-app`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-usecase/src/lib.rs` | create | BrowseUsecase, SearchUsecase, GetPluginUsecase, ListVersionsUsecase |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-api/src/lib.rs` | create | BrowseRequest/Response, SearchRequest/Response, GetPluginRequest/Response |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-adapter-postgres/migrations/V1__initial.sql` | create | plugins + plugin_versions + plugin_ratings + plugin_categories tables |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-adapter-postgres/migrations/V2__tsvector_search.sql` | create | GIN index on plugin description tsvector |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-adapter-postgres/src/lib.rs` | create | PostgresPluginRepo + PostgresSearchIndex impl |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-rest/src/lib.rs` | create | Axum routes: GET /v1/plugins, GET /v1/plugins/:id, GET /v1/plugins/search |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-sdk/src/lib.rs` | create | PluginCatalogClient (TS-codegen-ready) |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-app/src/main.rs` | create | composition root: bind PostgresPluginRepo to BrowseUsecase, serve Axum router |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
// usecase
pub struct BrowseUsecase<R: PluginRepo, S: SearchIndex> {
    repo: Arc<R>,
    index: Arc<S>,
}

impl<R: PluginRepo, S: SearchIndex> BrowseUsecase<R, S> {
    pub async fn search(&self, req: SearchRequest) -> Result<SearchResponse, ApiError> {
        let ids = self.index.search(&req.into()).await?;
        let plugins = self.repo.fetch_many(&ids).await?;
        let ranked = oya_plugin_app_store_plugin_catalog_domain::search::rank(&plugins, &req.into());
        Ok(SearchResponse::from(ranked))
    }
}

// adapter-postgres
#[async_trait]
impl SearchIndex for PostgresSearchIndex {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<PluginId>, PluginCatalogError> {
        sqlx::query!("SELECT id FROM plugins WHERE search_tsv @@ plainto_tsquery($1) ORDER BY ts_rank(search_tsv, plainto_tsquery($1)) DESC LIMIT $2", query.text, query.limit as i64)
            .fetch_all(&self.pool)
            .await
            .map(|rows| rows.into_iter().map(|r| PluginId::from(r.id)).collect())
            .map_err(PluginCatalogError::from)
    }
}
```

Layer assignment compliance (per ADR-0105 13-layer enum):
- `*-kernel` crates declare port traits + value types only; no dependencies on other project crates.
- `*-domain` crates implement pure domain logic; depend on `*-kernel` only.
- `*-usecase` crates orchestrate domain calls; depend on `*-kernel` + `*-domain` only.
- `*-adapter*` crates implement port traits against concrete backends; depend on `*-kernel` + `*-domain` + `*-usecase`; NEVER imported directly by `*-rest` or `*-app`.
- `*-rest` crates expose HTTP routes; depend on `*-kernel` + `*-api` + `*-usecase`.
- `*-worker` crates run long-lived loops; same dependency rules as `*-rest`.
- `*-app` crates are composition roots; the only crates allowed to wire concrete `*-adapter*` instances to `*-usecase` ports.

Port-in-kernel rule (per ADR-0064 SWEEP-I) is enforced by the `port-location` CI lane.

## Acceptance Gates

All gates must exit 0 before this IP is `verified`:

```bash
cargo check -p oya-plugin-app-store-plugin-catalog-usecase --all-features
cargo build -p oya-plugin-app-store-plugin-catalog-usecase --all-features
cargo clippy -p oya-plugin-app-store-plugin-catalog-usecase --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-plugin-catalog-usecase --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-plugin-catalog-usecase --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `test_browse_usecase_pagination` | Pagination cursor stable across requests |
| `test_search_usecase_full_text` | Full-text search returns expected hits ranked by ts_rank |
| `test_get_plugin_usecase_not_found` | Returns NotFound for unknown id |
| `test_postgres_search_index_tsvector` | GIN index used (EXPLAIN ANALYZE) |
| `test_postgres_repo_optimistic_concurrency` | Concurrent updates produce one winner |
| `test_rest_get_plugins_happy_path` | 200 + JSON shape |
| `test_rest_get_plugins_unauthenticated` | 401 for missing token |
| `test_rest_get_plugins_tenant_filter` | Tenant-scoped filter applied |
| `test_sdk_search_retry_on_5xx` | SDK retries with exponential backoff |
| `test_app_startup_shutdown_smoke` | App binds port, accepts /health, shuts down gracefully on SIGTERM |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-003-plugin-catalog-usecase-api-adapter-rest-sdk-app/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Search p95 > 200ms.
- GIN index not used (EXPLAIN ANALYZE shows seq scan).
- Optimistic concurrency results in lost updates.
- REST returns 5xx > 0.1% in steady state.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app`
- `microservice`: `plugin-app-store`
- `milestone`: `M04-ecosystem-substrate`
- `phase`: `P01-plugin-app-store-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-004-plugin-lifecycle-state-machine`](IP-004-plugin-lifecycle-state-machine.md)

## References

- PRD §plugin-catalog
- Postgres tsvector docs
- Axum 0.7 docs
- ADR-0185 (OpenAPI 3.2 codegen)
