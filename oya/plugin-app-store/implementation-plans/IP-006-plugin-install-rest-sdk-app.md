---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-006-plugin-install-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006-plugin-install-rest-sdk-app: plugin-install rest + sdk + app + adapter-postgres

## Intent

Expose plugin-install BC over REST + SDK; persist installations to Postgres; compose into the plugin-app-store app binary.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-plugin-install-rest`
- `oya-plugin-app-store-plugin-install-sdk`
- `oya-plugin-app-store-plugin-install-app`
- `oya-plugin-app-store-plugin-install-adapter-postgres`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-rest/src/lib.rs` | create | POST /v1/installations, DELETE /v1/installations/:id, GET /v1/installations |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-adapter-postgres/migrations/V1__installations.sql` | create | installations table + capability_grants table |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-sdk/src/lib.rs` | create | InstallClient (codegen-ready) |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-app/src/main.rs` | create | composition root |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
// REST
async fn install(
    State(orch): State<Arc<InstallOrchestrator<...>>>,
    Json(req): Json<InstallRequest>,
) -> Result<(StatusCode, Json<Installation>), ApiError> {
    let installation = orch.install(req).await?;
    Ok((StatusCode::CREATED, Json(installation)))
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
cargo check -p oya-plugin-app-store-plugin-install-rest --all-features
cargo build -p oya-plugin-app-store-plugin-install-rest --all-features
cargo clippy -p oya-plugin-app-store-plugin-install-rest --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-plugin-install-rest --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-plugin-install-rest --no-deps
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
| `test_rest_install_happy_path` | 201 + Installation body |
| `test_rest_install_capability_mismatch_rejected` | 400 with structured reason |
| `test_rest_install_unauthenticated` | 401 |
| `test_rest_install_tenant_mismatch_rejected` | 403 |
| `test_postgres_installations_table_isolation` | Tenant A row invisible to Tenant B |
| `test_sdk_install_retry_on_5xx` | Exponential backoff with jitter |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-006-plugin-install-rest-sdk-app/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Tenant isolation breach in Postgres queries.
- REST returns 5xx > 0.05% in steady state.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-006-plugin-install-rest-sdk-app`
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

[`IP-007-vetting-pipeline-kernel-domain`](IP-007-vetting-pipeline-kernel-domain.md)

## References

- PRD §plugin-install
- Axum docs
- sqlx docs
