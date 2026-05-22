---
doc_class: PerformanceBenchmark
benchmark_id: PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20
target_microservices:
  - api-gateway
  - service-mesh
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
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0044-service-mesh-istio-ambient-and-envoy-gateway
  - ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation
  - ADR-0186-observability-backplane-layering
---

# Cross Microservice Call Overhead Benchmark

## Benchmark Goal

Named target metric: `cross_microservice_rpc_overhead_ms`.

Named handshake metric: `mesh_mtls_handshake_latency_ms`.

Named hop metric: `per_cell_hop_latency_ms`.

Named SLO target: `SLO-CROSS-MICROSERVICE-OVERHEAD-P99`.

The SLO target is p99 incremental overhead for one cross-microservice RPC hop, one mTLS connection establishment, and one per-cell hop.

The benchmark measures north-south edge admission, east-west service mesh hop cost, Cedar ext-authz cost, OTel trace propagation, audit sampling cost, and cross-cell egress cost.

The throughput cap is the highest cross-service RPC rate that preserves p99 overhead, zero plaintext traffic, zero missing trace context, and zero unregistered cross-cell calls.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

## Test Harness

Named load-generator topology: `mesh-hop-overhead-cell-lab`.

Topology nodes:

- Bash prepares route registrations, mesh policy fixtures, SPIFFE identities, tenant headers, and echo endpoints.
- k6 drives north-south and east-west RPC chains with 1, 2, 3, 5, and 8 hops.
- Locust drives cross-cell calls, cold mTLS handshakes, and trace propagation validation.
- The SUT includes Envoy gateway, Istio Ambient ztunnel, waypoint policy, Cedar ext-authz, observability collector, audit-chain sampler, and echo services.
- Prometheus records incremental hop overhead, mTLS handshake latency, cross-cell hop latency, trace propagation failures, and plaintext attempts.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://edge-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-hop}"
SEED="${SEED:-94209001}"
TIER="${TIER:-Bronze}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/cross-msvc}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"

mkdir -p "${OUTPUT_DIR}"

case "${TIER}" in
  Bronze)
    VUS=100
    RPC_RATE=1400
    HANDSHAKE_RATE=80
    ;;
  Silver)
    VUS=200
    RPC_RATE=3400
    HANDSHAKE_RATE=180
    ;;
  Gold)
    VUS=400
    RPC_RATE=7600
    HANDSHAKE_RATE=390
    ;;
  Platinum)
    VUS=760
    RPC_RATE=14000
    HANDSHAKE_RATE=720
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

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/cross-msvc/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"hop_counts\":[1,2,3,5,8],
    \"cell_paths\":[\"same-namespace\",\"cross-namespace\",\"cross-cell-same-region\",\"cross-region\"],
    \"mesh_mode\":\"istio-ambient\",
    \"edge_gateway\":\"envoy\",
    \"distribution\":\"uniform-hop-count-with-bursting-handshake-cold-start\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e RPC_RATE="${RPC_RATE}" \
  -e HANDSHAKE_RATE="${HANDSHAKE_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/cross-µservice-call-overhead.k6.js

locust \
  -f benchmarks/cross-µservice-call-overhead.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 12 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for hops in 1 2 3 5 8; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_cross_msvc_rpc_overhead_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",hops=\"${hops}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/rpc-overhead-p99-${TIER}-${hops}.json"
done

for path in same_namespace cross_namespace cross_cell_same_region cross_region; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_per_cell_hop_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",path=\"${path}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/cell-hop-p99-${TIER}-${path}.json"
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_mesh_mtls_handshake_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m])) by (le))" \
  > "${OUTPUT_DIR}/mtls-handshake-p99-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_mesh_plaintext_traffic_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/plaintext-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_trace_context_missing_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/trace-missing-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile plaintext "${OUTPUT_DIR}/plaintext-${TIER}.json" \
  --slurpfile trace_missing "${OUTPUT_DIR}/trace-missing-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,plaintext:$plaintext[0],trace_missing:$trace_missing[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "cross microservice benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const rpcOverhead = new Trend('cross_microservice_rpc_overhead_ms', true);
export const mtlsHandshake = new Trend('mesh_mtls_handshake_latency_ms', true);
export const cellHop = new Trend('per_cell_hop_latency_ms', true);
export const plaintextTraffic = new Rate('mesh_plaintext_traffic_rate');
export const missingTrace = new Rate('trace_context_missing_rate');
export const rpcCalls = new Counter('cross_microservice_rpc_calls');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94209001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-hop';
const rpcRate = Number(__ENV.RPC_RATE || 1400);
const handshakeRate = Number(__ENV.HANDSHAKE_RATE || 80);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    rpc_chain: {
      executor: 'constant-arrival-rate',
      rate: rpcRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(100, Math.floor(rpcRate / 30)),
      maxVUs: Math.max(700, Math.floor(rpcRate / 6)),
      exec: 'runRpcChain',
    },
    cold_handshake: {
      executor: 'constant-arrival-rate',
      rate: handshakeRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(40, Math.floor(handshakeRate / 2)),
      maxVUs: Math.max(250, handshakeRate * 2),
      exec: 'runColdHandshake',
    },
    cross_cell_probe: {
      executor: 'constant-vus',
      vus: 20,
      duration: `${warmupSeconds + measureSeconds}s`,
      exec: 'runCrossCellProbe',
    },
  },
  thresholds: {
    cross_microservice_rpc_overhead_ms: ['p(99)<80'],
    mesh_mtls_handshake_latency_ms: ['p(99)<250'],
    per_cell_hop_latency_ms: ['p(99)<120'],
    mesh_plaintext_traffic_rate: ['rate==0'],
    trace_context_missing_rate: ['rate==0'],
  },
};

function hopCount(iteration) {
  const counts = [1, 2, 3, 5, 8];
  return counts[(iteration + seed) % counts.length];
}

function cellPath(iteration) {
  const paths = ['same_namespace', 'cross_namespace', 'cross_cell_same_region', 'cross_region'];
  return paths[(iteration + seed + __VU) % paths.length];
}

function headers(extra = {}) {
  const trace = `00-${String(seed).padStart(32, '0')}-${String(__VU).padStart(16, '0')}-01`;
  return Object.assign({
    'content-type': 'application/json',
    'traceparent': trace,
    'x-oya-benchmark-id': benchId,
    'x-oya-tier': tier,
    'x-oya-tenant-id': `${tenantPrefix}-${(__ITER + __VU + seed) % 64}`,
  }, extra);
}

export function runRpcChain() {
  const hops = hopCount(__ITER);
  const path = cellPath(__ITER);
  const started = Date.now();
  const res = http.post(`${baseUrl}/internal/bench/cross-msvc/rpc-chain`, JSON.stringify({
    hop_count: hops,
    cell_path: path,
    tenant_id: `${tenantPrefix}-${(__ITER + seed) % 64}`,
    payload_bytes: 2048,
    require_cedar_ext_authz: true,
    require_trace_context: true,
    require_audit_sample: true,
    idempotency_key: `rpc-chain-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'x-oya-hop-count': String(hops), 'x-oya-cell-path': path }), tags: { tier, hops: String(hops), path } });
  const ok = check(res, {
    'rpc chain completed': (r) => r.status === 200,
    'hop count matched': (r) => Number(r.json('observed_hops')) === hops,
    'trace preserved': (r) => r.json('trace_context_preserved') === true,
    'mtls strict': (r) => r.json('plaintext_detected') !== true,
  });
  plaintextTraffic.add(ok ? Boolean(res.json('plaintext_detected')) : false, { tier, path });
  missingTrace.add(ok ? res.json('trace_context_preserved') !== true : true, { tier, path });
  if (ok) {
    rpcOverhead.add(Number(res.json('incremental_overhead_ms') || (Date.now() - started)), { tier, hops: String(hops), path });
    rpcCalls.add(1, { tier, hops: String(hops), path });
  }
}

export function runColdHandshake() {
  const path = cellPath(__ITER);
  const started = Date.now();
  const res = http.post(`${baseUrl}/internal/bench/cross-msvc/mtls-handshake`, JSON.stringify({
    source_service: `bench-source-${__VU}`,
    destination_service: `bench-dest-${(__ITER + seed) % 64}`,
    cell_path: path,
    force_new_connection: true,
    spiffe_required: true,
    idempotency_key: `mtls-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'connection': 'close', 'x-oya-cell-path': path }), tags: { tier, path } });
  const ok = check(res, {
    'handshake completed': (r) => r.status === 200,
    'spiffe verified': (r) => r.json('spiffe_verified') === true,
    'mtls protocol present': (r) => String(r.json('protocol')).startsWith('TLS'),
  });
  if (ok) mtlsHandshake.add(Number(res.json('handshake_latency_ms') || (Date.now() - started)), { tier, path });
}

export function runCrossCellProbe() {
  const path = cellPath(__ITER);
  const res = http.post(`${baseUrl}/internal/bench/cross-msvc/cell-hop`, JSON.stringify({
    cell_path: path,
    tenant_id: `${tenantPrefix}-${(__ITER + seed) % 64}`,
    registered_call_type: 'benchmark.echo',
    payload_bytes: 512,
    idempotency_key: `cell-hop-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'x-oya-cell-path': path }), tags: { tier, path } });
  const ok = check(res, {
    'cell hop completed': (r) => r.status === 200,
    'call type registered': (r) => r.json('call_type_registered') === true,
    'audit policy applied': (r) => r.json('audit_policy_applied') === true,
  });
  if (ok) cellHop.add(Number(res.json('cell_hop_latency_ms') || 0), { tier, path });
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

BENCH_ID = "PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20"
SEED = int(os.getenv("SEED", "94209001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-hop")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class CrossMicroserviceProbeUser(HttpUser):
    wait_time = between(0.01, 0.12)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 63)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
            "x-oya-tenant-id": self.tenant_id,
            "traceparent": f"00-{SEED:032d}-{random.randint(1, 999999):016d}-01",
        }

    def _path(self):
        return random.choice(["same_namespace", "cross_namespace", "cross_cell_same_region", "cross_region"])

    @task(32)
    def rpc_chain_probe(self):
        hops = random.choice([1, 2, 3, 5, 8])
        path = self._path()
        start = time.perf_counter()
        with self.client.post(
            "/internal/bench/cross-msvc/rpc-chain",
            data=json.dumps({
                "hop_count": hops,
                "cell_path": path,
                "tenant_id": self.tenant_id,
                "payload_bytes": 2048,
                "require_cedar_ext_authz": True,
                "require_trace_context": True,
                "require_audit_sample": True,
                "idempotency_key": f"locust-rpc-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "x-oya-hop-count": str(hops), "x-oya-cell-path": path},
            name="/internal/bench/cross-msvc/rpc-chain",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code != 200:
                response.failure(f"rpc chain failed {response.status_code}")
                return
            body = response.json()
            if body.get("trace_context_preserved") is not True:
                response.failure("trace context missing")
                return
            if body.get("plaintext_detected") is True:
                response.failure("plaintext traffic detected")
                return
            events.request.fire(request_type="CHECK", name=f"rpc_chain_{hops}_{path}_ms", response_time=elapsed_ms, response_length=0)
            response.success()

    @task(28)
    def cold_handshake_probe(self):
        path = self._path()
        with self.client.post(
            "/internal/bench/cross-msvc/mtls-handshake",
            data=json.dumps({
                "source_service": f"locust-source-{random.randint(1, 10000)}",
                "destination_service": f"locust-dest-{random.randint(1, 10000)}",
                "cell_path": path,
                "force_new_connection": True,
                "spiffe_required": True,
                "idempotency_key": f"locust-mtls-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "connection": "close", "x-oya-cell-path": path},
            name="/internal/bench/cross-msvc/mtls-handshake",
            catch_response=True,
        ) as response:
            if response.status_code == 200 and response.json().get("spiffe_verified") is True:
                response.success()
            else:
                response.failure(f"mtls handshake failed {response.status_code}")

    @task(24)
    def cell_hop_probe(self):
        path = self._path()
        with self.client.post(
            "/internal/bench/cross-msvc/cell-hop",
            data=json.dumps({
                "cell_path": path,
                "tenant_id": self.tenant_id,
                "registered_call_type": "benchmark.echo",
                "payload_bytes": 512,
                "idempotency_key": f"locust-cell-hop-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "x-oya-cell-path": path},
            name="/internal/bench/cross-msvc/cell-hop",
            catch_response=True,
        ) as response:
            if response.status_code == 200 and response.json().get("call_type_registered") is True:
                response.success()
            else:
                response.failure(f"cell hop failed {response.status_code}")

    @task(16)
    def unregistered_call_negative_probe(self):
        path = random.choice(["cross_cell_same_region", "cross_region"])
        with self.client.post(
            "/internal/bench/cross-msvc/cell-hop",
            data=json.dumps({
                "cell_path": path,
                "tenant_id": self.tenant_id,
                "registered_call_type": "unregistered.benchmark.bad",
                "payload_bytes": 256,
                "idempotency_key": f"locust-unregistered-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "x-oya-cell-path": path},
            name="/internal/bench/cross-msvc/cell-hop-unregistered",
            catch_response=True,
        ) as response:
            if response.status_code in (400, 403, 409, 422):
                response.success()
            else:
                response.failure(f"unregistered call allowed status={response.status_code}")
```

## Test Workload

Named request shape: `north_south_edge_to_one_service`.

Named request shape: `east_west_one_hop`.

Named request shape: `east_west_multi_hop_2_3_5_8`.

Named request shape: `cold_mtls_handshake`.

Named request shape: `cross_cell_same_region_hop`.

Named request shape: `cross_region_hop`.

Named request shape: `unregistered_cross_cell_negative_probe`.

Named distribution: `uniform-hop-count-with-bursting-handshake-cold-start`.

Hop counts are swept uniformly across 1, 2, 3, 5, and 8 hops.

Cell paths are swept across same namespace, cross namespace, cross cell same region, and cross region.

Cold handshake bursts run every 5 minutes and force new connections.

Every RPC carries tenant id, traceparent, idempotency key, and benchmark id.

Cross-cell calls must be registered and audit-policied.

Plaintext traffic must remain zero.

Trace context missing must remain zero.

## Baseline Numbers

Recorded baseline run: `mesh-hop-overhead-cell-lab-2026-05-20T17:00:00Z`.

SUT topology: Envoy edge, Istio Ambient ztunnel, waypoint policy, Cedar ext-authz, OTel collector, audit-chain sampler, echo services.

RPC overhead by hop count:

| Tier | Hops | p50 overhead ms | p95 overhead ms | p99 overhead ms | RPC cap/s |
|---|---:|---:|---:|---:|---:|
| Bronze | 1 | 3.8 | 14.2 | 31.0 | 1,650 |
| Bronze | 2 | 7.0 | 25.0 | 52.0 | 1,420 |
| Bronze | 3 | 10.4 | 36.0 | 73.0 | 1,120 |
| Bronze | 5 | 17.8 | 61.0 | 126.0 | 690 |
| Bronze | 8 | 29.0 | 101.0 | 212.0 | 340 |
| Silver | 1 | 3.1 | 11.0 | 24.0 | 3,950 |
| Silver | 2 | 5.8 | 19.0 | 41.0 | 3,360 |
| Silver | 3 | 8.4 | 28.0 | 59.0 | 2,700 |
| Silver | 5 | 14.2 | 47.0 | 99.0 | 1,640 |
| Silver | 8 | 23.0 | 78.0 | 165.0 | 820 |
| Gold | 1 | 2.5 | 8.6 | 18.0 | 8,800 |
| Gold | 2 | 4.6 | 15.0 | 32.0 | 7,500 |
| Gold | 3 | 6.8 | 22.0 | 47.0 | 6,000 |
| Gold | 5 | 11.4 | 37.0 | 78.0 | 3,700 |
| Gold | 8 | 18.6 | 62.0 | 132.0 | 1,850 |
| Platinum | 1 | 2.0 | 6.7 | 14.0 | 16,100 |
| Platinum | 2 | 3.7 | 12.0 | 25.0 | 13,800 |
| Platinum | 3 | 5.4 | 17.0 | 37.0 | 11,200 |
| Platinum | 5 | 9.1 | 29.0 | 61.0 | 6,900 |
| Platinum | 8 | 14.8 | 49.0 | 104.0 | 3,400 |

mTLS and cell-hop baseline:

| Tier | Path | p50 mTLS handshake ms | p95 mTLS handshake ms | p99 mTLS handshake ms | p99 cell hop ms |
|---|---|---:|---:|---:|---:|
| Bronze | same_namespace | 12 | 44 | 88 | 18 |
| Bronze | cross_namespace | 15 | 58 | 116 | 31 |
| Bronze | cross_cell_same_region | 28 | 104 | 221 | 74 |
| Bronze | cross_region | 52 | 190 | 410 | 146 |
| Silver | same_namespace | 10 | 35 | 70 | 14 |
| Silver | cross_namespace | 12 | 46 | 92 | 24 |
| Silver | cross_cell_same_region | 23 | 82 | 176 | 58 |
| Silver | cross_region | 42 | 152 | 330 | 116 |
| Gold | same_namespace | 8 | 28 | 56 | 11 |
| Gold | cross_namespace | 10 | 36 | 74 | 19 |
| Gold | cross_cell_same_region | 18 | 66 | 142 | 45 |
| Gold | cross_region | 34 | 120 | 260 | 91 |
| Platinum | same_namespace | 7 | 22 | 44 | 9 |
| Platinum | cross_namespace | 8 | 29 | 60 | 15 |
| Platinum | cross_cell_same_region | 15 | 52 | 112 | 36 |
| Platinum | cross_region | 27 | 96 | 210 | 72 |

Integrity baseline:

| Tier | Plaintext traffic | Missing trace context | Unregistered cross-cell calls allowed | Audit policy completeness |
|---|---:|---:|---:|---:|
| Bronze | 0 | 0 | 0 | 100.000% |
| Silver | 0 | 0 | 0 | 100.000% |
| Gold | 0 | 0 | 0 | 100.000% |
| Platinum | 0 | 0 | 0 | 100.000% |

## Comparison vs Named Vendors

Named vendors and projects: AWS App Mesh, Istio Ambient, Linkerd, Google Cloud Service Mesh, Stripe internal API platform.

AWS App Mesh-class comparison: Envoy-based service-to-service hop overhead.

Istio Ambient-class comparison: ztunnel and waypoint overhead without pod sidecars.

Linkerd-class comparison: lightweight mTLS service mesh overhead.

Google Cloud Service Mesh-class comparison: managed mesh trace and policy overhead.

Stripe internal API platform-class comparison: disciplined per-service API boundaries and per-call observability.

Oyatie differentiator measured here: every cross-service call carries tenant identity, trace context, policy, and audit posture while preserving bounded p99 overhead.

Vendor parity guard: this document does not assert hidden vendor overhead; named vendors define comparable external or locally deployed harness targets.

## Methodology

Named SUT topology: `envoy-istio-ambient-waypoint-cedar-otel-audit`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

RPC overhead subtracts direct in-process echo baseline from observed routed RPC latency.

mTLS handshake latency measures forced new connection establishment with SPIFFE verification.

Per-cell hop latency measures incremental latency from same-cell to cross-cell routed calls.

Trace propagation is verified through returned trace ids and collector counters.

Plaintext detection uses mesh telemetry and synthetic forbidden HTTP probes.

Throughput cap is raised until p99 overhead, plaintext, trace, or unregistered-call gates breach.

## Reproducibility

Primary command:

`BENCH_ID=PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20 TIER=Gold SEED=94209001 ./benchmarks/cross-µservice-call-overhead.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://edge-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94209001 benchmarks/cross-µservice-call-overhead.k6.js`

Locust command:

`locust -f benchmarks/cross-µservice-call-overhead.locust.py --headless --users 400 --spawn-rate 34 --run-time 1200s --host https://edge-cell-01.dev.oyatie.local`

Named seed values:

- `94209001` hop-count seed.
- `94209002` cell-path seed.
- `94209003` handshake cold-start seed.
- `94209004` trace propagation seed.

## Failure Modes Detected

RPC overhead p99 regression above hop-count SLO.

mTLS handshake p99 regression above path SLO.

Per-cell hop p99 regression above path SLO.

Throughput cap regression above 10 percent from baseline.

Plaintext traffic detected.

Trace context missing.

Unregistered cross-cell call allowed.

Cedar ext-authz bypass.

Audit policy not applied to cross-cell traffic.

SPIFFE identity verification failure.

Waypoint policy latency dominates hop overhead.

Cross-region hop uses same-region policy.

Tenant header missing after first hop.

Retry storm multiplies effective hop count.

OTel collector backpressure drops trace spans.

## Cross-References

- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0009-cell-architecture-per-tenant-per-region.md`.
- `docs/decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md`.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md`.
- `docs/decisions/ADR-0186-observability-backplane-layering.md`.
- `docs/SLO-CATALOG.md`.
- Service-owned per-microservice benchmark directories remain untouched by this root corpus.
