---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-001-storage-backend-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: cloud-secrets + axis-foundry-evidence
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance, oya-governance-cross-pack-replication-forbidden, oya-governance-evidence-index-append-only]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Storage backend IaC (Postgres + evidence-blob-store proxy to audit-chain WORM)

## Intent

Helm + Kustomize for Postgres HA evidence index + `evidence-blob-store` chart (which configures the audit-chain WORM bucket consumption — foundry-evidence does NOT own its own WORM bucket per ADR-0131 substrate split). Per-pack overlay. LTS pins per `docs/standards/foundry-evidence.md` (Slice D extension).

## ChangeSet boundary

Pure IaC. 3 Helm chart bundles (evidence-builder + postgres + evidence-blob-store) + shared Kustomize base + pack-kr overlay (M01 launch). Cross-cutting Terraform-managed subscription to audit-chain substrate's per-pack S3 export.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/iac/helm/postgres/Chart.yaml` | create | upstream dep on bitnami/postgresql-ha at pinned LTS |
| `microservices/foundry/iac/helm/postgres/values.yaml` | create | HA primary + replica; INSERT-only role for `foundry_evidence_writer`; SELECT for `foundry_evidence_reader`; SELECT+UPDATE+DELETE for `foundry_evidence_retention_cascader` (Cedar-gated RPC only) |
| `microservices/foundry/iac/helm/evidence-builder/Chart.yaml` | create | deploys pack-builder + recorder REST + bridge worker + regulator-export worker + archive-cascade worker |
| `microservices/foundry/iac/helm/evidence-builder/values.yaml` | create | per-component replicas + resources; SPIFFE binding; PDB; HPA |
| `microservices/foundry/iac/helm/evidence-blob-store/Chart.yaml` | create | wraps S3 bucket-policy chart for the audit-chain WORM bucket SUBSCRIPTION (read-side; foundry-evidence reads from substrate's bucket via cross-µservice IAM) |
| `microservices/foundry/iac/helm/evidence-blob-store/values.yaml` | create | per-pack substrate-bucket reference; cross-µservice IAM principal |
| `microservices/foundry/iac/kustomize/base/kustomization.yaml` | create | shared base |
| `microservices/foundry/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (3y retention reference; KR-pinned) |
| `microservices/foundry/iac/terraform/oci-evidence-blob-store-iam.tf` | create | cross-µservice IAM grant to read substrate-owned WORM bucket |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/postgres
helm lint microservices/foundry/iac/helm/evidence-builder
helm lint microservices/foundry/iac/helm/evidence-blob-store
kubectl --dry-run=client apply -k microservices/foundry/iac/kustomize/overlays/pack-kr
terraform plan microservices/foundry/iac/terraform/
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
cargo run -p oya-dev-cli -- gate validate cross-pack-replication-forbidden --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate evidence-index-append-only --microservice foundry-evidence
```

## Halt Conditions

- Postgres role grant LEAN check fails — block; writer must be INSERT-only.
- Cross-µservice IAM grant attempts to write to substrate-owned WORM — block; foundry-evidence is a READ-side consumer of WORM, not writer.
- Helm version pin drift — block; LTS only.

## Next IP

[`IP-002-self-slo-manifest.md`](IP-002-self-slo-manifest.md)

## References

- ADR-0117 §"Cloud-native infra"; ADR-0131 §"Substrate split".
- `microservices/foundry/policy/evidence-pack-integrity.md` §"EPI-04".
- `microservices/foundry/policy/data-residency.md`.
- `microservices/audit-chain/iac/helm/audit-storage/values.yaml` (substrate-side WORM config).
