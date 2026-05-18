---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-002-plugin-catalog-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002-plugin-catalog-kernel-domain: oya-plugin-app-store-plugin-catalog-{kernel,domain}

## Intent

Pure kernel + domain layer for plugin-catalog BC: Plugin, PluginVersion, PluginRating, PluginCategory, PluginAuthor entities + value types + pure search/filter/rank logic. No I/O; no async; no allocations beyond necessary. Bedrock for IP-003 usecase wiring.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-plugin-catalog-kernel`
- `oya-plugin-app-store-plugin-catalog-domain`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-kernel/Cargo.toml` | create | kernel manifest; no deps beyond serde + thiserror |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-kernel/src/lib.rs` | create | module declarations only |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-kernel/src/entities.rs` | create | Plugin, PluginVersion, PluginRating, PluginCategory, PluginAuthor |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-kernel/src/value_types.rs` | create | PluginId (ULID), VersionString (semver), Slug, Rating (0..=5) |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-kernel/src/ports.rs` | create | PluginRepo, RatingRepo, CategoryRepo, SearchIndex port traits |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-kernel/src/errors.rs` | create | PluginCatalogError variants |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-domain/Cargo.toml` | create | domain manifest; depends on kernel only |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-domain/src/lib.rs` | create | module declarations |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-domain/src/search.rs` | create | pure search + filter + rank algorithm |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-domain/src/rating_aggregate.rs` | create | pure rating aggregation (avg + weighted by recency) |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-domain/src/category_tree.rs` | create | pure category tree traversal |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-domain/tests/search_property.rs` | create | proptest: search is monotonic + idempotent |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
// kernel: ports.rs
pub trait PluginRepo: Send + Sync {
    fn fetch_by_id(&self, id: &PluginId) -> impl Future<Output = Result<Option<Plugin>, PluginCatalogError>> + Send;
    fn list_published(&self, filter: PublishedFilter, pagination: Pagination) -> impl Future<Output = Result<Page<Plugin>, PluginCatalogError>> + Send;
}

pub trait SearchIndex: Send + Sync {
    fn search(&self, query: &SearchQuery) -> impl Future<Output = Result<Vec<PluginId>, PluginCatalogError>> + Send;
}

// domain: search.rs
pub fn rank(plugins: &[Plugin], query: &SearchQuery) -> Vec<RankedPlugin> {
    // pure: text match score + rating boost + recency boost + install-count boost
    // monotonic in score; idempotent for identical inputs
    plugins.iter().map(|p| RankedPlugin { plugin: p.clone(), score: compute_score(p, query) })
                  .sorted_by(|a, b| b.score.partial_cmp(&a.score).unwrap())
                  .collect()
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
cargo check -p oya-plugin-app-store-plugin-catalog-kernel --all-features
cargo build -p oya-plugin-app-store-plugin-catalog-kernel --all-features
cargo clippy -p oya-plugin-app-store-plugin-catalog-kernel --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-plugin-catalog-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-plugin-catalog-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate port-location --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `test_plugin_id_ulid_parse_roundtrip` | PluginId parses and roundtrips ULID format |
| `test_version_string_semver_validation` | VersionString rejects non-semver inputs |
| `test_rating_bounded_0_5` | Rating constructor rejects out-of-range values |
| `test_search_monotonic` | proptest: more search hits never lowers a hit's rank |
| `test_search_idempotent` | proptest: identical query returns identical rank |
| `test_rating_aggregate_weighted_recency` | Recent ratings weighted higher than old |
| `test_category_tree_dfs_traversal` | DFS traversal visits every node exactly once |
| `test_category_tree_no_cycles` | Cycle detection rejects self-referential category trees |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-002-plugin-catalog-kernel-domain/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Domain functions exhibit non-determinism (different output on same input).
- Kernel imports any crate from another product µservice.
- Domain imports any adapter or rest crate.
- Coverage falls below 95% line for domain crate.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-002-plugin-catalog-kernel-domain`
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

[`IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app`](IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app.md)

## References

- ADR-0056 (BNF v4.1)
- ADR-0105 (13-layer enum + check family patterns)
- ADR-0064 (port-in-kernel rule)
- ADR-0213 §plugin-catalog BC
- PRD §plugin-catalog
