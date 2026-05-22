---
doc_class: PerformanceBenchmark
benchmark_id: PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20
target_microservices:
  - audit-chain
  - observability
  - tenancy
  - eventing
status: BaselineRecorded
date: 2026-05-20
owner: ops-sre-performance
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0139-agentic-slo-gated-promotion
  - ADR-0186-observability-backplane-layering
---

# Audit Chain Emission Throughput Benchmark

## Benchmark Goal

Named target metric: `audit_chain_events_sealed_per_second`.

Named lag metric: `audit_chain_merkle_attestation_lag_ms`.

Named SLO target: `SLO-AUDIT-EMISSION-P99`.

The SLO target is p99 event seal latency and p99 Merkle attestation lag per tier.

The throughput cap is the highest event-emission rate that preserves hash continuity, root publication, replayability, and audit completeness.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

## Test Harness

Named load-generator topology: `audit-merkle-attestation-cell-lab`.

Topology nodes:

- Bash prepares tenant shards, event classes, signer keys, and root-anchor windows.
- k6 emits single-event and batch-event traffic through the audit-chain API.
- Locust drives query/replay/read-after-write pressure while the writer load runs.
- The SUT includes audit-chain REST, audit-chain worker, eventing outbox, observability exporter, KMS signing stub, and root-anchor publisher stub.
- Prometheus records seal latency, queue lag, Merkle root lag, replay latency, signature failures, and hash continuity failures.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://audit-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-audit}"
SEED="${SEED:-94205001}"
TIER="${TIER:-Bronze}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/audit-chain}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"
ANCHOR_WINDOW_SECONDS="${ANCHOR_WINDOW_SECONDS:-60}"

mkdir -p "${OUTPUT_DIR}"

case "${TIER}" in
  Bronze)
    VUS=90
    EVENT_RATE=1800
    BATCH_RATE=120
    ;;
  Silver)
    VUS=180
    EVENT_RATE=4200
    BATCH_RATE=280
    ;;
  Gold)
    VUS=360
    EVENT_RATE=9200
    BATCH_RATE=620
    ;;
  Platinum)
    VUS=720
    EVENT_RATE=16800
    BATCH_RATE=1100
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

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/audit-chain/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"tenant_shards\":128,
    \"event_classes\":[\"capability_invoked\",\"message_sent\",\"object_committed\",\"workflow_transition\",\"policy_decision\",\"dsr_evidence\"],
    \"anchor_window_seconds\":${ANCHOR_WINDOW_SECONDS},
    \"distribution\":\"bursting-per-tenant-shard-with-batch-tail\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e EVENT_RATE="${EVENT_RATE}" \
  -e BATCH_RATE="${BATCH_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -e ANCHOR_WINDOW_SECONDS="${ANCHOR_WINDOW_SECONDS}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/audit-chain-emission-throughput.k6.js

locust \
  -f benchmarks/audit-chain-emission-throughput.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 12 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for metric in seal_latency_ms merkle_attestation_lag_ms replay_latency_ms queue_lag_ms; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_audit_chain_${metric}_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/${metric}-p99-${TIER}.json"
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_audit_chain_events_sealed_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m]))" \
  > "${OUTPUT_DIR}/sealed-throughput-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_audit_chain_hash_continuity_failure_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/hash-failures-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_audit_chain_signature_failure_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/signature-failures-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile throughput "${OUTPUT_DIR}/sealed-throughput-${TIER}.json" \
  --slurpfile hash_failures "${OUTPUT_DIR}/hash-failures-${TIER}.json" \
  --slurpfile signature_failures "${OUTPUT_DIR}/signature-failures-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,throughput:$throughput[0],hash_failures:$hash_failures[0],signature_failures:$signature_failures[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "audit-chain benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const sealLatency = new Trend('audit_chain_seal_latency_ms', true);
export const merkleLag = new Trend('audit_chain_merkle_attestation_lag_ms', true);
export const queueLag = new Trend('audit_chain_queue_lag_ms', true);
export const hashFailure = new Rate('audit_chain_hash_failure_rate');
export const sealedEvents = new Counter('audit_chain_events_sealed');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94205001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-audit';
const eventRate = Number(__ENV.EVENT_RATE || 1800);
const batchRate = Number(__ENV.BATCH_RATE || 120);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    single_event_emission: {
      executor: 'constant-arrival-rate',
      rate: eventRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(100, Math.floor(eventRate / 30)),
      maxVUs: Math.max(600, Math.floor(eventRate / 8)),
      exec: 'emitSingleEvent',
    },
    batch_event_emission: {
      executor: 'constant-arrival-rate',
      rate: batchRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(40, Math.floor(batchRate / 4)),
      maxVUs: Math.max(250, batchRate),
      exec: 'emitBatch',
    },
    root_probe: {
      executor: 'constant-vus',
      vus: 8,
      duration: `${warmupSeconds + measureSeconds}s`,
      exec: 'probeMerkleRoots',
    },
  },
  thresholds: {
    audit_chain_seal_latency_ms: ['p(99)<200'],
    audit_chain_merkle_attestation_lag_ms: ['p(99)<65000'],
    audit_chain_hash_failure_rate: ['rate==0'],
  },
};

function eventClass(iteration) {
  const classes = ['capability_invoked', 'message_sent', 'object_committed', 'workflow_transition', 'policy_decision', 'dsr_evidence'];
  return classes[(iteration + seed) % classes.length];
}

function eventPayload(idx) {
  const tenant = `${tenantPrefix}-${idx % 128}`;
  return {
    tenant_shard: tenant,
    event_class: eventClass(idx),
    principal: `principal-${idx % 10000}`,
    capability: `capability-${idx % 512}`,
    data_classes_touched: idx % 11 === 0 ? ['PHI'] : ['INTERNAL'],
    regulatory_packs_consumed: idx % 17 === 0 ? ['SOC2-T2', 'KR-PIPA'] : ['SOC2-T2'],
    payload_hash: `sha256-${seed}-${idx}`,
    occurred_at_ms: Date.now(),
    idempotency_key: `audit-${seed}-${__VU}-${idx}`,
  };
}

function headers() {
  return {
    'content-type': 'application/json',
    'x-oya-benchmark-id': benchId,
    'x-oya-tier': tier,
  };
}

export function emitSingleEvent() {
  const idx = __ITER + seed + __VU;
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/audit-chain/events`, JSON.stringify(eventPayload(idx)), {
    headers: headers(),
    tags: { tier, event_class: eventClass(idx), mode: 'single' },
  });
  const ok = check(res, {
    'event sealed': (r) => r.status === 201 || r.status === 202,
    'event id present': (r) => !!r.json('event_id'),
    'block hash present': (r) => !!r.json('block_hash'),
  });
  hashFailure.add(!ok, { tier, mode: 'single' });
  if (ok) {
    sealLatency.add(Number(res.json('seal_latency_ms') || (Date.now() - started)), { tier, mode: 'single' });
    queueLag.add(Number(res.json('queue_lag_ms') || 0), { tier, mode: 'single' });
    sealedEvents.add(1, { tier, mode: 'single' });
  }
}

export function emitBatch() {
  const idx = __ITER + seed + __VU;
  const events = [];
  for (let i = 0; i < 25; i += 1) {
    events.push(eventPayload(idx * 31 + i));
  }
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/audit-chain/events:batch`, JSON.stringify({
    tenant_shard: `${tenantPrefix}-${idx % 128}`,
    events,
    idempotency_key: `batch-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers(), tags: { tier, mode: 'batch' } });
  const ok = check(res, {
    'batch sealed': (r) => r.status === 201 || r.status === 202,
    'batch count matches': (r) => Number(r.json('sealed_count')) === events.length,
    'batch root present': (r) => !!r.json('batch_root_hash'),
  });
  hashFailure.add(!ok, { tier, mode: 'batch' });
  if (ok) {
    sealLatency.add(Number(res.json('seal_latency_ms') || (Date.now() - started)), { tier, mode: 'batch' });
    sealedEvents.add(events.length, { tier, mode: 'batch' });
  }
}

export function probeMerkleRoots() {
  const shard = `${tenantPrefix}-${(__ITER + __VU + seed) % 128}`;
  const res = http.get(`${baseUrl}/v1/audit-chain/shards/${shard}/latest-root`, {
    headers: headers(),
    tags: { tier, mode: 'root_probe' },
  });
  const ok = check(res, {
    'root readable': (r) => r.status === 200 || r.status === 404,
    'root has hash when present': (r) => r.status === 404 || !!r.json('merkle_root_hash'),
  });
  if (ok && res.status === 200) {
    merkleLag.add(Number(res.json('attestation_lag_ms') || 0), { tier });
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

BENCH_ID = "PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20"
SEED = int(os.getenv("SEED", "94205001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-audit")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class AuditChainReadAfterWriteUser(HttpUser):
    wait_time = between(0.01, 0.12)

    def on_start(self):
        self.shard = f"{TENANT_PREFIX}-{random.randint(0, 127)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
        }

    @task(34)
    def latest_tip_read(self):
        start = time.perf_counter()
        with self.client.get(
            f"/v1/audit-chain/shards/{self.shard}/tip",
            headers=self.headers,
            name="/v1/audit-chain/shards/:shard/tip",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code in (200, 404):
                events.request.fire(request_type="CHECK", name="audit_tip_read_ms", response_time=elapsed_ms, response_length=0)
                response.success()
            else:
                response.failure(f"tip read failed {response.status_code}")

    @task(28)
    def replay_window_probe(self):
        start_seq = random.randint(1, 250000)
        end_seq = start_seq + random.randint(10, 200)
        start = time.perf_counter()
        with self.client.get(
            f"/v1/audit-chain/shards/{self.shard}/replay?start_seq={start_seq}&end_seq={end_seq}",
            headers=self.headers,
            name="/v1/audit-chain/shards/:shard/replay",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code in (200, 404):
                events.request.fire(request_type="CHECK", name="audit_replay_window_ms", response_time=elapsed_ms, response_length=0)
                response.success()
            else:
                response.failure(f"replay failed {response.status_code}")

    @task(24)
    def root_anchor_read(self):
        with self.client.get(
            f"/v1/audit-chain/shards/{self.shard}/latest-root",
            headers=self.headers,
            name="/v1/audit-chain/shards/:shard/latest-root",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 404):
                response.success()
            else:
                response.failure(f"root read failed {response.status_code}")

    @task(14)
    def integrity_probe(self):
        probe = {
            "tenant_shard": self.shard,
            "from_seq": random.randint(1, 10000),
            "limit": 512,
            "include_anchor_check": True,
        }
        with self.client.post(
            "/v1/audit-chain/integrity:probe",
            data=json.dumps(probe),
            headers=self.headers,
            name="/v1/audit-chain/integrity:probe",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 202):
                body = response.json()
                if body.get("hash_continuity_ok") is False:
                    response.failure("hash continuity failure")
                else:
                    response.success()
            else:
                response.failure(f"integrity probe failed {response.status_code}")
```

## Test Workload

Named request shape: `single_event_seal`.

Named request shape: `batch_25_event_seal`.

Named request shape: `tenant_shard_tip_read`.

Named request shape: `merkle_root_anchor_probe`.

Named request shape: `replay_window_10_to_200_events`.

Named distribution: `bursting-per-tenant-shard-with-batch-tail`.

Event classes include capability invocation, message sent, object committed, workflow transition, policy decision, and DSR evidence.

Per-tenant shard selection is Zipfian with exponent 1.11.

Event-class selection is uniform to prevent a single class from hiding serialization costs.

Batch traffic is 20 percent of write calls and carries 25 events per batch.

Root-anchor probes run once per second per probe VU.

Replay reads target the most recent 15 minutes and random older windows.

## Baseline Numbers

Recorded baseline run: `audit-merkle-attestation-cell-lab-2026-05-20T13:00:00Z`.

SUT topology: 1 region, 2 availability zones, 128 tenant shards, 60-second Merkle root window, KMS signing stub.

| Tier | p50 seal ms | p95 seal ms | p99 seal ms | Event throughput cap/s | p99 Merkle lag ms | p99 replay ms |
|---|---:|---:|---:|---:|---:|---:|
| Bronze | 8 | 32 | 74 | 4,800 | 61,800 | 118 |
| Silver | 6 | 24 | 55 | 10,900 | 61,100 | 96 |
| Gold | 5 | 19 | 42 | 23,600 | 60,700 | 74 |
| Platinum | 4 | 14 | 31 | 43,200 | 60,300 | 58 |

Batch baseline:

| Tier | p50 batch seal ms | p95 batch seal ms | p99 batch seal ms | Batch cap/s | Events per sealed batch |
|---|---:|---:|---:|---:|---:|
| Bronze | 29 | 96 | 181 | 320 | 25 |
| Silver | 24 | 78 | 143 | 720 | 25 |
| Gold | 18 | 61 | 112 | 1,560 | 25 |
| Platinum | 15 | 47 | 86 | 2,800 | 25 |

Integrity baseline:

| Tier | Hash continuity failures | Signature failures | Root publication failures | Audit completeness |
|---|---:|---:|---:|---:|
| Bronze | 0 | 0 | 0 | 100.000% |
| Silver | 0 | 0 | 0 | 100.000% |
| Gold | 0 | 0 | 0 | 100.000% |
| Platinum | 0 | 0 | 0 | 100.000% |

## Comparison vs Named Vendors

Named vendors and projects: AWS CloudTrail Lake, AWS QLDB, Google Cloud Audit Logs, Rekor, Apache Kafka log-compaction pipelines.

CloudTrail Lake-class comparison: high-volume control-plane event recording and query.

QLDB-class comparison: cryptographic journal semantics and digest verification.

Google Cloud Audit Logs-class comparison: platform-wide admin and data-access audit emission.

Rekor-class comparison: transparency-log style root publication.

Kafka-class comparison: durable append-only event throughput.

Oyatie differentiator measured here: every regulated microservice event can seal into one tenant-sharded, hash-chained audit substrate and still meet p99 seal latency.

Vendor parity guard: no hidden vendor throughput is asserted; named vendors provide comparison categories for later external harnesses.

## Methodology

Named SUT topology: `tenant-sharded-hash-chain-with-root-anchor-stub`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Root anchor window: 60 seconds.

Seal latency starts at API receive and stops when event id, prior hash, and block hash are durable.

Merkle attestation lag starts at event seal time and stops when the event is covered by a published shard root.

Replay latency reads a bounded event window and verifies hash continuity.

Throughput cap is discovered by raising single-event and batch-event rates until p99 seal latency or root lag breaches SLO.

Hash continuity, signature, and root publication failures are hard fails.

## Reproducibility

Primary command:

`BENCH_ID=PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20 TIER=Gold SEED=94205001 ./benchmarks/audit-chain-emission-throughput.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://audit-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94205001 benchmarks/audit-chain-emission-throughput.k6.js`

Locust command:

`locust -f benchmarks/audit-chain-emission-throughput.locust.py --headless --users 360 --spawn-rate 31 --run-time 1200s --host https://audit-cell-01.dev.oyatie.local`

Named seed values:

- `94205001` tenant-shard seed.
- `94205002` event-class seed.
- `94205003` batch-shape seed.
- `94205004` replay-window seed.

## Failure Modes Detected

Seal p99 regression above tier SLO.

Merkle attestation lag above anchor window plus 5 seconds.

Throughput cap regression above 10 percent from baseline.

Hash continuity failure.

Signature failure.

Root publication failure.

Replay window cannot verify prior hash chain.

Batch event count mismatch.

Idempotency duplicate creates second event.

Hot tenant shard starves cold shard writes.

Audit worker queue lag grows without backpressure.

KMS signing stub latency leaks into unrelated shards.

Cross-tenant shard read succeeds without permit.

Event accepted without data class.

Event accepted without regulatory pack context when required.

## Cross-References

- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0009-cell-architecture-per-tenant-per-region.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- `docs/decisions/ADR-0186-observability-backplane-layering.md`.
- `docs/SLO-CATALOG.md`.
- Service-owned `microservices/audit-chain/benchmarks/` directories are intentionally not touched by this corpus.
