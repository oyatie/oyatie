# IP-GITOPS-008 — ArgoCD AppProject bootstrap boundary

> ADR anchor: ADR-0202.
> Owner: `axis-cloud-iac`.
> Scope: `iac` only.

## Goal

Keep ArgoCD `AppProject` bootstrap in the existing
`k8s-namespace-bootstrap` module contract and keep application reconciliation in
the cloud-iac Kustomize/Helm roots. This prevents a bootstrap loop where an
application needs the project that would authorize itself.

## Real service paths

| Path | Boundary |
|---|---|
| `iac/tofu/modules/k8s-namespace-bootstrap/main.tofu` | declares `argocd_project_name` input and output |
| `iac/tofu/modules/k8s-namespace-bootstrap/README.md` | operator explanation for namespace/AppProject bootstrap |
| `iac/iac/helm/argocd/Chart.yaml` | ArgoCD control-plane chart wrapper |
| `iac/iac/helm/argocd/values.yaml` | ArgoCD control-plane values |
| `iac/iac/kustomize/overlays/pack-kr/kustomization.yaml` | first overlay that depends on bootstrap |
| `iac/policy/ci-scope.cedar` | enforcement that only applier/rollback identities mutate declared scope |

## Implementation contract

1. `AppProject` naming is an output of `k8s-namespace-bootstrap`, not a value
   scraped from an ArgoCD runtime object.
2. ArgoCD chart changes are validated independently from module changes.
3. Pack overlays may reference the project name only after bootstrap output is
   available.
4. Any future discipline checker must inspect real cloud-iac roots listed in
   this IP, not a generic Tier-A/Tier-B artifact tree.

## Counterpart refs

- `iac/cross-microservice-handoffs.md` rows for
  `application`, `payments`, and `developer-sdk` depend on cloud-iac chart
  validation/rendering before deploy.
- `iac/contracts/openapi/cloud-iac.yaml` operation
  `validateChartSignature` is the public chart validation surface that remains
  separate from AppProject bootstrap.

## Acceptance criteria

- `argocd_project_name` is present as both input and output in the namespace
  bootstrap module.
- `helm lint iac/iac/helm/argocd` passes before project
  bootstrap changes are promoted.
- This IP does not cite a nonexistent ArgoCD project manifest path.

## Validation commands

```bash
rg "argocd_project_name" iac/tofu/modules/k8s-namespace-bootstrap/main.tofu
helm lint iac/iac/helm/argocd
rg "validateChartSignature" iac/contracts/openapi/cloud-iac.yaml
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-GITOPS-008-argocd-project-bootstrap.md`.

## DR posture (per ADR-0343)

- Target source: `iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `86400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/seaweedfs-volume-failover.md`, `iac/manifest.json`, `iac/IP-GITOPS-008-argocd-project-bootstrap.md`.
