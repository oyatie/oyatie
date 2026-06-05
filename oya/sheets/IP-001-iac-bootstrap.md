---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-sheets + cloud-iac
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A IaC — CDN + WAF + Postgres + Valkey + S3 + OCI Object Storage (Arrow/Parquet) + gVisor + AV-scan sidecars

## Intent

Author Helm + Kustomize manifests for the sheets Layer-A substrate: OCI CDN (per-pack edge), OCI WAF, Postgres + Citus (workbook + cell + sharing-ACL + license + comments + version-pointers), Valkey (ephemeral CRDT + recalc-progress + WS lease), S3 (workbook snapshots + version-history + XLSX quarantine + export jobs), OCI Object Storage (Arrow/Parquet large-sheet blocks per ADR-SHEETS-0003), gVisor RuntimeClass (XLSX import/export sandboxing), ClamAV + OPSWAT MetaDefender sidecars (AV-scan), WebSocket gateway scaffold, under `microservices/sheets/iac/`. Versions pinned to LTS per `docs/standards/observability-slo.md` § "Layer-A components".

## ChangeSet boundary

One cohesive ChangeSet: 11 Helm chart bundles (sheets-postgres, sheets-valkey, cell-grid-rest, collab-crdt-worker, recalc-engine-worker, xlsx-export-worker, license-gate-cedar, sheets-cdn, sheets-waf, clamav-sidecar, opswat-sidecar) + 1 shared Kustomize base + per-pack overlays (pack-kr + pack-eu at M03 launch; 9 additional overlays scaffolded). No Rust code; pure IaC + values. Per-pack secret references via OpenBao SecretReference.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/sheets/iac/helm/sheets-postgres/{Chart.yaml,values.yaml}` | create |
| `microservices/sheets/iac/helm/sheets-valkey/{Chart.yaml,values.yaml}` | create |
| `microservices/sheets/iac/helm/visual-grid-rest/{Chart.yaml,values.yaml,templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml}` | create |
| `microservices/sheets/iac/helm/collab-crdt-worker/{Chart.yaml,values.yaml,templates/...}` | create |
| `microservices/sheets/iac/helm/recalc-engine-worker/{Chart.yaml,values.yaml,templates/...}` | create |
| `microservices/sheets/iac/helm/xlsx-export-worker/{Chart.yaml,values.yaml,templates/...}` | create (gVisor runtime class) |
| `microservices/sheets/iac/helm/license-gate-cedar/{Chart.yaml,values.yaml,templates/...}` | create |
| `microservices/sheets/iac/helm/sheets-cdn/{Chart.yaml,values.yaml}` | create |
| `microservices/sheets/iac/helm/sheets-waf/{Chart.yaml,values.yaml}` | create |
| `microservices/sheets/iac/helm/clamav-sidecar/{Chart.yaml,values.yaml}` | create |
| `microservices/sheets/iac/helm/opswat-sidecar/{Chart.yaml,values.yaml}` | create |
| `microservices/sheets/iac/kustomize/base/{kustomization,namespace,openbao-secret-references,service-mesh-tenant-headers,cdn-edge-config,gvisor-runtime-class}.yaml` | create |
| `microservices/sheets/iac/kustomize/overlays/pack-{kr,eu,us,us-hc,jp,sg,au,in,br,ae,ksa}/kustomization.yaml` | create (initial active: pack-kr + pack-eu) |

## Code Shape

LTS pins (`sheets-postgres/values.yaml`):

```yaml
citus:
  image:
    tag: "12.1.0"   # LTS pin (per ADR-SHEETS-0003 substrate; Postgres 16 LTS upstream)
  primary:
    replicas: 1
    resources:
      requests: {cpu: 2, memory: 8Gi}
      limits: {cpu: 4, memory: 16Gi}
  worker:
    replicas: 3
    resources:
      requests: {cpu: 4, memory: 16Gi}
      limits: {cpu: 8, memory: 32Gi}
  rls:
    enabled: true
  shardKey: tenant_id
  config:
    citus.shard_count: 32
    citus.replication_factor: 2
```

XLSX-export-worker template (`xlsx-export-worker/templates/deployment.yaml` excerpt):

```yaml
spec:
  template:
    spec:
      runtimeClassName: gvisor-sheets-import-export  # per ADR-SHEETS-0007
      containers:
        - name: xlsx-export-worker
          env:
            - name: GVISOR_BUDGET_RAM_MB
              value: "4096"
            - name: GVISOR_BUDGET_CPU_CORES
              value: "4"
            - name: GVISOR_BUDGET_WALL_CLOCK_SECONDS
              value: "300"
            - name: AV_SCAN_REQUIRED
              value: "true"
            - name: CALAMINE_VERSION
              value: "0.26"
            - name: RUST_XLSXWRITER_VERSION
              value: "0.79"
```

## Acceptance Gates

```bash
helm lint microservices/sheets/iac/helm/sheets-postgres
helm lint microservices/sheets/iac/helm/sheets-valkey
helm lint microservices/sheets/iac/helm/visual-grid-rest
helm lint microservices/sheets/iac/helm/collab-crdt-worker
helm lint microservices/sheets/iac/helm/recalc-engine-worker
helm lint microservices/sheets/iac/helm/xlsx-export-worker
helm lint microservices/sheets/iac/helm/license-gate-cedar
helm lint microservices/sheets/iac/helm/sheets-cdn
helm lint microservices/sheets/iac/helm/sheets-waf
helm lint microservices/sheets/iac/helm/clamav-sidecar
helm lint microservices/sheets/iac/helm/opswat-sidecar
kubectl --dry-run=client apply -k microservices/sheets/iac/kustomize/overlays/pack-kr
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice sheets
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
```

## Test Plan

- ≥ 1 helm-install + helm-test smoke per chart.
- E2E: spin up kind cluster; apply pack-kr overlay; verify all 11 component pods reach `Ready` within 10 min.
- Verify gVisor runtime class instantiated; sample workload runs in sandbox.
- Verify ClamAV + OPSWAT sidecars functional; sample EICAR test file refused.

## Halt Conditions

- Chart upstream-version drift from LTS pin — escalate.
- OpenBao secret-reference resolution failure — block.
- Citus RLS migration fails — block.
- gVisor runtime class fails to instantiate — block.
- kind cluster smoke fails — root-cause; do not mask.

## Next IP

[`IP-002-cargo-workspace-cell-grid-kernel-domain.md`](IP-002-cargo-workspace-cell-grid-kernel-domain.md)

## References

- ADR-0131 §"Per-microservice flat layout".
- ADR-SHEETS-0003 (large-sheet storage substrate).
- ADR-SHEETS-0007 (XLSX export fidelity + gVisor sandboxing).
- `microservices/sheets/multi-region.md`.
- `microservices/sheets/capacity-model.md`.
- `microservices/sheets/threat-model.md` §"Trust Boundaries" + §"T-S-04".
- Citus docs — `docs.citusdata.com`.
- Valkey Sentinel docs — `valkey.io/topics/sentinel/`.
- Apache Arrow 18.x — `arrow.apache.org/docs/`.
- Apache Parquet 18.x — `parquet.apache.org/`.
- OCI CDN docs — `docs.oracle.com/iaas/Content/CDN/`.
- OCI WAF docs — `docs.oracle.com/iaas/Content/WAF/`.
- gVisor — `gvisor.dev`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
