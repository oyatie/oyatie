---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-recordings + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap (Helm + Kustomize + OpenTofu)

## Intent

Author the recordings µservice's deployment substrate: Helm chart for the
core workloads (recording-rest, recording-ingest, transcript-worker,
diarization-worker, transcode-worker, search, retention-purge-worker,
legal-hold-engager, ediscovery-export-worker, share-link-rest, playback-rest,
ffmpeg-sandbox, watermark-stamper), upstream-dependency charts (Postgres
16, Valkey 8.1 (RESP3 wire-compatible), Meilisearch 0.10.0, Pandoc 3.x), Kustomize base + per-pack
overlays, Terraform-managed Grafana RBAC + CloudFront / self-host CDN per
pack.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (recordings) + 1 shared
Kustomize base + 4 per-pack Kustomize overlays + 1 OpenTofu module for
Grafana RBAC + CDN backend wiring. No code; pure IaC + values. Per-pack
secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/recordings/iac/helm/recordings/Chart.yaml` | create | upstream pins: postgres 16, valkey 8.1 (RESP3 wire-compatible), meilisearch 0.10.0, pandoc 3.x |
| `microservices/recordings/iac/helm/recordings/values.yaml` | create | per-BC replica sizing, OpenBao SecretReferences |
| `microservices/recordings/iac/helm/recordings/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | create | core Kubernetes resources |
| `microservices/recordings/iac/kustomize/base/kustomization.yaml` | create | shared base |
| `microservices/recordings/iac/kustomize/overlays/pack-{kr,eu,us-healthcare,us-financial}/kustomization.yaml` | create | per-pack overlays |
| `microservices/recordings/iac/terraform/grafana-rbac.tf` | create | Terraform-managed Grafana folder + roles |

## Acceptance Gates

```bash
helm lint microservices/recordings/iac/helm/recordings
kubectl --dry-run=client apply -k microservices/recordings/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/recordings/iac/kustomize/overlays/pack-us-healthcare
kubectl --dry-run=client apply -k microservices/recordings/iac/kustomize/overlays/pack-us-financial
terraform -chdir=microservices/recordings/iac/tofu validate
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice recordings
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
```

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0139; ADR-0131; ADR-0132.
- `multi-region.md`, `capacity-model.md`.
- `decisions/ADR-RECORDINGS-0004.md` (CDN backend).
- `decisions/ADR-RECORDINGS-0005.md` (storage tiering).
