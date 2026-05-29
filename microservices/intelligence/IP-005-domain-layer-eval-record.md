---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-005-domain-layer-eval-record
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

# IP-005: Domain layer — EvalRecord + canonicalen-set schema

## Intent

`EvalRecord` entity + canonicalen-set schema in `oya-intelligence-eval-domain`.

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-eval-domain/Cargo.toml` | create |
| `.../oya-intelligence-eval-domain/src/lib.rs` | create |
| `.../oya-intelligence-eval-domain/src/eval_record.rs` | create |
| `.../oya-intelligence-eval-domain/src/canonicalen_set.rs` | create |
| `.../oya-intelligence-eval-domain/src/eval_method.rs` | create |

## Code shape

```rust
pub struct EvalRecord {
    pub eval_id: Ulid,
    pub envelope_id: Ulid,
    pub score: f64,           // [0.0, 1.0]
    pub method: EvalMethod,
    pub canonicalen_set_version: retired-advancedenSetVersion,
    pub evaluated_at: SystemTime,
}

pub enum EvalMethod {
    retired-advancedenSetMatch,
    JudgeModelScore { judge_model: ModelId },
    ClassifierPass { classifier_id: ClassifierId },
}

pub struct retired-advancedenSetEntry {
    pub input: PromptText,
    pub expected_class: ExpectedOutputClass,
    pub allowed_variations: u32,
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-eval-domain
```

## Test plan

- `score` ∈ [0,1] invariant enforced.
- retired-advanceden-set JSONL round-trip.

## Next IP

[`IP-006-domain-layer-attribution.md`](IP-006-domain-layer-attribution.md)

## References

- `microservices/intelligence/capabilities/eval.yaml`.

## Wave 15 substance conversion — EvalRecord domain

### §A Problem

`capabilities/eval.yaml` names the eval capability, but the domain layer needs a durable record shape before CI,
workers, and audit tap can agree on what an evaluation result means.
This IP closes the gap between one-off test output and tenant/pack-scoped `EvalRecord` evidence.

### §B Approach

Define pure value objects for score, method, golden-set version, and evaluated timestamp in the eval domain crate.
The domain enforces score bounds and serializable corpus versioning without provider I/O.

### §C Deliverables

- `crates/oya-intelligence-eval-domain/src/eval_record.rs`
- `golden_set.rs`, `eval_method.rs`, and JSONL round-trip tests
- error variants for invalid scores, missing corpus version, and unsupported method

### §D Implementation

1. Encode score as a bounded newtype rather than a raw `f64`.
2. Carry `envelope_id` so eval evidence joins back to dispatch and audit tap.
3. Distinguish golden-set, judge-model, and classifier methods.
4. Version corpus rows so provider upgrades cannot rewrite old evidence silently.
5. Serialize rows with stable snake_case names for CI diffability.
6. Keep provider calls in IP-021 worker, outside this domain crate.

### §E Acceptance

Nextest must reject out-of-range scores, prove JSONL round-trip, and ensure every method variant serializes to the
contract used by IP-021.

### §F Evidence

Local anchors: `capabilities/eval.yaml`, refusal SLO files, and `manifest.json` eval bounded context.

### §G Counterparts

OpenAI Evals, Anthropic eval sets, and Vertex evaluation define comparable quality records; oyatie adds sealed,
tenant-scoped dispatch evidence.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-005-domain-layer-eval-record.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-005-domain-layer-eval-record.md` matched `attribution`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
