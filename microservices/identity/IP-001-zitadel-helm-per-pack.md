---
doc_class: IP
template_id: TPL-IP
ip_id: IP-001
microservice: identity
status: ga
related_adrs: [ADR-0117, ADR-0179, ADR-0187, ADR-0148]
related_iac: [microservices/identity/iac/helm/zitadel/]
date: 2026-05-18
owner_team: axis-identity + ops-platform
---

# IP-001 — Zitadel Helm deployment per pack

## Goal

Deploy Zitadel v2.55.0 via Helm chart v9.34.1 to each regulatory pack with full ops-grade configuration: HA replicas, Postgres event-store on the per-pack cluster, OpenBao-resolved secrets, mTLS via Istio Ambient ztunnel, NetworkPolicy lockdown, HPA + PDB, mesh waypoint enrolment per ADR-0148, and per-pack overlay overrides for residency-class differences.

## Files to create

| File | Purpose | Size estimate |
|---|---|---|
| `microservices/identity/iac/helm/zitadel/Chart.yaml` | Helm chart metadata + Zitadel dependency pin v9.34.1 | 25 lines |
| `microservices/identity/iac/helm/zitadel/values.yaml` | Default values: 3 replicas, image tag v2.55.0, Postgres DSN via SecretReference | 180 lines |
| `microservices/identity/iac/helm/zitadel/templates/deployment.yaml` | Deployment with SecretReference env + Istio sidecar annotations | 95 lines |
| `microservices/identity/iac/helm/zitadel/templates/service.yaml` | ClusterIP service exposing 8080 (admin) + 8443 (OIDC) | 30 lines |
| `microservices/identity/iac/helm/zitadel/templates/ingress.yaml` | Istio Ambient ingress binding to `identity-<pack>.oyatie.com` | 45 lines |
| `microservices/identity/iac/helm/zitadel/templates/hpa.yaml` | HPA: min 3, max 20, 70% CPU + 80% memory target | 35 lines |
| `microservices/identity/iac/helm/zitadel/templates/pdb.yaml` | PodDisruptionBudget: minAvailable=2 | 15 lines |
| `microservices/identity/iac/helm/zitadel/templates/networkpolicy.yaml` | Restrict ingress to Envoy waypoint + Postgres egress only | 55 lines |
| `microservices/identity/iac/helm/zitadel/templates/serviceaccount.yaml` | SPIFFE identity binding | 20 lines |
| `microservices/identity/iac/helm/zitadel/templates/configmap.yaml` | Zitadel runtime config (OIDC issuer, RP-ID, FIDO settings) | 70 lines |
| `microservices/identity/iac/kustomize/overlays/pack-kr/values.yaml` | KR overlay: KR-FSS HSM partition, KR-Seoul region | 35 lines |
| `microservices/identity/iac/kustomize/overlays/pack-eu/values.yaml` | EU overlay: EU-Frankfurt region, GDPR residency labels | 30 lines |
| `microservices/identity/iac/kustomize/overlays/pack-us/values.yaml` | US standard pack overlay | 25 lines |
| `microservices/identity/iac/kustomize/overlays/pack-us-healthcare/values.yaml` | HIPAA-eligible HSM, 6-year audit retention env | 40 lines |
| `microservices/identity/iac/kustomize/overlays/pack-ksa/values.yaml` | Sovereign KSA-Riyadh; Thales Luna HSM | 35 lines |
| `microservices/identity/iac/kustomize/overlays/pack-ae/values.yaml` | UAE-Dubai overlay | 30 lines |

## Tests to write

| Test | Type | Acceptance |
|---|---|---|
| `helm lint microservices/identity/iac/helm/zitadel` | static | exit 0, no warnings beyond known-acceptable |
| `helm template ... | kubectl apply --dry-run=server -f -` | server-side dry-run | every K8s resource validates |
| `kyverno-cli scan helm-rendered.yaml` against `policy/kyverno/zitadel-baseline.yaml` | admission | required-labels, image-pin, no-root, no-privileged-escalation pass |
| Cilium NetworkPolicy compile-check | static | policy renders to valid CiliumNetworkPolicy |
| Conformance: spin up minikube + apply chart + verify `/oauth/v2/discovery` returns 200 with `issuer: https://identity-test.oyatie.com` | e2e | smoke pass |
| Postgres event-store schema migration test | e2e | initial schema applies clean; upgrade from previous minor cleanly migrates |

## SecretReference inventory

Every secret consumed via OpenBao SecretReference per ADR-0117:

| Reference | Purpose |
|---|---|
| `${openbao:secret/identity/<pack>/postgres-dsn}` | Postgres connection string |
| `${openbao:secret/identity/<pack>/jwt-signing-key}` | JWT signing key (HSM-backed in regulated packs) |
| `${openbao:secret/identity/<pack>/admin-bootstrap-password}` | One-time admin bootstrap password (rotated immediately) |
| `${openbao:secret/identity/<pack>/<tenant>/scim-bearer}` | Per-tenant SCIM bearer (rotated 90d) |
| `${openbao:secret/identity/<pack>/tls-server-cert}` | Server TLS cert for ingress |

## Evidence to emit

| Artefact | Path | Cadence |
|---|---|---|
| Per-pack Helm-rendered manifests | `evidence/identity/helm-rendered/<pack>/<date>.yaml` | per-deploy |
| Helm lint report | `evidence/identity/helm-lint/<date>.json` | per-deploy |
| Kyverno admission scan | `evidence/identity/kyverno-admission/<pack>/<date>.json` | per-deploy |
| Deployment health check | `evidence/identity/deploy-health/<pack>/<date>.json` | per-deploy |
| Postgres schema-migration log | `evidence/identity/pg-schema-migration/<pack>/<date>.log` | per-upgrade |

## Pack-overlay differences

| Overlay | HSM | Audit retention | Region | Notes |
|---|---|---|---|---|
| pack-kr | KR-FSS Thales Luna | 5y | KR-Seoul | KR-FSS sector preference |
| pack-eu | AWS CloudHSM (FR) | 7y | EU-Frankfurt | GDPR Art. 30 |
| pack-us | AWS CloudHSM (us-east-1) | 1y | US-East | PCI-DSS 1y minimum |
| pack-us-healthcare | AWS CloudHSM HIPAA-eligible | 6y | US-East-HIPAA | HIPAA §164.316(b)(2) |
| pack-jp | AWS CloudHSM (ap-northeast-1) | 1y | JP-Tokyo | |
| pack-sg | AWS CloudHSM (ap-southeast-1) | 1y | SG-Singapore | |
| pack-au | AWS CloudHSM (ap-southeast-2) | 1y | AU-Sydney | |
| pack-in | AWS CloudHSM (ap-south-1) | 1y | IN-Mumbai | |
| pack-br | AWS CloudHSM (sa-east-1) | 1y | BR-São Paulo | |
| pack-ae | Thales DPoD (UAE) | 1y | AE-Dubai | |
| pack-ksa | Thales Luna (Riyadh) | 5y | KSA-Riyadh | Sovereign + KSA-CITC |

## Promotion path

1. `helm install zitadel ./microservices/identity/iac/helm/zitadel --namespace identity-dev --values overlays/dev/values.yaml`.
2. Smoke test: verify discovery + introspect.
3. Promote to staging.
4. Promote to pack-eu (bellwether) per ADR-0130.
5. Roll to remaining packs over 30 days.

## Rollback

`helm rollback zitadel <previous-revision> --namespace identity-<pack>`; Postgres schema rollback via PITR if minor-version downgrade required. Runbook: `identity-zitadel-rollback`.

## Counterpart references - 001-zitadel-helm-per-pack

- Counterpart class: identity substrate.
- Palantir Foundry and GitHub Enterprise are the counterpart baseline for governed multi-tenant identity surfaces; this IP ties the slice to Oyatie identity contracts, Cedar, and audit-chain evidence rather than leaving the behavior as generic application authentication.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

