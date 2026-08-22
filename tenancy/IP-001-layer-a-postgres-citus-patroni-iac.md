---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-001-layer-a-postgres-citus-patroni-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + axis-tenancy
acceptance_lanes: [cargo-check, helm-lint, kubectl-apply-dry-run, governance-per-microservice-layout, governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A Postgres + Citus + Patroni IaC

## Intent

Author Helm + Kustomize manifests for Postgres 16 + Citus 12.x + Patroni HA + Valkey under `microservices/tenancy/iac/helm/`. Deploys to the per-pack tenancy Kubernetes namespace per `multi-region.md`. Versions pinned to LTS per `docs/standards/observability-slo.md` §"Version Pinning". Pack-kr overlay activated at M01 launch.

## ChangeSet boundary

3 Helm chart bundles (postgres, citus, patroni) + 1 Kustomize base + pack-kr overlay + terraform postgres-rbac.tf. No code; pure IaC + values. Per-pack secret references via OpenBao. Valkey deployed via cloud-k8s shared chart (referenced, not bundled here).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tenancy/iac/helm/postgres/Chart.yaml` | create | upstream dep on bitnami/postgresql or zalando/postgres-operator at pinned LTS |
| `microservices/tenancy/iac/helm/postgres/values.yaml` | create | Postgres 16; replication setup; OpenBao SecretRef for passwords |
| `microservices/tenancy/iac/helm/citus/Chart.yaml` | create | Citus 12.x; multi-tenant sharding extension |
| `microservices/tenancy/iac/helm/citus/values.yaml` | create | Coordinator + worker count per capacity-model.md XS tier; shard placement strategy |
| `microservices/tenancy/iac/helm/patroni/Chart.yaml` | create | Patroni HA manager |
| `microservices/tenancy/iac/helm/patroni/values.yaml` | create | DCS=etcd; cluster topology 1 primary + 2 sync replicas |
| `microservices/tenancy/iac/kustomize/base/kustomization.yaml` | create | base referencing all 3 charts + Valkey |
| `microservices/tenancy/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (initial active) |
| `microservices/tenancy/iac/terraform/postgres-rbac.tf` | create | Terraform-managed Postgres roles (tenancy_app, tenancy-admin-jit, auditor-jit) |

## Crate Naming

n/a — IaC only.

## Code Shape

`postgres/values.yaml` (excerpt):

```yaml
postgresql:
  image:
    tag: "16.3"  # LTS pin
  primary:
    replicaCount: 1
    persistence:
      size: 100Gi
  readReplicas:
    replicaCount: 2
    persistence:
      size: 100Gi
  auth:
    existingSecret: tenancy-postgres-credentials  # OpenBao-managed
  postgresqlExtendedConf:
    - shared_preload_libraries: 'citus'
    - row_security: on
    - force_row_level_security: on  # per policy/rls-isolation.md Invariant RLS-01
```

`patroni/values.yaml` (excerpt):

```yaml
patroni:
  scope: tenancy-cluster
  namespace: tenancy
  dcs:
    type: etcd
    etcd:
      hosts:
        - etcd-0.etcd.tenancy.svc:2379
        - etcd-1.etcd.tenancy.svc:2379
        - etcd-2.etcd.tenancy.svc:2379
  bootstrap:
    dcs:
      synchronous_mode: true
      synchronous_node_count: 2  # quorum 2-of-(1 primary + 2 sync replicas)
```

`postgres-rbac.tf` (excerpt):

```hcl
resource "postgresql_role" "tenancy_app" {
  name        = "tenancy_app"
  login       = true
  password    = var.tenancy_app_password  # from OpenBao
  bypass_row_level_security = false  # ENFORCED per policy/rls-isolation.md Invariant RLS-04
}

resource "postgresql_role" "tenancy_admin_jit" {
  name                       = "tenancy_admin_jit"
  login                      = true
  bypass_row_level_security  = true  # only this JIT role can; OpenBao 2-person rule
}
```

## Acceptance Gates

```bash
helm lint microservices/tenancy/iac/helm/postgres
helm lint microservices/tenancy/iac/helm/citus
helm lint microservices/tenancy/iac/helm/patroni
kubectl --dry-run=client apply -k microservices/tenancy/iac/kustomize/overlays/pack-kr
tofu validate microservices/tenancy/iac/terraform/
cargo run -p dev-cli -- gate validate per-microservice-layout --microservice tenancy
cargo run -p dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- Per PHASE-01 IaC class: ≥ 1 helm-install + helm-test smoke per chart; 1 against kind/k3d cluster.
- Test files: `microservices/tenancy/tests/iac/{postgres,citus,patroni}.bats` running `helm install --dry-run` + `helm test`.
- E2E: spin up kind cluster; apply pack-kr overlay; verify Postgres + Citus + Patroni pods reach `Ready` within 10 min.
- Synthetic primary-failover drill in kind: kill primary; verify Patroni elects new primary ≤ 10s.

## Halt Conditions

- Postgres / Citus / Patroni version drift from LTS pin — escalate to `docs/standards/observability-slo.md` PR.
- OpenBao secret resolution failure — block; engage cloud-secrets.
- kind smoke fails — root-cause.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/tenancy/IP-001-layer-a-postgres-citus-patroni-iac.md` matched `multi-region`; anchors `microservices/tenancy/runbooks/dr-pair-promotion-drill.md, crates/tenancy-api/src/lib.rs`; type anchor `crates/tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Next IP

[`IP-002-tenant-lifecycle-kernel.md`](IP-002-tenant-lifecycle-kernel.md)

## References

- Bominal ADR-0018 + `multi-region.md`.
- `capacity-model.md` XS tier.
- Postgres docs — `postgresql.org/docs/16/`.
- Citus docs — `docs.citusdata.com`.
- Patroni docs — `patroni.readthedocs.io`.
