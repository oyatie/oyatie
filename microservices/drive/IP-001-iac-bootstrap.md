---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-drive + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

# IP-001: IaC bootstrap — Helm + Kustomize for Garage + MinIO + SeaweedFS + Postgres + Redis + Meilisearch + Tika + ClamAV + OPSWAT + LibreOffice-gVisor

## Intent

Author Helm + Kustomize manifests for the drive µservice substrate. Three object-store backends per ADR-DRIVE-0001 (Garage primary; MinIO secondary; SeaweedFS archive); Postgres 16 LTS for metadata; Redis 7.4 LTS for upload-session + delta cache; Meilisearch + Tika for full-text; ClamAV + OPSWAT for scan; LibreOffice 24 LTS in gVisor for preview per ADR-DRIVE-0005; OpenBao Transit for tenant-DEK envelope per ADR-DRIVE-0004. Pack-aware overlays for 11 packs.

## ChangeSet boundary

10 Helm template files + Kustomize base + 11 pack overlays (pack-kr + pack-eu first; remaining 9 follow). No Rust code; pure IaC + values. All secrets via `${openbao:secret/drive/...}` SecretReferences.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/drive/iac/helm/Chart.yaml` | created | dependencies: garage 1.0.1, minio RELEASE-2024-08-17, seaweedfs 3.71.0, postgres 16.4.0, redis 7.4.0, meilisearch 1.10.3, apache-tika 2.9.2, clamav 1.4.1, opswat 5.10.0, libreoffice-gvisor 24.8.4 |
| `microservices/drive/iac/helm/values.yaml` | created | per-BC replica sizing; object-store backend gating per ADR-DRIVE-0001; OpenBao SecretReferences |
| `microservices/drive/iac/helm/templates/deployment.yaml` | created | per-BC Deployment (11 BCs); gVisor RuntimeClass on preview workers |
| `microservices/drive/iac/helm/templates/service.yaml` | created | per-BC Service |
| `microservices/drive/iac/helm/templates/hpa.yaml` | created | per-BC HPA (CPU 70%; min 3, max 100) |
| `microservices/drive/iac/helm/templates/pdb.yaml` | created | PodDisruptionBudget min-available 50% |
| `microservices/drive/iac/helm/templates/networkpolicy.yaml` | created | mesh-only ingress; egress to OpenBao + Postgres + Redis + object-store + Meili + Tika + ClamAV + OPSWAT + cross-µservice mesh; preview workers DENY all egress except object-store + OpenBao |
| `microservices/drive/iac/helm/templates/servicemonitor.yaml` | created | Prometheus scrape config |
| `microservices/drive/iac/helm/templates/prometheusrule.yaml` | created | per-BC fast-burn + slow-burn + zero-tolerance alert rules |
| `microservices/drive/iac/kustomize/base/kustomization.yaml` | created | shared base |
| `microservices/drive/iac/kustomize/base/namespace.yaml` | created | drive namespace + PSS restricted |
| `microservices/drive/iac/kustomize/base/serviceaccount.yaml` | created | per-BC SA + OpenBao role binding |
| `microservices/drive/iac/kustomize/overlays/pack-kr/kustomization.yaml` | created | initial active pack |
| `microservices/drive/iac/kustomize/overlays/pack-eu/kustomization.yaml` | created | EU pack (OPSWAT enabled; T2 HR-overlay refused) |
| `microservices/drive/iac/kustomize/overlays/pack-us/...` | successor-IP | US pack (SEC 17a-4) |
| `microservices/drive/iac/kustomize/overlays/pack-us-healthcare/...` | successor-IP | HIPAA pack (MinIO + OPSWAT enabled) |
| `microservices/drive/iac/kustomize/overlays/pack-jp/...` | successor-IP | JP pack |
| (pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa) | successor-IP | per-pack overlays |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/drive/iac/helm
kubectl --dry-run=client apply -k microservices/drive/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice drive
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm lint + helm-test per chart against kind/k3d cluster.
- E2E smoke: spin kind cluster; apply pack-kr overlay; verify all 11 BC deployments + Garage + Postgres + Redis + Meilisearch + Tika + ClamAV + LibreOffice-gVisor reach Ready within 15 min.
- S3 SigV4 smoke: `aws s3 cp` against the Garage Service; expect 200.
- gVisor smoke: verify preview-worker pods have `runtimeClassName: gvisor`.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate per `docs/standards/observability-slo.md`.
- OpenBao secret-reference resolution fails — block.
- Helm chart fails kubectl-dry-run — root-cause; do not mask.

## Next IP

[`IP-002-file-store-kernel.md`](IP-002-file-store-kernel.md)

## References

- ADR-0117 (data residency); ADR-0131 (per-µservice flat layout); ADR-0133.
- ADR-DRIVE-0001 (object-storage substrate); ADR-DRIVE-0004 (encryption KMS); ADR-DRIVE-0005 (preview sandbox).
- Garage — `garagehq.deuxfleurs.fr`; MinIO — `min.io`; SeaweedFS — `github.com/seaweedfs/seaweedfs`.
- Postgres CloudNativePG operator — `cloudnative-pg.io`.
- gVisor — `gvisor.dev`.
