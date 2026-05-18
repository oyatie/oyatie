---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-docs + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

# IP-001: IaC bootstrap — Helm + Kustomize for Postgres + S3 + Redis + ClamAV + OPSWAT + gVisor

## Intent

Author Helm + Kustomize manifests for the docs µservice substrate. Postgres 16 LTS for document metadata (per-tenant + per-block RLS per ADR-DOCS-0004); S3-compatible object storage for content blobs + attachments (per-tenant prefix; Object Lock for legal-hold); Redis 7.4 LTS cluster mode for collab presence + CRDT op spool + cache; ClamAV scanner (default); OPSWAT MetaDefender (pack-us-healthcare overlay); gVisor pool for export workers per ADR-DOCS-0003. Pack-aware overlays for 11 packs.

## ChangeSet boundary

10 Helm template files + Kustomize base + per-pack overlay (pack-kr + pack-eu first; us/jp/sg/au/in/br/ae/ksa/us-healthcare follow). No Rust code; pure IaC + values. All secrets via `${openbao:secret/docs/...}` SecretReferences.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/docs/iac/helm/Chart.yaml` | created in this ChangeSet | dependencies: postgres 16.4, redis 7.4, ClamAV 1.3, OPSWAT MetaDefender container 5.x, gVisor 2024-Q4 LTS |
| `microservices/docs/iac/helm/values.yaml` | created | per-BC replica sizing; gVisor pool size; OpenBao SecretReferences |
| `microservices/docs/iac/helm/templates/deployment.yaml` | created | per-BC Deployment (8 BCs) |
| `microservices/docs/iac/helm/templates/service.yaml` | created | per-BC Service |
| `microservices/docs/iac/helm/templates/hpa.yaml` | created | per-BC HPA (CPU 70%; min 5 max 100) |
| `microservices/docs/iac/helm/templates/pdb.yaml` | created | PodDisruptionBudget min-available 50% |
| `microservices/docs/iac/helm/templates/networkpolicy.yaml` | created | mesh-only ingress; egress to OpenBao + Postgres + Redis + S3 + ClamAV + cross-µservice mTLS for embed-resolver |
| `microservices/docs/iac/helm/templates/servicemonitor.yaml` | created | Prometheus scrape config |
| `microservices/docs/iac/helm/templates/prometheusrule.yaml` | created | per-BC fast-burn + slow-burn alert rules |
| `microservices/docs/iac/kustomize/base/kustomization.yaml` | created | shared base |
| `microservices/docs/iac/kustomize/overlays/pack-kr/kustomization.yaml` | created | initial active pack |
| `microservices/docs/iac/kustomize/overlays/pack-eu/kustomization.yaml` | created | EU pack (eIDAS + EU AI Act overlays) |
| (additional packs: us, us-healthcare, jp, sg, au, in, br, ae, ksa) | successor-IP | per-pack overlays |

## Acceptance Gates

```bash
helm lint microservices/docs/iac/helm
kubectl --dry-run=client apply -k microservices/docs/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice docs
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm lint + helm-test per chart against kind/k3d cluster.
- E2E smoke: spin kind cluster; apply pack-kr overlay; verify all 8 BC deployments + Postgres + S3 emulator + Redis + ClamAV reach Ready within 10 min.
- gVisor sandbox smoke: spawn an export job; verify tmpfs-only + no network egress.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate per `docs/standards/observability-slo.md`.
- OpenBao secret-reference resolution fails — block.
- Helm chart fails kubectl-dry-run — root-cause; do not mask.

## Next IP

[`IP-002-document-store-kernel.md`](IP-002-document-store-kernel.md)

## References

- ADR-0117 (data residency); ADR-0131 (per-µservice flat layout); ADR-0133.
- ADR-DOCS-0001 (Loro CRDT); ADR-DOCS-0003 (export pipeline backends).
- Postgres CloudNativePG operator — `cloudnative-pg.io`.
- Redis cluster mode — `redis.io/docs/management/scaling/`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
- gVisor — `gvisor.dev`.
