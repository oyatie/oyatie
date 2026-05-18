---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-005-output-validator-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-foundry-guardrails-output-validator-kernel

## Intent

Scaffold the `kernel` layer crate for the `output-validator` BC: port traits (sealed) + entity types (`ProviderOutput`, `Validation`, `RedactionDiff`, `BlockReason`) + errors. Zero I/O. Foundation for output-validator BC.

## ChangeSet boundary

One new Rust crate at `microservices/foundry/src/crates/oya-foundry-guardrails-output-validator-kernel/`. Sibling to IP-004's prompt-classifier-kernel.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-output-validator-kernel/Cargo.toml` | create |
| `.../src/{lib.rs,entities.rs,ports.rs,errors.rs}` | create |
| `Cargo.toml` (workspace) | update |
| `catalog/oya-foundry-guardrails-output-validator-kernel.yaml` | create |

## Code Shape

```rust
// src/entities.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderOutput {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub text: String,
    #[data_class(INTERNAL_ONLY)]
    pub tool_args: Option<serde_json::Value>,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id_hashed: String,
    #[data_class(INTERNAL_ONLY)]
    pub invocation_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Validation {
    #[data_class(AUDIT)]
    pub verdict: super::Verdict,
    #[data_class(INTERNAL_ONLY)]
    pub block_reason: Option<super::BlockReason>,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub redaction_diff: Option<RedactionDiff>,
    #[data_class(INTERNAL_ONLY)]
    pub detector_versions: std::collections::BTreeMap<String, String>,
    #[data_class(AUDIT)]
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RedactionDiff {
    #[data_class(INTERNAL_ONLY)]
    pub original_hash: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub redacted_text: String,
    #[data_class(INTERNAL_ONLY)]
    pub redacted_spans: Vec<RedactedSpan>,
}
```

```rust
// src/ports.rs
#[async_trait]
pub trait OutputValidator: Send + Sync + Sealed {
    async fn validate(&self, output: &ProviderOutput) -> Result<Validation, KernelError>;
}

#[async_trait]
pub trait SecretLeakDetector: Send + Sync + Sealed {
    async fn detect(&self, text: &str) -> Result<Vec<SecretLeakHit>, KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-output-validator-kernel --all-features
cargo nextest run -p oya-foundry-guardrails-output-validator-kernel --all-features
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-foundry-guardrails-output-validator-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-foundry-guardrails-output-validator-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-foundry-guardrails-output-validator-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-foundry-guardrails-output-validator-kernel
```

## Test Plan

Per kernel class. 1 test per public type + 1 per port + 1 sealed-trait smoke. Coverage 90% / 80%.

## Halt Conditions

Same as IP-004.

## Next IP

[`IP-006-autonomy-tier-gate-kernel-and-cedar-adapter.md`](IP-006-autonomy-tier-gate-kernel-and-cedar-adapter.md)

## References

- ADR-0056, ADR-0105, ADR-0140 (retired per ADR-0145).
- PRD §"Bounded Contexts".
