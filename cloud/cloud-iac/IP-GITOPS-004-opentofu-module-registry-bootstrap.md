# IP-GITOPS-004 — OpenTofu module registry bootstrap

> ADR anchor: ADR-0202.
> Owner: `axis-cloud-iac`.
> Scope: `microservices/cloud-iac/tofu/modules`.

## Goal

Harden the existing cloud-iac OpenTofu module registry as the canonical service
module catalog. The registry already exists at `microservices/cloud-iac/tofu/modules/`;
this IP records its real module surfaces and avoids references to a nonexistent
composition tree.

## Real module inventory

| Module path | Existing entrypoint | Existing operator doc | Required exported contract |
|---|---|---|---|
| `microservices/cloud-iac/tofu/modules/cloud-account/` | `main.tofu` | `README.md` | `account_id`, `account_alias` |
| `microservices/cloud-iac/tofu/modules/vpc/` | `main.tofu` | `README.md` | VPC/network identifiers consumed downstream |
| `microservices/cloud-iac/tofu/modules/dns/` | `main.tofu` | `README.md` | DNS zone/record identifiers |
| `microservices/cloud-iac/tofu/modules/kms/` | `main.tofu` | `README.md` | KMS key references for encrypted state/backups |
| `microservices/cloud-iac/tofu/modules/secrets-bootstrap/` | `main.tofu` | `README.md` | `bao_root_token_secret_id`, `bao_unseal_keys_secret_ids`, `pki_mount_path` |
| `microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap/` | `main.tofu` | `README.md` | `namespace_name`, `service_account_name`, `argocd_project_name` |

## Implementation contract

1. Every module keeps a single visible `main.tofu` entrypoint and a sibling
   `README.md` that describes input/output ownership.
2. Provider-specific resources may be added inside the existing module
   directory for that concern; do not create a parallel module registry.
3. Composition ordering is documented through module outputs and the cloud-iac
   apply contracts, not through a nonexistent service-local composition path.
4. State, KMS, OpenBao, namespace, and AppProject bootstrap remain split across
   the six existing modules.

## Counterpart refs

- `microservices/cloud-iac/cross-microservice-handoffs.md` outbound
  `cloud-secrets` rows define secret reference reads used by
  `secrets-bootstrap`.
- `microservices/cloud-iac/cross-microservice-handoffs.md` outbound `cloud-k8s`
  rows define where namespace/bootstrap outputs are ultimately applied.

## Acceptance criteria

- Exactly six first-level module directories exist under
  `microservices/cloud-iac/tofu/modules/`.
- Each module has `main.tofu` and `README.md`.
- This IP contains no nonexistent composition-tree citation.

## Validation commands

```bash
find microservices/cloud-iac/tofu/modules -mindepth 1 -maxdepth 1 -type d | sort
find microservices/cloud-iac/tofu/modules -mindepth 2 -maxdepth 2 \\( -name main.tofu -o -name README.md \\) | sort
```
