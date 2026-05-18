---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-002-translate-router-kernel
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: oya-translate-router-kernel

## Intent

Scaffold the kernel layer per ADR-0105: port traits (sealed), entities, value objects, error types. Zero I/O. Zero business logic. Foundation for every other router layer crate plus all TM / termbase / QE / langdetect / doc / bulk / stream / adapter crates.

## ChangeSet boundary

One new Rust crate at `microservices/translate/src/crates/oya-translate-router-kernel/`. Workspace member registered. Catalog row at `catalog/oya-translate-router-kernel.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-translate-router-kernel/Cargo.toml` | create |
| `src/crates/oya-translate-router-kernel/src/lib.rs` | create — module surface |
| `src/crates/oya-translate-router-kernel/src/entities.rs` | create |
| `src/crates/oya-translate-router-kernel/src/ports.rs` | create |
| `src/crates/oya-translate-router-kernel/src/errors.rs` | create |
| `Cargo.toml` (workspace) | update — register crate |
| `catalog/oya-translate-router-kernel.yaml` | create |

## Crate Naming

```
NAME: oya-translate-router-kernel
JUSTIFICATION:
- microservice = translate (microservices/translate/)
- bc-tokens = router (primary BC; capability-routed engine selection)
- layer = kernel (ADR-0105 13-value enum; inner/pure; ports + entities only)
- exemptions claimed: none
```

## Entity Surface

```rust
// entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vendor {
    Anthropic,
    OpenAI,
    GoogleTranslate,
    DeepL,
    InHouse,                 // foundry-runtime-served
    MicrosoftTranslator,     // tracked; M02
    AmazonTranslate,         // tracked; M02
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentClass {
    UiString,
    Marketing,
    Legal,
    Medical,           // EU AI Act high-risk class flag per ADR-TRANSLATE-0003
    Employment,        // EU AI Act high-risk class flag per ADR-TRANSLATE-0003
    Credit,            // EU AI Act high-risk class flag per ADR-TRANSLATE-0003
    CodeComment,
    Narrative,
    Subtitle,
    GeneralText,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityTier { Draft, Standard, Premium, Eidas }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanguageTag(pub String);     // RFC 5646 BCP 47

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub tenant_id: String,
    pub pack: String,
    pub source_lang: LanguageTag,
    pub target_lang: LanguageTag,
    pub text: String,                    // ≤ 500 chars per PRD; checked in domain
    pub content_class: ContentClass,
    pub quality_tier: QualityTier,
    pub project_id: Option<String>,
    pub use_tm: bool,
    pub constraints: RoutingConstraints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutingConstraints {
    pub forbidden_vendors: Vec<Vendor>,
    pub forbidden_transports: Vec<String>,
    pub prefer_in_house: bool,
    pub cost_ceiling_per_call_usd: Option<f64>,
    pub latency_ceiling_p99_ms: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub decision_id: String,
    pub selected_vendor: Vendor,
    pub selected_region: String,
    pub residency_compliant: bool,
    pub reason: String,
    pub candidate_set: Vec<EngineCandidate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineCandidate {
    pub vendor: Vendor,
    pub region: String,
    pub language_pair_supported: bool,
    pub capability_fit_score: f64,
    pub cost_per_1k_chars_usd: f64,
    pub p99_latency_ms: u32,
    pub availability_rolling_15m: f64,
    pub eligible: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationResult {
    pub decision_id: String,
    pub translated_text: String,
    pub source_lang_detected: Option<LanguageTag>,
    pub engine: Vendor,
    pub model_id: String,
    pub region: String,
    pub cost_usd: f64,
    pub latency_ms: u64,
    pub request_hash: String,            // BLAKE3 hex
    pub response_hash: String,           // BLAKE3 hex
    pub envelope_signature: String,      // Ed25519 hex
    pub evidence_ref: String,            // audit-chain event id
    pub leverage_match: Option<LeverageMatch>,
    pub qe_score: Option<QualityScore>,
    pub residency_compliant: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeverageMatch {
    pub tm_unit_id: String,
    pub match_kind: MatchKind,
    pub similarity_pct: u8,             // 0..=100
    pub source_segment_hash: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchKind { Exact100, Ice, Fuzzy75to99, Mt }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualityScore {
    pub score: f32,                      // 0.0..=100.0
    pub model_id: String,
    pub eu_ai_act_classification: EuAiActClassification,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EuAiActClassification { LowRisk, LimitedRisk, HighRisk }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineHealthSnapshot {
    pub vendor: Vendor,
    pub region: String,
    pub availability_rolling_15m: f64,
    pub p99_latency_ms: u32,
    pub error_rate_rolling_5m: f64,
    pub cost_per_1k_chars_usd: f64,
    pub demoted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResidencyConstraint {
    pub pack: String,
    pub permitted_vendors: Vec<Vendor>,
    pub permitted_regions: Vec<String>,
    pub default_deny: bool,             // always true per ADR-TRANSLATE-0004
}
```

## Port Surface

```rust
// ports.rs (excerpt)
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait TranslateInvoker: Send + Sync + Sealed {
    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, RouterError>;
}

#[async_trait]
pub trait EngineRouter: Send + Sync + Sealed {
    async fn decide(&self, req: &TranslationRequest) -> Result<RoutingDecision, RouterError>;
}

#[async_trait]
pub trait TmLeverageQuery: Send + Sync + Sealed {
    async fn lookup(&self, tenant: &str, project: Option<&str>, src: &LanguageTag, tgt: &LanguageTag, text: &str)
        -> Result<Option<LeverageMatch>, RouterError>;
}

#[async_trait]
pub trait TermbaseQuery: Send + Sync + Sealed {
    async fn enforce(&self, tenant: &str, project: Option<&str>, src: &LanguageTag, tgt: &LanguageTag, text: &str)
        -> Result<Vec<TermEnforcement>, RouterError>;
}

#[async_trait]
pub trait QualityEstimator: Send + Sync + Sealed {
    async fn score(&self, src: &str, tgt: &str, src_lang: &LanguageTag, tgt_lang: &LanguageTag, content_class: ContentClass)
        -> Result<QualityScore, RouterError>;
}

#[async_trait]
pub trait LanguageDetector: Send + Sync + Sealed {
    async fn detect(&self, text: &str) -> Result<LanguageDetection, RouterError>;
}

#[async_trait]
pub trait DocumentTranslator: Send + Sync + Sealed {
    async fn translate_document(&self, doc: &DocumentJob) -> Result<DocumentResult, RouterError>;
}

#[async_trait]
pub trait EngineHealthMonitor: Send + Sync + Sealed {
    async fn snapshot(&self, vendor: Vendor, region: &str) -> Result<EngineHealthSnapshot, KernelError>;
}

#[async_trait]
pub trait TenantPolicyRepository: Send + Sync + Sealed {
    async fn load(&self, tenant_id: &str, pack: &str) -> Result<TenantPolicy, KernelError>;
}

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

## Acceptance Gates

```bash
cargo check -p oya-translate-router-kernel --all-features
cargo build -p oya-translate-router-kernel --all-features
cargo clippy -p oya-translate-router-kernel --all-features -- -D warnings
cargo nextest run -p oya-translate-router-kernel --all-features
cargo deny check
cargo doc -p oya-translate-router-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-translate-router-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-translate-router-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-translate-router-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-translate-router-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_translation_request_construction` | entity invariants |
| `test_routing_decision_serde` | serde roundtrip |
| `test_language_tag_bcp47_parse` | RFC 5646 conformance |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_residency_constraint_default_deny` | `default_deny == true` invariant |
| `test_content_class_eu_ai_act_high_risk_set` | medical/employment/credit classified high-risk |
| `test_no_credential_byte_in_kernel` | grep crate src for credential patterns; 0 hits |

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic.
- Any I/O reachable from kernel.
- Any credential-shaped type that could leak.

## Next IP

[`IP-003-translate-router-domain.md`](IP-003-translate-router-domain.md)
