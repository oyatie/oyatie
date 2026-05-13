---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P18-cloud-tenancy
impl_plan_id: IP-001-cloud-tenancy-kernel-scaffold
status: pending
owner: council-cloud
blocked_by:
  - impl_plan: P13-tenancy/IP-001
    reason: "CloudTenantAdapter reads TenantCellPlacer output; cloud.cells FK references tenancy.tenants"
  - impl_plan: P08-kms/IP-001
    reason: "CloudKmsAdapter wraps oya-kms-kernel KmsMasterKeyStore for per-cell DEK envelope"
  - impl_plan: P06-secrets/IP-001
    reason: "OCI Vault credentials stored via oya-secrets-kernel SecretReference port"
acceptance_lanes:
  - cargo-check
  - cargo-build
  - cargo-clippy
  - cargo-nextest
  - cargo-deny
  - lean-a1
  - lean-a2
  - lean-a3
  - lean-a4
---

# IP-001-cloud-tenancy-kernel-scaffold: Scaffold All 21 Cloud Crates — 8 BC Kernels + DDL + OCI ARM64 Manifests + Cell Lifecycle

## Intent

Scaffolds all 21 cloud crates across 8 BCs, authors the complete Postgres DDL for
cloud resource tracking tables, implements the cell lifecycle state machine (create →
active → draining → decommissioned), wires KMS envelope encryption through the P08
kernel port, and generates the OCI ARM64 deployment manifest skeleton. After this IP
merges, the full cloud infrastructure substrate is ready for per-tenant provisioning;
`oya-cloud-storage-sdk` is the cross-µservice SDK surface for other products.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 21 cloud workspace members; add oci-sdk = "0.1" to workspace deps |
| `crates/oya-cloud-tenancy-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-cloud-tenancy-kernel/src/lib.rs` | create | CloudTenantStore + OciNamespaceProvisioner ports; CloudTenant + OciNamespace + OciRegion types |
| `crates/oya-cloud-tenancy-domain/Cargo.toml` | create | Depends on kernel only |
| `crates/oya-cloud-tenancy-domain/src/lib.rs` | create | CloudTenantDomainLogic: namespace_for_tenant(); validate_region_support() |
| `crates/oya-cloud-tenancy-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-cloud-tenancy-application/src/lib.rs` | create | ProvisionTenantNamespaceUseCase; DecommissionTenantUseCase |
| `crates/oya-cloud-tenancy-adapter/Cargo.toml` | create | Depends on application + domain + kernel + oya-tenancy-kernel + oci-sdk + sqlx |
| `crates/oya-cloud-tenancy-adapter/src/lib.rs` | create | OciCloudTenantAdapter: impl CloudTenantStore; OciNamespaceProvisionerAdapter |
| `crates/oya-cloud-iam-kernel/Cargo.toml` | create | ServiceAccountStore + RoleBindingPort ports |
| `crates/oya-cloud-iam-kernel/src/lib.rs` | create | ServiceAccount + RoleBinding types |
| `crates/oya-cloud-iam-adapter/Cargo.toml` | create | Depends on iam-kernel + oya-secrets-kernel + oci-sdk + sqlx |
| `crates/oya-cloud-iam-adapter/src/lib.rs` | create | OciIamAdapter: impl ServiceAccountStore (credentials via SecretReference) |
| `crates/oya-cloud-kms-kernel/Cargo.toml` | create | CloudKmsPort port (wraps P08 KmsMasterKeyStore) |
| `crates/oya-cloud-kms-kernel/src/lib.rs` | create | PerCellDekManager type; CloudKmsPort sealed |
| `crates/oya-cloud-kms-adapter/Cargo.toml` | create | Depends on cloud-kms-kernel + oya-kms-kernel + oci-sdk + sqlx |
| `crates/oya-cloud-kms-adapter/src/lib.rs` | create | OciVaultKmsAdapter: impl CloudKmsPort; envelope encrypt via P08 KmsMasterKeyStore |
| `crates/oya-cloud-compute-kernel/Cargo.toml` | create | NodePoolManager + HpaConfigStore ports |
| `crates/oya-cloud-compute-kernel/src/lib.rs` | create | NodePoolSpec + HpaConfig types |
| `crates/oya-cloud-compute-adapter/Cargo.toml` | create | Depends on compute-kernel + oci-sdk; generates OKE manifests |
| `crates/oya-cloud-compute-adapter/src/lib.rs` | create | OkeNodePoolAdapter: generates HPA + VPA YAML manifests; ARM64 node pool shapes |
| `crates/oya-cloud-storage-kernel/Cargo.toml` | create | ObjectStoragePort port |
| `crates/oya-cloud-storage-kernel/src/lib.rs` | create | ObjectStoragePort; BucketId + ObjectKey + StorageClass types |
| `crates/oya-cloud-storage-adapter/Cargo.toml` | create | Depends on storage-kernel + oci-sdk + sqlx |
| `crates/oya-cloud-storage-adapter/src/lib.rs` | create | OciObjectStorageAdapter: impl ObjectStoragePort |
| `crates/oya-cloud-storage-sdk/Cargo.toml` | create | Depends on storage-kernel ONLY (sdk-kernel-only rule) |
| `crates/oya-cloud-storage-sdk/src/lib.rs` | create | CloudStorageClient: thin typed client; re-exports storage-kernel types; no adapter deps |
| `crates/oya-cloud-network-kernel/Cargo.toml` | create | VcnManager + SecurityListPort + SubnetPort ports |
| `crates/oya-cloud-network-kernel/src/lib.rs` | create | Vcn + Subnet + SecurityRule types |
| `crates/oya-cloud-network-adapter/Cargo.toml` | create | Depends on network-kernel + oci-sdk + sqlx |
| `crates/oya-cloud-network-adapter/src/lib.rs` | create | OciVcnAdapter: per-cell VCN + private subnet + security list |
| `crates/oya-cloud-billing-kernel/Cargo.toml` | create | UsageMeterPort port |
| `crates/oya-cloud-billing-kernel/src/lib.rs` | create | UsageEvent + UsageSummary types (billing skeleton; full engine M03) |
| `crates/oya-cloud-billing-adapter/Cargo.toml` | create | Depends on billing-kernel + sqlx |
| `crates/oya-cloud-billing-adapter/src/lib.rs` | create | PgUsageMeterAdapter: records usage events to cloud.usage_events |
| `crates/oya-cloud-cell-kernel/Cargo.toml` | create | CellLifecycleStore + CellHealthPort ports |
| `crates/oya-cloud-cell-kernel/src/lib.rs` | create | Cell + CellState + CellSpec types; CellState enum: Creating/Active/Draining/Decommissioned |
| `crates/oya-cloud-cell-application/Cargo.toml` | create | Depends on cell-kernel |
| `crates/oya-cloud-cell-application/src/lib.rs` | create | CreateCellUseCase, DrainCellUseCase, DecommissionCellUseCase, CellHealthCheckUseCase |
| `crates/oya-cloud-cell-adapter/Cargo.toml` | create | Depends on cell-application + kernel + oci-sdk + sqlx |
| `crates/oya-cloud-cell-adapter/src/lib.rs` | create | PgCellLifecycleStore + OciCellHealthAdapter |
| `crates/oya-cloud-app/Cargo.toml` | create | Composition root; depends on all cloud layers |
| `crates/oya-cloud-app/src/main.rs` | create | DI assembly for all 8 BCs |
| `contracts/cloud.openapi.yaml` | create | provisionCell, getCellStatus, provisionTenantNamespace, getStorageEndpoint |
| `migrations/cloud/V001__cloud_schema.sql` | create | Full DDL (see Code Shape) |
| `deploy/cloud/oci-arm64-node-pool.yaml` | create | OKE ARM64 node pool manifest (VM.Standard.A1.Flex; autoscaling) |
| `docs/standards/bounded-contexts.md` | update | Register 8 cloud BCs |

---

## Code Shape

### `migrations/cloud/V001__cloud_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS cloud;

CREATE TABLE cloud.cells (
    cell_id text PRIMARY KEY,           -- e.g., 'kr-1-cell-a'
    region text NOT NULL CHECK (region IN ('KR','US','EU','JP')),
    state text NOT NULL DEFAULT 'creating' CHECK (state IN (
        'creating','active','draining','decommissioned'
    )),
    oci_compartment_id text NULL,
    oci_vcn_id text NULL,
    max_tenants int NOT NULL DEFAULT 1000,
    current_tenant_count int NOT NULL DEFAULT 0,
    arm64_node_count int NOT NULL DEFAULT 2,   -- OCI A1 Always Free: 2 nodes
    created_at timestamptz NOT NULL DEFAULT now(),
    activated_at timestamptz NULL,
    decommissioned_at timestamptz NULL
);
CREATE INDEX idx_cells_active ON cloud.cells (region, state) WHERE state = 'active';
COMMENT ON TABLE cloud.cells IS 'No tenant_id column — cells are global infrastructure';

CREATE TABLE cloud.tenant_namespaces (
    namespace_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL UNIQUE,
    cell_id text NOT NULL REFERENCES cloud.cells(cell_id),
    oci_namespace text NOT NULL UNIQUE,
    provisioned_at timestamptz NOT NULL DEFAULT now(),
    deprovisioned_at timestamptz NULL
);
CREATE INDEX idx_tenant_namespaces_cell ON cloud.tenant_namespaces (cell_id)
    WHERE deprovisioned_at IS NULL;

CREATE TABLE cloud.service_accounts (
    sa_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    microservice text NOT NULL,
    oci_sa_ocid text NOT NULL,
    secret_ref_id uuid NOT NULL,    -- FK to secrets.refs(ref_id) via app layer
    created_at timestamptz NOT NULL DEFAULT now(),
    rotated_at timestamptz NULL
);
CREATE UNIQUE INDEX idx_sa_tenant_ms ON cloud.service_accounts (tenant_id, microservice);

CREATE TABLE cloud.cell_deks (
    dek_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cell_id text NOT NULL REFERENCES cloud.cells(cell_id),
    tenant_id uuid NOT NULL,
    encrypted_dek bytea NOT NULL,   -- envelope-encrypted with P08 KMS master key
    kms_key_id uuid NOT NULL,       -- FK to kms.key_versions
    created_at timestamptz NOT NULL DEFAULT now(),
    rotated_at timestamptz NULL,
    revoked_at timestamptz NULL
);
CREATE UNIQUE INDEX idx_cell_dek_active ON cloud.cell_deks (cell_id, tenant_id)
    WHERE revoked_at IS NULL;

CREATE TABLE cloud.storage_buckets (
    bucket_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    cell_id text NOT NULL REFERENCES cloud.cells(cell_id),
    oci_bucket_name text NOT NULL UNIQUE,
    storage_class text NOT NULL DEFAULT 'Standard' CHECK (storage_class IN (
        'Standard','InfrequentAccess','Archive'
    )),
    size_bytes bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_storage_buckets_tenant ON cloud.storage_buckets (tenant_id, cell_id);

CREATE TABLE cloud.usage_events (
    event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    microservice text NOT NULL,
    metric_name text NOT NULL,
    metric_value numeric(20,4) NOT NULL,
    recorded_at timestamptz NOT NULL DEFAULT now()
) PARTITION BY RANGE (recorded_at);
CREATE TABLE cloud.usage_events_2026_05 PARTITION OF cloud.usage_events
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');
CREATE INDEX idx_usage_events_tenant ON cloud.usage_events (tenant_id, microservice, recorded_at DESC);
```

---

## Acceptance Gates

```bash
cargo check --workspace --all-features                                    # exit 0
cargo build --workspace --all-features                                    # exit 0
cargo clippy --workspace --all-features -- -D warnings                    # exit 0
cargo nextest run --workspace --all-features                              # exit 0
cargo nextest run -p oya-cloud-cell-adapter --test arm64_manifest_valid   # exit 0
cargo deny check                                                          # exit 0
oya gate validate lean-a1 --phase P18-cloud-tenancy
oya gate validate lean-a2 --phase P18-cloud-tenancy
oya gate validate sdk-kernel-only --crate oya-cloud-storage-sdk          # exit 0
oya gate validate shardability --phase P18-cloud-tenancy
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_cell_state_transitions` | Creating→Active→Draining→Decommissioned; invalid transitions rejected |
| `test_storage_sdk_kernel_only_dep` | oya-cloud-storage-sdk Cargo.toml has no adapter/infrastructure deps |
| `test_namespace_for_tenant_unique` | Two tenants get distinct OCI namespaces |
| `test_dek_envelope_encrypt_decrypt` | CloudKmsAdapter wraps P08 KmsMasterKeyStore; round-trip succeeds |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_provision_tenant_namespace` | OciCloudTenantAdapter creates namespace row; cell_id FK valid |
| `integration_arm64_manifest_valid` | Generated OKE ARM64 manifest parses as valid Kubernetes YAML |
| `integration_cell_lifecycle` | Create→drain→decommission state machine; drain blocks new tenants |
| `integration_storage_bucket_provisioning` | OciObjectStorageAdapter creates bucket record |
| `integration_usage_meter_partition` | Usage event written to correct monthly partition |

---

## Load Test

| Scenario | Target | Pass criterion |
|---|---|---|
| Get cell status | p99 ≤50ms at 1k RPS | `http_req_duration{p(99)}<50` |
| Cloud storage upload (SDK) | p99 ≤200ms at 500 RPS | `http_req_duration{p(99)}<200` |
| Provision tenant namespace | p99 ≤500ms at 100 RPS | `http_req_duration{p(99)}<500` |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-cloud \
  --intent "IP-001-cloud-tenancy-kernel-scaffold: 8 cloud BCs + ARM64 manifests" \
  --ttl 3600 \
  crates/oya-cloud-tenancy-kernel/src/lib.rs::CloudTenantStore \
  crates/oya-cloud-cell-kernel/src/lib.rs::CellLifecycleStore \
  crates/oya-cloud-storage-sdk/src/lib.rs::CloudStorageClient \
  migrations/cloud/V001__cloud_schema.sql::cloud.cells
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-cloud-tenancy-kernel-scaffold merged; 8 cloud BCs; OCI ARM64 manifest; cell lifecycle; storage SDK verified sdk-kernel-only; next: IP-002-cloud-iam-kms-wiring" \
  -i high \
  -k "M02,P18,IP-001,cloud"
```

---

## Halt Conditions

1. `oya-cloud-storage-sdk` Cargo.toml imports an adapter/infrastructure crate — escalate; violates sdk-kernel-only rule.
2. Cell lifecycle state machine allows re-activation of decommissioned cell — escalate; irreversible state.
3. LEAN-A2: non-cloud product crate importing cloud adapter directly (not via SDK) — escalate.
4. DEK envelope encryption not using P08 KmsMasterKeyStore port — escalate; no parallel KMS path.

---

## Next IP Pointer

`IP-002-cloud-iam-kms-wiring.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0117 (cloud-native infra), ADR-0021 (OCI A1), ADR-0056 v4.1 (cloud dual-role + public_layers)
