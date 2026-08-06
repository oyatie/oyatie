---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-014-tests-load-drills-observability-slos
status: pending
owner: ops-sre-reliability + axis-tenancy
acceptance_lanes: [cargo-nextest, oya-governance-openslo-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Tests + load drills + OpenSLO manifests for tenancy

## Intent

Author k6 load tests; Patroni-failover availability drill; quarterly synthetic cross-tenant probe; tenancy OpenSLO manifests at `microservices/tenancy/slos/{availability,latency,correctness,freshness}.openslo.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/tenancy/tests/load/tenant-validate-100krps.js` | create — k6 100k RPS sustained |
| `microservices/tenancy/tests/load/patroni-failover-availability.sh` | create — induce Patroni primary loss; measure validate availability |
| `microservices/tenancy/tests/integration/synthetic_cross_tenant_probe.rs` | create — quarterly drill |
| `microservices/tenancy/tests/e2e/dsr_cascade_proof.rs` | create — full DSR drill across all M01 µservices |
| `microservices/tenancy/slos/availability.openslo.yaml` | create |
| `microservices/tenancy/slos/latency.openslo.yaml` | create |
| `microservices/tenancy/slos/correctness.openslo.yaml` | create — RLS-no-cross-tenant probe success rate ≥ 100% |
| `microservices/tenancy/slos/freshness.openslo.yaml` | create — RLS drift detection within 5min |
| `microservices/tenancy/slos/waivers.md` | create — empty register |

## Code Shape

`slos/availability.openslo.yaml`:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: tenancy-availability
  labels:
    microservice: tenancy
    sli: availability
    pack: pack-kr
spec:
  service: tenancy
  indicator:
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(http_requests_total{job="oya-tenancy-tenant-lifecycle-rest",status!~"5.."}[5m]))'
        total:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(http_requests_total{job="oya-tenancy-tenant-lifecycle-rest"}[5m]))'
  objectives:
    - displayName: "99.99% over rolling 30d (tightest in catalog)"
      target: 0.9999
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

`slos/correctness.openslo.yaml`:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: tenancy-correctness-rls-probe
spec:
  service: tenancy
  indicator:
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(oya_tenancy_synthetic_cross_tenant_probe_success_total[5m]))'
        total:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(oya_tenancy_synthetic_cross_tenant_probe_total[5m]))'
  objectives:
    - displayName: "100% — RLS MUST refuse cross-tenant; any failure = Sev-1"
      target: 1.0
  timeWindow: [{duration: 7d, isRolling: true}]
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
k6 run microservices/tenancy/tests/load/tenant-validate-100krps.js
bash microservices/tenancy/tests/load/patroni-failover-availability.sh
cargo nextest run -p oya-tenancy-tenant-lifecycle-adapter-postgres --test synthetic_cross_tenant_probe
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice tenancy
```

## Test Plan

- 100k RPS sustained 10min; p99 ≤ 5ms; error rate ≤ 0.01%.
- Patroni failover: 10s blip; overall availability ≥ 99.99% during 10min window with one induced primary loss.
- Cross-tenant probe: zero rows returned across all paths.
- DSR cascade: end-to-end proof-of-erasure across all M01 µservices.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/tenancy/IP-014-tests-load-drills-observability-slos.md` matched `p99, SLO`; anchors `microservices/tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Next IP

[`IP-015-legacy-crates-migration.md`](IP-015-legacy-crates-migration.md)
