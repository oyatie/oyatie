---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate
phase: P15-data-boundary
status: Proposed
acceptance_lanes: []
entry_gate: 'M02/P03-identity complete; oya-identity-kernel ships; cargo check clean;
  grit done

  on all P03 symbols. P14-policy SHOULD be complete (Cedar engine available) but P15

  may begin scaffolding in parallel; Cedar integration for DUB enforcement completes

  after P14 exits.

  '
exit_gate: 'All P15 impl-plan acceptance gates green; 12 data classes declared in
  kernel; HARD_DENY

  enforced at runtime for PHI/PCI/PIPA/children (verified by integration tests); DUB

  Cedar policy fragment deployed; 2 BCs registered (data-boundary-engine,

  data-boundary-classification); all crates pass cargo check/build/clippy/nextest/deny;

  oya gate validate lean-a1/a2/a3/a4 exit 0; grit done on all P15 symbols; ICM

  phase-complete row emitted.

  '
depends_on:
- milestone: M02
  phase: P03-identity
  reason: DataUseRequest carries principal_id from identity kernel; Person-pillar
    objects require identity.Person type to determine data class applicability.
- milestone: M02
  phase: P14-policy
  reason: DUB enforcement uses Cedar PolicyEvaluator for HARD_DENY decisions; DUB
    Cedar policy fragment loaded into policy.tenant_rule_packs by this phase.
owner_team: council-architecture
purpose: "Delivers the Data-Use-Boundary (DUB) substrate: the runtime enforcement layer that prevents regulated data classes (PHI, PCI, PIPA, children's data) from crossing prohibited boundaries."
---
# P15-data-boundary: Data-Use-Boundary Substrate — 12 Data Classes + HARD_DENY Runtime + Cedar DUB Policy

## Purpose

Delivers the Data-Use-Boundary (DUB) substrate: the runtime enforcement layer that
prevents regulated data classes (PHI, PCI, PIPA, children's data) from crossing
prohibited boundaries. Per oyatie ADR-0008, every data mutation in the system must
declare its data class; the DUB engine evaluates the declared class against the
request context and issues HARD_DENY for prohibited combinations.

This is not a soft guardrail — HARD_DENY means the database write does not happen.
The DUB Cedar policy fragment is the canonical source of truth for which (data_class,
action, principal_type, jurisdiction) combinations are unconditionally forbidden. Every
Ontology Action Type that touches a regulated data class routes through the DUB check
before the write is committed.

12 data classes per ADR-0008:
PHI (Protected Health Information), PCI (Payment Card Industry), PIPA (Korean Personal
Information Protection Act), Children (data of minors), BiometricId, FinancialAccount,
GovernmentId, Employment, LocationHistory, Communications, Genetic, Behavioral.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `data-boundary` | `engine` | `crates/oya-data-boundary-engine-kernel/` | `oya-data-boundary-engine-kernel` |
| `data-boundary` | `engine` | `crates/oya-data-boundary-engine-domain/` | `oya-data-boundary-engine-domain` |
| `data-boundary` | `engine` | `crates/oya-data-boundary-engine-application/` | `oya-data-boundary-engine-application` |
| `data-boundary` | `engine` | `crates/oya-data-boundary-engine-adapter/` | `oya-data-boundary-engine-adapter` |
| `data-boundary` | `engine` | `crates/oya-data-boundary-engine-grpc/` | `oya-data-boundary-engine-grpc` |
| `data-boundary` | `engine` | `crates/oya-data-boundary-engine-app/` | `oya-data-boundary-engine-app` |
| `data-boundary` | `classification` | `crates/oya-data-boundary-classification-kernel/` | `oya-data-boundary-classification-kernel` |
| `data-boundary` | `classification` | `crates/oya-data-boundary-classification-adapter/` | `oya-data-boundary-classification-adapter` |
| `data-boundary` | all | `contracts/data_boundary.proto` | — |
| `data-boundary` | all | `migrations/data_boundary/V001__data_boundary_schema.sql` | — |
| `data-boundary` | all | `cedar/data_boundary.cedar` | — |

Naming justification:

```
NAME: oya-data-boundary-engine-kernel
JUSTIFICATION:
- microservice = data-boundary: DUB substrate; ADR-0056 v4.1; oyatie ADR-0008
  (2 tokens: data-boundary; both are load-bearing — "data" alone is too generic,
  "boundary" alone insufficient; together: Data-Use-Boundary)
- bc-tokens = engine: multiple BCs (engine, classification); engine owns the
  HARD_DENY runtime evaluation loop
- layer = kernel: sealed ports DubEvaluator + DubAuditStore; 12 DataClass enum;
  DataUseRequest + DubDecision types; ZERO I/O
- exemptions claimed: none

NAME: oya-data-boundary-classification-kernel
JUSTIFICATION:
- microservice = data-boundary, bc-tokens = classification: the BC that owns
  per-object data class declarations; separate from the evaluation engine BC
- layer = kernel: DataClassificationStore port; ObjectDataClass type
- exemptions claimed: none
```

### Out-of-scope

- Per-jurisdiction DUB overlay matrices (e.g., EU GDPR vs KR PIPA specific rules) — deferred to M03
- Data subject access request (DSAR) workflows — deferred to M03 (uses Workflow engine)
- Data retention enforcement (deletion schedules) — deferred to M03

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-data-boundary-kernel-scaffold.md`](IP-001-data-boundary-kernel-scaffold.md) | Scaffold all 8 DUB crates; 12 DataClass enum; DubEvaluator + DubAuditStore ports; full DDL | pending | `council-architecture` |
| [`IP-002-data-boundary-cedar-dub-policy.md`](IP-002-data-boundary-cedar-dub-policy.md) | Author cedar/data_boundary.cedar fragment; load into policy engine; integration test HARD_DENY | pending | `council-architecture` |
| [`IP-003-data-boundary-load-tests.md`](IP-003-data-boundary-load-tests.md) | k6 load test; DUB evaluation p99 ≤10ms; HARD_DENY audit log write p99 ≤20ms | pending | `council-architecture` |

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
oya gate validate lean-a1 --phase P15-data-boundary
oya gate validate lean-a2 --phase P15-data-boundary
oya gate validate lean-a3 --phase P15-data-boundary
oya gate validate lean-a4 --phase P15-data-boundary
```

### DUB-specific gates

```bash
# HARD_DENY enforced — must not be bypassable
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_phi    # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_pci    # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_pipa   # exit 0
cargo nextest run -p oya-data-boundary-engine-adapter --test hard_deny_children  # exit 0
# Cedar DUB policy fragment loads cleanly
cargo nextest run -p oya-data-boundary-engine-adapter --test cedar_dub_policy_load  # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-data-boundary-engine-kernel` | `kernel` | Yes — DubEvaluator, DubAuditStore | N/A |
| `oya-data-boundary-engine-domain` | `domain` | N/A | N/A |
| `oya-data-boundary-engine-application` | `application` | N/A | N/A |
| `oya-data-boundary-engine-adapter` | `adapter` | N/A | Yes — CedarDubEvaluator, PgDubAuditStore |
| `oya-data-boundary-engine-grpc` | `grpc` | N/A | No direct adapter import |
| `oya-data-boundary-engine-app` | `app` | N/A | Unrestricted inward |
| `oya-data-boundary-classification-kernel` | `kernel` | Yes — DataClassificationStore | N/A |
| `oya-data-boundary-classification-adapter` | `adapter` | N/A | Yes — PgDataClassificationStore |

### Port traits declared in kernel

```rust
// oya-data-boundary-engine-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// The 12 data classes per oyatie ADR-0008
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataClass {
    Phi,            // Protected Health Information
    Pci,            // Payment Card Industry
    Pipa,           // Korean Personal Information Protection Act
    Children,       // Data of minors (COPPA / KPIPA child provisions)
    BiometricId,
    FinancialAccount,
    GovernmentId,
    Employment,
    LocationHistory,
    Communications,
    Genetic,
    Behavioral,
}

impl DataClass {
    /// Returns true if this class triggers HARD_DENY enforcement
    pub fn is_hard_deny_class(&self) -> bool {
        matches!(self, DataClass::Phi | DataClass::Pci | DataClass::Pipa | DataClass::Children)
    }
}

#[async_trait::async_trait]
pub trait DubEvaluator: Send + Sync + sealed::Sealed {
    /// Returns DubDecision::Allow or DubDecision::HardDeny
    /// HardDeny MUST prevent the downstream write from executing
    async fn evaluate(&self, tenant_id: TenantId, request: DataUseRequest) -> Result<DubDecision, DubError>;
}

#[async_trait::async_trait]
pub trait DubAuditStore: Send + Sync + sealed::Sealed {
    async fn record_deny(&self, tenant_id: TenantId, request: &DataUseRequest, reason: &str) -> Result<(), DubError>;
    async fn query_denies(&self, tenant_id: TenantId, from: chrono::DateTime<chrono::Utc>, limit: u32) -> Result<Vec<DubDenyEntry>, DubError>;
}

// oya-data-boundary-classification-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait DataClassificationStore: Send + Sync + sealed::Sealed {
    async fn get_class(&self, tenant_id: TenantId, object_id: ObjectId) -> Result<Option<DataClass>, DubError>;
    async fn declare_class(&self, tenant_id: TenantId, object_id: ObjectId, class: DataClass, declared_by: PrincipalId) -> Result<(), DubError>;
}
```

### CI lanes that must green

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P15-data-boundary` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P15-data-boundary` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P15-data-boundary` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P15-data-boundary` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P15-data-boundary` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `data-boundary-engine` | `data-boundary` | pending |
| `data-boundary-classification` | `data-boundary` | pending |

---

## Grit Claim Symbols

```
crates/oya-data-boundary-engine-kernel/src/lib.rs::DubEvaluator
crates/oya-data-boundary-engine-kernel/src/lib.rs::DubAuditStore
crates/oya-data-boundary-engine-kernel/src/lib.rs::DataClass
crates/oya-data-boundary-classification-kernel/src/lib.rs::DataClassificationStore
cedar/data_boundary.cedar::HardDenyPhi
migrations/data_boundary/V001__data_boundary_schema.sql::data_boundary.object_classifications
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P15-data-boundary started; 12 DataClass enum; HARD_DENY for PHI/PCI/PIPA/children; Cedar DUB policy fragment" \
  -i high \
  -k "M02,P15,phase-start,data-boundary"

icm store \
  -t context-oyatie \
  -c "Phase P15-data-boundary complete; HARD_DENY enforcement verified; audit log live; DUB Cedar policy deployed; next: P16-records" \
  -i high \
  -k "M02,P15,phase-complete,data-boundary"
```

---

## References

- oyatie ADRs cited: ADR-0008 (data-use-boundary), ADR-0056 v4.1
- Bominal ADRs inherited: ADR-0007 (Cedar), ADR-0028 (audit-chain), ADR-0132 (data ownership pillars)
- M02b-substrate-schema-foundation §6-N (data-boundary outlined)
