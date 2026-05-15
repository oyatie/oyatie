---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P18-cloud-tenancy
status: Proposed
acceptance_lanes: []
entry_gate: 'M02/P13-tenancy complete (TenantProductRegistry live; oyatie.set_current_tenant()

  deployed); M02/P08-kms complete (oya-kms-kernel ships with envelope encryption ports);

  M02/P06-secrets complete (oya-secrets-kernel ships with SecretReference port);

  cargo check clean; grit done on all P13/P08/P06 symbols; ICM phase-handoff emitted.

  '
exit_gate: 'All P18 impl-plan acceptance gates green; 8 BCs registered (cloud-tenancy,

  cloud-iam, cloud-kms, cloud-compute, cloud-storage, cloud-network, cloud-billing,

  cloud-cell); per-cell OCI ARM64 deployment manifest generated; KMS envelope

  encryption wired; all crates pass cargo check/build/clippy/nextest/deny;

  oya gate validate lean-a1/a2/a3/a4 exit 0; grit done on all P18 symbols;

  ICM phase-complete row emitted.

  '
depends_on:
- milestone: M02
  phase: P13-tenancy
  reason: Cloud tenancy substrate reads TenantProductRegistry to determine which cloud
    resources to provision per tenant; cell assignment uses TenantCellPlacer output.
- milestone: M02
  phase: P08-kms
  reason: Cloud KMS BC wraps oya-kms-kernel envelope encryption ports; per-cell DEK
    management extends P08 KMS kernel.
- milestone: M02
  phase: P06-secrets
  reason: Cloud IAM credentials and OCI Vault references stored via oya-secrets-kernel
    SecretReference port; no plaintext credentials in cloud-tenancy tables.
owner_team: council-cloud
purpose: "Delivers the cloud tenancy substrate: the multi-tenant runtime infrastructure layer that provisions and manages per-tenant OCI resources."
---
# P18-cloud-tenancy: Cloud Tenancy + IAM + KMS + Billing-Skeleton + Compute + Storage + Network — Multi-Tenant Runtime Substrate

## Purpose

Delivers the cloud tenancy substrate: the multi-tenant runtime infrastructure layer that
provisions and manages per-tenant OCI resources. Per Bominal ADR-0117 (cloud-native
infrastructure architecture), the cloud µservice plays a dual role — product µservice AND
infrastructure substrate — with `public_layers = ["sdk"]` exemption in the microservice
registry for cross-µservice SDK imports.

Eight BCs cover the full cloud runtime surface: `cloud-tenancy` (per-tenant OCI namespace
+ quota management), `cloud-iam` (service account + role binding per tenant), `cloud-kms`
(per-cell DEK envelope encryption extending P08 KMS), `cloud-compute` (OKE node pool +
HPA/VPA manifests), `cloud-storage` (OCI Object Storage bucket provisioning), `cloud-network`
(VCN + subnet + security-list per cell), `cloud-billing` (usage metering skeleton; full
billing engine in M03), `cloud-cell` (cell lifecycle: create/drain/decommission).

OCI A1 ARM64 Always Free profile is the launch target (ADR-0021 amended 2026-04-28);
the architecture stages from Stage 0 (OCI A1 VM) through Stage 3 (multi-region OKE)
without rearchitecting per ADR-0117 §1.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `cloud` | `tenancy` | `crates/oya-cloud-tenancy-kernel/` | `oya-cloud-tenancy-kernel` |
| `cloud` | `tenancy` | `crates/oya-cloud-tenancy-domain/` | `oya-cloud-tenancy-domain` |
| `cloud` | `tenancy` | `crates/oya-cloud-tenancy-application/` | `oya-cloud-tenancy-application` |
| `cloud` | `tenancy` | `crates/oya-cloud-tenancy-adapter/` | `oya-cloud-tenancy-adapter` |
| `cloud` | `iam` | `crates/oya-cloud-iam-kernel/` | `oya-cloud-iam-kernel` |
| `cloud` | `iam` | `crates/oya-cloud-iam-adapter/` | `oya-cloud-iam-adapter` |
| `cloud` | `kms` | `crates/oya-cloud-kms-kernel/` | `oya-cloud-kms-kernel` |
| `cloud` | `kms` | `crates/oya-cloud-kms-adapter/` | `oya-cloud-kms-adapter` |
| `cloud` | `compute` | `crates/oya-cloud-compute-kernel/` | `oya-cloud-compute-kernel` |
| `cloud` | `compute` | `crates/oya-cloud-compute-adapter/` | `oya-cloud-compute-adapter` |
| `cloud` | `storage` | `crates/oya-cloud-storage-kernel/` | `oya-cloud-storage-kernel` |
| `cloud` | `storage` | `crates/oya-cloud-storage-adapter/` | `oya-cloud-storage-adapter` |
| `cloud` | `storage` | `crates/oya-cloud-storage-sdk/` | `oya-cloud-storage-sdk` |
| `cloud` | `network` | `crates/oya-cloud-network-kernel/` | `oya-cloud-network-kernel` |
| `cloud` | `network` | `crates/oya-cloud-network-adapter/` | `oya-cloud-network-adapter` |
| `cloud` | `billing` | `crates/oya-cloud-billing-kernel/` | `oya-cloud-billing-kernel` |
| `cloud` | `billing` | `crates/oya-cloud-billing-adapter/` | `oya-cloud-billing-adapter` |
| `cloud` | `cell` | `crates/oya-cloud-cell-kernel/` | `oya-cloud-cell-kernel` |
| `cloud` | `cell` | `crates/oya-cloud-cell-application/` | `oya-cloud-cell-application` |
| `cloud` | `cell` | `crates/oya-cloud-cell-adapter/` | `oya-cloud-cell-adapter` |
| `cloud` | all | `crates/oya-cloud-app/` | `oya-cloud-app` |
| `cloud` | all | `contracts/cloud.openapi.yaml` | — |
| `cloud` | all | `migrations/cloud/V001__cloud_schema.sql` | — |

Naming justification:

```
NAME: oya-cloud-tenancy-kernel
JUSTIFICATION:
- microservice = cloud: the cloud infrastructure µservice; dual-role (product +
  substrate); public_layers = ["sdk"] in microservice registry per ADR-0056 v4.1
  cloud dual-role mechanism; ADR-0117
- bc-tokens = tenancy: per-tenant OCI namespace BC; separate from iam/kms/compute/
  storage/network/billing/cell BCs at same layer
- layer = kernel: CloudTenantStore + OciNamespaceProvisioner sealed ports; CloudTenant
  + OciNamespace + CellId types; ZERO I/O
- exemptions claimed: none

NAME: oya-cloud-storage-sdk
JUSTIFICATION:
- microservice = cloud, bc-tokens = storage: object storage BC
- layer = sdk: public_layers = ["sdk"] — this SDK is the cross-µservice surface
  (Connect, Records, etc. may import oya-cloud-storage-sdk per the cloud dual-role
  LEAN-A2 exemption); depends on storage-kernel only; ADR-0056 sdk-kernel-only rule
- exemptions claimed: cloud.public_layers["sdk"] — explicitly allowed by ADR-0056
  §"Cloud Dual-Role + public_layers Mechanism"
```

### Out-of-scope

- Full billing engine with invoice generation — deferred to M03
- Multi-region active-active OKE setup — deferred to M04+ (Stage 3 per ADR-0117)
- ScyllaDB provisioning — deferred to when IoT threshold hit (ADR-0117 §Cassandra/ScyllaDB)

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-cloud-tenancy-kernel-scaffold.md`](IP-001-cloud-tenancy-kernel-scaffold.md) | Scaffold all 21 cloud crates; 8 BC kernels; full DDL; OCI ARM64 deployment manifest skeleton | pending | `council-cloud` |
| [`IP-002-cloud-iam-kms-wiring.md`](IP-002-cloud-iam-kms-wiring.md) | Cloud IAM service account provisioning; cloud KMS DEK wrapping P08 KMS kernel | pending | `council-cloud` |
| [`IP-003-cloud-compute-storage-network.md`](IP-003-cloud-compute-storage-network.md) | OKE node pool manifests; OCI Object Storage bucket provisioning; VCN/subnet per cell | pending | `council-cloud` |
| [`IP-004-cloud-cell-lifecycle.md`](IP-004-cloud-cell-lifecycle.md) | Cell create/drain/decommission state machine; cell health check | pending | `council-cloud` |
| [`IP-005-cloud-load-tests.md`](IP-005-cloud-load-tests.md) | k6 load tests; cell provisioning p99 ≤500ms; storage SDK p99 ≤50ms | pending | `council-cloud` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P18-cloud-tenancy
oya gate validate lean-a2 --phase P18-cloud-tenancy
oya gate validate lean-a3 --phase P18-cloud-tenancy
oya gate validate lean-a4 --phase P18-cloud-tenancy
```

### Cloud-specific gates

```bash
# SDK depends only on kernel (sdk-kernel-only lane)
oya gate validate sdk-kernel-only --crate oya-cloud-storage-sdk   # exit 0
# Cross-µservice SDK import allowed via public_layers exemption
oya gate validate lean-a2 --exemption cloud-sdk-public-layers      # exit 0
# OCI ARM64 deployment manifests valid
cargo nextest run -p oya-cloud-cell-adapter --test arm64_manifest_valid  # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments (representative subset)

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-cloud-tenancy-kernel` | `kernel` | Yes — CloudTenantStore, OciNamespaceProvisioner | N/A |
| `oya-cloud-tenancy-adapter` | `adapter` | N/A | Yes — OciCloudTenantAdapter |
| `oya-cloud-iam-kernel` | `kernel` | Yes — ServiceAccountStore, RoleBindingPort | N/A |
| `oya-cloud-kms-kernel` | `kernel` | Yes — CloudKmsPort (wraps P08 KmsMasterKeyStore) | N/A |
| `oya-cloud-compute-kernel` | `kernel` | Yes — NodePoolManager, HpaConfigStore | N/A |
| `oya-cloud-storage-kernel` | `kernel` | Yes — ObjectStoragePort | N/A |
| `oya-cloud-storage-sdk` | `sdk` | N/A — depends on storage-kernel only | N/A |
| `oya-cloud-network-kernel` | `kernel` | Yes — VcnManager, SecurityListPort | N/A |
| `oya-cloud-billing-kernel` | `kernel` | Yes — UsageMeterPort | N/A |
| `oya-cloud-cell-kernel` | `kernel` | Yes — CellLifecycleStore, CellHealthPort | N/A |
| `oya-cloud-app` | `app` | N/A | Unrestricted inward |

### CI lanes that must green

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P18-cloud-tenancy` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P18-cloud-tenancy` | exit 0 |
| `sdk-kernel-only` | `oya gate validate sdk-kernel-only --crate oya-cloud-storage-sdk` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P18-cloud-tenancy` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P18-cloud-tenancy` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `cloud-tenancy` | `cloud` | pending |
| `cloud-iam` | `cloud` | pending |
| `cloud-kms` | `cloud` | pending |
| `cloud-compute` | `cloud` | pending |
| `cloud-storage` | `cloud` | pending |
| `cloud-network` | `cloud` | pending |
| `cloud-billing` | `cloud` | pending |
| `cloud-cell` | `cloud` | pending |

---

## Grit Claim Symbols

```
crates/oya-cloud-tenancy-kernel/src/lib.rs::CloudTenantStore
crates/oya-cloud-iam-kernel/src/lib.rs::ServiceAccountStore
crates/oya-cloud-kms-kernel/src/lib.rs::CloudKmsPort
crates/oya-cloud-compute-kernel/src/lib.rs::NodePoolManager
crates/oya-cloud-storage-kernel/src/lib.rs::ObjectStoragePort
crates/oya-cloud-storage-sdk/src/lib.rs::CloudStorageClient
crates/oya-cloud-cell-kernel/src/lib.rs::CellLifecycleStore
contracts/cloud.openapi.yaml::provisionCell
migrations/cloud/V001__cloud_schema.sql::cloud.cells
```

TTL: `--ttl 3600`. Fallback: ICM `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P18-cloud-tenancy started; 8 cloud BCs; OCI ARM64 launch profile; ADR-0117 stages; depends P13/P08/P06" \
  -i high \
  -k "M02,P18,phase-start,cloud"

icm store \
  -t context-oyatie \
  -c "Phase P18-cloud-tenancy complete; 8 BCs live; cloud-storage-sdk public_layers exemption verified; cell lifecycle state machine; next: P19-application" \
  -i high \
  -k "M02,P18,phase-complete,cloud"
```

---

## References

- Bominal ADRs inherited: ADR-0117 (cloud-native infra), ADR-0021 (OCI A1 launch profile), ADR-0009 (cell architecture)
- oyatie ADRs cited: ADR-0056 v4.1 (cloud dual-role + public_layers)
- M02-substrate-schema-foundation §6-N (cloud outlined)
