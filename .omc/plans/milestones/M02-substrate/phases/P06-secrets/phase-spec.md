---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P06-secrets
status: Proposed
entry_gate: |
  M01-P05 complete; oya-tenancy-kernel ships; OpenBao reachable in dev
  (docker: openbao/openbao); cargo check exits 0.
exit_gate: |
  All secrets crates compile; secrets.refs table verified (NO plaintext
  values stored in Postgres — only references); OpenBao adapter retrieves
  secret by reference in integration test; rotation workflow compiles; Cedar
  policy lints; grit done; ICM row emitted.
depends_on:
  - milestone: M01
    phase: P05-scaffold-locks
    reason: "workspace scaffold prerequisite"
owner_team: council-architecture
---

# P06-secrets: Secrets substrate — SecretReference port, OpenBao adapter (DAY-1 DEFAULT), HSM-per-cell production path

## Purpose

This phase delivers the complete Secrets substrate. The design principle is absolute: raw secrets NEVER enter Postgres. The `secrets.refs` table stores only opaque references (vault paths); actual secret values live exclusively in OpenBao (day-1 default per user preference) or in the per-cell HSM for production. The `SecretReferencePort` trait in kernel decouples all product code from the vault implementation — callers request a secret by reference ID and receive the decrypted value from the adapter without knowing whether the backing store is OpenBao dev, OpenBao HA, or HSM. This enforces zero-secret-in-database from day one and satisfies PCI DSS §3.4 + PIPA Article 29 storage encryption requirements without coupling products to vault internals.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `secrets` | `refs`, `rotation` | `crates/oya-secrets-{refs,rotation}-{kernel,domain,application,adapter}/`, `crates/oya-secrets-worker/`, `crates/oya-secrets-rest/`, `crates/oya-secrets-app/` | 2×4 + 1 worker + 1 rest + 1 app = 12 crates |

Naming justification:

```
NAME: oya-secrets-refs-kernel
JUSTIFICATION:
- microservice = secrets: secret reference management substrate
- bc-tokens = refs: the SecretReference BC — stores only opaque vault paths,
  never plaintext values; distinct from rotation (key/secret rotation lifecycle)
- layer = kernel: SecretReferencePort trait + SecretRef, SecretId types; zero I/O
- exemptions claimed: none

NAME: oya-secrets-rotation-kernel
JUSTIFICATION:
- microservice = secrets: same µservice
- bc-tokens = rotation: the rotation lifecycle BC — tracks version history,
  triggers rotation workers, notifies consumers via Workflow events
- layer = kernel: RotationSchedulePort + RotationRecord types
- exemptions claimed: none
```

### Out-of-scope

- KMS envelope encryption for data-at-rest field encryption — owned by P08-kms.
- HSM hardware provisioning — infra concern owned by oya-cloud phase.
- Product-specific secret types (DB passwords, API keys) — products register
  references; this phase owns the reference store and retrieval contract.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL (refs only) + SecretReferencePort + OpenBao adapter + rotation worker + Cedar + load test | pending | `council-architecture` |

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
oya gate validate lean-a1 --phase P06-secrets
oya gate validate lean-a2 --phase P06-secrets
oya gate validate lean-a3 --phase P06-secrets
oya gate validate lean-a4 --phase P06-secrets
```

### Zero-secret-in-database gate (critical invariant)

```bash
# Verify no plaintext value column exists in secrets schema
psql $DATABASE_URL -c "\d secrets.refs" | grep -v "value\|plaintext\|secret_data"
# Must: no rows containing actual secret material in Postgres at any time

# OpenBao round-trip integration test
cargo nextest run -p oya-secrets-refs-adapter --test openbao_round_trip   # exit 0
# Verify rotation increments version
cargo nextest run -p oya-secrets-rotation-application --test rotation_lifecycle  # exit 0
```

### Load test gate

```bash
k6 run tests/load/smoke-secrets-retrieve.js --env BASE_URL=http://localhost:8084
# Pass: p99 ≤200ms on secret retrieval (OpenBao LRU cache hit path); 0 errors
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-secrets-refs-kernel` | `kernel` | Yes — `SecretReferencePort`, `SecretStore` | N/A | No |
| `oya-secrets-rotation-kernel` | `kernel` | Yes — `RotationSchedulePort` | N/A | No |
| `oya-secrets-refs-domain` | `domain` | N/A — SecretRef value object, invariants | N/A | No |
| `oya-secrets-refs-adapter` | `adapter` | N/A | Yes — `OpenBaoAdapter`, `PgSecretRefAdapter` | No |
| `oya-secrets-worker` | `worker` | N/A | No direct adapter | No |
| `oya-secrets-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-secrets-refs-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait SecretReferencePort: Send + Sync + sealed::Sealed {
    /// Retrieve the decrypted secret value by reference ID.
    /// Value is NEVER persisted; returned only in memory, zeroed after use.
    async fn retrieve(&self, tenant_id: TenantId, ref_id: SecretRefId)
        -> Result<ZeroizingSecret, SecretsError>;
    /// Register a new secret reference; vault path stored in Postgres, value in vault.
    async fn register(&self, tenant_id: TenantId, vault_path: VaultPath, metadata: SecretMeta)
        -> Result<SecretRefId, SecretsError>;
    /// Soft-delete a reference; secret remains in vault until vault TTL expires.
    async fn revoke(&self, tenant_id: TenantId, ref_id: SecretRefId) -> Result<(), SecretsError>;
}

#[async_trait::async_trait]
pub trait SecretStore: Send + Sync + sealed::Sealed {
    /// Write a secret to the vault backend (OpenBao / HSM). Returns vault path.
    async fn write(&self, tenant_id: TenantId, path: &VaultPath, value: &ZeroizingSecret)
        -> Result<VaultPath, SecretsError>;
    /// Read from vault. Value zeroed after caller drops ZeroizingSecret.
    async fn read(&self, tenant_id: TenantId, path: &VaultPath)
        -> Result<ZeroizingSecret, SecretsError>;
    async fn rotate(&self, tenant_id: TenantId, path: &VaultPath)
        -> Result<SecretVersion, SecretsError>;
}

// oya-secrets-rotation-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait RotationSchedulePort: Send + Sync + sealed::Sealed {
    async fn schedule(&self, tenant_id: TenantId, ref_id: SecretRefId,
        interval_days: u32) -> Result<(), SecretsError>;
    async fn due_for_rotation(&self, tenant_id: TenantId)
        -> Result<Vec<SecretRefId>, SecretsError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P06-secrets` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P06-secrets` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P06-secrets` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `refs` | `secrets` | pending |
| `rotation` | `secrets` | pending |

---

## Grit Claim Symbols

```
crates/oya-secrets-refs-kernel/src/ports.rs::SecretReferencePort
crates/oya-secrets-refs-kernel/src/ports.rs::SecretStore
crates/oya-secrets-rotation-kernel/src/ports.rs::RotationSchedulePort
crates/oya-secrets-refs-adapter/src/openbao.rs::OpenBaoAdapter
crates/oya-secrets-worker/src/rotation_worker.rs::RotationWorker
migrations/secrets/V001__secrets_refs_init.sql::secrets_schema
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P06-secrets started; scope: 12 crates (refs/rotation BCs); OpenBao DAY-1 DEFAULT per user preference; zero-secret-in-database invariant enforced at DDL level" \
  -i high \
  -k "M02,P06,phase-start,secrets"

icm store \
  -t context-oyatie \
  -c "Phase P06-secrets complete; secrets.refs DDL verified (no plaintext column); OpenBao adapter integration test green; rotation lifecycle tested; next: P07-observability" \
  -i high \
  -k "M02,P06,phase-complete,secrets"
```

---

## References

- Bominal ADRs inherited: ADR-0043 (OpenBao/secrets adapter — if present; else inferred from stack)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05
- unblocks: P08-kms (KMS uses Secrets for key material), all product phases with sensitive fields
