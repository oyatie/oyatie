# IP-004: enforcement-kernel — Cedar evaluation port + cache contract

- Bounded context: enforcement
- Layer: kernel
- Crate: `oya-consent-graph-enforcement-kernel`
- Acceptance status: ga
- Authority: ADR-0214 §2.3 (Cedar enforcement gate), ADR-0090 (Cedar policy engine adoption),
  ADR-0105 (kernel layer rules), ADR-0064 (canonical-base neutrality — Cedar is the only enforcement
  engine across all packs; pack overlay rules layered on top).
- Depends on: `oya-consent-graph-agreement-kernel` (for shared types `AgreementId`, `CedarPolicyId`,
  `EntityScope`, `SharingTerms`, `SovereigntyCfg`).

## 1. Goal

Define the pure types + ports that govern Cedar evaluation at every cross-tenant hop, without binding
the kernel to the Cedar crate itself. The Cedar runtime lives in `enforcement-domain` (which depends
on `cedar-policy`); the kernel must remain swappable in principle (research item: OPA fallback?
unlikely but the port lets us).

## 2. Scope

In:
- `EnforcementRequest` and `EnforcementDecision` types.
- `EnforcementOutcome` enum (Permit / Deny / Indeterminate).
- `PolicyCache` port.
- `EnforcementClock` port.
- `PolicyEvaluator` port (implemented in `enforcement-domain` via Cedar).
- `EnforcementEvent` value object for audit emission.

Out:
- Cedar crate binding (→ `enforcement-domain`).
- gRPC server impl (→ `enforcement-rest`/`enforcement-api`).
- HTTP routes (→ `enforcement-rest`).

## 3. Types

### 3.1 `EnforcementRequest`
```rust
pub struct EnforcementRequest {
    pub request_id: Ulid,                // for audit linking
    pub grantor: TenantId,
    pub grantee: TenantId,
    pub principal: PrincipalId,          // user/service principal in grantee tenant
    pub action: ActionName,              // e.g., "project.subscribe", "project.read", "attested.query"
    pub resource: ResourceRef,           // {entity_type, entity_id (optional for subscribe)}
    pub context: EnforcementContext,
    pub agreement_id_hint: Option<AgreementId>,  // optimization; resolver re-checks
}

pub struct EnforcementContext {
    pub purpose_of_use: PurposeOfUse,
    pub request_time: Timestamp,
    pub request_region: Region,
    pub tenant_class: TenantClass,         // demo_trial/paid from principal claim
    pub prior_revocation_check_ms_ago: u64,        // 0 if just-checked; bound by 200ms
    pub trace_id: TraceId,
}
```

### 3.2 `EnforcementDecision`
```rust
pub struct EnforcementDecision {
    pub outcome: EnforcementOutcome,
    pub matched_agreement: Option<AgreementId>,
    pub determining_policy_id: Option<CedarPolicyId>,
    pub reasons: Vec<DeterminingReason>,    // human-readable explanation (for audit + debugging)
    pub eval_duration_ns: u64,              // hot-path latency observability
    pub cache_hit: bool,
}

pub enum EnforcementOutcome {
    Permit,
    Deny { reason: DenyReason },
    Indeterminate { reason: String },        // policy compile error, etc. — treated as Deny upstream
}

pub enum DenyReason {
    NoAgreement,                             // no matching active agreement
    AgreementRevoked,                        // agreement revoked, cache invalidation in flight
    AgreementSuspended,
    AgreementExpired,
    ScopeNotPermitted,                       // requested field/entity outside scope
    PurposeOfUseMismatch,                    // context.purpose != agreement.terms.purpose
    SovereigntyViolation,                    // grantee region not eligible
    RateLimitExceeded,
    CedarPolicyDeny,                         // explicit Cedar deny rule fired
    PartnerSuspended,                        // partner-directory marked peer Suspended
    InvalidPrincipal,                        // principal not in grantee tenant
    StaleRevocationCheck,                    // prior_revocation_check_ms_ago > 200ms
}
```

### 3.3 `PolicyCache` port
```rust
pub trait PolicyCache: Send + Sync {
    fn get(&self, agreement_id: AgreementId) -> Option<Arc<CompiledPolicyHandle>>;
    fn put(&self, agreement_id: AgreementId, handle: Arc<CompiledPolicyHandle>);
    fn invalidate(&self, agreement_id: AgreementId);
    fn invalidate_all_for_pair(&self, grantor: TenantId, grantee: TenantId);  // bulk revocation
    fn stats(&self) -> PolicyCacheStats;
}

pub struct CompiledPolicyHandle {
    pub agreement_id: AgreementId,
    pub policy_id: CedarPolicyId,
    pub compiled_at: Timestamp,
    pub schema_fingerprint: [u8; 32],   // canonical-base + pack overlay fingerprint
    pub opaque: Arc<dyn Any + Send + Sync>,  // typed in enforcement-domain as CedarPolicySet
}
```

Cache must be thread-safe (Arc<RwLock> or dashmap underneath); kernel exposes only the trait.

### 3.4 `PolicyEvaluator` port
```rust
pub trait PolicyEvaluator: Send + Sync {
    fn evaluate(
        &self,
        policy: &CompiledPolicyHandle,
        request: &EnforcementRequest,
    ) -> Result<EnforcementDecision, EvaluatorError>;
}
```

## 4. Cache invalidation contract (per ADR-SVC-CG-002)

Invalidation must propagate within these bounds:
- Local pod cache: ≤10ms after revocation event received from Pulsar.
- Cross-pod cluster: ≤500ms (Pulsar fan-out + per-pod subscriber).
- Cross-region: ≤1s (cross-region Pulsar mirror).

Stale-read budget: an enforcement decision rendered on a cached policy is permitted to be at most
1s out-of-date. This is encoded by the `prior_revocation_check_ms_ago` field: if a caller passes a
value >200ms, the kernel returns `Deny { reason: StaleRevocationCheck }`. Callers are responsible
for piggy-backing a freshness ping on every request (which is cheap — a single Pulsar tailing read).

## 5. Tests (kernel only)

| Test | Assertion |
|------|-----------|
| `decision_serde_roundtrip` | `EnforcementDecision` serializes/deserializes losslessly |
| `outcome_indeterminate_treated_as_deny_upstream` | matcher in domain converts Indeterminate → Deny |
| `stale_revocation_check_denies` | request with `prior_revocation_check_ms_ago=300` → Deny |
| `cache_invalidate_all_for_pair_bulk` | bulk invalidation removes N agreements in one call |
| `compiled_policy_handle_send_sync` | static assertion via `static_assertions::assert_impl_all!` |

## 6. Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum EnforcementKernelError {
    #[error("agreement not found: {0}")] AgreementNotFound(AgreementId),
    #[error("policy cache poisoned")] CachePoisoned,
    #[error("evaluator error: {0}")] Evaluator(#[from] EvaluatorError),
}
```

## 7. Dependencies

- `serde`, `thiserror`, `ulid`
- `oya-shared-{tenant-id, time, region}`
- `oya-consent-graph-agreement-kernel`
- `static_assertions`

**No** Cedar, **no** Pulsar, **no** Tokio, **no** Postgres.

## 8. Public API discipline

The Cedar runtime is hidden behind `CompiledPolicyHandle.opaque: Arc<dyn Any>`. Callers must never
downcast outside the `enforcement-domain` crate. A custom clippy lint (or a `pub(crate)` boundary in
the domain layer) keeps this honest.

## 9. Performance targets

The kernel is a hot path: `evaluate` will be called ~100K req/s at peak. Hot-path budget within
kernel (excluding evaluator + cache I/O): ≤100ns per call.

## 10. Verification

- `cargo build` + `cargo test` clean.
- `oya-check-layer-bnf-conformance` clean.
- `cargo flamegraph` on a 10K-evaluation hot loop shows <1% time in kernel-layer code
  (Cedar dominates).

## 11. Risk

- **R**: `Arc<dyn Any>` allows downstream to bypass type discipline.
  **M**: Tracked by `cargo-deny ban-deny-downcast` lint at the domain layer's compile time.
- **R**: Cache invalidation race: revocation event arrives mid-evaluation.
  **M**: Evaluation reads cache snapshot once; revocation event invalidates *next* lookup. The 200ms
  freshness budget covers the race.
- **R**: Indeterminate decisions leak (e.g., Cedar evaluator panics).
  **M**: `enforcement-usecase` wraps eval in `catch_unwind` and converts panics to `Indeterminate`;
  unit-tested via injected panicking evaluator.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: OneTrust/TrustArc expose policy and preference enforcement through tenant RBAC workflows, Cookiebot gates tags/categories, and Snowflake/Databricks rely on RBAC/table controls. This kernel's Cedar evaluation port is the Oyatie-specific boundary: every cross-tenant hop returns Allow/Deny/Indeterminate under cache invalidation rules instead of treating consent as a detached record.
