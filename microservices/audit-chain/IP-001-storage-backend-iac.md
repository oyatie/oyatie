---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-audit-chain-substrate
impl_plan_id: IP-001-storage-backend-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: cloud-secrets + axis-audit-chain
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance, oya-governance-cross-pack-replication-forbidden]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Storage backend IaC (Postgres + S3 WORM + HSM operator)

## Intent

Helm + Kustomize for Postgres HA + S3 WORM bucket (Object Lock Compliance mode) + OCI Cloud-HSM operator under `microservices/audit-chain/iac/`. Deploys per-pack. Versions pinned to LTS per `docs/standards/audit-chain.md` (Slice D extension).

## ChangeSet boundary

Pure IaC. 3 Helm chart bundles + shared Kustomize base + pack-kr overlay (M01 launch). HSM partition provisioning via OCI OpenTofu (separate manifest). Per-pack secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/audit-chain/iac/helm/postgres/Chart.yaml` | create | upstream dep on bitnami/postgresql-ha at pinned LTS |
| `microservices/audit-chain/iac/helm/postgres/values.yaml` | create | HA primary + replica; INSERT-only role for `audit_emitter`; SELECT+INSERT for `audit_sealer` |
| `microservices/audit-chain/iac/helm/audit-storage/Chart.yaml` | create | wraps S3 bucket-policy Helm chart for OCI Object Storage |
| `microservices/audit-chain/iac/helm/audit-storage/values.yaml` | create | Object Lock Compliance mode; per-pack retention window; SSE-KMS |
| `microservices/audit-chain/iac/helm/hsm-operator/Chart.yaml` | create | OCI Cloud-HSM operator |
| `microservices/audit-chain/iac/helm/hsm-operator/values.yaml` | create | per-pack partition reference; SPIFFE-bound PKCS#11 cert |
| `microservices/audit-chain/iac/kustomize/base/kustomization.yaml` | create | shared base |
| `microservices/audit-chain/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (3y retention; KR-pinned) |
| `microservices/audit-chain/iac/terraform/oci-cloud-hsm-partition.tf` | create | Terraform-managed HSM partition lifecycle |

## Acceptance Gates

```bash
helm lint microservices/audit-chain/iac/helm/postgres
helm lint microservices/audit-chain/iac/helm/audit-storage
helm lint microservices/audit-chain/iac/helm/hsm-operator
kubectl --dry-run=client apply -k microservices/audit-chain/iac/kustomize/overlays/pack-kr
tofu plan microservices/audit-chain/iac/terraform/
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice audit-chain
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
cargo run -p oya-dev-cli -- gate validate cross-pack-replication-forbidden --microservice audit-chain
```

## Halt Conditions

- Object Lock Compliance mode policy fails to deploy — block; this is the load-bearing immutability invariant.
- HSM partition OpenTofu fails — block; engage cloud-secrets.
- Postgres role grant LEAN check fails — block; emitter must be INSERT-only.

## Next IP

[`IP-002-self-slo-manifest.md`](IP-002-self-slo-manifest.md)

## References

- ADR-0117 §"Cloud-native infra"; ADR-0028; ADR-0131.
- `microservices/audit-chain/policy/seal-integrity.md` §"SI-06..SI-12".
- `microservices/audit-chain/policy/data-residency.md`.
- OCI Cloud-HSM docs.
