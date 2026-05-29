---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + cloud-iac
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A IaC — CDN + WAF + Postgres (Citus) + Valkey + WebSocket gateway

## Intent

Author Helm + Kustomize manifests for the workflow-studio Layer-A substrate: OCI CDN (per-pack edge), OCI WAF, Postgres + Citus (editor session + per-seat license attribution + draft persistence), Valkey (ephemeral CRDT + WS lease coordination), WebSocket gateway (axum-WS-based, fronted by Istio), under `microservices/workflow-studio/iac/`. Deploys to the workflow-studio namespace on each pack's regional cluster per `multi-region.md`. Versions pinned to LTS per `docs/standards/observability-slo.md` § "Layer-A components".

## ChangeSet boundary

One cohesive ChangeSet: 8 Helm chart bundles (CDN/WAF integration, Postgres + Citus, Valkey HA, WS gateway, visual-canvas REST, node-library-registry REST, license-gate-cedar, studio composition-root deployment) + 1 shared Kustomize base + per-pack overlays (pack-kr at M03 launch; 10 additional overlays scaffolded). No Rust code; pure IaC + values. Per-pack secret references via OpenBao SecretReference (no raw secrets in repo).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/workflow-studio/iac/helm/studio-postgres/{Chart.yaml,values.yaml,values-pack-kr.yaml}` | create | Citus 12.x distributed Postgres; tenant_id shard key; RLS enabled |
| `microservices/workflow-studio/iac/helm/studio-valkey/{Chart.yaml,values.yaml}` | create | Valkey 8.1 (RESP wire-compatible); Sentinel HA; per-cell cluster |
| `microservices/workflow-studio/iac/helm/visual-canvas-rest/{Chart.yaml,values.yaml,templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml}` | create | Editor REST Deployment + Service + HPA + PDB + NetworkPolicy + Prometheus monitoring |
| `microservices/workflow-studio/iac/helm/collab-crdt-worker/{Chart.yaml,values.yaml,templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml}` | create | WebSocket gateway Deployment + HPA + PDB |
| `microservices/workflow-studio/iac/helm/node-library-registry-rest/{Chart.yaml,values.yaml,templates/...}` | create | Node library REST + CDN-fronted distribution |
| `microservices/workflow-studio/iac/helm/license-gate-cedar/{Chart.yaml,values.yaml}` | create | Cedar evaluator sidecar; default-deny fail-closed |
| `microservices/workflow-studio/iac/helm/studio-cdn/{Chart.yaml,values.yaml}` | create | OCI CDN edge config; per-tenant cache key |
| `microservices/workflow-studio/iac/helm/studio-waf/{Chart.yaml,values.yaml}` | create | OCI WAF ruleset; CSP enforcement; rate limits |
| `microservices/workflow-studio/iac/kustomize/base/kustomization.yaml` | create | Shared base referencing all 8 charts |
| `microservices/workflow-studio/iac/kustomize/overlays/pack-{kr,eu,us,us-hc,jp,sg,au,in,br,ae,ksa}/kustomization.yaml` | create | Per-pack overlay (initial active pack: pack-kr) |
| `microservices/workflow-studio/iac/terraform/cdn-edge-config.tf` | create | OCI CDN per-pack edge configuration |
| `microservices/workflow-studio/iac/terraform/node-library-publishers.tf` | create | Allowed publisher set per pack (per threat-model.md T-S-04) |

## Code Shape

Helm chart skeleton (`studio-postgres/values-pack-kr.yaml`):

```yaml
citus:
  image:
    tag: "12.1.0"   # LTS pin
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
  initdb:
    secret:
      name: studio-postgres-init
      key: init.sql
    sqlScripts:
      enable-pgaudit.sql: |
        CREATE EXTENSION IF NOT EXISTS pgaudit;
        ALTER SYSTEM SET pgaudit.log = 'ROLE,DDL,WRITE';
  s3Backup:
    endpoint: "${OCI_OBJECTSTORAGE_ENDPOINT}"
    bucket: "${OCI_STUDIO_POSTGRES_BACKUP_BUCKET}"
    accessKeyId: "${openbao:secret/workflow-studio/postgres-backup/access-key}"
    secretAccessKey: "${openbao:secret/workflow-studio/postgres-backup/secret-key}"
```

WebSocket gateway template (`collab-crdt-worker/templates/deployment.yaml` excerpt):

```yaml
spec:
  replicas: 3
  template:
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            - topologyKey: topology.kubernetes.io/zone
      containers:
        - name: collab-crdt-worker
          image: registry.oyatie.dev/workflow-studio/collab-crdt-worker:{{ .Values.image.tag }}
          ports:
            - containerPort: 8080
              name: ws
            - containerPort: 9090
              name: metrics
          env:
            - name: VALKEY_LEASE_TTL_SECONDS
              value: "300"
            - name: WS_PER_TENANT_RATE_LIMIT
              value: "100"
            - name: OPENBAO_TOKEN_PATH
              value: "secret/workflow-studio/collab-crdt-worker/spiffe"
          resources:
            requests: {cpu: 1, memory: 2Gi}
            limits: {cpu: 4, memory: 8Gi}
          livenessProbe:
            httpGet: {path: /health, port: ws}
          readinessProbe:
            httpGet: {path: /ready, port: ws}
```

## Acceptance Gates

```bash
helm lint microservices/workflow-studio/iac/helm/studio-postgres
helm lint microservices/workflow-studio/iac/helm/studio-valkey
helm lint microservices/workflow-studio/iac/helm/visual-canvas-rest
helm lint microservices/workflow-studio/iac/helm/collab-crdt-worker
helm lint microservices/workflow-studio/iac/helm/node-library-registry-rest
helm lint microservices/workflow-studio/iac/helm/license-gate-cedar
helm lint microservices/workflow-studio/iac/helm/studio-cdn
helm lint microservices/workflow-studio/iac/helm/studio-waf
kubectl --dry-run=client apply -k microservices/workflow-studio/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice workflow-studio
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- Per Phase-01 §"Per-IP Test Coverage Threshold" IaC class: ≥ 1 helm-install + helm-test smoke per chart.
- `tests/iac/{studio-postgres,studio-valkey,visual-canvas-rest,collab-crdt-worker,node-library-registry-rest,license-gate-cedar,studio-cdn,studio-waf}.bats` running `helm install --dry-run` + `helm test`.
- E2E: spin up kind cluster; apply pack-kr overlay; verify all 8 component pods reach `Ready` within 10 min.

## Halt Conditions

- Any chart upstream-version drift from the LTS pin — escalate to docs/standards/observability-slo.md PR.
- OpenBao secret-reference resolution failure — block; engage cloud-secrets µservice.
- Citus RLS migration fails — block; engage cloud-postgres team.
- kind cluster smoke fails — root-cause; do not mask.

## Next IP

[`IP-002-visual-canvas-kernel-domain.md`](IP-002-visual-canvas-kernel-domain.md)

## References

- ADR-0131 §"Per-microservice flat layout".
- `microservices/workflow-studio/multi-region.md`.
- `microservices/workflow-studio/capacity-model.md`.
- `microservices/workflow-studio/threat-model.md` §"Trust Boundaries" + §"T-D-01".
- `docs/standards/observability-slo.md` §"Layer-A components".
- Citus docs — `docs.citusdata.com`.
- Valkey Sentinel docs — `valkey.io/docs/management/sentinel/`.
- OCI CDN docs — `docs.oracle.com/iaas/Content/CDN/`.
- OCI WAF docs — `docs.oracle.com/iaas/Content/WAF/`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md` matched [`multi-region`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/capacity-model.md`, `microservices/workflow-studio/compliance.md`, `microservices/workflow-studio/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
