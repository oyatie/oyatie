---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-001-layer-a-cdn-postgres-redis-s3-ws-gateway-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + cloud-iac
acceptance_lanes: [cargo-check, iac-validate, per-microservice-layout]
depends_on: []
---

# IP-001: Layer-A IaC — CDN, WAF, Postgres, Redis, S3, WS gateway, gVisor export pool

## Intent

Author the Layer-A infrastructure for slides: CDN edge cache + WAF + Postgres (Citus) deck metadata + Redis (per-cell sentinel) for CRDT cache + S3 for deck snapshots/assets + WebSocket gateway service + gVisor export-worker pool. Per ADR-0131 + multi-region.md per-pack topology.

## ChangeSet boundary

Files under `microservices/slides/iac/`:
- `helm/Chart.yaml` + `values.yaml` + 7 templates per BC family
- `kustomize/base/` + `overlays/pack-kr/` + `overlays/pack-eu/`

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/Chart.yaml` | create |
| `iac/helm/values.yaml` | create |
| `iac/helm/templates/deployment.yaml` | create |
| `iac/helm/templates/service.yaml` | create |
| `iac/helm/templates/hpa.yaml` | create |
| `iac/helm/templates/pdb.yaml` | create |
| `iac/helm/templates/networkpolicy.yaml` | create |
| `iac/helm/templates/servicemonitor.yaml` | create |
| `iac/helm/templates/prometheusrule.yaml` | create |
| `iac/kustomize/base/kustomization.yaml` | create |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | create |
| `iac/kustomize/overlays/pack-eu/kustomization.yaml` | create |

## Code Shape

`iac/helm/values.yaml`:

```yaml
image:
  registry: registry.oyatie.dev
  repository: slides/slides-runtime
  tag: "1.0.0"

replicaCount: 4

# Per-tenant editor session cap; per threat-model T-D-01
perTenantSessionCap: 50

# OIDC + Cedar
oidc:
  issuer: https://auth.oyatie.dev
  audience: slides
cedar:
  evaluatorEndpoint: "http://acl.slides.svc.cluster.local:8080"
  failClosed: true  # T-D-07

# HPA
hpa:
  minReplicas: 4
  maxReplicas: 50
  targetCPUUtilizationPercentage: 70

# gVisor export-pool
exportPool:
  replicaCount: 4
  minReplicas: 4
  maxReplicas: 100
  perJobMemoryGiB:
    import: 2
    mp4: 4
    pdf: 2
    pptx: 2

# Secret references (OpenBao)
secrets:
  spiffeIdentityPath: "${openbao:secret/slides/spiffe}"
  postgresConnectionPath: "${openbao:secret/slides/postgres}"
  redisAuthPath: "${openbao:secret/slides/redis}"
  s3KmsKeyPath: "${openbao:secret/slides/s3-kms-per-pack}"
  ed25519SigningKeyPath: "${openbao:secret/slides/audit-chain-signing}"
  collabHmacKeyPath: "${openbao:secret/slides/collab-hmac}"
```

## Acceptance Gates

```bash
helm lint iac/helm/
kubectl apply --dry-run=client -k iac/kustomize/overlays/pack-kr/
kubectl apply --dry-run=client -k iac/kustomize/overlays/pack-eu/
oya gate validate per-microservice-layout --microservice slides
```

## Test Plan

| Test | Verifies |
|---|---|
| `helm lint` | values + templates parseable |
| `kubectl apply --dry-run` per pack | overlay valid |
| Network policy egress allowlist test | only cross-µservice SDK consumers reachable |
| OpenBao reference test | secrets resolve, no plaintext |

## Halt Conditions

- Helm chart lint fail — STOP.
- Per-pack overlay drift detector flags critical drift — STOP.
- OpenBao reference resolution fails — STOP.

## Next IP

IP-002.

## References

- ADR-0131 per-microservice flat layout.
- multi-region.md.
- workflow-studio IP-001 (parallel pattern).
