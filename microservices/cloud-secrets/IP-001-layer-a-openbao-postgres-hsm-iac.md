---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-001-layer-a-openbao-postgres-hsm-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-secrets + ops-sre
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, version-pinning-conformance, per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A OpenBao + Postgres-HA + HSM-operator IaC

## Intent

Author Helm + Kustomize manifests for the Layer-A cloud-secrets stack: OpenBao 2.x LTS, Patroni-HA Postgres (OpenBao storage backend), HSM operator (drives OCI Cloud-HSM / Thales Luna PKCS#11 client config). Deploys to the dedicated cloud-secrets cluster per `multi-region.md`. Versions pinned to LTS per `docs/standards/version-pinning.md`.

## ChangeSet boundary

One cohesive ChangeSet: 3 Helm chart bundles (openbao, postgres, hsm-operator) + 1 shared Kustomize base + pack-kr Kustomize overlay (initial active pack). No code; pure IaC + values. All secret references in values use `${openbao:secret/...}` form — chicken-and-egg note: bootstrap KEK in HSM ceremony (4-eye) precedes Helm install; bootstrap-only secrets live in HSM-resident OpenBao seed-state.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cloud-secrets/iac/helm/openbao/Chart.yaml` | create | OpenBao 2.x LTS chart |
| `microservices/cloud-secrets/iac/helm/openbao/values.yaml` | create | HA Raft 5-node; auto-unseal PKCS#11; Postgres storage; mTLS |
| `microservices/cloud-secrets/iac/helm/postgres/Chart.yaml` | create | Patroni-HA Postgres chart |
| `microservices/cloud-secrets/iac/helm/postgres/values.yaml` | create | 3-node HA; LUKS at rest; encrypted backups |
| `microservices/cloud-secrets/iac/helm/hsm-operator/Chart.yaml` | create | HSM operator chart |
| `microservices/cloud-secrets/iac/helm/hsm-operator/values.yaml` | create | PKCS#11 client; partition discovery; attestation cron |
| `microservices/cloud-secrets/iac/kustomize/base/kustomization.yaml` | create | Shared base referencing all 3 charts |
| `microservices/cloud-secrets/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (Thales Luna; OCI ap-seoul-1) |

## Crate Naming

n/a — IaC only.

## Code Shape (excerpts)

`openbao/values.yaml`:

```yaml
openbao:
  fullnameOverride: openbao
  image:
    repository: openbao/openbao
    tag: "2.0.0"   # LTS pinned per docs/standards/version-pinning.md
  server:
    ha:
      enabled: true
      replicas: 5
      raft:
        enabled: true
        setNodeId: true
    auditDevice:
      type: file
      filePath: /var/log/openbao/audit/audit.log
    autoUnseal:
      type: pkcs11
      pkcs11:
        lib: /usr/local/lib/pkcs11/libCryptoki2_64.so
        slot: 0
        pin: "${openbao_bootstrap:hsm-pin}"   # bootstrap-only seed
        keyLabel: "${openbao_bootstrap:kek-alias}"
        mechanism: 0x1086   # CKM_AES_GCM
  service:
    type: ClusterIP
    annotations:
      service.alpha.kubernetes.io/tolerate-unready-endpoints: "true"
  ingress:
    enabled: false   # mTLS-only; in-cluster
```

`postgres/values.yaml`:

```yaml
patroni:
  replicaCount: 3
  postgresql:
    parameters:
      ssl: on
      synchronous_commit: remote_apply
  storage:
    storageClassName: oci-bv-luks
    size: 500Gi
  backup:
    enabled: true
    schedule: "0 */1 * * *"
    encryption:
      kmsKey: "${openbao:secret/shared/cloud-secrets/postgres-backup-kek}"
```

`hsm-operator/values.yaml`:

```yaml
hsmOperator:
  vendor: "${pack_overlay:hsm_vendor}"   # oci-cloud-hsm | thales-luna
  partitionId: "${openbao:secret/shared/cloud-secrets/hsm-partition-id}"
  attestation:
    cadenceCron: "0 3 * * *"   # 03:00 UTC daily
    verifierEndpoint: "${openbao:secret/shared/cloud-secrets/attestation-verifier-endpoint}"
```

## Acceptance Gates

```bash
helm lint microservices/cloud-secrets/iac/helm/openbao
helm lint microservices/cloud-secrets/iac/helm/postgres
helm lint microservices/cloud-secrets/iac/helm/hsm-operator
kubectl --dry-run=client apply -k microservices/cloud-secrets/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
cargo run -p oya-dev-cli -- gate validate lean-a11 --microservice cloud-secrets   # no raw secrets in IaC
```

## Test Plan

- helm-lint + helm template snapshot per chart.
- kind-cluster smoke: apply pack-kr overlay; all 3 components reach Ready within 10 min.
- Bootstrap drill: 4-eye HSM KEK ceremony → seed-state populated → first Helm install → auto-unseal succeeds.

## Halt Conditions

- OpenBao chart upstream version drift from LTS pin.
- Raw secret in any values file (LEAN-A11 BLOCKER).
- kind smoke fails — root-cause; do not mask.

## Next IP

`IP-002-secretreference-uri-spec.md`

## References

- ADR-0131 (Cloud split)
- `microservices/cloud-secrets/multi-region.md`
- `microservices/cloud-secrets/capacity-model.md`
- `docs/standards/version-pinning.md`
- OpenBao 2.x release notes
