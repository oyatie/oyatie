---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-015-kernel-guardrail-eu-ai-act
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0105]
---

# IP-015: Kernel — EU AI Act Annex III high-risk refusal wiring

## Intent

Wire the EU AI Act Annex III high-risk refusal layer into `oya-intelligence-guardrails-kernel`.
Implements the `AnnexIiiRefusalPort` trait. Backed by `policy/eu-ai-act-high-risk.cedar`.
When `pack-eu` active + request classifies as Annex III high-risk use case → RefusalDecision
with human-oversight queue routing per Art. 14.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-guardrails-kernel/src/annex_iii.rs` | create |
| `crates/oya-intelligence-guardrails-kernel/src/annex_iii_classifier.rs` | create |
| `crates/oya-intelligence-guardrails-domain/src/annex_iii_refusal.rs` | create |

## Code shape

```rust
pub trait AnnexIiiRefusalPort: Send + Sync {
    /// Returns Some(RefusalDecision) if the request falls under Annex III.
    /// Returns None if not high-risk.
    async fn evaluate(&self, envelope: &DispatchEnvelope)
        -> Result<Option<RefusalDecision>, GuardrailError>;
}

pub enum AnnexIiiCategory {
    BiometricIdentification,
    CriticalInfrastructure,
    Education,
    Employment,
    EssentialServices,
    LawEnforcement,
    MigrationAsylumBorderControl,
    JusticeElections,
}
```

## Key implementation notes

- Cedar evaluation: `policy/eu-ai-act-high-risk.cedar` is the gate; Rust trait calls library-first `oya-shared-policy-eval`.
- Classification: per-request intent classifier (lightweight ML + rule-based) determines Annex III category from prompt + system-prompt.
- Human-oversight queue: `RefusalDecision::AnnexIiiHighRisk { category, queue_id }` routes to the human-review surface.
- Audit: emit `EuAiActAnnexIiiRefusalEmitted` per ADR-0263.
- Transparency: Art. 13 obligation — brand-ux-surface renders `RefusalBanner` with category explanation.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-guardrails-kernel -- annex_iii
buck2 build //:quality-lane-registry-authority-check # lane=eu-ai-act-annex-iii-refusal --microservice intelligence
```

## Test plan

- retired-advanceden set: 50 prompts per Annex III category; verify classification accuracy ≥ 95%.
- Non-high-risk prompts: verify no false-positive refusals (false-positive SLO ≤ 2%).
- Audit emission: every refusal emits sealed event.

## References

- `microservices/intelligence/policy/eu-ai-act-high-risk.cedar`.
- `microservices/intelligence/compliance.md §pack-overlay-roster`.
- EU AI Act 2024/1689 Annex III.
- ADR-0255 §9 (EU AI Act posture).

## Wave 15 substance conversion — EU AI Act high-risk refusal

### §A Problem

The architecture promises EU AI Act Annex III handling, but the guardrail kernel needs a concrete classifier and
refusal path for high-risk use.
This IP closes the gap between policy text, refusal taxonomy, and auditable high-risk dispatch denial.

### §B Approach

Add Annex III classifier logic in the guardrails kernel and bind it to `policy/eu-ai-act-high-risk.cedar`.
The classifier returns typed refusal reasons that IP-003 serializes and IP-009 audits.

### §C Deliverables

- `crates/oya-intelligence-guardrails-kernel/src/eu_ai_act.rs`
- Annex III category fixtures mapped to `RefusalReason`
- golden-set rows consumed by IP-021

### §D Implementation

1. Encode all Annex III categories as explicit enum values.
2. Evaluate tenant pack and purpose before provider routing.
3. Refuse high-risk dispatch lacking required RFIA metadata.
4. Preserve non-high-risk prompts to protect false-positive SLO.
5. Emit refusal decisions with category, policy hash, and evidence id.
6. Add eval cases for each category and clean negatives.

### §E Acceptance

The Annex III gate must prove category coverage, false-positive control, and sealed audit emission for every
refusal.

### §F Evidence

Local anchors: `policy/eu-ai-act-high-risk.cedar`, `compliance.md`, refusal SLOs, and ADR-0255 EU posture.

### §G Counterparts

OpenAI, Anthropic, and Google offer provider safety controls, but oyatie closes the EU enterprise gap with
platform-owned Annex III classification, refusal, and audit evidence.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-015-kernel-guardrail-eu-ai-act.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-015-kernel-guardrail-eu-ai-act.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
