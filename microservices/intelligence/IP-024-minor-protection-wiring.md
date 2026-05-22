---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-024-minor-protection-wiring
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0292, ADR-0255, ADR-0244]
---

# IP-024: Minor-protection wiring (ADR-0292)

## Intent

Wire ADR-0292 minor-user doctrine into the intelligence dispatch pipeline. If the principal
is classified as `audience_type = MINOR_TARGETED` (age < 13 COPPA or 14-17 KOSA tier),
apply the enhanced refusal baseline and refuse unsafe content classes unconditionally.
Crisis-line bypass for minors remains (documentation-rigor §3.2.5 row 9).

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-guardrails-kernel/src/minor_protection.rs` | create |
| `crates/oya-intelligence-guardrails-usecase/src/minor_dispatch.rs` | create |

## Refusal rules for MINOR_TARGETED audience

| Rule | Age tier | Refusal class |
|---|---|---|
| Any adult-content generation | < 13 AND 14-17 | `MinorProtectionRefusal::AdultContent` |
| Violence / graphic content | < 13 AND 14-17 | `MinorProtectionRefusal::Violence` |
| Gambling-related content | < 13 AND 14-17 | `MinorProtectionRefusal::Gambling` |
| Substance / drug content | < 13 AND 14-17 | `MinorProtectionRefusal::Substance` |
| Social-comparison harmful content (KOSA) | 14-17 | `MinorProtectionRefusal::KosaHarmful` |
| Data collection beyond minimum necessary | < 13 | `MinorProtectionRefusal::CoppaDataMinimum` |

## Crisis-line exception (§3.2.5 row 9)

A minor's safety report / crisis-line dispatch (`audience_type = EMERGENCY_SERVICES + bot_class = CRISIS_TRIAGE`)
MUST pass through regardless of age tier. Parental-consent wall is forbidden on mandatory-reporting paths.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-guardrails-kernel -- minor_protection
cargo run -p oya-dev-cli -- gate validate minor-protection-refusal-coverage --microservice intelligence
cargo run -p oya-dev-cli -- gate validate minor-crisis-line-bypass --microservice intelligence
```

## References

- ADR-0292 (minor user doctrine).
- `microservices/intelligence/compliance.md §minor-protection`.
- `microservices/intelligence/policy/critical-path-emergency-services.cedar`.
- documentation-rigor §3.2.5 row 9 (child safety + mandatory reporting).

## Wave 15 substance conversion — minor protection dispatch path

### §A Problem

The dispatch surface serves `MINOR_TARGETED` callers according to `manifest.json`, but the old slice only named
minor protection without proving where it enters the dispatch pipeline.
This IP closes the gap between identity/audience claims, refusal taxonomy, emergency-services bypass, and provider
selection for child and teen users.
The core risk is accidental provider dispatch of unsafe content because a caller treated minor status as UI metadata
instead of a policy input.

### §B Approach

Add a guardrail kernel module that converts age/audience claims into refusal decisions before credential resolution.
The module consumes `audience_type = MINOR_TARGETED`, age tier, pack overlay, and crisis context.
It shares the existing refusal taxonomy from IP-003 and respects the emergency-services life-safety bypass in
`policy/critical-path-emergency-services.cedar`.

### §C Deliverables

- Create `crates/oya-intelligence-guardrails-kernel/src/minor_protection.rs`.
- Create `crates/oya-intelligence-guardrails-usecase/src/minor_dispatch.rs`.
- Add tests for COPPA, teen KOSA-style harmful-content, and mandatory-reporting bypass paths.
- Bind new reasons into `RefusalReason` coverage and `policy/refusal-baseline.cedar` fixtures.
- Add eval cases to `data/eval/intelligence/refusal-canonical-set.jsonl`.

### §D Implementation

1. Require identity/session claims to include audience type and age tier for minor-targeted contexts.
2. Refuse adult, gambling, substance, graphic violence, and harmful social-comparison classes before provider call.
3. Apply stricter data-minimization logging for minor calls in the audit tap.
4. Preserve crisis-line and mandatory-reporting dispatch when emergency attestation is valid.
5. Return typed `MinorProtectionRefusal` variants that serialize through `RefusalDecision`.
6. Add canonical-set rows so model/provider upgrades cannot weaken the refusal floor.
7. Verify no minor path can select a provider before minor guardrail evaluation succeeds.

### §E Acceptance

Acceptance requires `minor-protection-refusal-coverage`, `minor-crisis-line-bypass`, and nextest coverage for the
kernel module.
The proof set must include one rejected unsafe minor dispatch, one permitted crisis dispatch, and one audit record
without raw prompt leakage.

### §F Evidence

Local anchors: `policy/critical-path-emergency-services.cedar`, `policy/refusal-baseline.cedar`,
`slos/refusal-false-negative-rate.openslo.yaml`, `runbooks/assist-draft-policy-refusal.md`.
Doctrine anchors: ADR-0292, ADR-0255, ADR-0244, ADR-0263.

### §G Counterparts

| Counterpart | Relevant behaviour | Oyatie closure |
|---|---|---|
| OpenAI safety policies | Provider-level minor safety controls | Move minor protection into tenant-scoped platform guardrails before provider call |
| Anthropic constitutional safety | Model-side refusal behaviour | Add deterministic policy evidence and audit records around refusal |
| Google AI safety classifiers | Safety classification gates | Bind classification to Cedar, pack overlays, and emergency-services bypass |
