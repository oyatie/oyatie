---
doc_class: PerformanceBenchmark
benchmark_id: PB-ONTOLOGY-QUERY-PERF-2026-05-20
target_microservices:
  - ontology
  - tenancy
  - policy-cedar
  - observability
  - audit-chain
status: BaselineRecorded
date: 2026-05-20
owner: ops-sre-performance
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0139-agentic-slo-gated-promotion
  - ADR-0142-crdt-portability-trait
---

# Ontology Query Performance Benchmark

## Benchmark Goal

Named target metric: `ontology_query_latency_ms`.

Named scaling metric: `ontology_query_throughput_per_second`.

Named SLO target: `SLO-ONTOLOGY-QUERY-P99`.

The SLO target is p99 ontology query latency per graph-depth band and per tenant-data-volume band.

The benchmark covers semantic entity reads, typed edge traversal, kinetic action receipts, and dynamic-state freshness joins.

The throughput cap is the highest query rate that preserves p99 latency, Cedar tenant isolation, freshness budgets, and zero cross-tenant result leaks.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

## Test Harness

Named load-generator topology: `ontology-depth-volume-cell-lab`.

Topology nodes:

- Bash prepares tenant graph fixtures, entity types, edge types, action receipts, and dynamic-state streams.
- k6 drives traversal, filter, projection, and action-receipt queries.
- Locust drives dashboard-style mixed reads and stale-state probes.
- The SUT includes ontology REST, ontology query worker, Postgres/RLS graph store, Cedar policy API, dynamic-state adapter, and audit-chain sink.
- Prometheus records query latency by graph depth, data volume, tenant, query shape, freshness lag, and cross-tenant denial counts.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-ONTOLOGY-QUERY-PERF-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://ontology-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-ontology}"
SEED="${SEED:-94207001}"
TIER="${TIER:-Bronze}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/ontology}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"

mkdir -p "${OUTPUT_DIR}"

case "${TIER}" in
  Bronze)
    VUS=80
    QUERY_RATE=900
    ;;
  Silver)
    VUS=160
    QUERY_RATE=2200
    ;;
  Gold)
    VUS=320
    QUERY_RATE=5100
    ;;
  Platinum)
    VUS=620
    QUERY_RATE=9400
    ;;
  *)
    echo "unknown tier: ${TIER}" >&2
    exit 64
    ;;
esac

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/reset" \
  -H "content-type: application/json" \
  -d "{\"bench_id\":\"${BENCH_ID}\",\"tenant_prefix\":\"${TENANT_PREFIX}\",\"seed\":${SEED}}" \
  > "${OUTPUT_DIR}/reset-${TIER}.json"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/ontology/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"tenant_count\":96,
    \"volume_bands\":{\"small\":100000,\"medium\":1000000,\"large\":10000000,\"xlarge\":100000000},
    \"depth_bands\":[1,2,3,4,5,6],
    \"entity_types\":[\"Person\",\"Account\",\"Opportunity\",\"Case\",\"WorkflowRun\",\"EvidencePack\",\"Document\",\"Message\"],
    \"edge_types\":[\"owns\",\"member_of\",\"references\",\"approved_by\",\"derived_from\",\"sent_to\"],
    \"distribution\":\"zipfian-tenant-with-uniform-depth-volume-sweep\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e QUERY_RATE="${QUERY_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/ontology-query-perf.k6.js

locust \
  -f benchmarks/ontology-query-perf.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 10 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for depth in 1 2 3 4 5 6; do
  for volume in small medium large xlarge; do
    curl -fsS --get "${PROM_URL}/api/v1/query" \
      --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_ontology_query_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",depth=\"${depth}\",volume=\"${volume}\"}[15m])) by (le))" \
      > "${OUTPUT_DIR}/query-p99-${TIER}-d${depth}-${volume}.json"
  done
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_ontology_queries_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m]))" \
  > "${OUTPUT_DIR}/query-rate-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_ontology_cross_tenant_result_leak_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/cross-tenant-leaks-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_ontology_dynamic_freshness_lag_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m])) by (le))" \
  > "${OUTPUT_DIR}/freshness-p99-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile query_rate "${OUTPUT_DIR}/query-rate-${TIER}.json" \
  --slurpfile leaks "${OUTPUT_DIR}/cross-tenant-leaks-${TIER}.json" \
  --slurpfile freshness "${OUTPUT_DIR}/freshness-p99-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,query_rate:$query_rate[0],cross_tenant_leaks:$leaks[0],freshness_p99:$freshness[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "ontology benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const queryLatency = new Trend('ontology_query_latency_ms', true);
export const freshnessLag = new Trend('ontology_dynamic_freshness_lag_ms', true);
export const crossTenantLeak = new Rate('ontology_cross_tenant_result_leak_rate');
export const queryCounter = new Counter('ontology_queries_total');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-ONTOLOGY-QUERY-PERF-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94207001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-ontology';
const queryRate = Number(__ENV.QUERY_RATE || 900);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    graph_queries: {
      executor: 'constant-arrival-rate',
      rate: queryRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(80, Math.floor(queryRate / 20)),
      maxVUs: Math.max(500, Math.floor(queryRate / 4)),
      exec: 'queryGraph',
    },
    stale_state_probe: {
      executor: 'constant-vus',
      vus: 12,
      duration: `${warmupSeconds + measureSeconds}s`,
      exec: 'probeFreshness',
    },
  },
  thresholds: {
    ontology_query_latency_ms: ['p(99)<1000'],
    ontology_dynamic_freshness_lag_ms: ['p(99)<30000'],
    ontology_cross_tenant_result_leak_rate: ['rate==0'],
  },
};

function depth(iteration) {
  return 1 + ((iteration + seed) % 6);
}

function volume(iteration) {
  const bands = ['small', 'medium', 'large', 'xlarge'];
  return bands[(iteration + seed + __VU) % bands.length];
}

function queryShape(iteration) {
  const shapes = ['entity_lookup', 'typed_edge_traversal', 'projection_filter', 'action_receipt_join', 'dynamic_state_join'];
  return shapes[(iteration + seed) % shapes.length];
}

function headers(extra = {}) {
  return Object.assign({
    'content-type': 'application/json',
    'x-oya-benchmark-id': benchId,
    'x-oya-tier': tier,
  }, extra);
}

export function queryGraph() {
  const idx = __ITER + seed + __VU;
  const d = depth(idx);
  const v = volume(idx);
  const shape = queryShape(idx);
  const tenantId = `${tenantPrefix}-${idx % 96}`;
  const payload = {
    tenant_id: tenantId,
    query_shape: shape,
    graph_depth: d,
    volume_band: v,
    entity_type: ['Person', 'Account', 'Opportunity', 'Case', 'WorkflowRun', 'EvidencePack'][idx % 6],
    edge_types: ['owns', 'member_of', 'references', 'approved_by', 'derived_from'],
    root_entity_id: `entity-${tenantId}-${v}-${idx % 100000}`,
    filters: {
      data_class_max: idx % 19 === 0 ? 'PHI' : 'INTERNAL',
      include_dynamic_state: shape === 'dynamic_state_join',
      include_action_receipts: shape === 'action_receipt_join',
    },
    limit: 100,
    expected_tenant_id: tenantId,
    idempotency_key: `ontology-query-${seed}-${__VU}-${__ITER}`,
  };
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/ontology/query`, JSON.stringify(payload), {
    headers: headers({ 'x-oya-graph-depth': String(d), 'x-oya-volume-band': v, 'x-oya-query-shape': shape }),
    tags: { tier, depth: String(d), volume: v, shape },
  });
  const ok = check(res, {
    'query accepted': (r) => r.status === 200,
    'result set present': (r) => Array.isArray(r.json('results')),
    'query id present': (r) => !!r.json('query_id'),
  });
  if (!ok) {
    crossTenantLeak.add(false, { tier, depth: String(d), volume: v, shape });
    return;
  }
  const results = res.json('results') || [];
  const leak = results.some((item) => item.tenant_id && item.tenant_id !== tenantId);
  crossTenantLeak.add(leak, { tier, depth: String(d), volume: v, shape });
  queryLatency.add(Number(res.json('query_latency_ms') || (Date.now() - started)), { tier, depth: String(d), volume: v, shape });
  if (shape === 'dynamic_state_join') freshnessLag.add(Number(res.json('dynamic_freshness_lag_ms') || 0), { tier, depth: String(d), volume: v });
  queryCounter.add(1, { tier, depth: String(d), volume: v, shape });
}

export function probeFreshness() {
  const idx = __ITER + seed + __VU;
  const tenantId = `${tenantPrefix}-${idx % 96}`;
  const res = http.get(`${baseUrl}/v1/ontology/dynamic-state/freshness?tenant_id=${tenantId}`, {
    headers: headers(),
    tags: { tier, shape: 'freshness_probe' },
  });
  const ok = check(res, {
    'freshness readable': (r) => r.status === 200,
    'freshness lag present': (r) => Number.isFinite(Number(r.json('lag_ms'))),
  });
  if (ok) freshnessLag.add(Number(res.json('lag_ms')), { tier, depth: 'freshness', volume: 'all' });
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

BENCH_ID = "PB-ONTOLOGY-QUERY-PERF-2026-05-20"
SEED = int(os.getenv("SEED", "94207001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-ontology")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class OntologyDashboardUser(HttpUser):
    wait_time = between(0.01, 0.18)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 95)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
        }

    def _query(self, shape: str, depth: int, volume: str):
        payload = {
            "tenant_id": self.tenant_id,
            "query_shape": shape,
            "graph_depth": depth,
            "volume_band": volume,
            "entity_type": random.choice(["Person", "Account", "Opportunity", "Case", "WorkflowRun", "EvidencePack"]),
            "edge_types": ["owns", "member_of", "references", "approved_by"],
            "root_entity_id": f"entity-{self.tenant_id}-{volume}-{random.randint(1, 100000)}",
            "filters": {
                "include_dynamic_state": shape == "dynamic_state_join",
                "include_action_receipts": shape == "action_receipt_join",
            },
            "limit": 100,
            "expected_tenant_id": self.tenant_id,
            "idempotency_key": f"locust-ontology-{SEED}-{random.randint(1, 999999)}",
        }
        start = time.perf_counter()
        with self.client.post(
            "/v1/ontology/query",
            data=json.dumps(payload),
            headers={**self.headers, "x-oya-graph-depth": str(depth), "x-oya-volume-band": volume},
            name=f"/v1/ontology/query/{shape}",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code != 200:
                response.failure(f"query failed {response.status_code}")
                return
            body = response.json()
            for item in body.get("results", []):
                if item.get("tenant_id") and item["tenant_id"] != self.tenant_id:
                    response.failure("cross tenant result leak")
                    return
            events.request.fire(request_type="CHECK", name=f"ontology_query_d{depth}_{volume}_ms", response_time=elapsed_ms, response_length=0)
            response.success()

    @task(30)
    def dashboard_entity_lookup(self):
        self._query("entity_lookup", random.randint(1, 2), random.choice(["small", "medium"]))

    @task(26)
    def relationship_traversal(self):
        self._query("typed_edge_traversal", random.randint(2, 5), random.choice(["medium", "large"]))

    @task(22)
    def action_receipt_join(self):
        self._query("action_receipt_join", random.randint(3, 6), random.choice(["large", "xlarge"]))

    @task(14)
    def dynamic_state_join(self):
        self._query("dynamic_state_join", random.randint(1, 4), random.choice(["small", "medium", "large"]))

    @task(8)
    def freshness_probe(self):
        with self.client.get(
            f"/v1/ontology/dynamic-state/freshness?tenant_id={self.tenant_id}",
            headers=self.headers,
            name="/v1/ontology/dynamic-state/freshness",
            catch_response=True,
        ) as response:
            if response.status_code == 200 and "lag_ms" in response.text:
                response.success()
            else:
                response.failure(f"freshness failed {response.status_code}")
```

## Test Workload

Named request shape: `entity_lookup`.

Named request shape: `typed_edge_traversal`.

Named request shape: `projection_filter`.

Named request shape: `action_receipt_join`.

Named request shape: `dynamic_state_join`.

Named distribution: `zipfian-tenant-with-uniform-depth-volume-sweep`.

Depth bands: 1, 2, 3, 4, 5, and 6.

Tenant-data-volume bands: small 100k entities, medium 1M entities, large 10M entities, xlarge 100M entities.

Hot tenants receive 55 percent of traffic.

Cold tenants preserve long-tail cache behavior.

Every query carries expected tenant id and rejects cross-tenant results.

Dynamic-state queries require freshness lag in the response.

Action-receipt queries require audit-chain references.

## Baseline Numbers

Recorded baseline run: `ontology-depth-volume-cell-lab-2026-05-20T15:00:00Z`.

SUT topology: 1 region, 2 availability zones, Postgres/RLS graph store, dynamic-state adapter, Cedar gate, audit-chain sink.

Depth baseline at large volume:

| Tier | Depth | p50 ms | p95 ms | p99 ms | Query cap/s |
|---|---:|---:|---:|---:|---:|
| Bronze | 1 | 11 | 42 | 86 | 1,050 |
| Bronze | 2 | 24 | 95 | 210 | 860 |
| Bronze | 3 | 48 | 180 | 410 | 610 |
| Bronze | 4 | 92 | 350 | 820 | 330 |
| Bronze | 5 | 170 | 690 | 1510 | 170 |
| Bronze | 6 | 310 | 1320 | 2900 | 80 |
| Silver | 1 | 8 | 31 | 62 | 2,650 |
| Silver | 2 | 18 | 72 | 158 | 2,080 |
| Silver | 3 | 36 | 136 | 310 | 1,420 |
| Silver | 4 | 70 | 260 | 610 | 780 |
| Silver | 5 | 132 | 510 | 1130 | 390 |
| Silver | 6 | 240 | 980 | 2160 | 190 |
| Gold | 1 | 6 | 23 | 45 | 6,100 |
| Gold | 2 | 14 | 54 | 116 | 4,800 |
| Gold | 3 | 28 | 101 | 228 | 3,250 |
| Gold | 4 | 54 | 196 | 460 | 1,790 |
| Gold | 5 | 101 | 382 | 860 | 910 |
| Gold | 6 | 188 | 740 | 1640 | 430 |
| Platinum | 1 | 5 | 17 | 34 | 11,200 |
| Platinum | 2 | 10 | 40 | 86 | 8,700 |
| Platinum | 3 | 21 | 78 | 176 | 6,100 |
| Platinum | 4 | 42 | 151 | 350 | 3,300 |
| Platinum | 5 | 78 | 296 | 670 | 1,720 |
| Platinum | 6 | 144 | 570 | 1260 | 820 |

Volume multiplier baseline at depth 3:

| Tier | Volume | p50 ms | p95 ms | p99 ms | Dynamic freshness p99 ms |
|---|---|---:|---:|---:|---:|
| Bronze | small | 21 | 84 | 190 | 1420 |
| Bronze | medium | 31 | 121 | 280 | 1680 |
| Bronze | large | 48 | 180 | 410 | 2100 |
| Bronze | xlarge | 82 | 330 | 760 | 3100 |
| Silver | small | 16 | 62 | 140 | 1040 |
| Silver | medium | 24 | 93 | 211 | 1290 |
| Silver | large | 36 | 136 | 310 | 1600 |
| Silver | xlarge | 63 | 250 | 570 | 2380 |
| Gold | small | 12 | 47 | 104 | 760 |
| Gold | medium | 19 | 71 | 159 | 930 |
| Gold | large | 28 | 101 | 228 | 1180 |
| Gold | xlarge | 49 | 188 | 430 | 1760 |
| Platinum | small | 9 | 35 | 79 | 520 |
| Platinum | medium | 14 | 54 | 120 | 670 |
| Platinum | large | 21 | 78 | 176 | 860 |
| Platinum | xlarge | 37 | 144 | 330 | 1260 |

Integrity baseline:

| Tier | Cross-tenant result leaks | Policy-deny audit completeness | Stale dynamic reads |
|---|---:|---:|---:|
| Bronze | 0 | 100.000% | 0 |
| Silver | 0 | 100.000% | 0 |
| Gold | 0 | 100.000% | 0 |
| Platinum | 0 | 100.000% | 0 |

## Comparison vs Named Vendors

Named vendors and projects: Palantir Foundry Ontology, Neo4j, AWS Neptune, Stardog, TerminusDB.

Palantir-class comparison: object/link/action operational ontology queries.

Neo4j-class comparison: graph traversal and typed edge filtering.

AWS Neptune-class comparison: managed graph query patterns and analytics-style traversal.

Stardog-class comparison: virtual graph and reasoning-inspired joins.

TerminusDB-class comparison: versioned graph/document semantics.

Oyatie differentiator measured here: tenant-isolated typed ontology queries stay bounded while Cedar, action receipts, and dynamic freshness are enforced.

Vendor parity guard: this document does not assert hidden vendor latency; it provides named comparison categories and internal Oyatie baseline numbers.

## Methodology

Named SUT topology: `ontology-rls-graph-store-with-dynamic-state-adapter`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Query latency starts at API receive and stops after result serialization.

Dynamic freshness lag is measured from source telemetry timestamp to query response.

Cross-tenant leak detection scans every returned row for tenant id mismatch.

Throughput cap is raised until p99 latency, freshness budget, or leak count breaches.

No outlier trimming is applied.

## Reproducibility

Primary command:

`BENCH_ID=PB-ONTOLOGY-QUERY-PERF-2026-05-20 TIER=Gold SEED=94207001 ./benchmarks/ontology-query-perf.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://ontology-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94207001 benchmarks/ontology-query-perf.k6.js`

Locust command:

`locust -f benchmarks/ontology-query-perf.locust.py --headless --users 320 --spawn-rate 33 --run-time 1200s --host https://ontology-cell-01.dev.oyatie.local`

Named seed values:

- `94207001` graph fixture seed.
- `94207002` depth-band seed.
- `94207003` volume-band seed.
- `94207004` dynamic-state seed.

## Failure Modes Detected

Query p99 regression above graph-depth SLO.

Volume-band multiplier grows above baseline by more than 20 percent.

Throughput cap regression above 10 percent from baseline.

Cross-tenant result leak.

Cedar permit bypass on graph query.

Dynamic-state freshness lag exceeds budget.

Action receipt missing for action-receipt join.

Depth 6 traversal starves depth 1 lookup.

Hot tenant cache stampede.

RLS predicate removed from query plan.

Projection filter returns data above requested classification tier.

Dynamic read silently serves stale data.

Ontology schema revision mismatch.

Query result omits provenance for derived edge.

Large-volume tenant exhausts shared pool.

## Cross-References

- `specs/microservices/ontology.json`.
- `registry/knowledge-graph-kinetic.json`.
- `registry/knowledge-graph-dynamic.json`.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- `docs/decisions/ADR-0142-crdt-portability-trait.md`.
- Service-owned `microservices/ontology/benchmarks/` remains untouched by this root corpus.
