# IP-GITOPS-006 — Secret bootstrap for cloud-iac IaC

> ADR anchor: ADR-0202, ADR-0173.
> Owner: `axis-cloud-iac + axis-cloud-secrets`.
> Scope: `iac` only.

## Goal

Bind cloud-iac bootstrap to the existing service-local OpenTofu module and
handoff contracts for OpenBao-backed secret references. This IP does not define
OpenBao internals; it defines the cloud-iac side of secret-reference
consumption.

## Real service paths

| Path | Contract |
|---|---|
| `iac/tofu/modules/secrets-bootstrap/main.tofu` | module variables and OpenBao seed outputs |
| `iac/tofu/modules/secrets-bootstrap/README.md` | operator-facing module contract |
| `iac/iac/helm/seaweedfs/values.yaml` | `openbao://secret/cloud-iac/seaweedfs/*` references |
| `iac/iac/helm/opentofu/values.yaml` | OpenTofu runner secret/config references |
| `iac/policy/ci-scope.cedar` | worker action scoping for registry/apply/render identities |
| `iac/policy/iac-isolation.md` | secret and apply isolation invariants |

## Implementation contract

1. `secrets-bootstrap` exposes references, not raw secret values.
2. SeaweedFS S3 access keys remain references in values files:
   `openbao://secret/cloud-iac/seaweedfs/s3-access-key` and
   `openbao://secret/cloud-iac/seaweedfs/s3-secret-key`.
3. Cloud-iac workers consume secret references through policy-controlled
   handoffs; no IP may add literal credentials to chart values, module files, or
   documentation.
4. Rotation readiness is validated through downstream secret-rotated and
   secret-revoked event handling, not by manual token copying.

## Counterpart refs

- `iac/cross-microservice-handoffs.md` outbound rows to
  `cloud-secrets` define signer and kubeconfig secret-reference reads.
- `iac/contracts/asyncapi/cloud-iac-events.yaml` subscribed
  channels `cloud-secrets.secret.rotated` and `cloud-secrets.secret.revoked`
  define rotation/revocation behavior.

## Acceptance criteria

- No literal access key or secret key is introduced in cloud-iac IaC.
- `secrets-bootstrap/main.tofu` remains the only cloud-iac OpenTofu module for
  OpenBao seed outputs.
- SeaweedFS and runner charts reference secrets by indirection.

## Validation commands

```bash
rg "openbao://secret/cloud-iac" iac/iac iac/tofu
rg "cloud-secrets.secret.rotated|cloud-secrets.secret.revoked" iac/contracts/asyncapi/cloud-iac-events.yaml iac/cross-microservice-handoffs.md
rg "accessKeySecretRef|secretKeySecretRef" iac/iac/helm/seaweedfs/values.yaml
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-GITOPS-006-secret-bootstrap-tier-b.md`.
