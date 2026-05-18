---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-001-postgres-layer-a-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [helm-lint, helm-install-smoke, oya-check-postgres-rls-enforced, oya-check-secrets-via-openbao]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: HA Postgres Layer-A IaC

## Intent

Helm chart for HA PostgreSQL (Patroni-managed primary + replica per pack region); OpenBao-issued per-pod credentials; row-level security mandatory; WAL archive to encrypted S3.

## ChangeSet boundary

`microservices/foundry/iac/helm/postgres/{Chart.yaml, values.yaml}`. No Rust changes.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/foundry/iac/helm/postgres/Chart.yaml` | create |
| `microservices/foundry/iac/helm/postgres/values.yaml` | create |
| `microservices/foundry/iac/kustomize/base/kustomization.yaml` | update (helmCharts list) |

## Substrate selections

- Postgres 16 LTS (cite postgresql.org/docs/16/).
- Patroni for HA + Patroni controller for K8s.
- pgBouncer for connection pooling (2 replicas).
- WAL archive to S3-compatible bucket (per-pack region; SSE-KMS).

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/postgres
helm install --dry-run --debug -n foundry-supervisor postgres microservices/foundry/iac/helm/postgres
cargo run -p oya-dev-cli -- gate validate postgres-rls-enforced --microservice foundry-supervisor
cargo run -p oya-dev-cli -- gate validate secrets-via-openbao --microservice foundry-supervisor
```

## Test Plan

| Test | Verifies |
|---|---|
| Helm lint | chart syntactic |
| Helm install smoke (kind cluster) | chart deploys; primary + replica come up |
| Patroni status (`patronictl list`) | HA elected |
| RLS policy applied on every tenant-scoped table | LEAN lane |
| OpenBao SecretReference materialized | secret-scanner lane |

## Halt Conditions

- `multitenancy_enabled` analog (`row_security`) not enabled.
- Direct DB credentials in values.yaml (not via OpenBao reference).

## Next IP

[`IP-002-redis-layer-a-iac.md`](IP-002-redis-layer-a-iac.md)

## References

- PRD §"Performance" + §"Horizontal Scalability".
- `policy/supervisor-isolation.md` TI-P-*.
- PostgreSQL HA — `postgresql.org/docs/current/high-availability.html`.
- Patroni — `patroni.readthedocs.io`.
- `capacity-model.md` §"Postgres Sizing".
