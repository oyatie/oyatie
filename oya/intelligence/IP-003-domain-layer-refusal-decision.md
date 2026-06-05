---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-003-domain-layer-refusal-decision
status: pending
owner: axis-intelligence + council-privacy
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-refusal-baseline-floor]
---

# IP-003: Domain layer — RefusalDecision entity + reason taxonomy

## Intent

Author the `RefusalDecision` entity and the closed `RefusalReason` enum in the
`oya-intelligence-guardrails-domain` crate. The enum is the canonical taxonomy; mutation requires
2-person-rule + ops-legal sign-off per pack-eu + multispectrum-review v2.4.0.

## Concrete file targets

| Path | Action |
|---|---|
| `microservices/intelligence/crates/oya-intelligence-guardrails-domain/Cargo.toml` | create |
| `microservices/intelligence/crates/oya-intelligence-guardrails-domain/src/lib.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-guardrails-domain/src/refusal_decision.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-guardrails-domain/src/refusal_reason.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-guardrails-domain/src/pack_overlay.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-guardrails-domain/tests/reason_taxonomy_coverage.rs` | create |

## Code shape

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusalDecision {
    pub decision_id: Ulid,
    pub envelope_id: Ulid,
    pub reason: RefusalReason,
    pub gate: GateId,
    pub pack_overlay_applied: Option<Pack>,
    pub copy_localized: Option<RefusalCopy>,
    pub decided_at: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    Csam,
    Violence,
    SelfHarm,
    Extremism,
    Coppa,
    EuAiActAnnexIiiCat1,
    EuAiActAnnexIiiCat2,
    EuAiActAnnexIiiCat3,
    EuAiActAnnexIiiCat4,
    EuAiActAnnexIiiCat5,
    EuAiActAnnexIiiCat6,
    EuAiActAnnexIiiCat7,
    EuAiActAnnexIiiCat8,
    PciScopeRefused,
    DataResidencyViolation,
    CostCapExceeded,
    CredentialUnavailable,
    ProviderSaturated,
    ConsentMissing,
    ModalityBudgetExceeded,
    PhiProviderNotBaa,
    GdprArt9LawfulBasisMissing,
    PipaArt23ConsentMissing,
    PromptInjectionDetected,
    RfiaRequired,
}
```

## Acceptance gates

```bash
cargo check  -p oya-intelligence-guardrails-domain
cargo clippy -p oya-intelligence-guardrails-domain -- -D warnings
cargo nextest run -p oya-intelligence-guardrails-domain
buck2 build //:quality-lane-registry-authority-check # lane=refusal-reason-coverage --microservice intelligence
```

## Test plan

- Round-trip serialise/deserialise across all enum variants.
- `reason_taxonomy_coverage` test ensures the enum's exhaustive match covers each
  `policy/refusal-baseline.cedar` + `policy/eu-ai-act-high-risk.cedar` rule.

## Halt conditions

- Removing a variant fails the `oya-governance-refusal-baseline-floor` lane.

## Next IP

[`IP-004-domain-layer-routing-decision.md`](IP-004-domain-layer-routing-decision.md)

## References

- ADR-0255, EU AI Act Annex III, GDPR Art. 9, KR PIPA Art. 23.
- `microservices/intelligence/policy/refusal-baseline.cedar`.
- `microservices/intelligence/policy/eu-ai-act-high-risk.cedar`.
