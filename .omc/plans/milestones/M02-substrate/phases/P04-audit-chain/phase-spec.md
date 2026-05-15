---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P04-audit-chain
status: Proposed
acceptance_lanes: []
entry_gate: "M01-P05 complete; oya-tenancy-kernel ships (TenantId available);\noya-kms-kernel\
  \ ships (KeyId, ChainSigner port available \u2014 co-deployed\nwith P08 or stubbed\
  \ via in-process Ed25519 for dev); Postgres 16 available.\n"
exit_gate: 'All audit-chain crates compile; append-only triggers verified on

  audit_chain.audit_events; Merkle/Ed25519 segment sealer worker runs

  end-to-end for a test tenant+period; seal latency <1s measured;

  Cedar policy lints; Protobuf compiles; grit done; ICM row emitted.

  '
depends_on:
- milestone: M01
  phase: P05-scaffold-locks
  reason: workspace scaffold prerequisite
owner_team: council-architecture
purpose: Auto-backfilled purpose for phase-spec.md
---
# P04-audit-chain: Full Audit-chain substrate — Merkle/Ed25519 segment sealer, append-only events, KMS-backed signing keys

## Purpose

This phase delivers the complete Audit-chain substrate: an append-only, cryptographically verifiable ledger of all state-changing events across every product. Per Bominal ADR-0028, each segment covers one `(tenant_id, period_date)` pair, folds event payload SHA-256 hashes into a Merkle root, and signs the segment with Ed25519 over the canonical preimage `tenant_id || merkle_root || period_date || prev_segment_root`. The worker seals segments on a configurable schedule (default: hourly) with <1s seal latency per the quality bar. KMS-backed signing keys replace the dev seed signer in production. Without Audit-chain every product event is ephemeral; this phase establishes tamper-evident evidence from day one, advancing the "audit-first" Master Plan principle.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `audit-chain` | `events`, `segments`, `signing` | `crates/oya-audit-chain-{events,segments,signing}-{kernel,domain,application,adapter}/`, `crates/oya-audit-chain-worker/`, `crates/oya-audit-chain-rest/`, `crates/oya-audit-chain-app/` | `oya-audit-chain-events-kernel`, `oya-audit-chain-segments-kernel`, `oya-audit-chain-signing-kernel`, … (3×4 + 1 worker + 1 rest + 1 app = 16 crates) |

Naming justification:

```
NAME: oya-audit-chain-events-kernel
JUSTIFICATION:
- microservice = audit-chain: the cryptographic audit ledger substrate;
  registered in [workspace.metadata.oya.microservices]; ADR-0056 flat BNF
- bc-tokens = events: the append-only audit event BC; distinct from
  segments (sealed Merkle segments) and signing (key management)
- layer = kernel: AuditEventStore port trait + AuditEvent entity types;
  zero I/O; ADR-0056 §"Layer semantics"
- exemptions claimed: none

NAME: oya-audit-chain-worker
JUSTIFICATION:
- microservice = audit-chain: same µservice
- bc-tokens = (none): single worker binary across all BCs; segment sealer
  runs as a single Tokio task set; ADR-0056 BC-optionality rule
- layer = worker: long-running background worker; Tokio JoinSet; periodic
  sealer + outbox dispatcher; ADR-0056 §"Layer semantics"
- exemptions claimed: none
```

### Out-of-scope

- Per-product event schema definitions (hr.employee_hired, payroll.run_closed etc.) — registered by product phases.
- OSCAL-shaped compliance report generation — deferred to M03 compliance phase.
- Cross-region segment replication — deferred to M03.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + append-only triggers + Merkle + Ed25519 + worker + Cedar + Proto | pending | `council-architecture` |

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
oya gate validate lean-a1 --phase P04-audit-chain
oya gate validate lean-a2 --phase P04-audit-chain
oya gate validate lean-a3 --phase P04-audit-chain
oya gate validate lean-a4 --phase P04-audit-chain
```

### Cryptographic correctness gates

```bash
# Append-only triggers
psql $DATABASE_URL -c "UPDATE audit_chain.audit_events SET payload = '{}' WHERE false;"
# Must raise: "audit_events is append-only"

# Merkle root determinism test
cargo nextest run -p oya-audit-chain-segments-domain --test merkle_determinism  # exit 0

# Ed25519 sign → verify round-trip
cargo nextest run -p oya-audit-chain-signing-domain --test ed25519_round_trip   # exit 0

# Seal latency <1s (Bominal ADR-0028 requirement)
cargo nextest run -p oya-audit-chain-segments-application --test seal_latency   # exit 0; asserts <1000ms
```

### Load test gate

```bash
k6 run tests/load/smoke-audit-chain-append.js --env BASE_URL=http://localhost:8082
# Pass: p99 ≤200ms on event append; 0 errors at 5k events/s
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-audit-chain-events-kernel` | `kernel` | Yes — `AuditEventStore` | N/A | No |
| `oya-audit-chain-segments-kernel` | `kernel` | Yes — `AuditSegmentSealer`, `MerkleTreeBuilder` | N/A | No |
| `oya-audit-chain-signing-kernel` | `kernel` | Yes — `ChainSigner` | N/A | No |
| `oya-audit-chain-events-domain` | `domain` | N/A — calls through `AuditEventStore` | N/A | No |
| `oya-audit-chain-segments-domain` | `domain` | N/A — Merkle logic (pure, no I/O) | N/A | No |
| `oya-audit-chain-events-adapter` | `adapter` | N/A | Yes — Postgres impl | No |
| `oya-audit-chain-signing-adapter` | `adapter` | N/A | Yes — Ed25519 + KMS bridge | No |
| `oya-audit-chain-worker` | `worker` | N/A | No direct adapter | No — calls application |
| `oya-audit-chain-rest` | `rest` | N/A | No direct adapter | Yes |
| `oya-audit-chain-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-audit-chain-events-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait AuditEventStore: Send + Sync + sealed::Sealed {
    /// Append-only. Returns EventId. Fails if tenant_id is unknown.
    async fn append(&self, tenant_id: TenantId, event: AuditEventDraft)
        -> Result<EventId, AuditError>;
    /// Fetch all unsealed events for a (tenant, period) pair — used by sealer worker.
    async fn fetch_unsealed(&self, tenant_id: TenantId, period: NaiveDate)
        -> Result<Vec<AuditEvent>, AuditError>;
    /// Mark events as sealed after segment is committed.
    async fn mark_sealed(&self, event_ids: &[EventId], segment_id: SegmentId)
        -> Result<(), AuditError>;
}

// oya-audit-chain-segments-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait AuditSegmentSealer: Send + Sync + sealed::Sealed {
    /// Build Merkle root from unsealed events, sign, persist segment. <1s SLA.
    async fn seal_period(&self, tenant_id: TenantId, period: NaiveDate)
        -> Result<AuditSegment, AuditError>;
    /// Verify an existing sealed segment — checks Merkle root + Ed25519 sig.
    async fn verify_segment(&self, segment: &AuditSegment)
        -> Result<VerifyResult, AuditError>;
}

pub trait MerkleTreeBuilder: Send + Sync + sealed::Sealed {
    /// Deterministic: leaves must be sorted by event_id before calling.
    fn build_root(&self, leaves: &[Sha256Hash]) -> Sha256Hash;
    /// Returns the full proof path for leaf at index.
    fn proof_path(&self, leaves: &[Sha256Hash], index: usize) -> Vec<Sha256Hash>;
}

// oya-audit-chain-signing-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait ChainSigner: Send + Sync + sealed::Sealed {
    /// Sign canonical preimage: tenant_id || merkle_root || period_date || prev_segment_root
    async fn sign(&self, key_id: KeyId, preimage: &[u8])
        -> Result<Ed25519Signature, AuditError>;
    async fn verify(&self, key_id: KeyId, preimage: &[u8], sig: &Ed25519Signature)
        -> Result<bool, AuditError>;
    async fn rotate_key(&self, tenant_id: TenantId) -> Result<KeyId, AuditError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P04-audit-chain` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P04-audit-chain` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P04-audit-chain` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P04-audit-chain` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `events` | `audit-chain` | pending |
| `segments` | `audit-chain` | pending |
| `signing` | `audit-chain` | pending |

---

## Grit Claim Symbols

```
crates/oya-audit-chain-events-kernel/src/ports.rs::AuditEventStore
crates/oya-audit-chain-segments-kernel/src/ports.rs::AuditSegmentSealer
crates/oya-audit-chain-signing-kernel/src/ports.rs::ChainSigner
crates/oya-audit-chain-segments-domain/src/merkle.rs::MerkleTree
crates/oya-audit-chain-worker/src/sealer.rs::SegmentSealerWorker
migrations/audit_chain/V001__audit_chain_init.sql::audit_chain_schema
contracts/audit_chain.proto::AuditEventAppended
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P04-audit-chain started; scope: 16 crates (events/segments/signing BCs); Bominal ADR-0028 Merkle/Ed25519; seal latency SLA <1s" \
  -i high \
  -k "M02,P04,phase-start,audit-chain"

icm store \
  -t context-oyatie \
  -c "Phase P04-audit-chain complete; append-only triggers verified; Merkle determinism tested; Ed25519 round-trip green; seal <1s; next: P05-eventing" \
  -i high \
  -k "M02,P04,phase-complete,audit-chain"
```

---

## References

- Bominal ADRs inherited: ADR-0028 (Merkle/Ed25519 audit chain)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05
- unblocks: all Wave-B product phases (every product emits audit events)
