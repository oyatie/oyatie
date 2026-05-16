---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P06-secrets
impl_plan_id: IP-P06-secrets-substrate
status: pending
owner: council-architecture
blocked_by: []
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
purpose: "Delivers the complete Secrets substrate: 12 crates across 2 BCs (refs, rotation), `secrets.refs` DDL storing only vault paths (never plaintext), OpenBao HTTP adapter (DAY-1 default), `ZeroizingSecret` type that zeroes memory on drop."
---
# IP-P06-secrets-substrate: Scaffold 12 secrets crates with SecretReferencePort, OpenBao adapter, rotation worker

## Intent

Delivers the complete Secrets substrate: 12 crates across 2 BCs (refs, rotation), `secrets.refs` DDL storing only vault paths (never plaintext), OpenBao HTTP adapter (DAY-1 default), `ZeroizingSecret` type that zeroes memory on drop, rotation worker, Cedar policy, load test p99≤200ms on cache-hit retrieval path.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-secrets-refs-kernel/Cargo.toml` | create | SecretReferencePort + SecretStore port traits; ZeroizingSecret type |
| `crates/oya-secrets-refs-kernel/src/types.rs` | create | SecretRefId, VaultPath, SecretMeta, ZeroizingSecret (zeroize crate), SecretVersion |
| `crates/oya-secrets-refs-kernel/src/ports.rs` | create | SecretReferencePort + SecretStore sealed traits |
| `crates/oya-secrets-rotation-kernel/Cargo.toml` | create | RotationSchedulePort trait |
| `crates/oya-secrets-rotation-kernel/src/ports.rs` | create | RotationSchedulePort sealed trait |
| `crates/oya-secrets-refs-domain/src/secret_ref.rs` | create | SecretRef value object; vault path validation; no plaintext fields |
| `crates/oya-secrets-refs-application/src/retrieve.rs` | create | RetrieveSecretUseCase: lookup ref → call SecretStore::read → return ZeroizingSecret |
| `crates/oya-secrets-refs-application/src/register.rs` | create | RegisterSecretUseCase: write to vault → store ref in Postgres |
| `crates/oya-secrets-refs-adapter/src/openbao.rs` | create | OpenBaoAdapter: HTTP client to OpenBao API; LRU DEK cache (ttl=5min) |
| `crates/oya-secrets-refs-adapter/src/postgres.rs` | create | PgSecretRefAdapter: CRUD on secrets.refs (paths only) |
| `crates/oya-secrets-rotation-domain/src/schedule.rs` | create | RotationSchedule value object; rotation_due_at computation |
| `crates/oya-secrets-rotation-application/src/rotate.rs` | create | RotateSecretUseCase: read old → write new version → update ref |
| `crates/oya-secrets-rotation-adapter/src/postgres.rs` | create | PgRotationScheduleStore |
| `crates/oya-secrets-worker/src/rotation_worker.rs` | create | RotationWorker: daily check; call RotateSecretUseCase for due refs |
| `crates/oya-secrets-rest/src/routes.rs` | create | POST /secrets/v1/refs, GET /secrets/v1/refs/{id}/retrieve (returns value, never stored), DELETE /secrets/v1/refs/{id} |
| `crates/oya-secrets-app/src/main.rs` | create | composition root |
| `migrations/secrets/V001__secrets_refs_init.sql` | create | DDL with strict no-plaintext enforcement |
| `contracts/secrets/secrets.proto` | create | Protobuf schema |
| `policy/secrets/secrets.cedar` | create | Cedar policy |
| `tests/load/smoke-secrets-retrieve.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 12 secrets crates |

---

## Crate Naming

```
NAME: oya-secrets-refs-kernel
JUSTIFICATION:
- microservice = secrets: secret reference management substrate; OpenBao adapter
- bc-tokens = refs: SecretReference BC — vault paths only, never plaintext values
- layer = kernel: SecretReferencePort + SecretStore port traits + ZeroizingSecret type
- exemptions claimed: none
```

---

## Code Shape

### `migrations/secrets/V001__secrets_refs_init.sql`

```sql
CREATE SCHEMA IF NOT EXISTS secrets;

-- Secret references: vault paths ONLY — no secret values ever stored in Postgres
-- This table is intentionally minimal. Adding any column named 'value', 'plaintext',
-- 'secret_data', 'secret_value', or 'key_material' is FORBIDDEN and will fail
-- the oya-check-shardability-cli secret-column-ban check.
CREATE TABLE secrets.refs (
    ref_id          uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid    NOT NULL,
    vault_path      text    NOT NULL,      -- e.g., "secret/data/t/{tenant_id}/db/primary"
    description     text    NOT NULL,
    secret_engine   text    NOT NULL DEFAULT 'openbao',   -- 'openbao' | 'hsm'
    version         int     NOT NULL DEFAULT 1,
    rotation_due_at timestamptz NULL,
    revoked_at      timestamptz NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE secrets.refs ENABLE ROW LEVEL SECURITY;
ALTER TABLE secrets.refs FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON secrets.refs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_secrets_refs_tenant
    ON secrets.refs (tenant_id, ref_id)
    WHERE revoked_at IS NULL;
CREATE INDEX idx_secrets_refs_rotation_due
    ON secrets.refs (tenant_id, rotation_due_at)
    WHERE revoked_at IS NULL AND rotation_due_at IS NOT NULL;

-- Verify no plaintext columns exist (enforced by oya-check-shardability-cli secret-column-ban)
-- The following trigger rejects any ALTER TABLE that adds a forbidden column name
CREATE OR REPLACE FUNCTION secrets.reject_plaintext_column()
RETURNS event_trigger AS $$
DECLARE
    obj record;
BEGIN
    FOR obj IN SELECT * FROM pg_event_trigger_ddl_commands() LOOP
        IF obj.command_tag = 'ALTER TABLE' AND
           EXISTS (
               SELECT 1 FROM information_schema.columns
               WHERE table_schema = 'secrets'
               AND column_name IN ('value','plaintext','secret_data','secret_value','key_material','raw_key')
           ) THEN
            RAISE EXCEPTION 'secrets schema: plaintext/value columns are forbidden per security policy';
        END IF;
    END LOOP;
END $$ LANGUAGE plpgsql;

CREATE EVENT TRIGGER secrets_no_plaintext_columns
    ON ddl_command_end
    WHEN TAG IN ('ALTER TABLE')
    EXECUTE FUNCTION secrets.reject_plaintext_column();

-- Rotation schedule
CREATE TABLE secrets.rotation_schedules (
    schedule_id         uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid    NOT NULL,
    ref_id              uuid    NOT NULL REFERENCES secrets.refs(ref_id),
    interval_days       int     NOT NULL,
    last_rotated_at     timestamptz NULL,
    next_rotation_at    timestamptz NOT NULL,
    created_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE secrets.rotation_schedules ENABLE ROW LEVEL SECURITY;
ALTER TABLE secrets.rotation_schedules FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON secrets.rotation_schedules
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
```

### `crates/oya-secrets-refs-kernel/src/types.rs`

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secret value held in memory. Zeroed when dropped.
/// NEVER persist this type. NEVER log this type.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ZeroizingSecret(Vec<u8>);

impl ZeroizingSecret {
    pub fn new(bytes: Vec<u8>) -> Self { Self(bytes) }
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

// Explicitly prevent Debug/Display printing of secret value
impl std::fmt::Debug for ZeroizingSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ZeroizingSecret([REDACTED])")
    }
}

impl std::fmt::Display for ZeroizingSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VaultPath(String);

impl VaultPath {
    /// Validate OpenBao path format: "secret/data/t/{tenant_id}/..." 
    pub fn new(path: impl Into<String>) -> Result<Self, SecretsError> {
        let p = path.into();
        if p.is_empty() || p.contains("..") || !p.starts_with("secret/") {
            return Err(SecretsError::InvalidVaultPath(p));
        }
        Ok(Self(p))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

### `crates/oya-secrets-refs-adapter/src/openbao.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use reqwest::Client;
use oya_secrets_refs_kernel::ports::SecretStore;
use oya_secrets_refs_kernel::types::{VaultPath, ZeroizingSecret, SecretVersion, TenantId};

struct CacheEntry { secret: Vec<u8>, inserted_at: Instant }

pub struct OpenBaoAdapter {
    http: Client,
    base_url: String,
    token: String,
    /// Per-tenant LRU cache. TTL = 5 minutes to avoid KMS rate limits.
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: Duration,
}

impl OpenBaoAdapter {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            http: Client::builder().timeout(Duration::from_secs(5)).build().unwrap(),
            base_url: base_url.into(),
            token: token.into(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 min cache TTL
        }
    }
    
    fn cache_key(tenant_id: TenantId, path: &VaultPath) -> String {
        format!("{}/{}", tenant_id, path.as_str())
    }
}

#[async_trait::async_trait]
impl SecretStore for OpenBaoAdapter {
    async fn write(&self, tenant_id: TenantId, path: &VaultPath, value: &ZeroizingSecret)
        -> Result<VaultPath, oya_secrets_refs_kernel::SecretsError>
    {
        let url = format!("{}/v1/{}", self.base_url, path.as_str());
        let body = serde_json::json!({ "data": { "value": base64::encode(value.as_bytes()) } });
        let resp = self.http.post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&body)
            .send().await
            .map_err(|e| oya_secrets_refs_kernel::SecretsError::VaultUnavailable(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(oya_secrets_refs_kernel::SecretsError::VaultWrite(resp.status().to_string()));
        }
        Ok(path.clone())
    }

    async fn read(&self, tenant_id: TenantId, path: &VaultPath)
        -> Result<ZeroizingSecret, oya_secrets_refs_kernel::SecretsError>
    {
        let key = Self::cache_key(tenant_id, path);
        // Cache hit path
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&key) {
                if entry.inserted_at.elapsed() < self.cache_ttl {
                    return Ok(ZeroizingSecret::new(entry.secret.clone()));
                }
            }
        }
        // Cache miss — fetch from OpenBao
        let url = format!("{}/v1/{}", self.base_url, path.as_str());
        let resp = self.http.get(&url)
            .header("X-Vault-Token", &self.token)
            .send().await
            .map_err(|e| oya_secrets_refs_kernel::SecretsError::VaultUnavailable(e.to_string()))?;
        let json: serde_json::Value = resp.json().await
            .map_err(|e| oya_secrets_refs_kernel::SecretsError::VaultRead(e.to_string()))?;
        let encoded = json["data"]["data"]["value"].as_str()
            .ok_or_else(|| oya_secrets_refs_kernel::SecretsError::VaultRead("missing value field".into()))?;
        let bytes = base64::decode(encoded)
            .map_err(|e| oya_secrets_refs_kernel::SecretsError::VaultRead(e.to_string()))?;
        // Populate cache
        let mut cache = self.cache.write().await;
        cache.insert(key, CacheEntry { secret: bytes.clone(), inserted_at: Instant::now() });
        Ok(ZeroizingSecret::new(bytes))
    }

    async fn rotate(&self, tenant_id: TenantId, path: &VaultPath)
        -> Result<SecretVersion, oya_secrets_refs_kernel::SecretsError>
    {
        // Invalidate cache for this path
        let key = Self::cache_key(tenant_id, path);
        self.cache.write().await.remove(&key);
        // OpenBao KV v2 rotate: POST /v1/{path} with new value generated by vault
        let url = format!("{}/v1/{}/rotate", self.base_url, path.as_str());
        let resp = self.http.post(&url)
            .header("X-Vault-Token", &self.token)
            .send().await
            .map_err(|e| oya_secrets_refs_kernel::SecretsError::VaultUnavailable(e.to_string()))?;
        let json: serde_json::Value = resp.json().await
            .map_err(|e| oya_secrets_refs_kernel::SecretsError::VaultRead(e.to_string()))?;
        let version = json["data"]["version"].as_u64().unwrap_or(1) as u32;
        Ok(SecretVersion(version))
    }
}
```

### `contracts/secrets/secrets.proto`

```proto
syntax = "proto3";
package oyatie.secrets.v1;

message SecretRegistered {
    string tenant_id  = 1;
    string ref_id     = 2;
    string vault_path = 3;   // path only; never the value
    int64  timestamp_ms = 4;
}

message SecretRotated {
    string tenant_id    = 1;
    string ref_id       = 2;
    int32  new_version  = 3;
    int64  timestamp_ms = 4;
}

message SecretRevoked {
    string tenant_id  = 1;
    string ref_id     = 2;
    int64  timestamp_ms = 3;
}
```

### `tests/load/smoke-secrets-retrieve.js`

```javascript
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 50, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<200'],  // cache-hit path ≤200ms
    http_req_failed: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8084';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';
const REF_ID = __ENV.REF_ID || '00000000-0000-0000-0000-000000000002';

export default function () {
  // Retrieve secret by ref (cache-hit path after first request)
  const res = http.get(`${BASE_URL}/secrets/v1/refs/${REF_ID}/retrieve`, {
    headers: { 'X-Tenant-Id': TENANT_ID, 'Authorization': `Bearer ${__ENV.TEST_TOKEN}` },
  });
  check(res, { 'retrieve 200': (r) => r.status === 200 });
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-secrets-refs-kernel --all-features     # exit 0
cargo check -p oya-secrets-refs-adapter --all-features    # exit 0
cargo clippy --workspace --all-features -- -D warnings     # exit 0
cargo nextest run --workspace --all-features               # exit 0
psql $DATABASE_URL -f migrations/secrets/V001__secrets_refs_init.sql  # exit 0
# Verify no plaintext column exists
psql $DATABASE_URL -c "\d secrets.refs" | grep -E "value|plaintext|secret_data" && exit 1 || exit 0
# OpenBao round-trip (requires openbao running on localhost:8200 in dev mode)
cargo nextest run -p oya-secrets-refs-adapter --test openbao_round_trip  # exit 0
# Rotation lifecycle
cargo nextest run -p oya-secrets-rotation-application --test rotation_lifecycle  # exit 0
# Load test
k6 run tests/load/smoke-secrets-retrieve.js --env BASE_URL=http://localhost:8084
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_zeroing_secret_memory_zeroed_on_drop` | ZeroizingSecret bytes are zeroed after drop |
| `test_zeroing_secret_debug_redacted` | Debug/Display output is `[REDACTED]` |
| `test_vault_path_rejects_dotdot` | `..` in path → InvalidVaultPath error |
| `test_vault_path_must_start_secret` | Non-`secret/` prefix rejected |
| `test_cache_hit_returns_without_http` | Second retrieve within TTL → no HTTP call |
| `test_cache_invalidated_on_rotate` | After rotate, cache entry removed |
| `test_rotation_due_computed_correctly` | schedule interval=30d → next_rotation_at = last + 30d |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_openbao_write_read_round_trip` | Write secret → read back → bytes match |
| `integration_openbao_rotate_new_version` | Rotate → version increments → read returns new value |
| `integration_zero_secret_in_postgres` | After register, confirm no secret bytes in secrets.refs |
| `integration_rls_cross_tenant_blocked` | Tenant A cannot read tenant B refs |

---

## Clean Architecture Compliance

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-secrets-refs-kernel` | `kernel` | `zeroize` (external) | all project layers |
| `oya-secrets-refs-domain` | `domain` | `refs-kernel` | `adapter`, presentation |
| `oya-secrets-refs-application` | `application` | `refs-domain`, `refs-kernel` | `adapter`, presentation |
| `oya-secrets-refs-adapter` | `adapter` | `refs-application`, `refs-kernel`; `reqwest` (external) | presentation |
| `oya-secrets-worker` | `worker` | `*-application`, `*-kernel` | direct adapter |
| `oya-secrets-app` | `app` | all | none |

---

## Load Test

```bash
k6 run tests/load/smoke-secrets-retrieve.js --env BASE_URL=http://localhost:8084
# Pass: p99 ≤200ms (cache-hit); 0 errors at 50 VUs/60s
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P06-secrets: 12 crates + OpenBao adapter + ZeroizingSecret + rotation" \
  --ttl 7200 \
  crates/oya-secrets-refs-kernel/src/ports.rs::SecretReferencePort \
  crates/oya-secrets-refs-adapter/src/openbao.rs::OpenBaoAdapter \
  migrations/secrets/V001__secrets_refs_init.sql::secrets_schema
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P06-secrets merged; 12 crates; OpenBao DAY-1; ZeroizingSecret zeroes on drop; zero-secret-in-postgres invariant enforced; rotation worker; next: P07-observability/impl-plan" \
  -i high \
  -k "M02,P06,IP-P06,secrets"
```

---

## Next IP Pointer

`phases/P07-observability/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Bominal ADR-0043 (secrets / OpenBao adapter pattern)
- `zeroize` crate: https://docs.rs/zeroize
