---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-guardrails
deciders: ops-sre-reliability, axis-foundry-guardrails, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-guardrails/cost-budget.md
  - microservices/foundry-guardrails/multi-region.md
  - microservices/foundry-guardrails/policy/tenant-isolation.md
  - microservices/foundry-guardrails/PRD.md (Performance)
review_cadence: quarterly + on every classifier-model rollout + on every Cedar bundle change
doc_status: published
---

# Capacity Sizing Model (foundry-guardrails µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for every Layer-A component (Cedar engine + classifier-model serving + Postgres rule store + Cosign + object storage) and Layer-B component (`oya-foundry-guardrails-*-{kernel..app}`). Drives `cost-budget.md` and `multi-region.md`. Numbers cite ONNX-runtime + Postgres + Cedar public benchmarks; verify-at-deploy markers called out where upstream may have moved.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants per pack | `N_tenants` | OpenBao tenant-resolver |
| Per-tenant invocations/sec | `I_inv_per_sec_per_tenant` | tier-bound; trial 10 RPS / production 100 RPS / sandbox 5 RPS / internal 1000 RPS |
| Total pack invocations/sec | `I_pack_total = sum across tenants` | derived |
| Pre-invocation classification depth (heuristic + classifier + LLM-judge%) | `D_class` | default: heuristic always + classifier always + LLM-judge for 5% (ambiguous) |
| Post-output validation depth | `D_validate` | default: heuristic always + classifier 50% + LLM-judge 2% |
| Cedar evaluations per invocation | `C_cedar` | default: 4 (autonomy-ceiling + tenant-scope + pack-rules + entitlement) |
| Rule-store lookups per invocation | `R_rule` | default: 2 (per-tenant rules + per-pack base rules) |

## Classifier-Model Serving Sizing

### Formulae

```
classifier_qps_pack         = I_pack_total × (D_class.classifier_pct + D_validate.classifier_pct)
classifier_qps_per_pod      = 1000 (single-model BERT-class baseline; 5000 burst with p99 ≤ 80ms)
classifier_replicas_pack    = ceil(classifier_qps_pack / 1000) × 1.3 (buffer) × 1.2 (HA)

# Multi-model serving: 1 pod per model is naive; per pack we run:
#   - 1 PII/PHI classifier (BERT-base, ~110M params, ONNX int8)
#   - 1 Jailbreak classifier (BERT-base or Llama-Guard-class, ~1B params, ONNX int8)
#   - 1 Content-safety classifier (multi-label; Llama-Guard-class)
#   - 1 AI-slop classifier (small heuristic + BERT-small)
# Total models per pack = 4; replicas per model independent.
```

### Reference-architecture baselines

| Scale tier | N_tenants | I_pack_total | classifier_qps_pack | replicas per model |
|---|---|---|---|---|
| **XS** (M01 launch; ~5-20 tenants) | 20 | 2k RPS | 1k RPS classifier | 4 / model × 4 models = 16 pods |
| **S** (~100 tenants) | 100 | 10k RPS | 5k RPS | 8 / model × 4 = 32 pods |
| **M** (~1k tenants) | 1000 | 100k RPS | 50k RPS | 70 / model × 4 = 280 pods |
| **L** (~10k tenants) | 10000 | 1M RPS | 500k RPS | 700 / model × 4 = 2800 pods |

Per-pack: each pack has its own classifier-serving cluster sized at active-tenants-in-pack tier. DR-pair packs add 0.6× warm-standby.

References: ONNX Runtime production deployments — `onnxruntime.ai/docs/performance/`; Hugging Face Text Classification API benchmarks; Microsoft DeepSpeed-MII inference benchmarks. Verify-at-deploy: 2026-05-17 numbers.

### LLM-judge fallback sizing

LLM-judge invocations go through foundry-providers, not in-cluster. Per-tenant budget: 100/hour soft / 500/hour hard. Total pack budget ≈ N_tenants × avg-rate × usage-coefficient.

```
llm_judge_invocations_per_hour_pack = N_tenants × (I_inv_per_sec_per_tenant × 3600 × 0.05)
# 5% ambiguous fraction in classify; 2% in validate
# Per-pack: capped by foundry-providers budget per tenant
```

## Cedar Engine Sizing

```
cedar_eval_qps_per_pod   = 5000 (single-pod Cedar v4 baseline; CPU-bound)
cedar_eval_qps_pack      = I_pack_total × C_cedar
cedar_replicas_pack      = ceil(cedar_eval_qps_pack / 5000) × 1.3 × 1.2

# Sidecar pattern: Cedar engine runs in-process within rest/worker pods.
# Standalone evaluator pool also exists for batch-mode rule evaluations.
```

Reference baselines (per pack):

| Tier | Cedar evaluations/sec | Cedar standalone replicas (for batch) |
|---|---|---|
| XS | 8k | 2 |
| S | 40k | 4 |
| M | 400k | 20 |
| L | 4M | 200 |

References: AWS Cedar engine benchmarks — `aws.amazon.com/blogs/security/iam-cedar-policy-language/`; Cedar v4 performance docs.

## Postgres Rule-Store Sizing

```
postgres_writes_per_day  = (rule_mutations_per_day_pack + cedar_fragment_mutations_per_day)
postgres_reads_per_sec   = I_pack_total × R_rule
postgres_primary_qps     = postgres_reads_per_sec / 3 (with 2 read replicas)

# Reads scale with read replicas; writes through primary.
# Per-pack: 1 primary + 2 read replicas (HA);
#   - XS: r6g.large instances (4 vCPU / 16 GB RAM)
#   - S: r6g.xlarge (8 vCPU / 32 GB)
#   - M: r6g.4xlarge (16 vCPU / 128 GB)
#   - L: r6g.8xlarge (32 vCPU / 256 GB) + horizontal sharding by tenant
```

Storage per pack: ~10 GB at XS (rule defs + mutation log); ~100 GB at S; ~1 TB at M; ~10 TB at L. Mostly mutation log + Cedar fragment history.

References: Postgres on OCI baseline — `oracle.com/cloud/database/`; AWS RDS Postgres benchmarks.

## Layer-B Sizing (oya-foundry-guardrails-*)

```
rest_replicas_pack       = max(4, ceil(I_pack_total / 500)) × 1.3 (HA)
worker_replicas_pack     = max(2, ceil(rule_hot_reload_qps / 100)) × 2 (HA)
app_replicas_pack        = 2 (HA composition root)
```

For M01 launch (pack-kr only, ~20 tenants):
- rest_replicas: 4 (HA min)
- worker_replicas: 2
- app_replicas: 2

## Headroom + Burst

- **Pre-warmed pool**: 2 standby pods per classifier model. Cold-start budget ≤ 500ms per ADR-0020.
- **HPA**: scales on CPU > 70% OR p99 latency > 40ms (classifier) / > 80ms (rest); ratchet up 2 replicas per scale-out event.
- **VPA**: for non-critical batch components.

## Per-Pack Multipliers

- **DR-pair packs** (pack-eu / pack-us / pack-au / pack-in / pack-br / pack-ae / pack-ksa): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention; isolated; dedicated HIPAA-eligible region).
- **Single-region packs** (pack-kr / pack-jp / pack-sg): 1.0× base.

## Storage Costs (per pack region)

```
Classifier-model artifacts (Cosign-signed; ONNX int8):
  - PII/PHI:           ~250 MB / version × 6 versions retained = 1.5 GB
  - Jailbreak:         ~500 MB / version × 6 versions retained = 3 GB
  - Content-safety:    ~500 MB / version × 6 versions retained = 3 GB
  - AI-slop:           ~50 MB  / version × 6 versions retained = 300 MB
  Total: ~8 GB / pack
  Object-storage cost: < $1/month

Postgres storage (per pack):
  XS: 10 GB; S: 100 GB; M: 1 TB; L: 10 TB
```

### Worked example: oyatie XS tier (M01 launch; 20 tenants pack-kr-only)

```
I_pack_total = 2000 RPS
classifier_qps_pack = 1000 RPS (heuristic always; classifier always; LLM-judge 5%)
classifier replicas = 4 / model × 4 models = 16 pods (r6g.large, 4 vCPU each)
cedar_eval_qps_pack = 8000
cedar engine: 2 standalone + sidecar-in-rest
rest_replicas = 4
worker_replicas = 2
app_replicas = 2
postgres: 1 primary r6g.large + 2 read replicas r6g.large

Total compute pods: 16 classifier + 2 cedar + 4 rest + 2 worker + 2 app + 3 postgres ≈ 29 pods + helpers
Storage: ~10 GB Postgres + ~8 GB classifier artifacts ≈ ~$25/month storage
```

Cost projections in `cost-budget.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice foundry-guardrails` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate `I_inv_per_sec_per_tenant` averages.
- Annual classifier-model benchmark refresh: re-verify ONNX-runtime baseline against current models.

## References

- ONNX Runtime — `onnxruntime.ai`.
- AWS Cedar v4 — `aws.amazon.com/security/cedar/`.
- Postgres on OCI — `oracle.com/cloud/database/`.
- Cosign — `docs.sigstore.dev/cosign/`.
- `microservices/foundry-guardrails/cost-budget.md`.
- `microservices/foundry-guardrails/multi-region.md`.
- `microservices/foundry-guardrails/policy/tenant-isolation.md`.
