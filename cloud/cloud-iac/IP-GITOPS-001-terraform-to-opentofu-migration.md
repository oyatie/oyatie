# IP-GITOPS-001 — Cloud-IaC OpenTofu module source migration

> ADR anchor: ADR-0202, ADR-0173.
> Owner: `axis-cloud-iac`.
> Scope: `microservices/cloud-iac` only.

## Goal

Make cloud-iac's IaC execution surface OpenTofu-native without claiming a
repo-wide migration owned by other microservices. The concrete source of truth is
the service-local module registry under `microservices/cloud-iac/tofu/modules/`
and the OpenTofu runner chart under `microservices/cloud-iac/iac/helm/opentofu/`.

## Real service paths

| Path | Contract this IP protects |
|---|---|
| `microservices/cloud-iac/tofu/modules/cloud-account/main.tofu` | account bootstrap variables and `account_id`/`account_alias` outputs |
| `microservices/cloud-iac/tofu/modules/vpc/main.tofu` | network module surface consumed before namespace bootstrap |
| `microservices/cloud-iac/tofu/modules/dns/main.tofu` | DNS module surface for pack/environment endpoints |
| `microservices/cloud-iac/tofu/modules/kms/main.tofu` | KMS key material references for state and backup encryption |
| `microservices/cloud-iac/tofu/modules/secrets-bootstrap/main.tofu` | OpenBao seed outputs consumed by secret projection |
| `microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap/main.tofu` | `namespace_name`, `service_account_name`, and `argocd_project_name` outputs |
| `microservices/cloud-iac/iac/helm/opentofu/Chart.yaml` | runner deployment package |
| `microservices/cloud-iac/iac/helm/opentofu/values.yaml` | runner image/version/resource contract |

## Implementation contract

1. Keep all cloud-iac module entrypoints in `.tofu` files. Do not introduce new
   `.tf` module roots in this service.
2. Preserve each module README beside its `main.tofu`; the README is the
   operator-facing contract for inputs, outputs, and apply order.
3. When provider resources are wired, provider source declarations live in the
   existing module file for that domain, not in a new ad hoc composition path.
4. The OpenTofu runner chart is the only service-local execution primitive for
   module validation and plan/apply jobs.

## Counterpart refs

- `microservices/cloud-iac/cross-microservice-handoffs.md` outbound rows for
  `cloud-secrets` secret references and `cloud-k8s` apply/rollback calls define
  the non-local dependencies this IP must not inline.
- `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` operations
  `planPreview`, `triggerApply`, and `getApplyState` are the REST surfaces that
  expose OpenTofu plan/apply state to counterpart services.

## Acceptance criteria

- `find microservices/cloud-iac -name "*.tf"` returns no service-local module
  roots.
- `tofu validate` is run for each existing directory under
  `microservices/cloud-iac/tofu/modules/`.
- `helm lint microservices/cloud-iac/iac/helm/opentofu` passes before any
  runner image/version change is promoted.
- No task in this IP edits another microservice's IaC tree.

## Validation commands

```bash
find microservices/cloud-iac -name "*.tf" -print
find microservices/cloud-iac/tofu/modules -maxdepth 2 -name main.tofu -print
helm lint microservices/cloud-iac/iac/helm/opentofu
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-GITOPS-001-terraform-to-opentofu-migration.md`.
