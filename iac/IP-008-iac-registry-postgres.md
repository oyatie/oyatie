---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-008-iac-registry-postgres
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: oya-cloud-iac-iac-registry-{kernel,domain,usecase,api,adapter,adapter-postgres}

## Intent

Scaffold the iac-registry BC: kernel (port traits + entities) + domain (catalog versioning + provenance validation) + usecase (catalog orchestrator) + api + adapter + adapter-postgres (per-pack Postgres iac-state-index store). Registry is the catalog of charts + modules + overlays plus the per-pack apply-state index.

## ChangeSet boundary

Six new crates per ADR-0105. Plus the Postgres schema migration files at `iac/iac/postgres/migrations/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/src/crates/oya-cloud-iac-iac-registry-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-registry-domain/{Cargo.toml,src/lib.rs,src/catalog_version.rs,src/provenance_validate.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-registry-usecase/{Cargo.toml,src/lib.rs,src/catalog_orchestrator.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-registry-api/{Cargo.toml,src/lib.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-registry-adapter/{Cargo.toml,src/lib.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-registry-adapter-postgres/{Cargo.toml,src/lib.rs,src/state_index_store.rs,src/chart_catalog_store.rs}` | create |
| `iac/iac/postgres/migrations/{0001_initial.sql,0002_provenance.sql,0003_append_only_trigger.sql}` | create |
| `iac/catalog/oya-cloud-iac-iac-registry-*.yaml` | create (6 rows) |

## Code Shape

```sql
-- migrations/0001_initial.sql
CREATE TABLE apply_state_index (
  microservice         text not null,
  pack                 text not null check (pack ~ '^pack-[a-z-]+$'),
  jurisdiction         text not null check (jurisdiction in ('kr','eu','us','us-hc','jp','sg','au','in','br','ae','ksa')),
  environment          text not null check (environment in ('dev','staging','production')),
  current_sha          text not null check (current_sha ~ '^[a-f0-9]{40}$'),
  prior_sha            text,
  applied_at           timestamptz not null,
  applied_by           text not null,
  signature            bytea not null,
  data_class           text not null default 'AUDIT',
  pack_pinned          boolean not null default true,
  slsa_attestation_digest text,
  PRIMARY KEY (microservice, pack, environment, applied_at)
) PARTITION BY RANGE (applied_at);

CREATE TABLE chart_record (
  microservice  text not null,
  chart_name    text not null,
  version       text not null,
  digest        text not null,
  signed_by     text,
  registered_at timestamptz not null default now(),
  PRIMARY KEY (microservice, chart_name, version)
);

CREATE INDEX ix_apply_state_microservice_env ON apply_state_index (microservice, environment, applied_at DESC);
```

```sql
-- migrations/0003_append_only_trigger.sql
CREATE OR REPLACE FUNCTION refuse_mutation() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'apply_state_index is append-only; updates and deletes are refused';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER apply_state_index_append_only
BEFORE UPDATE OR DELETE ON apply_state_index
FOR EACH ROW EXECUTE FUNCTION refuse_mutation();
```

```rust
// adapter-postgres/src/state_index_store.rs
pub struct PostgresApplyStateIndexStore { pool: PgPool }

#[async_trait]
impl ApplyStateIndexStore for PostgresApplyStateIndexStore {
    async fn append(&self, record: &ApplyStateRecord) -> Result<(), RepositoryError> {
        sqlx::query!("INSERT INTO apply_state_index (...) VALUES (...)", ...).execute(&self.pool).await?;
        Ok(())
    }
    async fn latest(&self, ms: &str, env: Environment) -> Result<Option<ApplyStateRecord>, RepositoryError> { ... }
}
```

## Acceptance Gates

```bash
cargo check --workspace -p oya-cloud-iac-iac-registry-* --all-features
cargo nextest run --workspace -p oya-cloud-iac-iac-registry-* --all-features
psql -f iac/iac/postgres/migrations/0001_initial.sql  # against test Postgres
cloud-ci/oya-ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Test Plan

| Test | Layer | Verifies |
|---|---|---|
| `test_catalog_version_monotonic` | domain | version sequence enforced |
| `test_provenance_validate_chain` | domain | SLSA L3 attestation chain verified |
| `test_state_index_append_only` | adapter-postgres | UPDATE/DELETE refused at DB level |
| `integration_state_index_round_trip` | adapter-postgres | insert + read via real Postgres |
| `test_partition_by_range_works` | adapter-postgres | rows route to correct partition |

## Halt Conditions

- Append-only constraint not enforced at DB level — refuse.
- Cross-pack apply-state-index unification — refuse (per residency contract).

## Next IP

[`IP-009-iac-rollback-engine.md`](IP-009-iac-rollback-engine.md)

## References

- ADR-0105; ADR-0117.
- PRD §"Bounded Contexts" iac-registry BC.
- `iac/policy/data-residency.md` §"iac-state-index Jurisdiction Labels".
