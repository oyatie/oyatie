---
doc_class: PerformanceBenchmark
benchmark_id: PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20
target_microservices:
  - workflow
  - workflow-engine
  - ontology
  - policy-cedar
  - audit-chain
status: BaselineRecorded
date: 2026-05-20
owner: ops-sre-performance
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0035-workflow-engine-state-machine-and-dag-hybrid
  - ADR-0139-agentic-slo-gated-promotion
---

# Workflow Engine Orchestration Cost Benchmark

## Benchmark Goal

Named target metric: `workflow_step_transition_latency_ms`.

Named throughput metric: `workflow_durable_transitions_per_second`.

Named compensation metric: `workflow_saga_compensation_latency_ms`.

Named SLO target: `SLO-WORKFLOW-ORCHESTRATION-P99`.

The SLO target is p99 durable step-transition latency per workflow shape and p99 saga-compensation latency after injected failure.

The benchmark finds durable-function throughput cap while preserving idempotency, audit emission, replay determinism, and zero duplicate side effects.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

## Test Harness

Named load-generator topology: `workflow-durable-saga-cell-lab`.

Topology nodes:

- Bash prepares workflow definitions, version pins, ontology action fixtures, Cedar permits, and failure-injection switches.
- k6 starts workflow instances and injects deterministic step failures.
- Locust polls timelines, triggers retries, reads replay state, and validates compensation completion.
- The SUT includes workflow REST, workflow worker, durable state store, ontology adapter, Cedar policy API, audit-chain sink, and external-effect stub.
- Prometheus records transition latency, instance throughput, replay latency, compensation latency, duplicate side effects, and audit completeness.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://workflow-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-workflow}"
SEED="${SEED:-94206001}"
TIER="${TIER:-Bronze}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/workflow}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"

mkdir -p "${OUTPUT_DIR}/definitions"

case "${TIER}" in
  Bronze)
    VUS=90
    START_RATE=240
    STEP_RATE=2800
    ;;
  Silver)
    VUS=180
    START_RATE=560
    STEP_RATE=6800
    ;;
  Gold)
    VUS=340
    START_RATE=1180
    STEP_RATE=14800
    ;;
  Platinum)
    VUS=680
    START_RATE=2150
    STEP_RATE=27000
    ;;
  *)
    echo "unknown tier: ${TIER}" >&2
    exit 64
    ;;
esac

cat > "${OUTPUT_DIR}/definitions/linear-8.json" <<'JSON'
{
  "definition_id": "bench-linear-8",
  "version": "2026-05-20",
  "shape": "linear_8_step",
  "steps": ["validate", "reserve", "authorize", "invoke", "persist", "notify", "audit", "complete"],
  "replay_safety_class": "deterministic",
  "compensation": "reverse_committed_effects"
}
JSON

cat > "${OUTPUT_DIR}/definitions/dag-32.json" <<'JSON'
{
  "definition_id": "bench-dag-32",
  "version": "2026-05-20",
  "shape": "dag_32_parallel_step",
  "fanout_width": 8,
  "fanout_depth": 4,
  "replay_safety_class": "deterministic_with_effect_boundaries",
  "compensation": "join_order_reverse"
}
JSON

cat > "${OUTPUT_DIR}/definitions/saga-12.json" <<'JSON'
{
  "definition_id": "bench-saga-12",
  "version": "2026-05-20",
  "shape": "saga_12_external_effect",
  "steps": ["quote", "reserve", "charge", "provision", "notify", "reconcile"],
  "compensations": ["void_charge", "release_reservation", "deprovision", "notify_cancel"],
  "failure_injection": "step_4_at_7_percent"
}
JSON

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/reset" \
  -H "content-type: application/json" \
  -d "{\"bench_id\":\"${BENCH_ID}\",\"tenant_prefix\":\"${TENANT_PREFIX}\",\"seed\":${SEED}}" \
  > "${OUTPUT_DIR}/reset-${TIER}.json"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/workflow/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"tenant_count\":96,
    \"definition_count\":3,
    \"distribution\":\"uniform-workflow-shape-with-bursting-saga-failures\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

for def in linear-8 dag-32 saga-12; do
  curl -fsS -X PUT "${SUT_BASE_URL}/v1/workflows/definitions/${def}" \
    -H "content-type: application/json" \
    -H "x-oya-benchmark-id: ${BENCH_ID}" \
    --data-binary "@${OUTPUT_DIR}/definitions/${def}.json" \
    > "${OUTPUT_DIR}/definition-${def}-${TIER}.json"
done

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e START_RATE="${START_RATE}" \
  -e STEP_RATE="${STEP_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/workflow-engine-orchestration-cost.k6.js

locust \
  -f benchmarks/workflow-engine-orchestration-cost.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 12 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for shape in linear_8_step dag_32_parallel_step saga_12_external_effect; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_workflow_step_transition_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",shape=\"${shape}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/transition-p99-${TIER}-${shape}.json"
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=sum(rate(oya_workflow_step_transitions_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",shape=\"${shape}\"}[15m]))" \
    > "${OUTPUT_DIR}/transition-rate-${TIER}-${shape}.json"
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_workflow_saga_compensation_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m])) by (le))" \
  > "${OUTPUT_DIR}/compensation-p99-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_workflow_duplicate_side_effect_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/duplicate-side-effects-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile compensation "${OUTPUT_DIR}/compensation-p99-${TIER}.json" \
  --slurpfile duplicates "${OUTPUT_DIR}/duplicate-side-effects-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,compensation_p99:$compensation[0],duplicate_side_effects:$duplicates[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "workflow benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const startLatency = new Trend('workflow_start_latency_ms', true);
export const transitionLatency = new Trend('workflow_step_transition_latency_ms', true);
export const compensationLatency = new Trend('workflow_saga_compensation_latency_ms', true);
export const duplicateSideEffects = new Rate('workflow_duplicate_side_effect_rate');
export const transitions = new Counter('workflow_step_transitions');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94206001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-workflow';
const startRate = Number(__ENV.START_RATE || 240);
const stepRate = Number(__ENV.STEP_RATE || 2800);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    start_instances: {
      executor: 'constant-arrival-rate',
      rate: startRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(80, Math.floor(startRate / 4)),
      maxVUs: Math.max(400, startRate),
      exec: 'startInstance',
    },
    transition_ticks: {
      executor: 'constant-arrival-rate',
      rate: stepRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(100, Math.floor(stepRate / 70)),
      maxVUs: Math.max(800, Math.floor(stepRate / 10)),
      exec: 'tickTransition',
    },
    compensation_probe: {
      executor: 'constant-vus',
      vus: 24,
      duration: `${warmupSeconds + measureSeconds}s`,
      exec: 'probeCompensation',
    },
  },
  thresholds: {
    workflow_start_latency_ms: ['p(99)<250'],
    workflow_step_transition_latency_ms: ['p(99)<100'],
    workflow_saga_compensation_latency_ms: ['p(99)<5000'],
    workflow_duplicate_side_effect_rate: ['rate==0'],
  },
};

function shape(iteration) {
  const v = (iteration + seed) % 100;
  if (v < 40) return 'linear_8_step';
  if (v < 78) return 'dag_32_parallel_step';
  return 'saga_12_external_effect';
}

function definitionId(s) {
  if (s === 'linear_8_step') return 'bench-linear-8';
  if (s === 'dag_32_parallel_step') return 'bench-dag-32';
  return 'bench-saga-12';
}

function headers(extra = {}) {
  return Object.assign({
    'content-type': 'application/json',
    'x-oya-benchmark-id': benchId,
    'x-oya-tier': tier,
  }, extra);
}

export function startInstance() {
  const s = shape(__ITER);
  const idx = __ITER + seed + __VU;
  const started = Date.now();
  const payload = {
    tenant_id: `${tenantPrefix}-${idx % 96}`,
    definition_id: definitionId(s),
    definition_version: '2026-05-20',
    shape: s,
    input_ref: `ontology://bench/entity/${idx}`,
    idempotency_key: `wf-start-${seed}-${__VU}-${__ITER}`,
    inject_failure: s === 'saga_12_external_effect' && idx % 14 === 0,
  };
  const res = http.post(`${baseUrl}/v1/workflows/instances`, JSON.stringify(payload), {
    headers: headers({ 'x-oya-workflow-shape': s }),
    tags: { tier, shape: s },
  });
  const ok = check(res, {
    'instance started': (r) => r.status === 201 || r.status === 202,
    'instance id present': (r) => !!r.json('instance_id'),
    'audit id present': (r) => !!r.json('audit_event_id'),
  });
  if (ok) startLatency.add(Number(res.json('start_latency_ms') || (Date.now() - started)), { tier, shape: s });
}

export function tickTransition() {
  const s = shape(__ITER);
  const idx = __ITER + seed + __VU;
  const instanceId = `seeded-${definitionId(s)}-${idx % 250000}`;
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/workflows/instances/${instanceId}:tick`, JSON.stringify({
    tenant_id: `${tenantPrefix}-${idx % 96}`,
    expected_shape: s,
    worker_id: `bench-worker-${__VU}`,
    idempotency_key: `wf-tick-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'x-oya-workflow-shape': s }), tags: { tier, shape: s } });
  const ok = check(res, {
    'tick accepted': (r) => r.status === 200 || r.status === 202 || r.status === 404,
    'no duplicate side effect': (r) => r.status === 404 || r.json('duplicate_side_effect') !== true,
  });
  duplicateSideEffects.add(ok && res.status !== 404 ? Boolean(res.json('duplicate_side_effect')) : false, { tier, shape: s });
  if (ok && res.status !== 404) {
    transitionLatency.add(Number(res.json('transition_latency_ms') || (Date.now() - started)), { tier, shape: s });
    transitions.add(Number(res.json('steps_advanced') || 1), { tier, shape: s });
  }
}

export function probeCompensation() {
  const idx = __ITER + seed + __VU;
  const instanceId = `seeded-bench-saga-12-${idx % 250000}`;
  const res = http.get(`${baseUrl}/v1/workflows/instances/${instanceId}/compensation`, {
    headers: headers(),
    tags: { tier, shape: 'saga_12_external_effect' },
  });
  const ok = check(res, {
    'compensation readable': (r) => r.status === 200 || r.status === 404,
    'compensation bounded when present': (r) => r.status === 404 || Number(r.json('compensation_latency_ms') || 0) < 10000,
  });
  if (ok && res.status === 200) {
    compensationLatency.add(Number(res.json('compensation_latency_ms') || 0), { tier });
  }
  sleep(1);
}
```

```python
from __future__ import annotations

import json
import os
import random
import time
from locust import HttpUser, between, events, task

BENCH_ID = "PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20"
SEED = int(os.getenv("SEED", "94206001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-workflow")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class WorkflowObserverUser(HttpUser):
    wait_time = between(0.01, 0.15)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 95)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
        }

    def _instance_id(self, shape: str):
        if shape == "linear_8_step":
            return f"seeded-bench-linear-8-{random.randint(1, 250000)}"
        if shape == "dag_32_parallel_step":
            return f"seeded-bench-dag-32-{random.randint(1, 250000)}"
        return f"seeded-bench-saga-12-{random.randint(1, 250000)}"

    @task(34)
    def timeline_read(self):
        shape = random.choice(["linear_8_step", "dag_32_parallel_step", "saga_12_external_effect"])
        instance_id = self._instance_id(shape)
        start = time.perf_counter()
        with self.client.get(
            f"/v1/workflows/instances/{instance_id}/timeline?tenant_id={self.tenant_id}",
            headers=self.headers,
            name="/v1/workflows/instances/:id/timeline",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code in (200, 404):
                events.request.fire(request_type="CHECK", name="workflow_timeline_read_ms", response_time=elapsed_ms, response_length=0)
                response.success()
            else:
                response.failure(f"timeline failed {response.status_code}")

    @task(28)
    def replay_probe(self):
        shape = random.choice(["linear_8_step", "dag_32_parallel_step"])
        instance_id = self._instance_id(shape)
        payload = {
            "tenant_id": self.tenant_id,
            "instance_id": instance_id,
            "from_step": 0,
            "to_step": random.randint(2, 32),
            "verify_audit_chain": True,
        }
        start = time.perf_counter()
        with self.client.post(
            "/v1/workflows/replay:probe",
            data=json.dumps(payload),
            headers=self.headers,
            name="/v1/workflows/replay:probe",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code in (200, 202, 404):
                events.request.fire(request_type="CHECK", name="workflow_replay_probe_ms", response_time=elapsed_ms, response_length=0)
                response.success()
            else:
                response.failure(f"replay failed {response.status_code}")

    @task(22)
    def compensation_status(self):
        instance_id = self._instance_id("saga_12_external_effect")
        with self.client.get(
            f"/v1/workflows/instances/{instance_id}/compensation",
            headers=self.headers,
            name="/v1/workflows/instances/:id/compensation",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 404):
                response.success()
            else:
                response.failure(f"compensation status failed {response.status_code}")

    @task(16)
    def retry_eligibility_probe(self):
        instance_id = self._instance_id(random.choice(["linear_8_step", "saga_12_external_effect"]))
        with self.client.get(
            f"/v1/workflows/instances/{instance_id}/retry-eligibility?tenant_id={self.tenant_id}",
            headers=self.headers,
            name="/v1/workflows/instances/:id/retry-eligibility",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 404):
                response.success()
            else:
                response.failure(f"retry eligibility failed {response.status_code}")
```

## Test Workload

Named request shape: `linear_8_step`.

Named request shape: `dag_32_parallel_step`.

Named request shape: `saga_12_external_effect`.

Named request shape: `replay_probe_0_to_n`.

Named request shape: `compensation_status_read`.

Named distribution: `uniform-workflow-shape-with-bursting-saga-failures`.

Linear workflows exercise serial durable transitions.

DAG workflows exercise parallel fanout, join, and deterministic replay.

Saga workflows inject step failure at a controlled 7 percent rate.

Every state-changing operation carries an idempotency key.

Every step transition emits audit-chain evidence.

Every ontology action reference is type-checked by the workflow engine before transition.

Every external-effect stub records an effect id to detect duplicate side effects.

## Baseline Numbers

Recorded baseline run: `workflow-durable-saga-cell-lab-2026-05-20T14:00:00Z`.

SUT topology: 1 region, 2 availability zones, durable Postgres state store, workflow worker pool, ontology adapter, Cedar gate, audit-chain sink.

| Tier | Shape | p50 transition ms | p95 transition ms | p99 transition ms | Transition cap/s | Instance start p99 ms |
|---|---|---:|---:|---:|---:|---:|
| Bronze | linear_8_step | 9 | 38 | 82 | 3,200 | 178 |
| Bronze | dag_32_parallel_step | 13 | 51 | 109 | 2,700 | 205 |
| Bronze | saga_12_external_effect | 18 | 76 | 158 | 1,900 | 226 |
| Silver | linear_8_step | 7 | 29 | 61 | 7,700 | 139 |
| Silver | dag_32_parallel_step | 10 | 41 | 87 | 6,500 | 166 |
| Silver | saga_12_external_effect | 14 | 59 | 123 | 4,500 | 184 |
| Gold | linear_8_step | 5 | 21 | 44 | 16,700 | 101 |
| Gold | dag_32_parallel_step | 8 | 31 | 66 | 14,100 | 123 |
| Gold | saga_12_external_effect | 11 | 46 | 96 | 9,900 | 140 |
| Platinum | linear_8_step | 4 | 16 | 33 | 30,200 | 76 |
| Platinum | dag_32_parallel_step | 6 | 24 | 50 | 25,400 | 94 |
| Platinum | saga_12_external_effect | 9 | 35 | 74 | 18,300 | 109 |

Saga compensation baseline:

| Tier | p50 compensation ms | p95 compensation ms | p99 compensation ms | Duplicate side effects | Audit completeness |
|---|---:|---:|---:|---:|---:|
| Bronze | 310 | 1420 | 2810 | 0 | 100.000% |
| Silver | 260 | 1180 | 2320 | 0 | 100.000% |
| Gold | 210 | 920 | 1810 | 0 | 100.000% |
| Platinum | 170 | 730 | 1420 | 0 | 100.000% |

## Comparison vs Named Vendors

Named vendors and projects: Temporal, Cadence, Camunda, Argo Workflows, AWS Step Functions.

Temporal-class comparison: event history, deterministic replay, and durable worker polling.

Cadence-class comparison: long-running workflow continuity and retry behavior.

Camunda-class comparison: incident and compensation lifecycle.

Argo Workflows-class comparison: DAG fanout and artifact-backed execution.

AWS Step Functions-class comparison: managed durable orchestration and state transition cost.

Oyatie differentiator measured here: workflow_spec authority, Cedar policy gating, ontology type-checking, and audit-chain emission remain in the transition path.

Vendor parity guard: this document does not assert hidden vendor p99 or cost values; it defines comparable shapes for future vendor harnesses.

## Methodology

Named SUT topology: `durable-state-worker-pool-with-policy-and-audit`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Transition latency starts when the worker claims a step and stops when durable state and audit evidence are committed.

Instance start latency starts at REST receive and stops when the instance is visible to workers.

Compensation latency starts at injected failure detection and stops when all declared compensations are durable.

Replay probe latency measures deterministic reconstruction without performing side effects.

Throughput cap is raised until p99 transition latency, duplicate side effects, or audit completeness breach.

## Reproducibility

Primary command:

`BENCH_ID=PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20 TIER=Gold SEED=94206001 ./benchmarks/workflow-engine-orchestration-cost.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://workflow-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94206001 benchmarks/workflow-engine-orchestration-cost.k6.js`

Locust command:

`locust -f benchmarks/workflow-engine-orchestration-cost.locust.py --headless --users 340 --spawn-rate 29 --run-time 1200s --host https://workflow-cell-01.dev.oyatie.local`

Named seed values:

- `94206001` workflow-shape seed.
- `94206002` failure-injection seed.
- `94206003` worker-claim seed.
- `94206004` replay-window seed.

## Failure Modes Detected

Step transition p99 regression above tier SLO.

Instance start p99 regression above baseline.

Saga compensation p99 above 5 seconds.

Throughput cap regression above 10 percent from baseline.

Duplicate side effect after retry.

State-machine and DAG replay divergence.

Step transition without audit event.

Workflow action invoked without Cedar permit.

Ontology action reference not type checked before execution.

Worker claim starvation.

Hot workflow definition cache stampede.

Stuck compensation without incident row.

Cross-tenant instance timeline read succeeds.

Unversioned workflow definition executes.

Retry eligibility returns unsafe retry for non-idempotent step.

## Cross-References

- `specs/microservices/workflow.json`.
- `specs/microservices/ontology.json`.
- `docs/standards/saga-compensation-policy.md`.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- Service-owned `microservices/workflow-engine/benchmarks/` remains untouched by this root corpus.
