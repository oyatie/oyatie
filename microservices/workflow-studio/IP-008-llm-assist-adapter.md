---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-008-llm-assist-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + foundry-providers-team
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-llm-assist-validation-required]
depends_on: [IP-005, IP-007]
---

# IP-008: LLM-assist bridge adapter

## Intent

Author the `visual-canvas-adapter` extension that bridges to foundry-providers' LLM SDK for prose-to-spec drafting. Implements the four security controls per threat-model T-I-05 + T-S-05 + T-D-04:
1. PII redactor scrubs prose before LLM submission.
2. Prompt-injection classifier scrubs/refuses suspicious patterns.
3. LLM completion validated against canonical workflow_spec.v1.json schema BEFORE surfaced.
4. Streaming back to browser via WS (per Open Question 4 decision: stream-back for UX).

## ChangeSet boundary

One adapter-extension crate + one fixture corpus:
- Extends `oya-workflow-studio-visual-canvas-adapter` with LLM-assist hooks (separate module `llm_assist` within the existing adapter crate; gated by feature flag `llm-assist`).
- New crate: `oya-workflow-studio-llm-assist-bridge-domain` for pure redaction + classification logic.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-llm-assist-bridge-domain/{Cargo.toml,src/{lib.rs,redactor.rs,classifier.rs,schema_validate.rs},tests/{redactor.rs,classifier.rs,schema_validate.rs}}` | create |
| `src/crates/oya-workflow-studio-visual-canvas-adapter/src/llm_assist.rs` | create (added to existing crate) |
| `src/crates/oya-workflow-studio-visual-canvas-adapter/Cargo.toml` | update | add `llm-assist` feature gate |
| `microservices/workflow-studio/capabilities/eval/llm-assist-redactor-golden.jsonl` | create | PII redactor test corpus |
| `microservices/workflow-studio/capabilities/eval/llm-assist-injection-corpus.jsonl` | create | prompt injection adversarial corpus |
| `microservices/workflow-studio/catalog/oya-workflow-studio-llm-assist-bridge-domain.yaml` | create |

## Code Shape

`llm-assist-bridge-domain/src/redactor.rs`:

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap());
static PHONE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\+?\d{1,3}[-.\s]?)?\(?\d{2,4}\)?[-.\s]?\d{3,4}[-.\s]?\d{4}\b").unwrap());
static SSN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap());
static KR_RRN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{6}-[1-4]\d{6}\b").unwrap());
// Healthcare: MRN-like patterns
static MRN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bMRN[-:]?\s*\d{6,12}\b").unwrap());

pub fn redact_prose(prose: &str) -> RedactionResult {
    let mut out = prose.to_string();
    let mut count = 0;
    for re in [&*EMAIL_RE, &*PHONE_RE, &*SSN_RE, &*KR_RRN_RE, &*MRN_RE] {
        let matches: Vec<_> = re.find_iter(&out).map(|m| m.as_str().to_string()).collect();
        count += matches.len();
        out = re.replace_all(&out, "[REDACTED]").to_string();
    }
    RedactionResult { redacted_prose: out, redacted_count: count as u32 }
}

#[derive(Debug, Clone)]
pub struct RedactionResult {
    pub redacted_prose: String,
    pub redacted_count: u32,
}
```

`llm-assist-bridge-domain/src/classifier.rs`:

```rust
use once_cell::sync::Lazy;
use regex::RegexSet;

static INJECTION_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new(&[
        r"(?i)ignore\s+(?:all\s+)?(?:previous|prior|above)\s+instructions",
        r"(?i)system\s*prompt[\s:]+",
        r"(?i)you\s+are\s+now\s+(?:a|an)\s+",
        r"(?i)```\s*system\s*",
        r"(?i)<\s*\|im_start\s*\|\s*>\s*system",
        r"(?i)reveal\s+(?:your|the)\s+(?:system|hidden)\s+",
    ]).unwrap()
});

pub fn detect_injection(prose: &str) -> bool {
    INJECTION_PATTERNS.is_match(prose)
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-llm-assist-bridge-domain
cargo nextest run -p oya-workflow-studio-llm-assist-bridge-domain
cargo run -p oya-dev-cli -- gate validate llm-assist-validation-required --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_redactor_emails_phones_ssns` | golden corpus passes; redacted_count >= expected |
| `test_redactor_kr_rrn` | KR Resident Registration Number redacted (pack-kr) |
| `test_redactor_mrn` | medical record numbers redacted (pack-us-healthcare) |
| `test_classifier_known_injection_patterns` | adversarial corpus detected; 100% true-positive on curated set |
| `test_classifier_false_positive_rate_under_5pct` | benign prose corpus does not over-trigger |
| `test_schema_validation_rejects_extra_fields` | LLM completion with unknown fields rejected before user surface |
| `test_circuit_breaker_after_3_consecutive_timeouts` | breaker opens; subsequent requests fail-fast |

## Halt Conditions

- Redactor leaks PII in test corpus — STOP. T-I-05 invariant breach.
- Classifier misses known injection — improve classifier; add to corpus.

## Next IP

[`IP-009-license-gate-cedar-full.md`](IP-009-license-gate-cedar-full.md)

## References

- threat-model.md T-I-05, T-S-05, T-D-04.
- runbooks/copilot-degraded-fallback.md.
- OWASP Top 10 LLM Applications (2023) A01 + A02.
- EU AI Act 2024/1689 Arts. 9-15 + 26 + 50.
- NIST AI RMF 1.0.
