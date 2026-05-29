# IP-009: projection-gateway-kernel — zero-copy projection contract + sovereignty pin

- Bounded context: projection-gateway
- Layer: kernel
- Crate: `oya-consent-graph-projection-gateway-kernel`
- Acceptance status: ga
- Authority: ADR-0214 §2.5 (zero-copy projection, region-pinned), ADR-SVC-CG-004 (grantor-region
  topic ownership), ADR-0058 (ontology projection model), ADR-0105.
- Depends on: `oya-consent-graph-agreement-kernel`.

## 1. Goal

Define the pure types + ports for zero-copy, region-pinned cross-tenant projection: per-(grantor,
grantee, entity) Pulsar topic minted in the **grantor's** Pulsar cluster, scope-narrowed payload,
optional aggregate-mode k-anonymity guard. Kernel-pure: no Pulsar, no Ontology, no Cedar runtime; just
types and the contracts that downstream layers must honor.

## 2. Scope

In:
- `ProjectionTopic` value object (id, name, region, ACL).
- `ProjectionEvent` and `ProjectionPayload` types.
- `ProjectionContract` invariants (sovereignty, scope-narrowing, aggregate-k-anon).
- `ProjectionMinter` and `ProjectionEmitter` ports.
- `TopicAclEntry` value object.

Out:
- Pulsar adapter (→ IP-010).
- Scope-narrowing and aggregate-k-anon impl (→ IP-011, also pure but lives in `domain`).
- Subscriber side (lives in ontology cross-tenant extension IP-CT-001..005).

## 3. Types

### 3.1 `ProjectionTopic`
```rust
pub struct ProjectionTopic {
    pub topic_id: ProjectionTopicId,                // ULID, unique across cluster
    pub topic_name: TopicName,                      // canonical: oya.consent-graph.projection.v1.<grantor_short>.<grantee_short>.<entity_short>
    pub agreement_id: AgreementId,
    pub grantor: TenantId,
    pub grantee: TenantId,
    pub entity_type: String,
    pub region: Region,                             // ALWAYS grantor's region (sovereignty invariant)
    pub acl: Vec<TopicAclEntry>,                    // grantee-tenant-only on read
    pub mode: SharingMode,                          // Projection | Aggregate | AttestedQuery
    pub k_anonymity: Option<u32>,                   // populated for Aggregate mode
    pub created_at: Timestamp,
    pub destroyed_at: Option<Timestamp>,            // populated on revocation/expiration
}

pub struct TopicAclEntry {
    pub principal: PrincipalRef,                    // grantee-tenant or grantee-tenant.principal
    pub permissions: Vec<TopicPermission>,
    pub expires_at: Option<Timestamp>,
}

pub enum TopicPermission {
    Subscribe,
    PeekMessage,
    // No Publish; only grantor-side adapter writes
    // No Manage; only consent-graph projection-gateway-app manages ACLs
}
```

### 3.2 `ProjectionEvent`
```rust
pub struct ProjectionEvent {
    pub event_id: Ulid,
    pub agreement_id: AgreementId,
    pub entity_type: String,
    pub entity_id: Option<EntityId>,                // None for aggregate-mode rows
    pub payload: ProjectionPayload,
    pub emitted_at: Timestamp,
    pub schema_version: u16,
    pub redaction_applied: Vec<FieldName>,          // for audit; lists fields that were redacted
}

pub enum ProjectionPayload {
    Row(JsonValue),                                  // Projection mode: scope-narrowed row
    Aggregate(AggregateBucket),                      // Aggregate mode: tuple with k-anon guard
    AttestedAnswer(AttestedAnswerPayload),           // AttestedQuery mode: signed answer
}

pub struct AggregateBucket {
    pub group_by: HashMap<String, JsonValue>,        // dimensions
    pub measures: HashMap<String, AggregateValue>,   // computed measures
    pub k: u32,                                      // observed k (≥ k_anonymity)
    pub dp_noise: Option<DpNoise>,                   // differential privacy noise applied
}
```

### 3.3 Invariants
```rust
pub enum ProjectionInvariant {
    TopicRegionEqualsGrantorRegion,                  // sovereignty: topic in grantor cluster
    AclGranteeTenantOnly,                            // no third tenant on ACL
    AclSubscribeOnly,                                // no publish/manage in ACL
    SchemaVersionSupported,
    RedactionAppliedConsistentWithScope,             // emit-time check: payload omits scope-excluded fields
    AggregateModeKAnonHoldsAtEmit,                   // k_anonymity ≤ observed k
    NoCrossBorderTransferInRow,                      // row payload contains no field flagged with cross-border-forbidden classifier (PII/PHI)
    EventIdMonotonicWithinTopic,                     // monotone ulid per topic
}
```

The most critical invariant is **TopicRegionEqualsGrantorRegion** — this is the sovereignty
guarantee, kernel-checked at every topic creation and emit. A violation here is a P0 incident
(`regional-sovereignty-violation.md` runbook).

## 4. Ports

```rust
pub trait ProjectionMinter: Send + Sync {
    async fn mint(&self, agreement: &DataSharingAgreement) -> Result<ProjectionTopic, MintError>;
    async fn destroy(&self, topic_id: ProjectionTopicId) -> Result<(), DestroyError>;
    async fn set_acl(&self, topic_id: ProjectionTopicId, acl: Vec<TopicAclEntry>) -> Result<(), AclError>;
}

pub trait ProjectionEmitter: Send + Sync {
    async fn emit(&self, topic_id: ProjectionTopicId, event: ProjectionEvent) -> Result<EmitReceipt, EmitError>;
}

pub trait ProjectionScopeNarrower: Send + Sync {
    fn narrow(&self, raw_row: &JsonValue, scope: &EntityScope, terms: &SharingTerms)
        -> Result<JsonValue, NarrowError>;
}

pub trait ProjectionAggregator: Send + Sync {
    fn aggregate(&self, rows: &[JsonValue], terms: &SharingTerms)
        -> Result<Vec<AggregateBucket>, AggregateError>;
}
```

## 5. Sovereignty enforcement

The kernel exposes:
```rust
pub fn assert_grantor_region(topic: &ProjectionTopic, expected_grantor_region: Region)
    -> Result<(), ProjectionInvariant>
{
    if topic.region == expected_grantor_region { Ok(()) } else { Err(ProjectionInvariant::TopicRegionEqualsGrantorRegion) }
}
```

Called at:
- Topic mint (IP-010).
- Each event emit (IP-011) — defense-in-depth in case ACLs misconfigured.
- Subscriber side (IP-CT-001 of ontology extension) — re-asserts on receive.

A failed assertion logs at ERROR + emits `oya.consent-graph.sovereignty-violation` to audit-chain
(P0 incident) + increments `sovereignty_violation_total` metric (SLO target = 0).

## 6. Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProjectionKernelError {
    #[error("invariant violation: {0:?}")] Invariant(Vec<ProjectionInvariant>),
    #[error("mint error: {0}")] Mint(#[from] MintError),
    #[error("emit error: {0}")] Emit(#[from] EmitError),
    #[error("narrow error: {0}")] Narrow(#[from] NarrowError),
    #[error("aggregate error: {0}")] Aggregate(#[from] AggregateError),
}
```

## 7. Tests

| Test | Assertion |
|------|-----------|
| `topic_region_must_equal_grantor_region` | mint with mismatched region → invariant violation |
| `acl_must_be_grantee_only` | ACL with third tenant → violation |
| `acl_subscribe_only` | ACL with `Publish` perm → violation |
| `event_redaction_consistency` | event payload contains field outside scope → violation |
| `aggregate_k_anon_holds` | aggregate with k < k_anonymity → violation |
| `event_id_monotonic` | two events with non-monotonic ulid → violation |
| `assert_grantor_region_pure_function` | pure-function test, no I/O |

## 8. Dependencies

- `serde`, `thiserror`, `ulid`
- `oya-shared-{tenant-id, time, region, entity-id, field-name}`
- `oya-consent-graph-agreement-kernel`

**No** Pulsar, **no** Postgres, **no** async runtime (ports are async traits but kernel functions
themselves are sync).

## 9. Verification

- `cargo build` + `cargo test` clean.
- `oya-check-layer-bnf-conformance` clean.
- Property test: any randomly-generated `ProjectionTopic` either passes all invariants or has a
  specific violation enumerable.

## 10. Risk

- **R**: Cross-border-forbidden classifier is incomplete → PII leaks through.
  **M**: Classifier kept in `oya-shared-pii-classifier` crate, audited per regulatory pack; PR review
  required for any classifier change.
- **R**: Aggregate k value computed wrong → k-anon violation.
  **M**: `ProjectionAggregator::aggregate` returns the bucket with observed k; kernel verifies
  `observed_k ≥ required_k_anonymity` at emit time.
- **R**: ACL drift over agreement lifetime (e.g., grantee adds a principal).
  **M**: ACL is recomputed from agreement on every grant amendment; agreement state is the source of
  truth, topic ACL is a projection.

## 11. Notes on multi-grantor

Multi-grantor joins (one grantee subscribes to projections from N grantors) are out of scope for v1.
Each (grantor, grantee, entity) tuple is an independent topic. Workflow Studio is responsible for any
multi-source join logic on the grantee side, and is responsible for re-verifying enforcement at
join-time.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: Snowflake and Databricks provide zero-copy sharing, BigQuery Authorized Views provides view-level filtering, and CMP tools provide consent records without entity-stream projection. This kernel makes the Oyatie distinction explicit: projection is zero-copy, topic-backed, sovereignty-pinned, and always tied to a DataSharingAgreement instead of warehouse-level sharing alone.
