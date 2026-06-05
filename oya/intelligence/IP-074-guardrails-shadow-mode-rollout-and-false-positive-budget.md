---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-014-shadow-mode-rollout-and-false-positive-budget
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, shadow-enforce-promotion-readiness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Shadow→enforce rollout + per-tenant false-positive escalation budget

## Intent

Implement the shadow→enforce rule rollout primitive per ADR-0114 precedent. New rule lands in `shadow` status; engine emits shadow decisions to observability + foundry-evidence WITHOUT affecting live invocations; after ≥ 7d shadow + shadow-vs-enforce-delta review sign-off, promote-to-enforce LEAN lane permits status transition. Per-tenant FP escalation budget enforced in content-safety-rule-engine usecase.

## ChangeSet boundary

Add `shadow_mode` field to `RuleDefinition`; add shadow-runner orchestrator in `content-safety-rule-engine-usecase`; add FP budget tracker in `content-safety-rule-engine-usecase`; new LEAN lane `shadow-enforce-promotion-readiness` in dev-cli; rule-author dashboard entry in observability dashboards.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-content-safety-rule-engine-usecase/src/{shadow_runner.rs,fp_budget.rs}` | create |
| `crates/oya-dev-cli/src/foundation_audit_gates.rs` | update — `validate_shadow_enforce_promotion_readiness()` |
| `dashboards/shadow-vs-enforce-delta.json` | create |
| `dashboards/false-positive-rate.json` | already in dashboards list; populate via this IP |
| `policy/guardrail-enforcement.md` | update — shadow-mode duration default + FP budget tiers |

## Code Shape

```rust
// shadow_runner.rs
pub async fn run_shadow_evaluation(
    rule: &RuleDefinition,
    prompt: &Prompt,
    live_verdict: &Verdict,
    emitter: &dyn GuardrailDecisionEmitter,
) -> Result<(), UsecaseError> {
    let shadow_verdict = evaluate_with_rule(rule, prompt).await?;
    let delta = shadow_verdict.is_definite() && (shadow_verdict != *live_verdict);
    if delta {
        // emit shadow-vs-enforce delta event without affecting live
        emitter.emit_shadow_delta(rule, prompt, live_verdict, &shadow_verdict).await?;
    }
    Ok(())
}

// fp_budget.rs
pub struct FpBudget {
    pub tenant_id: String,
    pub used_this_month: u32,
    pub total_this_month: u32,
}

impl FpBudget {
    pub fn mark_false_positive(&mut self, decision_id: &str) -> Result<MarkResult, FpBudgetError> {
        if self.used_this_month >= self.total_this_month {
            return Err(FpBudgetError::Exceeded);
        }
        self.used_this_month += 1;
        Ok(MarkResult { remaining: self.total_this_month - self.used_this_month })
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-content-safety-rule-engine-usecase --all-features
cargo nextest run -p oya-foundry-guardrails-content-safety-rule-engine-usecase --all-features
buck2 build //:quality-lane-registry-authority-check # lane=shadow-enforce-promotion-readiness --rule <rule-id>
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_shadow_decision_emitted` | shadow runner emits delta event |
| `test_shadow_does_not_affect_live` | live invocation unaffected by shadow rule |
| `test_promote_refused_without_7d` | early promote attempt → refused |
| `test_fp_budget_exhausted` | over-budget mark → error |
| `test_fp_budget_resets_monthly` | rollover ok |

## Halt Conditions

- Shadow affects live invocation — refactor.
- FP budget bypassable — refactor.

## Next IP

[`IP-015-sdk-rust-and-typescript.md`](IP-015-sdk-rust-and-typescript.md)

## References

- ADR-0114 (canary observability rollback precedent).
- `policy/guardrail-enforcement.md`.
- ADR-0139 (shadow→enforce gate model).

## Wave 15 counterpart anchor

- Counterparts: AWS Bedrock Guardrails, OpenAI Moderation, Anthropic safety tooling, and NVIDIA NeMo Guardrails.
- Gap closure: this IP closes inline prompt, output, autonomy, jailbreak, and false-positive-budget enforcement before tenant-visible release.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
