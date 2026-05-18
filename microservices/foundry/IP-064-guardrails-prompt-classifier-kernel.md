---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-004-prompt-classifier-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, data-class, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-foundry-guardrails-prompt-classifier-kernel

## Intent

Scaffold the `kernel` layer crate per ADR-0105: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation for every other prompt-classifier layer crate.

## ChangeSet boundary

One new Rust crate at `microservices/foundry/src/crates/oya-foundry-guardrails-prompt-classifier-kernel/`. Workspace member added. Catalog row at `microservices/foundry/catalog/`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-foundry-guardrails-prompt-classifier-kernel/Cargo.toml` | create | minimal deps (`async-trait`, `serde`, `chrono`) |
| `src/crates/oya-foundry-guardrails-prompt-classifier-kernel/src/lib.rs` | create | module decl + pub use surface |
| `.../src/entities.rs` | create | `Prompt`, `Classification`, `DataClassTag`, `ClassifierModelVersion`, `ClassifierEnsembleResult` with `#[data_class(..)]` |
| `.../src/ports.rs` | create | `PromptClassifier`, `ClassifierModelServer`, `GuardrailDecisionEmitter` (sealed) |
| `.../src/errors.rs` | create | error variants per port + entity |
| `Cargo.toml` (workspace) | update | add member |
| `catalog/oya-foundry-guardrails-prompt-classifier-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-foundry-guardrails-prompt-classifier-kernel
JUSTIFICATION:
- microservice = foundry-guardrails
- bc-tokens = prompt-classifier
- layer = kernel (ADR-0105 13-value enum)
- exemptions: none
```

## Code Shape

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prompt {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub text: String,
    #[data_class(INTERNAL_ONLY)]
    pub session_turn_count: u32,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id_hashed: String,
    #[data_class(INTERNAL_ONLY)]
    pub capability_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Classification {
    #[data_class(AUDIT)]
    pub verdict: Verdict,
    #[data_class(INTERNAL_ONLY)]
    pub block_reason: Option<BlockReason>,
    #[data_class(INTERNAL_ONLY)]
    pub data_class_tags: Vec<DataClassTag>,
    #[data_class(INTERNAL_ONLY)]
    pub classifier_model_versions: std::collections::BTreeMap<String, String>,
    #[data_class(INTERNAL_ONLY)]
    pub ensemble_score: f64,
    #[data_class(AUDIT)]
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Verdict { Allow, Block, Redact }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockReason {
    Pii, Phi, JailbreakInjection, ContentSafetyToxicity,
    ContentSafetySelfHarm, ContentSafetySexual, ContentSafetyViolence,
    ContentSafetyMinors, ContentSafetyHate, ContentSafetyWeapons,
    ContentSafetyIllegal, AutonomyTierExceeded, PolicyDeny,
    LlmJudgeBudgetExceeded, AiSlop,
}
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait PromptClassifier: Send + Sync + Sealed {
    async fn classify(&self, prompt: &Prompt) -> Result<Classification, KernelError>;
}

#[async_trait]
pub trait ClassifierModelServer: Send + Sync + Sealed {
    async fn infer(&self, model_id: &str, input: &[u8]) -> Result<Vec<f32>, KernelError>;
    fn model_version(&self, model_id: &str) -> Result<ClassifierModelVersion, KernelError>;
}

#[async_trait]
pub trait GuardrailDecisionEmitter: Send + Sync + Sealed {
    async fn emit(&self, decision: &Classification, ctx: &EmitContext) -> Result<(), KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-prompt-classifier-kernel --all-features
cargo build -p oya-foundry-guardrails-prompt-classifier-kernel --all-features
cargo clippy -p oya-foundry-guardrails-prompt-classifier-kernel --all-features -- -D warnings
cargo nextest run -p oya-foundry-guardrails-prompt-classifier-kernel --all-features
cargo deny check
cargo doc -p oya-foundry-guardrails-prompt-classifier-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-foundry-guardrails-prompt-classifier-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-foundry-guardrails-prompt-classifier-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-foundry-guardrails-prompt-classifier-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-foundry-guardrails-prompt-classifier-kernel
```

## Test Plan

Per kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. Coverage 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_prompt_construction` | entity invariants |
| `test_classification_serde` | serde roundtrip |
| `test_verdict_enum_exhaustive` | match exhaustiveness |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_data_class_annotations_present` | every public field has `#[data_class(..)]` |

## Halt Conditions

- BNF v4.1 naming violation
- Any port trait introduces business logic — refactor to domain/usecase
- Any I/O reachable from kernel — refactor

## Next IP

[`IP-005-output-validator-kernel.md`](IP-005-output-validator-kernel.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer; ADR-0106 usecase rename; ADR-0140 (retired per ADR-0145) Cedar.
- PRD §"Bounded Contexts" port-trait table.
- Bominal ADR-0028 (data-class taxonomy).
