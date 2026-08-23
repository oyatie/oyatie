# Standard — GitOps + IaC + cluster lifecycle: tier boundaries

> ADR anchor: `docs/decisions/ADR-0709-general-live-apex.md`.
> Gate anchor: `crates/check-iac-tier-discipline/`.
> Authored 2026-05-18.

## TL;DR

Three tools, three tiers, zero overlap.

- **Tier A — ArgoCD** owns app deploy.
- **Tier B — OpenTofu** owns cloud-side resources.
- **Tier C — Cluster API** owns K8s cluster lifecycle.

If you cannot place a change cleanly in exactly one tier, you are
likely about to cross a boundary. Stop and re-read this standard.

## Boundary table

| Resource kind                | Owner Tier | Tool      |
| ---------------------------- | ---------- | --------- |
| ArgoCD Application CR        | A          | ArgoCD    |
| Helm release manifest        | A          | ArgoCD    |
| K8s Deployment / StatefulSet | A          | ArgoCD    |
| K8s DaemonSet / Job          | A          | ArgoCD    |
| K8s ConfigMap / Secret use   | A          | ArgoCD    |
| K8s namespace creation       | B          | OpenTofu  |
| K8s RBAC bootstrap           | B          | OpenTofu  |
| ArgoCD project bootstrap     | B          | OpenTofu  |
| VPC / subnet / route         | B          | OpenTofu  |
| IAM role + policy            | B          | OpenTofu  |
| KMS key                      | B          | OpenTofu  |
| DNS zone / record            | B          | OpenTofu  |
| RDS-equivalent instance      | B          | OpenTofu  |
| Object-storage bucket        | B          | OpenTofu  |
| OpenBao initial bootstrap    | B          | OpenTofu  |
| K8s cluster (the cluster)    | C          | ClusterAPI|
| Cluster upgrade plan         | C          | ClusterAPI|
| Cluster scale spec           | C          | ClusterAPI|

## Why this matters

A single tool spanning all three tiers always drifts. ADR-0202
lists the failure modes in detail. The pragmatic version:

- ArgoCD reconciling cloud-IAM is fragile (`cloud-provider`
  drift, partial-apply semantics).
- OpenTofu emitting per-pod manifests is fragile (no
  reconciliation loop, no health checks, no progressive
  rollout).
- Cluster API trying to ship apps re-implements ArgoCD worse.

## Where each tier's source lives

- Tier A: `microservices/<ms>/iac/argocd/`
- Tier B: `microservices/cloud-iac/tofu/modules/` + per-µservice
  `microservices/<ms>/iac/tofu/` (consumer of canonical modules)
- Tier C: `microservices/cloud-k8s/iac/cluster-api/`

## Canonical OpenTofu modules

Under `microservices/cloud-iac/tofu/modules/`:

- `cloud-account/` — root-of-trust account + organization
- `vpc/` — VPC + subnets + routing
- `dns/` — DNS zone + record bootstrap (publishes SPF / DKIM /
  DMARC records consumed by ADR-0201 email comms)
- `kms/` — KMS keys (per tenant, per region)
- `secrets-bootstrap/` — OpenBao initial seed
- `k8s-namespace-bootstrap/` — namespace + RBAC + network policy
  seed (so Tier-A ArgoCD can land app manifests safely)

## Discipline

- `check-iac-tier-discipline` is the advisory gate. It
  flips to BLOCKER at T+90d (post-Terraform-migration window).

## Migration timeline (Terraform → OpenTofu)

- T+0 (ADR-0202 land date): all new IaC is OpenTofu.
- T+30d: existing `.tf` files compile under `tofu`.
- T+60d: all execution goes through `tofu`.
- T+90d: residual Terraform usage is a BLOCKER violation.

## In-house posture

oyatie does NOT build replacements for ArgoCD / OpenTofu /
Cluster API. The in-house contribution is the boundary table
itself + the discipline gate. See ADR-0202 §"In-house roadmap".

## References

- ADR-0202 — GitOps + IaC + cluster lifecycle three-tier.
- ADR-0171 — multi-cluster federation.
- ADR-0173 — vendor lock-in avoidance.
- ArgoCD (Intuit / CNCF graduated).
- OpenTofu (Linux Foundation).
- Cluster API (Kubernetes SIG Cluster Lifecycle).
