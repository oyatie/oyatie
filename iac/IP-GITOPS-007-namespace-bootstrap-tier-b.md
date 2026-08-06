# IP-GITOPS-007 — Kubernetes namespace and AppProject bootstrap

> ADR anchor: ADR-0202.
> Owner: `axis-cloud-iac + axis-cloud-k8s`.
> Scope: `iac/tofu/modules/k8s-namespace-bootstrap`.

## Goal

Define the cloud-iac namespace/bootstrap contract around the existing
`k8s-namespace-bootstrap` OpenTofu module. The module is the service-local
authority for namespace, ServiceAccount, RBAC seed, NetworkPolicy seed, and
ArgoCD `AppProject` output references.

## Real service paths

| Path | Contract |
|---|---|
| `iac/tofu/modules/k8s-namespace-bootstrap/main.tofu` | module variables and outputs |
| `iac/tofu/modules/k8s-namespace-bootstrap/README.md` | operator contract |
| `iac/iac/kustomize/base/kustomization.yaml` | cloud-iac base rendered after bootstrap |
| `iac/iac/kustomize/overlays/pack-kr/kustomization.yaml` | first pack overlay |
| `iac/policy/ci-scope.cedar` | applier/rollback scope guards |
| `iac/contracts/openapi/cloud-iac.yaml` | apply state and rollback surfaces |

## Implementation contract

1. Module inputs remain `cluster_id`, `namespace`, `owner_team`,
   `argocd_project_name`, and `tags`.
2. Module outputs remain `namespace_name`, `service_account_name`, and
   `argocd_project_name`; consumers must not scrape provider internals.
3. Default-deny networking and RBAC are seeded before the Kustomize overlay is
   applied.
4. The cloud-iac applier cannot mutate resources outside the declared
   microservice apply scope enforced by `ci-scope.cedar`.

## Counterpart refs

- `iac/cross-microservice-handoffs.md` outbound
  `cloud-k8s` rows define apply and rollback calls after bootstrap.
- `iac/cross-microservice-handoffs.md` inbound
  `cloud-k8s` row defines apply-state reads from this service.

## Acceptance criteria

- `k8s-namespace-bootstrap/main.tofu` exposes the three named outputs.
- The pack overlay renders after namespace bootstrap without claiming ownership
  of another microservice's namespace.
- Cloud-k8s handoffs remain explicit REST calls, not implicit module side
  effects.

## Validation commands

```bash
rg "output \\\"namespace_name\\\"|output \\\"service_account_name\\\"|output \\\"argocd_project_name\\\"" iac/tofu/modules/k8s-namespace-bootstrap/main.tofu
rg "cloud-k8s|apply-state" iac/cross-microservice-handoffs.md
rg "apply_scope|cross-µservice apply" iac/policy/ci-scope.cedar
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-GITOPS-007-namespace-bootstrap-tier-b.md`.
