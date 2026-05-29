---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-001-domain-layer-dispatch-request
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-governance-per-microservice-layout, oya-governance-naming-justification]
---

# IP-001: Domain layer — DispatchRequest

## Intent

Author the `DispatchRequest` aggregate root + value objects in the `oya-intelligence-model-routing-domain`
crate. Pure types; no I/O; no async. Conforms to ADR-0056 BNF v4.1 + ADR-0105 layer enum.

## ChangeSet boundary

One cohesive ChangeSet. New crate `oya-intelligence-model-routing-domain` under
`microservices/intelligence/crates/`. No other crate edits.

## Concrete file targets

| Path | Action |
|---|---|
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/Cargo.toml` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/lib.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/dispatch_request.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/audience.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/modality.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/prompt.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/tool.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/src/config.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-model-routing-domain/tests/dispatch_request_invariants.rs` | create |

## Naming justification

`oya-intelligence-model-routing-domain` conforms to BNF v4.1: `oya-<microservice>-<bc>-<layer>`.
The `model-routing` BC owns this entity per ARCHITECTURE.md §5.

## Code shape

```rust
// dispatch_request.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DispatchRequest {
    pub envelope_version: EnvelopeVersion,        // newtype "1.0"
    pub envelope_id: Ulid,
    pub tenant_id: TenantId,                       // newtype "tenant:<hashed-id>"
    pub audience: Audience,
    pub purpose: Purpose,                          // newtype non-empty
    pub modality: Modality,
    pub provider_hint: ProviderHint,
    pub model_hint: Option<ModelId>,
    pub prompt: Prompt,
    pub tools: Vec<ToolDef>,
    pub config: DispatchConfig,
    pub secret_reference: SecretReference,
    pub pack: Option<Pack>,
    pub consent_grant_id: Option<Ulid>,
    pub rfia_id: Option<Ulid>,
    pub annex_iii_categories: Vec<AnnexIiiCategory>,
}

impl DispatchRequest {
    pub fn new(...) -> Result<Self, DispatchRequestError> { ... }
    pub fn requires_audit_tap_atomicity(&self) -> bool { true }
    pub fn modality_budget_class(&self) -> ModalityBudgetClass { ... }
}
```

Invariants enforced at construction:
- `tenant_id` matches `^tenant:[a-f0-9]{16}$`.
- `prompt.parts` non-empty.
- `config.temperature` ∈ [0, 2] when set.
- `config.top_p` ∈ [0, 1] when set.
- If `rfia_id == None`, then `annex_iii_categories` MUST be empty.

## Acceptance gates

```bash
cargo check  -p oya-intelligence-model-routing-domain
cargo clippy -p oya-intelligence-model-routing-domain -- -D warnings
cargo nextest run -p oya-intelligence-model-routing-domain
```

## Test plan

- Unit tests: constructor accepts valid input; rejects each invariant violation with the correct
  error variant.
- Property tests via `proptest`: round-trip serialisation; envelope_id uniqueness across 10k
  generated samples.

## Halt conditions

- Layer-enum-conformance lane refuses if any cross-layer import is added.
- Cargo deny check refuses unallowed dependency.

## Next IP

[`IP-002-domain-layer-secret-reference.md`](IP-002-domain-layer-secret-reference.md)

## References

- ADR-0255, ADR-0056, ADR-0105.
- `microservices/intelligence/contracts/openapi/intelligence-v1.yaml` (DispatchEnvelope schema).
- `microservices/intelligence/ARCHITECTURE.md` §5 (BC roster).

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
