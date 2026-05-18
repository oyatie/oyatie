# `cloud-account` OpenTofu module

> ADR anchor: ADR-0202 (Tier B).
> Canonical: yes.

Root-of-trust cloud account + organization. Consumed by `vpc`,
`dns`, `kms` modules downstream.

## Inputs

| Variable | Type | Required | Description |
| -------- | ---- | -------- | ----------- |
| `account_name` | string | yes | Logical account name |
| `billing_email` | string | yes | Billing contact |
| `tags` | map | no | Per-account tags |

## Outputs

| Output | Description |
| ------ | ----------- |
| `account_id` | Cloud-provider account identifier |
| `account_alias` | Cloud-provider account alias |

## Discipline

This module is Tier B (OpenTofu / cloud-side). It MUST NOT
declare per-pod manifests; `oya-check-iac-tier-discipline`
enforces.
