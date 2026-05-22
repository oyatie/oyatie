---
doc_class: ContractSpec
microservice: connect
contract_type: rust-trait-abi
version: "1.0.0"
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0145, ADR-0249, ADR-0258]
companion_docs:
  - microservices/connect/IP-013-connector-adapter-trait.md
  - microservices/connect/sdk-plan.md
  - microservices/connect/contracts/openapi/connect-integration.yaml
doc_status: published
versioning_policy: SemVer per ADR-0258; minor for additive optional methods; major for breaking invoke signature
deprecation_cadence: ≥6 months notice; sunset ADR required
---

# ConnectorAdapter Trait — ABI Contract v1.0.0

## Purpose

The `ConnectorAdapter` Rust trait is the stable ABI boundary between the connect microservice core and all first-party and third-party connector implementations. Any code implementing this trait for marketplace publishing (MPO flow per ADR-0249) MUST conform to this contract.

Hyperscaler precedent: Workato connector SDK (public trait contract for third-party connector developers); AWS Lambda extension API (stable ABI for out-of-process extensions).

## Trait definition (stable surface)

```rust
use async_trait::async_trait;
use serde_json::Value;

/// ConnectorMetadata — static, allocation-free, embeddable in static storage
#[derive(Debug, Clone)]
pub struct ConnectorMetadata {
    pub id: &'static str,          // matches catalog/connectors/<id>.yaml `name`
    pub display_name: &'static str,
    pub version: semver::Version,
    pub category: ConnectorCategory,
    pub emergency_services_class: bool, // per pagerduty.yaml / twilio.yaml
}

/// InvokeContext — per-call context injected by the adapter engine
pub struct InvokeContext<'a> {
    pub tenant_id: &'a TenantId,
    pub credential: AccessToken, // short-lived ≤60s per ADR-0296
    pub idempotency_key: &'a IdempotencyKey,
    pub rate_limit_profile: &'a RateLimitProfile,
    pub tracer: &'a dyn opentelemetry::trace::Tracer,
}

/// AdapterError — non-exhaustive to allow future variants without breaking callers
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("vendor returned {status}: {body}")]
    VendorError { status: u16, body: String },
    #[error("rate limited; retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
    #[error("credential error: {0}")]
    CredentialError(String),
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("payload schema error: {0}")]
    PayloadSchemaError(String),
    #[error("not retryable: {0}")]
    NotRetryable(String),
}

/// ConnectorAdapter — the stable trait
///
/// Invariants:
/// - `metadata()` MUST be idempotent and allocation-free.
/// - `invoke()` MUST NOT retain `ctx.credential` beyond the call scope.
/// - If `metadata().emergency_services_class == true`, `invoke()` for trigger actions
///   MUST NOT return `AdapterError::RateLimited` or a circuit-breaker error equivalent.
///   (The adapter engine enforces this at call time; adapter MAY also self-enforce.)
/// - `invoke()` MUST be cancel-safe (drop of the future does not leave vendor state corrupted).
#[async_trait]
pub trait ConnectorAdapter: Send + Sync + 'static {
    /// Static metadata for this connector implementation.
    fn metadata(&self) -> &ConnectorMetadata;

    /// Execute a named action against the vendor API.
    ///
    /// Callers:
    /// - `connector-adapter-domain::ConnectorAdapterService::invoke`
    ///
    /// Returns:
    /// - `Ok(Value)` — canonicalized vendor response
    /// - `Err(AdapterError::VendorError { .. })` — retryable if 5xx
    /// - `Err(AdapterError::RateLimited { .. })` — engine honors Retry-After
    /// - `Err(AdapterError::NotRetryable(..))` — engine routes immediately to DLQ
    async fn invoke(
        &self,
        ctx: &InvokeContext<'_>,
        action: &ActionName,
        payload: Value,
    ) -> Result<Value, AdapterError>;

    /// JSON Schema (draft-07) for the named action's input payload.
    /// Returns `None` if no schema defined (schema validation skipped).
    fn action_schema(&self, action: &ActionName) -> Option<Value> {
        let _ = action;
        None
    }

    /// Returns a fingerprint string identifying the current vendor API schema.
    /// Used by schema-drift-monitor to detect vendor-side schema changes.
    /// Default implementation returns a static hash of known action schemas.
    async fn vendor_schema_fingerprint(&self) -> Result<String, AdapterError> {
        Ok(format!("static:{}", self.metadata().version))
    }
}
```

## Versioning policy (ADR-0258)

| Change class | Version bump | Notice period |
|---|---|---|
| New optional method with default impl | Minor (1.1.0) | None required |
| New required method | Major (2.0.0) | ≥6 months; sunset ADR |
| Breaking `invoke` signature change | Major (2.0.0) | ≥6 months; sunset ADR |
| New `AdapterError` variant (non-exhaustive) | Minor | None (callers handle `_` arm) |
| `ConnectorMetadata` new field (default value) | Minor | None |

## Compliance requirements for MPO publishers

Every connector adapter published to the marketplace MUST:

1. Pass security review (Cedar gate `connector-catalog-publishing.cedar`).
2. Provide SBOM (`sbom_provided: true` in catalog entry).
3. Sign the adapter binary with cosign keyless OIDC (Sigstore TUF root).
4. Not retain `ctx.credential` beyond the `invoke()` call scope — verified by static analysis in CI.
5. Respect `metadata().emergency_services_class` semantics — if set, override RateLimited returns for trigger actions.

## Cross-references

- `IP-013-connector-adapter-trait.md` — implementation plan
- `catalog/connectors/pagerduty.yaml` — `emergency_services_class: true` example
- `policy/connector-catalog-publishing.cedar` — MPO publish gate
- `sdk-plan.md` — SDK and test harness documentation
