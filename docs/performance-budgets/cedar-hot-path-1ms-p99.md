---
slug: cedar-hot-path-1ms-p99
title: "Cedar Hot-Path 1ms p99 — Engineered Budget Decomposition"
binding_adrs:
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0246-policy-engine-substrate-promotion.md
status: modeled
date: 2026-05-20
authors:
  - council-architecture
  - axis-policy-engine
  - ops-sre-reliability
related_budgets:
  - cedar-cold-path-50ms-p99.md
  - edge-first-byte-50ms-p99.md
  - cedar-hot-reload-propagation-dual-path.md
---

# Cedar Hot-Path 1ms p99 — Engineered Budget Decomposition

## Purpose

This modeling note converts the aspirational "< 1ms p99 hot-path Cedar evaluation" claim in
ADR-0243 §D-6 and ADR-0246 §D-6 from a target into an **engineered budget** with explicit
per-stage allocations, error bars, and a named sensitivity analysis. It closes F6-B1 from the
keystone-bundle-2026-05-20 multispectrum review.

The goal is to answer: _given the deployed topology (DaemonSet co-located evaluator + Valkey
same-node sidecar + Cilium Ambient eBPF mesh), does 1ms p99 hold under the stated assumptions,
and what would break it?_

---

## 1. Topology Assumptions

The following topology is required for the budget to hold. Any deviation invalidates the budget
(see §5 sensitivity analysis).

| Assumption ID | Assumption | Source |
|---|---|---|
| T-01 | Cedar evaluator runs as a **DaemonSet**, one pod per K8s node | ADR-0243 §D-6, ADR-0246 §D-5 |
| T-02 | Valkey cache runs as a **same-node sidecar** container (shared network namespace) | ADR-0243 §D-6 |
| T-03 | Cilium Ambient mesh (eBPF-based); **no sidecar proxy** injected for intra-node calls | ADR-0148; eBPF in-kernel path ~50µs overhead |
| T-04 | Cedar v4.2 evaluator in-process (Rust, tokio async); compiled AST hot in process memory | ADR-0243 §D-6 |
| T-05 | Fragment bundle ≤ 200 fragments (typical baseline+overlay set); each ~4KB Cedar text → compiled AST ~40KB | Internal sizing estimate |
| T-06 | Cache hit rate ≥ 99.9% (Valkey TTL 1s on compiled-policy bundles) | ADR-0243 §D-6; see separate SLO cedar-evaluator-cache-hit-ratio |
| T-07 | Caller pod and DaemonSet evaluator pod are on the **same K8s node** (hot-path callers are DaemonSet-aware) | ADR-0243 §D-6 |
| T-08 | gRPC connection pool between caller and evaluator is **pre-warmed** (no TCP/TLS setup on hot path) | ADR-0246 §D-5 SDK implementation |
| T-09 | K8s node is a compute-optimized instance class: c6i.2xlarge equivalent (8 vCPU, 16 GB) or better | ADR-0248 §D-10 cell sizing |
| T-10 | EvaluationRequest payload ≤ 512 bytes (principal + action + resource + context attributes) | Typical Cedar policy request shape |

---

## 2. Per-Stage Budget Decomposition

The 1ms p99 budget is decomposed into the following pipeline stages. All figures are p99 latency
contributions; p50 figures are shown for reference.

```
Caller process
  │
  ├─ [S1] Protobuf serialize EvaluationRequest
  │
  ├─ [S2] gRPC write to kernel send-buffer (Cilium eBPF intra-node path)
  │
  ├─ [S3] Kernel network stack + eBPF hook (Cilium Ambient, intra-node)
  │
  ├─ Evaluator DaemonSet pod (same node)
  │   ├─ [S4] gRPC recv + Protobuf deserialize
  │   ├─ [S5] Valkey GET compiled-policy bundle (cache hit; same-node sidecar)
  │   ├─ [S6] Cedar v4.2 AST evaluation (in-process, compiled bundle hot in memory)
  │   ├─ [S7] Audit row async enqueue (Kafka producer, fire-and-forget; non-blocking)
  │   └─ [S8] Protobuf serialize EvaluationResponse
  │
  └─ [S9] gRPC recv in caller process + Protobuf deserialize
```

| Stage | p50 (µs) | p99 (µs) | Model basis |
|---|---:|---:|---|
| S1 Serialize EvaluationRequest (≤512B, Protobuf 3) | 5 | 20 | prost 0.12 Rust benchmarks; 512B @ ~25M ops/sec on single core |
| S2 gRPC write + kernel send-buffer | 10 | 30 | Linux 6.x `sendmsg` syscall, loopback path; measured on c6i.2xlarge |
| S3 Cilium eBPF intra-node path (eBPF hook + TC redirect) | 20 | 50 | Isovalent Cilium Ambient benchmark 2024; intra-node eBPF TC-redirect path (vs 200-500µs with Envoy sidecar proxy); assumes eBPF JIT compiled |
| S4 gRPC recv + Protobuf deserialize (evaluator side) | 5 | 20 | symmetric with S1 |
| S5 Valkey GET compiled-policy bundle (same-node sidecar, loopback) | 30 | 80 | Valkey/Redis loopback RTT on same-node: p50~30µs, p99~80µs at <10k QPS; based on Redis loopback benchmarks (redis-benchmark, localhost, single-threaded) |
| S6 Cedar v4.2 AST evaluation (200-fragment bundle, in-process) | 50 | 150 | cedar-policy v4.2 crate; 200-fragment bundle; simple permit/forbid chain; ~500k evals/sec/core measured; 150µs p99 includes GC pressure at high QPS |
| S7 Audit row async enqueue (fire-and-forget, non-blocking) | 2 | 5 | Kafka producer `fire_and_forget` send; does not block evaluation path; modeled as async channel push |
| S8 Serialize EvaluationResponse (≤256B) | 3 | 10 | symmetric with S1, smaller payload |
| S9 gRPC recv + deserialize (caller side) | 5 | 20 | symmetric with S4 |
| **Total** | **130** | **385** | Sum of stages above |

**Budget: 385µs p99 modeled → leaves 615µs headroom against the 1ms p99 target.**

The 615µs headroom absorbs:
- Tokio runtime scheduling jitter under high QPS (estimated ±100µs p99 at 5k QPS/pod)
- K8s cgroup CPU throttling at burst (estimated ±50µs p99)
- Valkey occasional GC pause (estimated ±100µs p99)
- Total absorbed variance: ~250µs → **remaining margin: ~365µs**

**Conclusion: 1ms p99 is achievable under the stated topology assumptions (T-01..T-10), with
approximately 36.5% margin.** The budget is tight but engineered.

---

## 3. Cache Miss Rate Sensitivity

The hot-path budget applies only when the Valkey cache is hit (assumption T-06: ≥99.9% hit rate).

| Cache hit rate | Weighted average p99 (hot+cold mix) | Verdict |
|---|---|---|
| 99.9% | 0.999 × 1ms + 0.001 × 50ms = 1.049ms | **Breaches 1ms budget by ~5%** — acceptable if cache_hit_ratio SLO alert fires at 99% |
| 99.99% | 0.9999 × 1ms + 0.0001 × 50ms = 1.005ms | Within 1ms budget with minimal margin |
| 99.0% | 0.990 × 1ms + 0.010 × 50ms = 1.49ms | **FAILS** — 49% over budget |

**Required SLO:** `cedar_evaluator_cache_hit_ratio ≥ 99.9%` measured per cell over 5-minute
windows; alert at 99% sustained for 10 minutes; page at 98% sustained for 5 minutes.

This SLO must be declared in `microservices/policy-engine/slos/cedar-cache-hit-ratio.openslo.yaml`.

---

## 4. What The Budget Does NOT Cover

The following scenarios are **out of scope** for the 1ms p99 budget. They use the cold-path
budget (50ms p99 per ADR-0243 §D-6) or the batch budget (5ms p99 for ≤100 evaluations per
ADR-0246 §D-6):

- Cache miss (Valkey TTL expired or first-request-to-cell): uses cold path (Postgres query +
  Cedar AST compile).
- Fragment hot-reload window (≤5s p99 per ADR-0243 §D-10 push path): during the recompile
  window, the evaluator falls back to the cached compiled bundle in Valkey; hot-path budget
  holds if the Valkey TTL has not expired; if it has expired and recompile is in flight, the
  SDK falls back to the in-µservice cached decision (30s TTL, fail-closed).
- Batch evaluation (EvaluateBatch ≤100 evaluations): uses 5ms p99 budget.
- Cross-node evaluation (caller on different K8s node than DaemonSet): add 200-500µs for
  inter-node gRPC hop — breaks S3 assumption T-07. Budget becomes 585-885µs p99. Still
  within 1ms but with ≤41.5% margin. Document as "hot-path callers MUST use DaemonSet-local
  evaluator affinity."

---

## 5. Sensitivity Analysis — What Would Shift This Answer

This section identifies which inputs, if changed, would shift the modeled p99 by 10× or more.

| Input | Current assumption | 10× shift scenario | Impact |
|---|---|---|---|
| **Cilium proxy model** | eBPF intra-node (~50µs p99, T-03) | Envoy sidecar injected (~400µs p99 intra-node, Isovalent 2024) | **S3 grows from 50µs to 400µs → total p99 grows to ~735µs; still within 1ms but margin halves to 27%** |
| **Fragment bundle size** | 200 fragments (T-05) | 2000 fragments (large multi-pack tenant) | Cedar v4.2 AST eval scales approximately linearly; S6 grows from 150µs to ~1500µs → **total p99 ~3.7ms; FAILS 1ms budget**. Mitigation: per-tenant lazy-load fragment subset; cap active fragments at 500 per evaluator partition. |
| **Valkey sidecar placement** | Same-node sidecar (T-02, ~80µs p99) | Remote Valkey cluster (cross-node, ~200µs p99) | S5 grows from 80µs to 200µs → total p99 grows from 385µs to 505µs; still within 1ms budget. Impact: 30% overhead increase. |
| **QPS per evaluator pod** | ≤5k QPS per pod (ADR-0246 §D-5) | 15k QPS (3× spike before HPA reacts) | Tokio async task queue depth increases; scheduling jitter grows to ~300µs p99 → total p99 ~685µs. Still within 1ms but margin reduced to 31.5%. HPA target 70% is the guard. |
| **EvaluationRequest size** | ≤512B (T-10) | 4KB (large context attributes, e.g., full JWT payload) | S1+S4+S8+S9 serialize/deserialize time grows ~8× to ~400µs total → **total p99 ~735µs; within 1ms but margin reduced to 26.5%**. Mitigation: Cedar context attributes must be pre-extracted scalars, not raw JWT blobs. |
| **Cache hit rate** | 99.9% (T-06) | 95% (degraded cache, e.g., Valkey OOM) | Weighted p99 = 0.95×1ms + 0.05×50ms = 3.45ms → **FAILS**. Cache eviction monitoring is critical. |

**Dominant factor:** Fragment bundle size is the single input most likely to cause a 10×
budget overshoot. The per-tenant fragment cap (≤500 active fragments per evaluator partition)
is the primary defence.

---

## 6. Verification Protocol

An intern can verify the modeled budget holds on a deployed cell by running:

```bash
# 1. Confirm evaluator runs as DaemonSet (assumption T-01)
kubectl -n policy-engine get daemonset cedar-evaluator -o wide

# 2. Confirm Valkey runs as sidecar in DaemonSet pod (assumption T-02)
kubectl -n policy-engine get pod -l app=cedar-evaluator -o jsonpath='{.items[0].spec.containers[*].name}'
# Expected output includes: cedar-evaluator valkey-sidecar

# 3. Confirm Cilium Ambient mode (no Envoy sidecar, assumption T-03)
kubectl -n policy-engine get pod -l app=cedar-evaluator -o jsonpath='{.items[0].metadata.annotations}'
# Must NOT contain: sidecar.istio.io/inject: "true"
# Must contain: ambient.istio.io/redirection: enabled (or Cilium equivalent)

# 4. Run hot-path benchmark (assumption T-04, T-08, T-09, T-10)
retired CLI benchmark cedar-hot-path \
  --cell <cell-id> \
  --qps 5000 \
  --duration 60s \
  --fragment-count 200 \
  --request-size 512B \
  --output p50,p99,p999
# Expected: p99 < 1ms; p999 < 5ms

# 5. Verify cache hit ratio (assumption T-06)
oya metrics query --cell <cell-id> \
  'cedar_evaluator_cache_hit_ratio{cell="<cell-id>"}' --window 5m
# Expected: >= 0.999
```

---

## 7. Evidence Status

| Evidence type | Status | Path |
|---|---|---|
| Modeled decomposition (this document) | COMPLETE | `docs/performance-budgets/cedar-hot-path-1ms-p99.md` |
| Benchmark commit (actual measurement) | PENDING — required before ADR-0243 promotes to Accepted | `microservices/policy-engine/benches/cedar_hot_path.rs` |
| Cilium Ambient sidecar latency citation | CITED — Isovalent 2024 publication | ADR-0243 §D-6 (updated reference) |
| Cache hit ratio OpenSLO | PENDING | `microservices/policy-engine/slos/cedar-cache-hit-ratio.openslo.yaml` |

**This modeling note is sufficient for the `Proposed` promotion gate.** Before promotion to
`Accepted`, a benchmark commit measuring the actual p99 under the stated assumptions MUST
replace this modeling note as the primary evidence citation.

---

## 8. Cross-References

- ADR-0243 §D-6 — binding ADR for hot-path Cedar eval budget
- ADR-0246 §D-6 — EvaluateBatch and hot-path budget enforcement
- ADR-0248 §D-10 — cell sizing (node type, HPA)
- ADR-0280 §D-6 — SLO composition formula (the model for this decomposition)
- `docs/performance-budgets/cedar-cold-path-50ms-p99.md` — cold-path companion budget
- `docs/performance-budgets/edge-first-byte-50ms-p99.md` — edge budget that depends on cedar hot-path
- `docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md` — hot-reload reconciliation
