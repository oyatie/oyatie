# IP-GITOPS-002 — Cloud-IaC reconciler composition root

> ADR anchor: ADR-0202, ADR-0171.
> Owner: `axis-cloud-iac`.
> Scope: `microservices/cloud-iac` only.

## Goal

Use the existing cloud-iac Helm and Kustomize roots as the declarative
composition surface for the GitOps reconciler stack. This IP does not invent
new ArgoCD root or per-microservice application paths that are not present in
this service.

## Real service paths

| Path | Role |
|---|---|
| `microservices/cloud-iac/iac/helm/argocd/Chart.yaml` | ArgoCD chart wrapper |
| `microservices/cloud-iac/iac/helm/argocd/values.yaml` | ArgoCD HA and policy values |
| `microservices/cloud-iac/iac/helm/helm-controller/Chart.yaml` | Flux Helm-controller chart wrapper |
| `microservices/cloud-iac/iac/helm/helm-controller/values.yaml` | Helm-controller deployment values |
| `microservices/cloud-iac/iac/helm/kustomize-controller/Chart.yaml` | Flux Kustomize-controller chart wrapper |
| `microservices/cloud-iac/iac/helm/kustomize-controller/values.yaml` | Kustomize-controller deployment values |
| `microservices/cloud-iac/iac/kustomize/base/kustomization.yaml` | shared reconciler base |
| `microservices/cloud-iac/iac/kustomize/overlays/pack-kr/kustomization.yaml` | active pack overlay |

## Implementation contract

1. The pack overlay is the first composition root. It may reference the
   cloud-iac Helm chart wrappers already present in `iac/helm/`.
2. New ArgoCD `Application` or `ApplicationSet` manifests must land under an
   existing cloud-iac IaC root, or this IP must first add that root explicitly.
3. The composition root must keep render/apply ownership inside cloud-iac:
   `iac-renderer-worker` renders desired state, `iac-applier-worker` mutates
   declared apply scope, and policy remains enforced by `policy/ci-scope.cedar`.
4. The overlay must not reach into another microservice's IaC tree.

## Counterpart refs

- `microservices/cloud-iac/cross-microservice-handoffs.md` inbound rows from
  `cell`, `payments`, and `developer-sdk` define callers that ask cloud-iac to
  render or validate deploy artifacts.
- `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml` publishes
  `RenderCompleted`, `ApplyStarted`, and `ApplyCompleted` events that counterpart
  services consume after reconciliation.

## Acceptance criteria

- `kubectl --dry-run=client apply -k microservices/cloud-iac/iac/kustomize/overlays/pack-kr`
  renders only cloud-iac-owned resources.
- `helm lint` passes for the ArgoCD, Helm-controller, and Kustomize-controller
  chart wrappers.
- No nonexistent ArgoCD root path reference remains in this IP.

## Validation commands

```bash
helm lint microservices/cloud-iac/iac/helm/argocd
helm lint microservices/cloud-iac/iac/helm/helm-controller
helm lint microservices/cloud-iac/iac/helm/kustomize-controller
kubectl --dry-run=client apply -k microservices/cloud-iac/iac/kustomize/overlays/pack-kr
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-GITOPS-002-argocd-app-of-apps-pattern.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `86400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-GITOPS-002-argocd-app-of-apps-pattern.md`.
