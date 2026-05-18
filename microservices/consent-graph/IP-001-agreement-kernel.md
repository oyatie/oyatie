# IP-001: agreement-kernel — DataSharingAgreement type + state machine

- Bounded context: agreement
- Layer: kernel (per ADR-0105 13-layer enum)
- Crate: `oya-consent-graph-agreement-kernel`
- Acceptance status: ga
- Authority: ADR-0214 §2.1 (entity definition), ADR-0110 (state machine semantics), ADR-0056 (BNF
  layer rules), ADR-0105 (kernel layer = pure types + ports + invariants, zero side effects).

## 1. Goal

Define the **pure** type system for the `DataSharingAgreement` aggregate, its lifecycle state machine,
its scope/terms value objects, and the ports (traits) that downstream layers (`domain`, `usecase`,
`adapter`) implement. Zero side effects. Zero I/O. Zero dependencies beyond `std`, `serde`,
`thiserror`, `oya-shared-time`, `oya-shared-tenant-id`, `oya-shared-ulid`, and
`oya-audit-chain-emission-kernel` (for the `ChainLinkPair` shared type).

## 2. Scope

In:
- `DataSharingAgreement` struct.
- `AgreementId`, `EntityScope`, `SharingTerms`, `SharingMode`, `AgreementState`,
  `SovereigntyCfg`, `ChainLinkPair`, `CedarPolicyId` types.
- `AgreementStateMachine` pure transition function.
- `AgreementInvariant` enum + check trait.
- Ports: `AgreementRepository`, `AgreementClock`, `AgreementIdGenerator`.

Out:
- Postgres bindings (→ `agreement-adapter`).
- HTTP routes (→ `agreement-rest`).
- Cedar compilation (→ `enforcement-domain`).
- Pulsar emission (→ `agreement-adapter`, `audit-bridge-*`).

## 3. Types

### 3.1 `AgreementId`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgreementId(pub Ulid);
impl AgreementId {
    pub fn now(gen: &impl AgreementIdGenerator) -> Self { Self(gen.next()) }
    pub fn as_str(&self) -> String { self.0.to_string() }
}
```
Why ULID, not UUIDv4: monotonic-by-time-prefix gives us free chronological ordering for audit-citable
references; matches what audit-chain uses for `SealId`.

### 3.2 `AgreementState` lifecycle
```rust
pub enum AgreementState {
    Drafted,                 // grantor authored, not yet visible to grantee
    Offered,                 // grantor offered to grantee, grantee may accept or decline
    Accepted,                // grantee accepted; Cedar policy compilation begins
    Active,                  // Cedar policy live; projection topic minted; enforcement live
    Suspended,               // temporarily blocked (e.g., compliance review) — projection paused but agreement intact
    Revoked { reason: RevocationReason, at: Timestamp },   // permanent terminal; not re-activatable
    Expired { at: Timestamp },   // expiration timestamp passed; identical to Revoked but distinct event class
}

pub enum RevocationReason {
    GrantorInitiated, GranteeInitiated, DataSubjectInitiated,
    ContractEnded, PolicyViolation, SecurityIncident, ExpirationCascade, DsarErasure,
}
```

### 3.3 `EntityScope`
```rust
pub struct EntityScope {
    pub entity_type: String,        // Ontology entity type, e.g., "FinishedGoodsInventory"
    pub field_set: FieldSet,        // either AllFields, Allow{Vec<String>}, or Deny{Vec<String>}
    pub predicate: Option<String>,  // Cedar predicate, e.g., "resource.sku in principal.open_po_skus"
}

pub enum FieldSet {
    AllFields,                                  // grantor permits the full entity (rare)
    Allow(Vec<FieldName>),                      // explicit allow-list (common)
    Deny(Vec<FieldName>),                       // explicit deny-list (use sparingly; allow-list preferred)
}
```

### 3.4 `SharingTerms`
```rust
pub struct SharingTerms {
    pub purpose_of_use: PurposeOfUse,           // e.g., "inventory-visibility", "eligibility-verification"
    pub mode: SharingMode,                      // Projection | Aggregate | AttestedQuery
    pub redaction: RedactionConfig,             // per-field redaction (mask, hash, null, range-bucket)
    pub k_anonymity: Option<u32>,               // required if mode == Aggregate; default k=5
    pub differential_privacy: Option<DpConfig>, // ε, δ for Aggregate mode
    pub max_qps: Option<u32>,                   // rate cap on grantee reads (1K default)
}
```

### 3.5 `SovereigntyCfg`
```rust
pub struct SovereigntyCfg {
    pub grantor_region: Region,
    pub permitted_grantee_regions: Vec<Region>,         // subset of grantee tenant's regions
    pub cross_border_transfer_permitted: bool,          // default false
    pub residency_overlay_pack: Option<PackId>,         // e.g., kr, eu, us-healthcare
}
```

### 3.6 `ChainLinkPair`
```rust
pub struct ChainLinkPair {
    pub grantor: ChainLink,    // {chain_id, seq, sealed_at}
    pub grantee: ChainLink,
}
```
Shared with `oya-audit-chain-emission-kernel`. The pair forms the bilateral cross-pointer required by
ADR-0214 §2.6 and IP-013.

## 4. State machine

Transitions are **pure** (`fn (state, event) -> Result<state, AgreementInvariantViolation>`).

| From | Event | To | Notes |
|------|-------|----|----|
| ∅ | Draft | Drafted | grantor scope authored |
| Drafted | Offer | Offered | grantor sends to grantee |
| Drafted | DiscardDraft | ∅ | discardable until Offered |
| Offered | Accept | Accepted | grantee acceptance, triggers Cedar compile |
| Offered | Decline | ∅ | grantee may decline |
| Offered | RevokeOffer | ∅ | grantor may rescind before acceptance |
| Accepted | Activate | Active | Cedar policy compiled + projection topic minted |
| Accepted | RevokeForCompile | Revoked | Cedar compile failed; deterministic terminal |
| Active | Suspend | Suspended | compliance review or breach-of-terms |
| Active | Amend | Drafted (new version) | scope change creates new agreement; old superseded |
| Active | Revoke{reason} | Revoked | either party may revoke |
| Active | Expire{at} | Expired | expiration timestamp reached |
| Suspended | Resume | Active | review cleared |
| Suspended | Revoke{reason} | Revoked | review rejected |
| Revoked / Expired | * | ⛔ | terminal; further events rejected |

Transitions outside this table return `AgreementInvariantViolation::IllegalTransition`.

## 5. Invariants

```rust
pub enum AgreementInvariant {
    GrantorNotGrantee,                  // grantor.tenant_id != grantee.tenant_id
    NonEmptyEntityType,                 // scope.entity_type != ""
    AggregateModeRequiresKAnonymity,    // mode==Aggregate ⟹ k_anonymity.is_some() && k>=2
    ExpirationInFuture,                 // if Some(exp), exp > created_at + 1h
    SovereigntyConfigConsistent,        // !cross_border_transfer_permitted ⟹ permitted_grantee_regions ⊆ {grantor_region}
    RevocableTrueByDefault,             // revocable=false requires legal-hold attestation field
    BilateralChainLinkPaired,           // both ChainLink halves present once Active
    SchemaVersionSupported,             // schema_version ∈ supported set
}
```

`check_all(&DataSharingAgreement) -> Result<(), Vec<AgreementInvariantViolation>>` is part of the kernel
surface so any layer can re-verify.

## 6. Ports

```rust
pub trait AgreementRepository: Send + Sync {
    fn create(&self, a: &DataSharingAgreement) -> Result<(), RepoError>;
    fn read(&self, id: AgreementId, tenant: TenantId) -> Result<Option<DataSharingAgreement>, RepoError>;
    fn update_optimistic(&self, a: &DataSharingAgreement, expected_version: u64) -> Result<(), RepoError>;
    fn list_active_by_grantor(&self, grantor: TenantId) -> Result<Vec<DataSharingAgreement>, RepoError>;
    fn list_active_by_grantee(&self, grantee: TenantId) -> Result<Vec<DataSharingAgreement>, RepoError>;
}

pub trait AgreementClock: Send + Sync {
    fn now(&self) -> Timestamp;
}

pub trait AgreementIdGenerator: Send + Sync {
    fn next(&self) -> Ulid;
}
```

Tenant-scoping is enforced at the port-contract level (every method takes `TenantId`). Adapters that
violate tenant-scoping fail the `oya-check-tenant-isolation` lint.

## 7. Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgreementKernelError {
    #[error("invariant violation: {0:?}")] Invariant(Vec<AgreementInvariant>),
    #[error("illegal transition from {from:?} on event {event}")] IllegalTransition { from: AgreementState, event: String },
    #[error("schema version {0} not supported")] UnsupportedSchema(u16),
}
```

## 8. Tests (kernel-only)

| Test | Assertion |
|------|-----------|
| `state_machine_happy_path` | Drafted → Offered → Accepted → Active → Revoked transitions cleanly |
| `state_machine_illegal_revoke_from_drafted` | Drafted + Revoke → IllegalTransition |
| `invariant_grantor_not_grantee` | grantor==grantee → violation |
| `invariant_aggregate_requires_k` | mode=Aggregate without k → violation |
| `invariant_sovereignty_cross_border_forbidden` | !cross_border + grantee_region outside grantor_region → violation |
| `bilateral_chain_link_pair_required_on_active` | Active state without ChainLinkPair → violation |
| `terminal_states_reject_events` | Revoked + any event → IllegalTransition |

100% line coverage required on kernel (per ADR-0105 kernel-layer policy).

## 9. Dependencies (Cargo)

- `std`
- `serde = { version = "1", features = ["derive"] }`
- `thiserror = "2"`
- `ulid = "1"`
- `oya-shared-time` (workspace)
- `oya-shared-tenant-id` (workspace)
- `oya-audit-chain-emission-kernel` (workspace, for `ChainLink` shared type only)

**No** Postgres, **no** Pulsar, **no** Cedar, **no** Tokio, **no** HTTP — those belong in lower layers.

## 10. Output

The crate exposes:
- `DataSharingAgreement` aggregate
- `agreement_state_machine::apply(state, event) -> Result<state, _>`
- `agreement_invariants::check_all(&DataSharingAgreement) -> Result<(), Vec<_>>`
- All ports above
- `AgreementKernelError`

Anything else is a layer violation and fails `oya-check-layer-bnf-conformance`.

## 11. Verification

- `cargo build -p oya-consent-graph-agreement-kernel` clean.
- `cargo test -p oya-consent-graph-agreement-kernel` 100% pass.
- `oya-check-layer-bnf-conformance` clean (no I/O, no async runtime, no HTTP dep).
- `oya-check-public-api-surface` snapshot reviewed.

## 12. Risk + mitigation

- **R**: State machine misses a transition → impl is stuck.
  **M**: Exhaustive table-driven `match` + clippy `wildcard_match_arms = warn`.
- **R**: Invariant under-specified → bad agreements slip through.
  **M**: 7 invariants enumerated + property test (proptest) generating random agreements + checking
  invariant-consistent.
- **R**: Schema evolution breaks downstream → grantee SDKs explode.
  **M**: `schema_version` field gated by ADR-0123 hyperscaler maturity claim gate + ADR-0064 canonical-base
  neutrality (no breaking changes without ADR + version bump + 6mo sunset).
