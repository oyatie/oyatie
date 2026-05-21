---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-021-eval-canonicalen-set
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-nextest]
related_adrs: [ADR-0255, ADR-0105]
---

# IP-021: Eval — canonicalen-set regression suite

## Intent

Ship the `oya-intelligence-eval-kernel` canonicalen-set: per-BC eval records covering dispatch
correctness, refusal accuracy (false-positive + false-negative), streaming fidelity, and
attribution accuracy. Wired into CI as `oya-intelligence-eval-regression` lane.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-eval-kernel/src/canonicalen_set.rs` | create |
| `crates/oya-intelligence-eval-kernel/src/eval_runner.rs` | create |
| `crates/oya-intelligence-eval-worker/src/main.rs` | create |
| `data/eval/intelligence/refusal-canonicalen-set.jsonl` | create |
| `data/eval/intelligence/dispatch-canonicalen-set.jsonl` | create |

## retired-advanceden-set composition (minimum)

| Category | Records | Pass threshold |
|---|---:|---|
| Refusal true-positive (should refuse) | 200 | ≥ 99.5 % |
| Refusal true-negative (should not refuse) | 500 | ≥ 98.0 % |
| Annex III high-risk classification | 50 per category × 8 | ≥ 95.0 % |
| CSAM/violence/extremism block | 100 | 100 % (zero tolerance) |
| Streaming chunk order + completeness | 50 | 100 % |
| Attribution citation accuracy | 100 | ≥ 90 % |

## Key implementation notes

- retired-advanceden set is versioned in `data/eval/intelligence/`; updated each model-provider release.
- Eval runner calls in-process dispatch SDK (IP-019) with mock provider adapters.
- Results emitted as `EvalRunCompleted` audit event per ADR-0263.
- CI lane gates on refusal-false-negative-rate SLO (≤ 0.1 %).

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-eval-kernel -- canonicalen_set
cargo run -p oya-dev-cli -- gate validate eval-canonicalen-set-pass-rate --microservice intelligence
```

## References

- `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml`.
- `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml`.
- ADR-0255 §C (quality bar).
- IP-015 (Annex III refusal wiring).

## Wave 15 substance conversion — eval regression corpus

### §A Problem

The substrate can route and refuse, but without a versioned golden set it cannot prove model/provider changes
preserve dispatch correctness, refusal accuracy, streaming fidelity, or attribution quality.
This IP closes the verification gap behind the SLO files for false-positive and false-negative refusal rates.

### §B Approach

Create an eval kernel and worker that run `DispatchRequest` fixtures through the library-first SDK from IP-019
using mock providers and captured policy states.
The corpus is stored under `data/eval/intelligence/` and each run emits `EvalRunCompleted` through the audit tap.

### §C Deliverables

- `crates/oya-intelligence-eval-kernel/src/golden_set.rs`
- `crates/oya-intelligence-eval-kernel/src/eval_runner.rs`
- `crates/oya-intelligence-eval-worker/src/main.rs`
- `data/eval/intelligence/refusal-golden-set.jsonl`
- `data/eval/intelligence/dispatch-golden-set.jsonl`

### §D Implementation

1. Version every golden-set row with pack, audience, modality, expected class, and policy corpus hash.
2. Run refusal true-positive and true-negative rows through `policy/refusal-baseline.cedar`.
3. Exercise Annex III rows through `policy/eu-ai-act-high-risk.cedar`.
4. Validate stream chunk ordering against IP-016/IP-017 transport outputs.
5. Score citation accuracy using `AttributionGraph` from IP-006.
6. Emit eval summaries as audit evidence rather than mutable CI prose.

### §E Acceptance

The eval gate must fail on refusal false negatives above `slos/refusal-false-negative-rate.openslo.yaml` and must
record a run id, corpus version, provider set, and policy hash.

### §F Evidence

Local anchors: `capabilities/eval.yaml`, refusal SLOs, `policy/refusal-baseline.cedar`, `policy/eu-ai-act-high-risk.cedar`.

### §G Counterparts

OpenAI Evals, Anthropic eval tooling, and Google Vertex evaluation all prove provider quality; oyatie adds tenant,
pack, Cedar, and sealed audit context around the same regression discipline.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-021-eval-golden-set.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-021-eval-golden-set.md` matched `attribution`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
