---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-008-kernel-guardrail-stack
status: pending
owner: axis-intelligence + council-privacy
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-refusal-baseline-floor]
---

# IP-008: Kernel — guardrail-stack port traits

## Intent

`oya-intelligence-guardrails-kernel`: port traits for pre-call classifier + post-call classifier +
PII detector + refusal-baseline + EU AI Act Annex III evaluator + abuse-defence gate.

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-guardrails-kernel/Cargo.toml` | create |
| `.../oya-intelligence-guardrails-kernel/src/lib.rs` | create |
| `.../oya-intelligence-guardrails-kernel/src/pre_call_classifier_port.rs` | create |
| `.../oya-intelligence-guardrails-kernel/src/post_call_classifier_port.rs` | create |
| `.../oya-intelligence-guardrails-kernel/src/pii_detector_port.rs` | create |
| `.../oya-intelligence-guardrails-kernel/src/refusal_baseline_port.rs` | create |
| `.../oya-intelligence-guardrails-kernel/src/annex_iii_port.rs` | create |
| `.../oya-intelligence-guardrails-kernel/src/abuse_defence_port.rs` | create |

## Code shape

```rust
#[async_trait]
pub trait PreCallClassifierPort: Send + Sync + 'static {
    async fn classify(&self, request: &DispatchRequest) -> Result<ClassificationSignals, ClassifierError>;
}

#[async_trait]
pub trait PostCallClassifierPort: Send + Sync + 'static {
    async fn classify(&self, request: &DispatchRequest, output: &DispatchOutput)
                      -> Result<ClassificationSignals, ClassifierError>;
}

#[async_trait]
pub trait RefusalBaselinePort: Send + Sync + 'static {
    async fn evaluate(&self, signals: &ClassificationSignals, pack: Pack)
                      -> Result<Option<RefusalDecision>, BaselineError>;
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-guardrails-kernel
```

## Next IP

[`IP-009-kernel-audit-tap.md`](IP-009-kernel-audit-tap.md)

## References

- `microservices/intelligence/policy/refusal-baseline.cedar`.
- `microservices/intelligence/policy/eu-ai-act-high-risk.cedar`.
- `microservices/intelligence/policy/abuse-defence.cedar`.

## Wave 15 substance conversion — guardrail kernel

### §A Problem

The guardrail stack is the only boundary preventing model dispatch from becoming provider-default safety.
The old slice named a stack but did not prove how refusal, abuse defence, EU AI Act, and pack overlays compose.

### §B Approach

Create pure kernel ports for pre-call and post-call checks, with Cedar policy evaluation outside provider adapters
and refusal decisions feeding IP-003 taxonomy.
The kernel returns typed decisions; it does not log raw prompts or mutate tenant state.

### §C Deliverables

- `crates/oya-intelligence-guardrails-kernel/src/guardrail_stack.rs`
- `pre_call.rs`, `post_call.rs`, `classifier_port.rs`, and `policy_eval_port.rs`
- tests covering abuse, Annex III, minor, and cost-cap refusal paths

### §D Implementation

1. Evaluate emergency-services bypass before abuse score gates.
2. Evaluate tenant/audience/pack authorization before provider routing.
3. Run pre-call classifiers for prompt injection, CSAM, violence, and minor unsafe content.
4. Run post-call checks for unsafe completion and citation/cost disclosure failures.
5. Return `RefusalDecision` with gate id, policy hash, and pack overlay.
6. Emit only metadata and hashes to audit tap.

### §E Acceptance

Nextest and `refusal-baseline-floor` must prove every Cedar refusal rule has a matching domain reason and an audit
event path.

### §F Evidence

Local anchors: `policy/refusal-baseline.cedar`, `policy/eu-ai-act-high-risk.cedar`,
`policy/abuse-defence.cedar`, and `runbooks/prompt-injection-detected.md`.

### §G Counterparts

OpenAI, Anthropic, and Google expose provider safety layers; oyatie closes the enterprise gap by putting Cedar,
pack overlays, and audit evidence before and after provider execution.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-008-kernel-guardrail-stack.md` matched `cost`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
