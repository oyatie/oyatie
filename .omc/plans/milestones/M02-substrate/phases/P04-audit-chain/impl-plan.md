---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P04-audit-chain
impl_plan_id: IP-P04-audit-chain-substrate
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
purpose: Auto-backfilled purpose for impl-plan.md
---
# IP-P04-audit-chain-substrate: Scaffold 16 audit-chain crates with Merkle/Ed25519 sealer, append-only DDL, KMS bridge

## Intent

Delivers the complete Audit-chain substrate: 16 crates across 3 BCs (events, segments, signing), full append-only Postgres DDL with UPDATE/DELETE denial triggers, Merkle tree builder, Ed25519 segment sealer worker, KMS-bridge port for production signing, Cedar policy, Protobuf event schema, k6 load test with <1s seal latency verified.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-audit-chain-events-kernel/Cargo.toml` | create | AuditEventStore port + AuditEvent types |
| `crates/oya-audit-chain-events-kernel/src/types.rs` | create | EventId, TenantId, AuditEvent, AuditEventDraft, SegmentId |
| `crates/oya-audit-chain-events-kernel/src/ports.rs` | create | AuditEventStore sealed trait |
| `crates/oya-audit-chain-segments-kernel/Cargo.toml` | create | AuditSegmentSealer + MerkleTreeBuilder |
| `crates/oya-audit-chain-segments-kernel/src/ports.rs` | create | AuditSegmentSealer + MerkleTreeBuilder sealed traits |
| `crates/oya-audit-chain-signing-kernel/Cargo.toml` | create | ChainSigner trait |
| `crates/oya-audit-chain-signing-kernel/src/ports.rs` | create | ChainSigner sealed trait; Ed25519Signature type |
| `crates/oya-audit-chain-events-domain/src/event.rs` | create | AuditEvent invariants; canonical JSON serialization |
| `crates/oya-audit-chain-segments-domain/src/merkle.rs` | create | SHA-256 Merkle tree; deterministic leaf ordering; proof paths |
| `crates/oya-audit-chain-segments-domain/src/sealer.rs` | create | seal_period pure logic: fetch events → build root → call ChainSigner |
| `crates/oya-audit-chain-signing-domain/src/preimage.rs` | create | canonical preimage: tenant_id ∥ merkle_root ∥ period_date ∥ prev_segment_root |
| `crates/oya-audit-chain-events-application/src/append.rs` | create | AppendEventUseCase: validate, store, stage outbox |
| `crates/oya-audit-chain-segments-application/src/seal.rs` | create | SealPeriodUseCase: orchestrate fetch→merkle→sign→persist |
| `crates/oya-audit-chain-signing-application/src/rotate.rs` | create | RotateSigningKeyUseCase |
| `crates/oya-audit-chain-events-adapter/src/postgres.rs` | create | PgAuditEventStore: append-only inserts; SELECT for seal |
| `crates/oya-audit-chain-segments-adapter/src/postgres.rs` | create | PgSegmentStore: insert sealed segment |
| `crates/oya-audit-chain-signing-adapter/src/ed25519.rs` | create | Ed25519SignerAdapter: dev mode (ring crate); production delegates to oya-kms-signing |
| `crates/oya-audit-chain-worker/src/sealer.rs` | create | SegmentSealerWorker: hourly tokio interval; calls SealPeriodUseCase for all active tenants |
| `crates/oya-audit-chain-rest/src/routes.rs` | create | GET /audit-chain/v1/segments/{id}, GET /audit-chain/v1/events (paginated) |
| `crates/oya-audit-chain-app/src/main.rs` | create | composition root |
| `migrations/audit_chain/V001__audit_chain_init.sql` | create | full DDL (see below) |
| `contracts/audit_chain/audit_chain.proto` | create | Protobuf schema |
| `policy/audit_chain/audit_chain.cedar` | create | Cedar policy |
| `tests/load/smoke-audit-chain-append.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 16 audit-chain crates |

---

## Crate Naming

```
NAME: oya-audit-chain-events-kernel
JUSTIFICATION:
- microservice = audit-chain: cryptographic audit ledger; Bominal ADR-0028
- bc-tokens = events: append-only audit event BC
- layer = kernel: AuditEventStore port trait + entity types
- exemptions claimed: none

NAME: oya-audit-chain-worker
JUSTIFICATION:
- microservice = audit-chain: same µservice
- bc-tokens = (none): single sealer worker; ADR-0056 BC-optionality
- layer = worker: periodic Tokio task; hourly segment sealer
- exemptions claimed: none
```

---

## Code Shape

### `migrations/audit_chain/V001__audit_chain_init.sql`

```sql
CREATE SCHEMA IF NOT EXISTS audit_chain;

-- Append-only audit events; partitioned by month
CREATE TABLE audit_chain.audit_events (
    event_id         uuid        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    period_date      date        NOT NULL,
    event_type       text        NOT NULL,
    payload          jsonb       NOT NULL,
    payload_hash     bytea       NOT NULL,   -- SHA-256(canonical_json(payload))
    prev_event_id    uuid        NULL,
    occurred_at      timestamptz NOT NULL DEFAULT now(),
    sealed_segment_id uuid       NULL        -- FK to audit_segments after seal
) PARTITION BY RANGE (occurred_at);

-- Monthly partitions (create 12 months ahead via worker)
CREATE TABLE audit_chain.audit_events_2026_05
    PARTITION OF audit_chain.audit_events
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

ALTER TABLE audit_chain.audit_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_chain.audit_events FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON audit_chain.audit_events
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_audit_events_period
    ON audit_chain.audit_events (tenant_id, period_date, occurred_at)
    WHERE sealed_segment_id IS NULL;

-- Append-only enforcement triggers
CREATE OR REPLACE FUNCTION audit_chain.deny_modification()
RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'audit_chain.audit_events is append-only: updates and deletes are forbidden';
END $$ LANGUAGE plpgsql;

CREATE TRIGGER no_update
    BEFORE UPDATE ON audit_chain.audit_events
    FOR EACH ROW EXECUTE FUNCTION audit_chain.deny_modification();
CREATE TRIGGER no_delete
    BEFORE DELETE ON audit_chain.audit_events
    FOR EACH ROW EXECUTE FUNCTION audit_chain.deny_modification();

-- Sealed Merkle segments
CREATE TABLE audit_chain.audit_segments (
    segment_id       uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    period_date      date        NOT NULL,
    event_count      int         NOT NULL,
    merkle_root      bytea       NOT NULL,   -- SHA-256 Merkle root
    signature        bytea       NOT NULL,   -- Ed25519 over canonical preimage
    sig_algorithm    text        NOT NULL DEFAULT 'Ed25519',
    prev_segment_root bytea      NULL,
    signing_key_id   uuid        NOT NULL,
    sealed_at        timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX idx_segments_per_period
    ON audit_chain.audit_segments (tenant_id, period_date);

-- Signing keys (references only; actual key material in KMS/OpenBao)
CREATE TABLE audit_chain.signing_keys (
    key_id       uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    uuid    NOT NULL,
    public_key   bytea   NOT NULL,   -- Ed25519 public key (32 bytes)
    kms_key_ref  text    NULL,       -- KMS ARN / OpenBao path for private key
    algorithm    text    NOT NULL DEFAULT 'Ed25519',
    created_at   timestamptz NOT NULL DEFAULT now(),
    rotated_at   timestamptz NULL,
    revoked_at   timestamptz NULL
);
CREATE INDEX idx_signing_keys_active
    ON audit_chain.signing_keys (tenant_id, created_at DESC)
    WHERE revoked_at IS NULL;

-- Audit chain outbox
CREATE TABLE audit_chain.outbox (
    outbox_id    uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    uuid        NOT NULL,
    topic        text        NOT NULL,
    key          text        NOT NULL,
    payload      jsonb       NOT NULL,
    published_at timestamptz NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_audit_chain_outbox_unpublished
    ON audit_chain.outbox (created_at) WHERE published_at IS NULL;
```

### `crates/oya-audit-chain-segments-domain/src/merkle.rs`

```rust
use sha2::{Digest, Sha256};

pub type Sha256Hash = [u8; 32];

/// Deterministic Merkle tree. Leaves MUST be sorted by event_id (UUID lexicographic)
/// before calling build_root to ensure determinism across nodes.
pub struct MerkleTree {
    leaves: Vec<Sha256Hash>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<Sha256Hash>) -> Self {
        assert!(!leaves.is_empty(), "MerkleTree requires at least one leaf");
        Self { leaves }
    }

    pub fn build_root(&self) -> Sha256Hash {
        let mut current = self.leaves.clone();
        while current.len() > 1 {
            current = current
                .chunks(2)
                .map(|pair| {
                    let mut h = Sha256::new();
                    h.update(pair[0]);
                    h.update(pair.get(1).unwrap_or(&pair[0])); // duplicate last if odd count
                    h.finalize().into()
                })
                .collect();
        }
        current[0]
    }

    /// Returns sibling hashes from leaf to root (proof of inclusion).
    pub fn proof_path(&self, index: usize) -> Vec<Sha256Hash> {
        let mut proof = Vec::new();
        let mut current = self.leaves.clone();
        let mut idx = index;
        while current.len() > 1 {
            let sibling = if idx % 2 == 0 {
                current.get(idx + 1).copied().unwrap_or(current[idx])
            } else {
                current[idx - 1]
            };
            proof.push(sibling);
            current = current
                .chunks(2)
                .map(|pair| {
                    let mut h = Sha256::new();
                    h.update(pair[0]);
                    h.update(pair.get(1).unwrap_or(&pair[0]));
                    h.finalize().into()
                })
                .collect();
            idx /= 2;
        }
        proof
    }
}

/// Canonical preimage per Bominal ADR-0028:
/// tenant_id_bytes(16) ∥ merkle_root(32) ∥ period_date_bytes(4 = days since epoch) ∥ prev_segment_root(32, or zeros if first)
pub fn canonical_preimage(
    tenant_id: uuid::Uuid,
    merkle_root: &Sha256Hash,
    period_date: chrono::NaiveDate,
    prev_segment_root: Option<&Sha256Hash>,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(84);
    buf.extend_from_slice(tenant_id.as_bytes());
    buf.extend_from_slice(merkle_root);
    let days = period_date.num_days_from_ce();
    buf.extend_from_slice(&days.to_be_bytes());
    match prev_segment_root {
        Some(r) => buf.extend_from_slice(r),
        None => buf.extend_from_slice(&[0u8; 32]),
    }
    buf
}
```

### `crates/oya-audit-chain-worker/src/sealer.rs`

```rust
use std::sync::Arc;
use tokio::time::{interval, Duration};
use oya_audit_chain_segments_application::SealPeriodUseCase;
use oya_tenancy_kernel::ports::TenantStore;

pub struct SegmentSealerWorker<S, T> {
    seal_use_case: Arc<SealPeriodUseCase<S>>,
    tenant_store: Arc<T>,
    interval_secs: u64,
}

impl<S, T> SegmentSealerWorker<S, T>
where
    S: oya_audit_chain_segments_kernel::ports::AuditSegmentSealer,
    T: TenantStore,
{
    pub fn new(seal_use_case: Arc<SealPeriodUseCase<S>>, tenant_store: Arc<T>, interval_secs: u64) -> Self {
        Self { seal_use_case, tenant_store, interval_secs }
    }

    pub async fn run(self) {
        let mut ticker = interval(Duration::from_secs(self.interval_secs));
        loop {
            ticker.tick().await;
            let today = chrono::Utc::now().date_naive();
            let yesterday = today.pred_opt().unwrap_or(today);
            match self.tenant_store.list_active().await {
                Ok(tenants) => {
                    for tenant in tenants {
                        let result = self.seal_use_case.seal_period(tenant.id, yesterday).await;
                        if let Err(e) = result {
                            tracing::error!(tenant_id=%tenant.id, period=%yesterday, error=%e, "segment seal failed");
                        }
                    }
                }
                Err(e) => tracing::error!(error=%e, "failed to list active tenants for seal"),
            }
        }
    }
}
```

### `contracts/audit_chain/audit_chain.proto`

```proto
syntax = "proto3";
package oyatie.audit_chain.v1;

message AuditEventAppended {
    string tenant_id    = 1;
    string event_id     = 2;
    string event_type   = 3;
    bytes  payload_hash = 4;
    string period_date  = 5;   // ISO 8601 date
    int64  timestamp_ms = 6;
}

message AuditSegmentSealed {
    string tenant_id        = 1;
    string segment_id       = 2;
    string period_date      = 3;
    int32  event_count      = 4;
    bytes  merkle_root      = 5;
    bytes  signature        = 6;
    string signing_key_id   = 7;
    int64  sealed_at_ms     = 8;
}
```

### `tests/load/smoke-audit-chain-append.js`

```javascript
import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

export const options = {
  vus: 100, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<200'],
    http_req_failed: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8082';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

export default function () {
  const res = http.post(`${BASE_URL}/audit-chain/v1/events`, JSON.stringify({
    event_type: 'test.load_test_event',
    payload: { test_id: uuidv4() },
  }), {
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-Id': TENANT_ID,
      'Authorization': `Bearer ${__ENV.TEST_TOKEN}`,
    },
  });
  check(res, { 'append 201': (r) => r.status === 201 });
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-audit-chain-events-kernel --all-features   # exit 0
cargo check -p oya-audit-chain-segments-domain --all-features  # exit 0
cargo clippy --workspace --all-features -- -D warnings         # exit 0
cargo nextest run --workspace --all-features                   # exit 0
psql $DATABASE_URL -f migrations/audit_chain/V001__audit_chain_init.sql  # exit 0
# Append-only trigger test
psql $DATABASE_URL -c "UPDATE audit_chain.audit_events SET payload='{}' WHERE false;" 2>&1 | grep "append-only"
# Merkle determinism
cargo nextest run -p oya-audit-chain-segments-domain --test merkle_determinism  # exit 0
# Ed25519 round-trip
cargo nextest run -p oya-audit-chain-signing-domain --test ed25519_round_trip   # exit 0
# Seal latency <1s
cargo nextest run -p oya-audit-chain-segments-application --test seal_latency   # exit 0
# Load test
k6 run tests/load/smoke-audit-chain-append.js --env BASE_URL=http://localhost:8082
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_merkle_root_deterministic` | Same leaves in same order → same root |
| `test_merkle_single_leaf` | Single event seals correctly |
| `test_merkle_odd_leaf_count` | Odd leaf count duplicates last leaf |
| `test_merkle_proof_path` | Proof path verifies inclusion |
| `test_canonical_preimage_no_prev` | First segment: prev_root = zeros |
| `test_ed25519_sign_verify_round_trip` | Sign preimage → verify passes |
| `test_ed25519_tampered_sig_rejected` | Modified signature fails verification |
| `test_append_only_trigger` | UPDATE on audit_events raises exception |
| `test_seal_latency_under_1s` | seal_period with 10k events completes <1000ms |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_append_seal_verify` | Append 100 events → seal period → verify segment |
| `integration_chain_continuity` | Segment N prev_root == Segment N-1 merkle_root |
| `integration_rls_cross_tenant` | Tenant A cannot read tenant B events |

---

## Clean Architecture Compliance

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-audit-chain-events-kernel` | `kernel` | nothing project-internal | all layers |
| `oya-audit-chain-segments-domain` | `domain` | `segments-kernel`, `events-kernel` | `adapter`, presentation |
| `oya-audit-chain-segments-application` | `application` | `segments-domain`, `*-kernel` | `adapter`, presentation |
| `oya-audit-chain-signing-adapter` | `adapter` | `signing-application`, `signing-kernel` | presentation |
| `oya-audit-chain-worker` | `worker` | `*-application`, `*-kernel` | direct adapter |
| `oya-audit-chain-app` | `app` | all | none |

---

## Load Test

```bash
k6 run tests/load/smoke-audit-chain-append.js --env BASE_URL=http://localhost:8082
# Pass: p99 ≤200ms append; 0 errors at 100 VUs/60s

# Seal latency measurement
cargo nextest run -p oya-audit-chain-segments-application --test seal_latency -- --nocapture
# Pass: "seal completed in Xms" where X < 1000
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P04-audit-chain: 16 crates + Merkle + Ed25519 + sealer worker" \
  --ttl 7200 \
  crates/oya-audit-chain-events-kernel/src/ports.rs::AuditEventStore \
  crates/oya-audit-chain-segments-domain/src/merkle.rs::MerkleTree \
  crates/oya-audit-chain-worker/src/sealer.rs::SegmentSealerWorker \
  migrations/audit_chain/V001__audit_chain_init.sql::audit_chain_schema
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P04-audit-chain merged; 16 crates; append-only triggers; Merkle/Ed25519; seal <1s; ADR-0028 compliant; next: P05-eventing/impl-plan" \
  -i high \
  -k "M02,P04,IP-P04,audit-chain"
```

---

## Next IP Pointer

`phases/P05-eventing/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Schema foundation: `.omc/plans/M02-substrate-schema-foundation.md §5`
- Bominal ADR-0028 (Merkle/Ed25519 audit chain)
