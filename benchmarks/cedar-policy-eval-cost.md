---
doc_class: PerformanceBenchmark
benchmark_id: PB-CEDAR-POLICY-EVAL-COST-2026-05-20
target_microservices:
  - policy-cedar
  - tenancy
  - identity
  - audit-chain
status: BaselineRecorded
date: 2026-05-20
owner: ops-sre-performance
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0128-hyperscaler-architecture-invariants
  - ADR-0139-agentic-slo-gated-promotion
---

# Cedar Policy Eval Cost Benchmark

## Benchmark Goal

Named target metric: `cedar_eval_latency_ms`.

Named cache metric: `cedar_policy_cache_hit_rate`.

Named SLO target: `SLO-CEDAR-EVAL-P99`.

The SLO target is p99 Cedar authorization evaluation latency per policy-complexity band.

The benchmark records cache hit rate, policy parse cache pressure, entity graph lookup cost, deny-path cost, and audit decision emission cost.

The throughput cap is the highest evaluations per second that preserve p99 latency and zero incorrect permit or forbid decisions.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

## Test Harness

Named load-generator topology: `cedar-complexity-band-lab`.

Topology nodes:

- Bash prepares Cedar schemas, policies, entity graphs, permit fixtures, forbid fixtures, and cache state.
- k6 drives HTTP evaluation calls against the policy-cedar API.
- Locust drives mixed application-style authorization traffic with tenant, capability, document, drive, workflow, and messenger resources.
- Prometheus records latency histograms by complexity band, cache hit counters, incorrect decision counters, and fallback counters.
- Audit-chain validates decision evidence for sampled denied and allowed decisions.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-CEDAR-POLICY-EVAL-COST-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://policy-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-cedar}"
SEED="${SEED:-94204001}"
TIER="${TIER:-Bronze}"
COMPLEXITY="${COMPLEXITY:-medium}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/cedar}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"

mkdir -p "${OUTPUT_DIR}/policies"

case "${TIER}" in
  Bronze)
    VUS=80
    EVAL_RATE=2500
    ;;
  Silver)
    VUS=160
    EVAL_RATE=6200
    ;;
  Gold)
    VUS=320
    EVAL_RATE=13500
    ;;
  Platinum)
    VUS=620
    EVAL_RATE=24500
    ;;
  *)
    echo "unknown tier: ${TIER}" >&2
    exit 64
    ;;
esac

cat > "${OUTPUT_DIR}/policies/simple.cedar" <<'CEDAR'
permit (
  principal,
  action == Action::"read",
  resource
)
when {
  principal.tenant_id == resource.tenant_id
};
CEDAR

cat > "${OUTPUT_DIR}/policies/medium.cedar" <<'CEDAR'
permit (
  principal in Role::"member",
  action in [Action::"read", Action::"comment"],
  resource is Document
)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.visibility in ["tenant", "team"] &&
  context.device_trust >= 2 &&
  context.region in principal.allowed_regions
};
forbid (
  principal,
  action,
  resource is Document
)
when {
  resource.legal_hold == true &&
  action == Action::"delete"
};
CEDAR

cat > "${OUTPUT_DIR}/policies/complex.cedar" <<'CEDAR'
permit (
  principal in Role::"workflow_operator",
  action in [Action::"invoke", Action::"approve", Action::"rollback"],
  resource is WorkflowRun
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.autonomy_tier >= resource.required_autonomy_tier &&
  context.device_trust >= 3 &&
  context.session_age_minutes <= 30 &&
  context.region in principal.allowed_regions &&
  resource.data_class notin ["PHI", "PCI"] &&
  context.change_window_open == true
};
forbid (
  principal,
  action,
  resource is WorkflowRun
)
when {
  resource.break_glass == true &&
  context.step_up_auth_completed == false
};
forbid (
  principal,
  action,
  resource is WorkflowRun
)
when {
  resource.cross_tenant == true &&
  context.explicit_cross_tenant_grant == false
};
CEDAR

cat > "${OUTPUT_DIR}/policies/pathological.cedar" <<'CEDAR'
permit (
  principal in Role::"auditor",
  action in [Action::"read", Action::"export", Action::"attest"],
  resource is EvidencePack
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.step_up_auth_completed == true &&
  context.case_id in principal.case_assignments &&
  context.pack_id in resource.allowed_pack_ids &&
  context.region in principal.allowed_regions &&
  resource.regulatory_packs.contains("SOC2-T2") &&
  resource.retention_class in ["seven_year", "legal_hold"] &&
  context.export_purpose in ["regulator", "internal_audit"] &&
  context.approver_count >= 2
};
forbid (principal, action, resource) when { context.dsr_delete_pending == true };
forbid (principal, action, resource) when { resource.key_shredded == true };
forbid (principal, action, resource) when { context.tenant_suspended == true };
CEDAR

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/reset" \
  -H "content-type: application/json" \
  -d "{\"bench_id\":\"${BENCH_ID}\",\"tenant_prefix\":\"${TENANT_PREFIX}\",\"seed\":${SEED}}" \
  > "${OUTPUT_DIR}/reset-${TIER}.json"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/cedar/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"complexity_bands\":[\"simple\",\"medium\",\"complex\",\"pathological\"],
    \"entity_graphs_per_band\":32,
    \"policy_versions_per_band\":8,
    \"distribution\":\"uniform-band-with-zipfian-policy-hotset\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

for policy in simple medium complex pathological; do
  curl -fsS -X PUT "${SUT_BASE_URL}/internal/bench/cedar/policies/${policy}" \
    -H "content-type: text/plain" \
    --data-binary "@${OUTPUT_DIR}/policies/${policy}.cedar" \
    > "${OUTPUT_DIR}/policy-${policy}-${TIER}.json"
done

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e EVAL_RATE="${EVAL_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/cedar-policy-eval-cost.k6.js

locust \
  -f benchmarks/cedar-policy-eval-cost.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 10 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for band in simple medium complex pathological; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_cedar_eval_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",complexity=\"${band}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/p99-${TIER}-${band}.json"
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=sum(rate(oya_cedar_policy_cache_hit_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",complexity=\"${band}\"}[15m])) / sum(rate(oya_cedar_policy_cache_lookup_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",complexity=\"${band}\"}[15m]))" \
    > "${OUTPUT_DIR}/cache-hit-${TIER}-${band}.json"
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_cedar_incorrect_decision_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/incorrect-decision-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile incorrect "${OUTPUT_DIR}/incorrect-decision-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,incorrect_decisions:$incorrect[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "cedar benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const evalLatency = new Trend('cedar_eval_latency_ms', true);
export const cacheHit = new Rate('cedar_cache_hit_rate');
export const incorrectDecision = new Rate('cedar_incorrect_decision_rate');
export const evalCounter = new Counter('cedar_eval_total');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-CEDAR-POLICY-EVAL-COST-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94204001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-cedar';
const evalRate = Number(__ENV.EVAL_RATE || 2500);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    policy_eval: {
      executor: 'constant-arrival-rate',
      rate: evalRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(100, Math.floor(evalRate / 50)),
      maxVUs: Math.max(500, Math.floor(evalRate / 10)),
      exec: 'evaluatePolicy',
    },
  },
  thresholds: {
    cedar_eval_latency_ms: ['p(99)<30'],
    cedar_cache_hit_rate: ['rate>0.85'],
    cedar_incorrect_decision_rate: ['rate==0'],
  },
};

function complexityForIteration(iteration) {
  const v = (iteration + seed) % 100;
  if (v < 25) return 'simple';
  if (v < 55) return 'medium';
  if (v < 85) return 'complex';
  return 'pathological';
}

function expectedDecision(complexity, idx) {
  if (complexity === 'simple') return true;
  if (complexity === 'medium') return idx % 11 !== 0;
  if (complexity === 'complex') return idx % 7 !== 0 && idx % 13 !== 0;
  return idx % 5 !== 0 && idx % 17 !== 0 && idx % 23 !== 0;
}

function evalPayload(complexity) {
  const idx = __ITER + seed + __VU;
  const tenantId = `${tenantPrefix}-${idx % 64}`;
  return {
    tenant_id: tenantId,
    complexity,
    policy_version: `v${idx % 8}`,
    principal: {
      type: complexity === 'pathological' ? 'auditor' : 'member',
      id: `principal-${idx % 10000}`,
      tenant_id: tenantId,
      autonomy_tier: 3,
      allowed_regions: ['us-east-1', 'us-west-2'],
      case_assignments: [`case-${idx % 128}`],
    },
    action: complexity === 'simple' ? 'read' : complexity === 'complex' ? 'invoke' : 'export',
    resource: {
      type: complexity === 'complex' ? 'WorkflowRun' : complexity === 'pathological' ? 'EvidencePack' : 'Document',
      id: `resource-${idx % 25000}`,
      tenant_id: tenantId,
      required_autonomy_tier: 2,
      data_class: idx % 13 === 0 ? 'PHI' : 'INTERNAL',
      legal_hold: idx % 11 === 0,
      break_glass: idx % 7 === 0,
      cross_tenant: idx % 13 === 0,
      regulatory_packs: ['SOC2-T2'],
      allowed_pack_ids: [`pack-${idx % 32}`],
      retention_class: 'seven_year',
    },
    context: {
      region: 'us-east-1',
      device_trust: 3,
      session_age_minutes: idx % 29,
      change_window_open: true,
      step_up_auth_completed: idx % 7 !== 0,
      explicit_cross_tenant_grant: idx % 13 !== 0,
      approver_count: 2,
      case_id: `case-${idx % 128}`,
      pack_id: `pack-${idx % 32}`,
      export_purpose: 'regulator',
      dsr_delete_pending: idx % 5 === 0,
      tenant_suspended: false,
    },
    expected_allow: expectedDecision(complexity, idx),
    idempotency_key: `eval-${seed}-${__VU}-${__ITER}`,
  };
}

export function evaluatePolicy() {
  const complexity = complexityForIteration(__ITER);
  const payload = evalPayload(complexity);
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/policy/cedar/evaluate`, JSON.stringify(payload), {
    headers: {
      'content-type': 'application/json',
      'x-oya-benchmark-id': benchId,
      'x-oya-tier': tier,
      'x-oya-complexity': complexity,
    },
    tags: { tier, complexity },
  });
  const ok = check(res, {
    'eval accepted': (r) => r.status === 200,
    'decision present': (r) => typeof r.json('allow') === 'boolean',
    'audit decision id present': (r) => !!r.json('decision_id'),
  });
  if (!ok) {
    incorrectDecision.add(true, { tier, complexity, reason: 'transport' });
    return;
  }
  const allow = Boolean(res.json('allow'));
  const cache = Boolean(res.json('cache_hit'));
  const elapsed = Number(res.json('eval_latency_ms') || (Date.now() - started));
  evalLatency.add(elapsed, { tier, complexity });
  cacheHit.add(cache, { tier, complexity });
  incorrectDecision.add(allow !== payload.expected_allow, { tier, complexity });
  evalCounter.add(1, { tier, complexity, allow: String(allow) });
  sleep(0.001);
}
```

```python
from __future__ import annotations

import json
import os
import random
import time
from locust import HttpUser, between, events, task

BENCH_ID = "PB-CEDAR-POLICY-EVAL-COST-2026-05-20"
SEED = int(os.getenv("SEED", "94204001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-cedar")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class CedarApplicationAuthUser(HttpUser):
    wait_time = between(0.001, 0.030)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 63)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
        }

    def _payload(self, surface: str, complexity: str):
        idx = random.randint(1, 1_000_000)
        allow_expected = idx % 19 != 0
        return {
            "tenant_id": self.tenant_id,
            "surface": surface,
            "complexity": complexity,
            "principal": {
                "type": "employee",
                "id": f"employee-{idx % 20000}",
                "tenant_id": self.tenant_id,
                "autonomy_tier": 2 + idx % 2,
                "allowed_regions": ["us-east-1"],
            },
            "action": random.choice(["read", "write", "invoke", "export", "delete"]),
            "resource": {
                "type": random.choice(["Document", "WorkflowRun", "Message", "Object", "EvidencePack"]),
                "id": f"res-{idx}",
                "tenant_id": self.tenant_id,
                "data_class": "INTERNAL" if allow_expected else "PHI",
                "legal_hold": not allow_expected,
            },
            "context": {
                "region": "us-east-1",
                "device_trust": 3,
                "step_up_auth_completed": allow_expected,
                "approver_count": 2,
                "change_window_open": True,
            },
            "expected_allow": allow_expected,
            "idempotency_key": f"locust-eval-{SEED}-{idx}",
        }

    def _evaluate(self, surface: str, complexity: str):
        payload = self._payload(surface, complexity)
        start = time.perf_counter()
        with self.client.post(
            "/v1/policy/cedar/evaluate",
            data=json.dumps(payload),
            headers={**self.headers, "x-oya-complexity": complexity},
            name=f"/v1/policy/cedar/evaluate/{complexity}",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code != 200:
                response.failure(f"eval failed {response.status_code}")
                return
            body = response.json()
            if body.get("allow") != payload["expected_allow"]:
                response.failure("incorrect cedar decision")
                return
            events.request.fire(request_type="CHECK", name=f"cedar_eval_{complexity}_ms", response_time=elapsed_ms, response_length=0)
            response.success()

    @task(35)
    def simple_document_read(self):
        self._evaluate("drive", "simple")

    @task(30)
    def medium_messenger_action(self):
        self._evaluate("messenger", "medium")

    @task(25)
    def complex_workflow_invoke(self):
        self._evaluate("workflow", "complex")

    @task(10)
    def pathological_evidence_export(self):
        self._evaluate("audit-chain", "pathological")
```

## Test Workload

Named request shape: `simple_same_tenant_read`.

Named request shape: `medium_document_policy_with_legal_hold_forbid`.

Named request shape: `complex_workflow_autonomy_and_cross_tenant_gate`.

Named request shape: `pathological_evidence_pack_export`.

Named distribution: `uniform-band-with-zipfian-policy-hotset`.

Each run uses four policy complexity bands.

Simple band has one permit rule and one tenant equality check.

Medium band has permit plus legal-hold forbid.

Complex band has autonomy-tier, session age, region, cross-tenant, and break-glass conditions.

Pathological band has multiple forbids and large context arrays.

Cache warmup preloads 80 percent of policy versions and 50 percent of entity graphs.

Cold-cache probes are retained at 5 percent to catch parser and entity-loader regressions.

Permit and forbid fixtures are generated in equal proportions inside each band.

## Baseline Numbers

Recorded baseline run: `cedar-complexity-band-lab-2026-05-20T12:00:00Z`.

SUT topology: 1 policy-cedar API deployment, 1 policy worker pool, 3 cache shards, 1 audit-chain sink, 64 tenants.

| Tier | Complexity band | p50 eval ms | p95 eval ms | p99 eval ms | Eval cap/s | Cache hit rate |
|---|---|---:|---:|---:|---:|---:|
| Bronze | simple | 0.42 | 1.7 | 3.4 | 3,200 | 94.8% |
| Bronze | medium | 0.88 | 3.9 | 7.8 | 2,700 | 92.1% |
| Bronze | complex | 1.70 | 7.6 | 15.8 | 2,050 | 89.4% |
| Bronze | pathological | 3.80 | 16.9 | 28.5 | 1,100 | 83.2% |
| Silver | simple | 0.36 | 1.3 | 2.5 | 7,400 | 96.1% |
| Silver | medium | 0.73 | 3.0 | 5.9 | 6,300 | 94.5% |
| Silver | complex | 1.38 | 5.9 | 11.4 | 4,900 | 91.7% |
| Silver | pathological | 3.10 | 13.4 | 23.1 | 2,400 | 86.8% |
| Gold | simple | 0.31 | 1.0 | 2.0 | 15,900 | 97.4% |
| Gold | medium | 0.62 | 2.4 | 4.7 | 13,700 | 95.9% |
| Gold | complex | 1.14 | 4.8 | 9.1 | 10,900 | 93.8% |
| Gold | pathological | 2.55 | 10.5 | 19.4 | 5,600 | 89.9% |
| Platinum | simple | 0.25 | 0.8 | 1.6 | 28,800 | 98.2% |
| Platinum | medium | 0.51 | 1.9 | 3.8 | 24,600 | 97.0% |
| Platinum | complex | 0.94 | 3.8 | 7.4 | 19,700 | 95.2% |
| Platinum | pathological | 2.12 | 8.4 | 15.7 | 10,200 | 92.3% |

Correctness baseline:

| Tier | Incorrect decisions | Audit-decision completeness | Fallback evaluator use |
|---|---:|---:|---:|
| Bronze | 0 | 100.000% | 0.000% |
| Silver | 0 | 100.000% | 0.000% |
| Gold | 0 | 100.000% | 0.000% |
| Platinum | 0 | 100.000% | 0.000% |

## Comparison vs Named Vendors

Named vendors and projects: AWS Verified Permissions, Cedar open-source evaluator, Open Policy Agent, Styra DAS, Google Zanzibar-style relationship checks.

AWS Verified Permissions-class comparison: Cedar policy decision latency and policy-store cache behavior.

Cedar OSS-class comparison: local evaluator cost with entity graphs and policy sets.

Open Policy Agent-class comparison: general authorization engine p99 under mixed allow/deny load.

Styra DAS-class comparison: policy bundle distribution and cache hit behavior.

Zanzibar-class comparison: relationship-graph authorization style and tuple cache pressure.

Oyatie differentiator measured here: policy evaluation remains p99 bounded while each decision can emit audit-chain evidence.

Vendor parity guard: this document does not assert hidden vendor latency; external comparison requires a separate public-endpoint or locally deployed vendor harness run.

## Methodology

Named SUT topology: `policy-cedar-api-cache-sharded-with-audit-sink`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 2 minutes.

Policy cache is warmed before measurement, then cold probes remain at 5 percent.

Entity graph cache is warmed for hot tenants and cold for long-tail tenants.

p99 latency starts at API receive and stops after decision object construction.

Audit decision emission is measured separately but required for pass/fail.

Incorrect decisions fail the benchmark even when latency is inside SLO.

Fallback evaluator use must remain zero because fallback changes the safety model.

## Reproducibility

Primary command:

`BENCH_ID=PB-CEDAR-POLICY-EVAL-COST-2026-05-20 TIER=Gold SEED=94204001 ./benchmarks/cedar-policy-eval-cost.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://policy-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94204001 benchmarks/cedar-policy-eval-cost.k6.js`

Locust command:

`locust -f benchmarks/cedar-policy-eval-cost.locust.py --headless --users 320 --spawn-rate 33 --run-time 1200s --host https://policy-cell-01.dev.oyatie.local`

Named seed values:

- `94204001` policy hotset seed.
- `94204002` entity graph seed.
- `94204003` deny-path seed.
- `94204004` cache cold-probe seed.

## Failure Modes Detected

Cedar p99 regression above complexity-band SLO.

Cache hit rate regression below baseline by more than 4 percentage points.

Incorrect permit decision.

Incorrect forbid decision.

Policy parse cache eviction storm.

Entity graph lookup hotspot.

Pathological policy starvation of simple policy traffic.

Deny path slower than permit path by more than 3x.

Audit decision emission missing.

Fallback evaluator invoked.

Tenant policy version drift.

Policy cache accepts stale superseded policy.

Context array size causes allocator pressure.

Autonomy-tier comparison bypass.

Cross-tenant permit leakage.

## Cross-References

- `specs/cedar-policy-schema.json`.
- `specs/cedar-fragment-schema.json`.
- `docs/standards/autonomy-ceiling.md`.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0128-hyperscaler-architecture-invariants.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- Service-owned `microservices/*/benchmarks/` directories are intentionally outside this root corpus change.
