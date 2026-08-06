---
doc_class: PerformanceBaselineIndex
consolidates:
  - benchmark_id: PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20
    source: benchmarks/audit-chain-emission-throughput.md
  - benchmark_id: PB-CEDAR-POLICY-EVAL-COST-2026-05-20
    source: benchmarks/cedar-policy-eval-cost.md
  - benchmark_id: PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20
    source: benchmarks/cross-µservice-call-overhead.md
  - benchmark_id: PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20
    source: benchmarks/drive-upload-download-throughput.md
  - benchmark_id: PB-MSG-THROUGHPUT-LATENCY-2026-05-20
    source: benchmarks/messenger-throughput-and-latency.md
  - benchmark_id: PB-MLS-KEY-DELIVERY-PERF-2026-05-20
    source: benchmarks/mls-key-delivery-perf.md
  - benchmark_id: PB-ONTOLOGY-QUERY-PERF-2026-05-20
    source: benchmarks/ontology-query-perf.md
  - benchmark_id: PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20
    source: benchmarks/workflow-engine-orchestration-cost.md
date: 2026-06-07
owner: ops-sre-performance
status: BaselineRecorded
consolidation_note: >
  This file consolidates all 8 top-level performance benchmark baselines into one
  document. The 8 source files have been git-rm'd. Gate keys in
  accounting-registry-producer and gate-baseline.generated.json still reference
  the original 8 paths; the spine MUST regen those producer faces after this
  consolidation so the gate-baseline debt keys map to PERFORMANCE-BASELINES.md
  instead of the 8 removed paths.
---

# Performance Baselines

This document is the single source of truth for all top-level Oyatie performance
benchmark baselines. It supersedes the 8 individual files that were previously at
`benchmarks/<name>.md`. Every `benchmark_id`, SLO, metric name, harness script,
baseline table, and cross-reference is preserved verbatim from the originals.

---

## Table of Contents

1. [Audit Chain Emission Throughput](#1-audit-chain-emission-throughput)
2. [Cedar Policy Eval Cost](#2-cedar-policy-eval-cost)
3. [Cross Microservice Call Overhead](#3-cross-microservice-call-overhead)
4. [Drive Upload Download Throughput](#4-drive-upload-download-throughput)
5. [Messenger Throughput And Latency](#5-messenger-throughput-and-latency)
6. [MLS Key Delivery Performance](#6-mls-key-delivery-performance)
7. [Ontology Query Performance](#7-ontology-query-performance)
8. [Workflow Engine Orchestration Cost](#8-workflow-engine-orchestration-cost)

---

## 1. Audit Chain Emission Throughput

**benchmark_id:** `PB-AUDIT-CHAIN-EMISSION-THROUGHPUT-2026-05-20`

**target_microservices:** audit-chain, observability, tenancy, eventing

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0009-cell-architecture-per-tenant-per-region, ADR-0139-agentic-slo-gated-promotion, ADR-0186-observability-backplane-layering

### Benchmark Goal

Named target metric: `audit_chain_events_sealed_per_second`.

Named lag metric: `audit_chain_merkle_attestation_lag_ms`.

Named SLO target: `SLO-AUDIT-EMISSION-P99`.

The SLO target is p99 event seal latency and p99 Merkle attestation lag per tier.

The throughput cap is the highest event-emission rate that preserves hash continuity, root publication, replayability, and audit completeness.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

### Test Harness

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

### Test Workload

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

### Baseline Numbers

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

### Comparison vs Named Vendors

Named vendors and projects: AWS CloudTrail Lake, AWS QLDB, Google Cloud Audit Logs, Rekor, Apache Kafka log-compaction pipelines.

CloudTrail Lake-class comparison: high-volume control-plane event recording and query.

QLDB-class comparison: cryptographic journal semantics and digest verification.

Google Cloud Audit Logs-class comparison: platform-wide admin and data-access audit emission.

Rekor-class comparison: transparency-log style root publication.

Kafka-class comparison: durable append-only event throughput.

Oyatie differentiator measured here: every regulated microservice event can seal into one tenant-sharded, hash-chained audit substrate and still meet p99 seal latency.

Vendor parity guard: no hidden vendor throughput is asserted; named vendors provide comparison categories for later external harnesses.

### Methodology

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

### Reproducibility

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

### Failure Modes Detected

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

### Cross-References

- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0706-observability-live-apex.md`.
- `docs/SLO-CATALOG.md`.
- Service-owned `microservices/audit-chain/benchmarks/` directories are intentionally not touched by this corpus.

---

## 2. Cedar Policy Eval Cost

**benchmark_id:** `PB-CEDAR-POLICY-EVAL-COST-2026-05-20`

**target_microservices:** policy-cedar, tenancy, identity, audit-chain

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0128-hyperscaler-architecture-invariants, ADR-0139-agentic-slo-gated-promotion

### Benchmark Goal

Named target metric: `cedar_eval_latency_ms`.

Named cache metric: `cedar_policy_cache_hit_rate`.

Named SLO target: `SLO-CEDAR-EVAL-P99`.

The SLO target is p99 Cedar authorization evaluation latency per policy-complexity band.

The benchmark records cache hit rate, policy parse cache pressure, entity graph lookup cost, deny-path cost, and audit decision emission cost.

The throughput cap is the highest evaluations per second that preserve p99 latency and zero incorrect permit or forbid decisions.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

### Test Harness

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

### Test Workload

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

### Baseline Numbers

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

### Methodology

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

### Reproducibility

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

### Failure Modes Detected

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

### Cross-References

- `specs/cedar-policy-schema.json`.
- `specs/cedar-fragment-schema.json`.
- `docs/standards/autonomy-ceiling.md`.
- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Service-owned `microservices/*/benchmarks/` directories are intentionally outside this root corpus change.

---

## 3. Cross Microservice Call Overhead

**benchmark_id:** `PB-CROSS-MICROSERVICE-CALL-OVERHEAD-2026-05-20`

**target_microservices:** api-gateway, service-mesh, tenancy, policy-cedar, observability, audit-chain

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0009-cell-architecture-per-tenant-per-region, ADR-0044-service-mesh-istio-ambient-and-envoy-gateway, ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation, ADR-0186-observability-backplane-layering

### Benchmark Goal

Named target metric: `cross_microservice_rpc_overhead_ms`.

Named handshake metric: `mesh_mtls_handshake_latency_ms`.

Named hop metric: `per_cell_hop_latency_ms`.

Named SLO target: `SLO-CROSS-MICROSERVICE-OVERHEAD-P99`.

The SLO target is p99 incremental overhead for one cross-microservice RPC hop, one mTLS connection establishment, and one per-cell hop.

The benchmark measures north-south edge admission, east-west service mesh hop cost, Cedar ext-authz cost, OTel trace propagation, audit sampling cost, and cross-cell egress cost.

The throughput cap is the highest cross-service RPC rate that preserves p99 overhead, zero plaintext traffic, zero missing trace context, and zero unregistered cross-cell calls.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

### Test Harness

Named load-generator topology: `mesh-hop-overhead-cell-lab`.

Topology nodes:

- Bash prepares route registrations, mesh policy fixtures, SPIFFE identities, tenant headers, and echo endpoints.
- k6 drives north-south and east-west RPC chains with 1, 2, 3, 5, and 8 hops.
- Locust drives cross-cell calls, cold mTLS handshakes, and trace propagation validation.
- The SUT includes Envoy gateway, Istio Ambient ztunnel, waypoint policy, Cedar ext-authz, observability collector, audit-chain sampler, and echo services.
- Prometheus records incremental hop overhead, mTLS handshake latency, cross-cell hop latency, trace propagation failures, and plaintext attempts.

### Test Workload

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

### Baseline Numbers

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

### Methodology

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

### Reproducibility

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

### Failure Modes Detected

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

### Cross-References

- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0706-observability-live-apex.md`.
- `docs/SLO-CATALOG.md`.
- Service-owned per-microservice benchmark directories remain untouched by this root corpus.

---

## 4. Drive Upload Download Throughput

**benchmark_id:** `PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20`

**target_microservices:** drive, tenancy, policy-cedar, audit-chain, object-storage

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0009-cell-architecture-per-tenant-per-region, ADR-0044-service-mesh-istio-ambient-and-envoy-gateway, ADR-0139-agentic-slo-gated-promotion

### Benchmark Goal

Named target metric: `drive_object_transfer_throughput_mib_per_second`.

Named latency metric: `drive_object_first_byte_latency_ms`.

Named SLO target: `SLO-DRV-TRANSFER-P99`.

The SLO target is p99 upload commit latency and p99 download first-byte latency per file-size band and per tier.

The benchmark finds the transfer throughput cap that keeps object integrity verification, tenant quota checks, Cedar checks, and audit-chain emission complete.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set; they are not a production availability or vendor-comparison claim.

### Test Harness

Named load-generator topology: `drive-band-matrix-cell-lab`.

Topology nodes:

- One bash fixture loader creates tenants, folders, quota classes, and sparse file payloads.
- k6 drives signed-upload create, chunk PUT, commit, metadata read, signed-download create, and ranged GET.
- Locust drives long-tail mixed downloads and concurrent resumable uploads.
- The SUT includes drive REST, drive worker, tenancy, Cedar, audit-chain, object-storage adapter, and malware-scan stub.
- Prometheus collects throughput, p50, p95, p99, checksum error rate, and quota-throttle counters.

### Test Workload

Named request shape: `small_1MiB_upload_commit`.

Named request shape: `medium_16MiB_resumable_upload`.

Named request shape: `large_128MiB_parallel_chunk_upload`.

Named request shape: `xlarge_512MiB_resumable_upload`.

Named request shape: `ranged_download_1MiB_first_byte`.

Named request shape: `full_download_band_sweep`.

Named distribution: `zipfian-hot-folder-with-uniform-band-sweep`.

Hot folders receive 60 percent of metadata reads.

File-size bands are swept uniformly during cap discovery.

The normal measurement run uses 42 percent small, 30 percent medium, 22 percent large, and 6 percent xlarge objects.

Uploads require object digest declaration before commit.

Downloads require signed URL creation before transfer.

Every committed upload emits a drive audit event.

Every cross-tenant denied metadata lookup emits a policy-denial audit event.

### Baseline Numbers

Recorded baseline run: `drive-band-matrix-cell-lab-2026-05-20T11:10:00Z`.

SUT topology: 1 region, 2 availability zones, 1 active cell, erasure-coded object-storage adapter, malware-scan stub enabled.

Upload baseline:

| Tier | Band | p50 commit ms | p95 commit ms | p99 commit ms | Throughput cap MiB/s |
|---|---|---:|---:|---:|---:|
| Bronze | small 1 MiB | 37 | 118 | 226 | 180 |
| Bronze | medium 16 MiB | 91 | 410 | 850 | 420 |
| Bronze | large 128 MiB | 510 | 1320 | 2450 | 760 |
| Bronze | xlarge 512 MiB | 1810 | 4920 | 8700 | 940 |
| Silver | small 1 MiB | 31 | 91 | 172 | 360 |
| Silver | medium 16 MiB | 76 | 310 | 620 | 840 |
| Silver | large 128 MiB | 420 | 1040 | 1880 | 1450 |
| Silver | xlarge 512 MiB | 1620 | 3990 | 7100 | 1760 |
| Gold | small 1 MiB | 24 | 70 | 133 | 690 |
| Gold | medium 16 MiB | 61 | 240 | 470 | 1610 |
| Gold | large 128 MiB | 330 | 830 | 1490 | 2850 |
| Gold | xlarge 512 MiB | 1310 | 3180 | 5900 | 3420 |
| Platinum | small 1 MiB | 20 | 54 | 104 | 1180 |
| Platinum | medium 16 MiB | 48 | 181 | 355 | 2850 |
| Platinum | large 128 MiB | 260 | 610 | 1090 | 5100 |
| Platinum | xlarge 512 MiB | 990 | 2440 | 4310 | 6200 |

Download baseline:

| Tier | Band | p50 TTFB ms | p95 TTFB ms | p99 TTFB ms | Throughput cap MiB/s |
|---|---|---:|---:|---:|---:|
| Bronze | small 1 MiB | 22 | 70 | 132 | 260 |
| Bronze | medium 16 MiB | 25 | 78 | 145 | 620 |
| Bronze | large 128 MiB | 31 | 90 | 168 | 980 |
| Bronze | xlarge 512 MiB | 39 | 120 | 235 | 1160 |
| Silver | small 1 MiB | 18 | 57 | 109 | 520 |
| Silver | medium 16 MiB | 21 | 65 | 124 | 1240 |
| Silver | large 128 MiB | 27 | 78 | 146 | 1900 |
| Silver | xlarge 512 MiB | 34 | 103 | 198 | 2280 |
| Gold | small 1 MiB | 15 | 43 | 82 | 980 |
| Gold | medium 16 MiB | 18 | 52 | 99 | 2300 |
| Gold | large 128 MiB | 23 | 63 | 119 | 3600 |
| Gold | xlarge 512 MiB | 29 | 86 | 166 | 4200 |
| Platinum | small 1 MiB | 12 | 35 | 68 | 1710 |
| Platinum | medium 16 MiB | 15 | 43 | 82 | 4100 |
| Platinum | large 128 MiB | 19 | 52 | 99 | 6500 |
| Platinum | xlarge 512 MiB | 24 | 69 | 134 | 7800 |

Integrity baseline:

| Tier | Checksum mismatch rate | Quota throttle correctness | Audit completeness |
|---|---:|---:|---:|
| Bronze | 0.000% | 100.000% | 100.000% |
| Silver | 0.000% | 100.000% | 100.000% |
| Gold | 0.000% | 100.000% | 100.000% |
| Platinum | 0.000% | 100.000% | 100.000% |

### Methodology

Named SUT topology: `drive-cell-object-adapter-with-scan-stub`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Each tier run starts from an empty namespace and preloads read fixtures before measurement.

Upload throughput counts committed object bytes only.

Download throughput counts bytes returned after signed URL authorization.

p99 upload latency is measured at commit completion, not final chunk receive.

p99 download latency is measured as first byte after signed URL creation.

Integrity validation requires checksum mismatch rate exactly zero.

Quota correctness requires all expected tier throttles to occur and no unexpected throttles outside configured caps.

### Reproducibility

Primary command:

`BENCH_ID=PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20 TIER=Gold SEED=94203001 ./benchmarks/drive-upload-download-throughput.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://drive-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94203001 benchmarks/drive-upload-download-throughput.k6.js`

Locust command:

`locust -f benchmarks/drive-upload-download-throughput.locust.py --headless --users 280 --spawn-rate 24 --run-time 1200s --host https://drive-cell-01.dev.oyatie.local`

Named seed values:

- `94203001` object-band seed.
- `94203002` hot-folder seed.
- `94203003` quota-throttle seed.
- `94203004` ranged-download seed.

### Failure Modes Detected

Upload commit p99 regression above tier SLO.

Download TTFB p99 regression above tier SLO.

Throughput cap regression above 10 percent from baseline.

Checksum mismatch above zero.

Committed object without audit event.

Download authorization without Cedar permit.

Signed URL reuse after expiry.

Quota throttle applied to the wrong tier.

Hot-folder metadata cache stampede.

Chunk commit accepts missing chunk.

Resumable upload duplicate chunk corrupts final digest.

Large-object transfer starves small-object first byte latency.

Cross-tenant metadata read returns object attributes.

Malware-scan stub timeout blocks unrelated tenants.

Object-storage adapter backpressure is not surfaced as retryable.

### Cross-References

- `docs/SLO-CATALOG.md`.
- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `microservices/drive/` remains service-owned and is intentionally not modified by this corpus.

---

## 5. Messenger Throughput And Latency

**benchmark_id:** `PB-MSG-THROUGHPUT-LATENCY-2026-05-20`

**target_microservices:** messenger, tenancy, policy-cedar, audit-chain

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0009-cell-architecture-per-tenant-per-region, ADR-0044-service-mesh-istio-ambient-and-envoy-gateway, ADR-0139-agentic-slo-gated-promotion

### Benchmark Goal

Named target metric: `messenger_message_delivery_latency_ms`.

Named throughput metric: `messenger_messages_delivered_per_second`.

Named SLO target: `SLO-MSG-DELIVERY-P99`.

The SLO target is p99 message delivery latency at or below 250 ms for Bronze, 200 ms for Silver, 160 ms for Gold, and 120 ms for Platinum during the steady-state window.

The throughput cap is the highest delivered-message rate that preserves delivery completeness at 99.99 percent and audit emission completeness at 100 percent.

This benchmark covers the send path, fanout path, receipt path, tenant-policy path, and audit emission path for direct messages, group messages, and large channels.

The comparison unit is one accepted message that reaches every intended recipient and produces the expected delivery receipts.

Baseline numbers below are recorded synthetic lab baselines for the named topology and seed set; they are not production claims.

### Test Harness

Named load-generator topology: `msg-four-tier-cell-lab`.

Topology nodes:

- One k6 coordinator generates HTTP send and read-receipt load.
- Three Locust workers generate WebSocket receive and acknowledgement pressure.
- One bash harness prepares tenants, channels, payloads, and extracts Prometheus snapshots.
- One cell-local SUT runs messenger REST, realtime gateway, tenancy, Cedar policy, audit-chain, and object storage stubs.
- Time is synchronized by cell NTP; samples with clock skew above 5 ms are rejected.

### Test Workload

Named request shape: `direct_dm_1_to_1`.

Named request shape: `group_thread_1_to_32`.

Named request shape: `large_channel_1_to_1000`.

Named request shape: `burst_channel_1_to_5000`.

Named distribution: `zipfian-with-burst-overlay`.

The Zipfian component selects hot tenants, hot channels, and hot participants with exponent 1.07.

The burst overlay injects a 45-second channel storm every 7 minutes.

Payload size defaults to 1 KiB UTF-8 body plus 512 bytes metadata.

Receipt pressure is enabled for every accepted message.

Audit probe lookup samples 1 percent of accepted messages.

Bronze uses a single-cell profile with 32 tenants and no cross-cell fanout.

Silver uses 64 tenants and one cross-namespace policy hop.

Gold uses 128 tenants and one cross-cell replication shadow.

Platinum uses 256 tenants, cross-cell shadow fanout, and strict delivery receipt audit validation.

### Baseline Numbers

Recorded baseline run: `msg-four-tier-cell-lab-2026-05-20T10:30:00Z`.

SUT topology: 1 region, 2 availability zones, 1 active cell, 1 shadow cell for Gold and Platinum.

| Tier | p50 delivery ms | p95 delivery ms | p99 delivery ms | Throughput cap msg/s | Receipt p99 ms | Audit completeness |
|---|---:|---:|---:|---:|---:|---:|
| Bronze | 24 | 118 | 213 | 1,450 | 241 | 100.000% |
| Silver | 21 | 92 | 174 | 3,100 | 205 | 100.000% |
| Gold | 18 | 71 | 139 | 6,350 | 166 | 100.000% |
| Platinum | 16 | 54 | 103 | 11,200 | 128 | 100.000% |

Per-shape baseline:

| Shape | Tier | p50 ms | p95 ms | p99 ms | Cap msg/s |
|---|---|---:|---:|---:|---:|
| direct_dm_1_to_1 | Bronze | 18 | 72 | 129 | 620 |
| direct_dm_1_to_1 | Platinum | 11 | 38 | 71 | 3,900 |
| group_thread_1_to_32 | Bronze | 25 | 116 | 211 | 530 |
| group_thread_1_to_32 | Platinum | 15 | 53 | 101 | 3,100 |
| large_channel_1_to_1000 | Bronze | 42 | 151 | 238 | 230 |
| large_channel_1_to_1000 | Platinum | 28 | 75 | 118 | 2,150 |
| burst_channel_1_to_5000 | Bronze | 59 | 181 | 249 | 70 |
| burst_channel_1_to_5000 | Platinum | 36 | 88 | 119 | 890 |

### Methodology

Named SUT topology: `single-region-dual-az-cell-with-shadow-replica`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 2 minutes.

Clock skew rejection threshold: 5 ms.

Outlier handling: no outlier trimming; p99 includes burst windows.

Throughput cap method: increase target arrival rate in 10 percent steps until p99 breaches the tier SLO or delivery completeness drops below 99.99 percent.

Latency measurement starts when REST accepts the message and stops when the recipient stream receives the delivery event.

Receipt latency measurement starts when delivery event is emitted and stops when receipt commit is visible.

Audit completeness is measured as accepted work-context messages divided by audit-chain message emission events.

Every run stores k6 JSON, Locust CSV, Prometheus snapshots, and fixture manifest.

### Reproducibility

Primary command:

`BENCH_ID=PB-MSG-THROUGHPUT-LATENCY-2026-05-20 TIER=Gold SEED=94202001 ./benchmarks/messenger-throughput-and-latency.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://msg-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94202001 benchmarks/messenger-throughput-and-latency.k6.js`

Locust command:

`locust -f benchmarks/messenger-throughput-and-latency.locust.py --headless --users 420 --spawn-rate 43 --run-time 1200s --host https://msg-cell-01.dev.oyatie.local`

Named seed values:

- `94202001` fixture shape seed.
- `94202002` hot-channel seed.
- `94202003` burst timing seed.
- `94202004` recipient cursor seed.

### Failure Modes Detected

Delivery p99 regression above tier SLO.

Throughput cap regression above 10 percent from baseline.

Receipt commit lag above delivery p99 by more than 80 ms.

WebSocket reconnect storm above 0.5 percent of sessions.

Hot-channel fanout queue saturation.

Tenant quota leak where Bronze tenants consume Silver worker capacity.

Cedar policy cache miss storm.

Audit-chain emission loss.

Idempotency-key duplicate delivery.

Cross-cell shadow replication lag above 2 seconds.

Unread cursor staleness above 500 ms.

Burst-channel backpressure without user-visible queued state.

Message accepted without `audit_event_id`.

Receipt accepted for the wrong tenant.

Channel read returning messages from a neighboring tenant.

### Cross-References

- `specs/microservices/messenger.json`.
- `docs/SLO-CATALOG.md`.
- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `microservices/messenger/` remains the service-owned doc set and is intentionally not modified by this corpus.

---

## 6. MLS Key Delivery Performance

**benchmark_id:** `PB-MLS-KEY-DELIVERY-PERF-2026-05-20`

**target_microservices:** messenger, identity, tenancy, audit-chain, policy-cedar

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0009-cell-architecture-per-tenant-per-region, ADR-0139-agentic-slo-gated-promotion

### Benchmark Goal

Named target metric: `mls_key_delivery_latency_ms`.

Named rotation metric: `mls_key_rotation_cadence_cost_ms`.

Named SLO target: `SLO-MLS-KEY-DELIVERY-P99`.

The benchmark covers RFC 9420-style MLS group key delivery, epoch advancement, member add/remove, device fanout, and rotation cadence cost.

The SLO target is p99 key package delivery latency and p99 epoch-rotation completion latency per group-size band and tier.

The throughput cap is the highest key-delivery and rotation rate that preserves zero unauthorized key delivery, zero stale epoch acceptance, and full audit evidence.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

### Test Harness

Named load-generator topology: `mls-epoch-rotation-cell-lab`.

Topology nodes:

- Bash prepares work and personal messenger groups, devices, identity keys, credential fixtures, and epoch states.
- k6 drives key package fetch, group commit, welcome delivery, update proposal, remove proposal, and epoch acknowledgement.
- Locust drives device reconnect, missed epoch recovery, and group membership churn.
- The SUT includes messenger key service, identity, tenancy, Cedar policy, audit-chain, and realtime delivery stubs.
- Prometheus records key delivery latency, epoch rotation latency, fanout queue lag, stale epoch rejection, and unauthorized key delivery.

### Test Workload

Named request shape: `key_package_delivery`.

Named request shape: `welcome_message_delivery`.

Named request shape: `epoch_rotation_cadence`.

Named request shape: `member_add_remove_rotation`.

Named request shape: `missed_epoch_recovery`.

Named request shape: `stale_epoch_negative_probe`.

Named distribution: `zipfian-group-hotset-with-burst-rotation`.

Group-size bands: dm 2 members, small 16 members, medium 128 members, large 1024 members, xlarge 5000 members.

Each member has 3 synthetic devices.

Rotation reasons include cadence, member remove, device update, and credential expiry.

Burst rotation windows run for 30 seconds every 6 minutes.

Unauthorized delivery probes use tenant and group mismatches.

Stale epoch probes attempt to submit messages two epochs behind current.

### Baseline Numbers

Recorded baseline run: `mls-epoch-rotation-cell-lab-2026-05-20T16:00:00Z`.

SUT topology: 1 region, 2 availability zones, messenger key service, identity key stub, realtime fanout stub, audit-chain sink.

Key delivery baseline:

| Tier | Group band | p50 delivery ms | p95 delivery ms | p99 delivery ms | Key delivery cap/s |
|---|---|---:|---:|---:|---:|
| Bronze | dm | 12 | 39 | 74 | 720 |
| Bronze | small | 18 | 61 | 118 | 610 |
| Bronze | medium | 39 | 146 | 310 | 420 |
| Bronze | large | 96 | 360 | 820 | 180 |
| Bronze | xlarge | 210 | 880 | 2100 | 48 |
| Silver | dm | 9 | 30 | 57 | 1,800 |
| Silver | small | 14 | 48 | 92 | 1,520 |
| Silver | medium | 31 | 112 | 240 | 1,060 |
| Silver | large | 76 | 280 | 630 | 470 |
| Silver | xlarge | 170 | 690 | 1640 | 120 |
| Gold | dm | 7 | 23 | 44 | 4,100 |
| Gold | small | 11 | 37 | 71 | 3,500 |
| Gold | medium | 24 | 86 | 182 | 2,460 |
| Gold | large | 59 | 214 | 482 | 1,080 |
| Gold | xlarge | 132 | 520 | 1260 | 280 |
| Platinum | dm | 6 | 18 | 34 | 7,600 |
| Platinum | small | 9 | 29 | 56 | 6,400 |
| Platinum | medium | 19 | 68 | 144 | 4,600 |
| Platinum | large | 47 | 168 | 378 | 2,020 |
| Platinum | xlarge | 105 | 410 | 990 | 520 |

Epoch rotation baseline:

| Tier | Group band | p50 rotation ms | p95 rotation ms | p99 rotation ms | Rotation cap/s |
|---|---|---:|---:|---:|---:|
| Bronze | dm | 21 | 80 | 160 | 65 |
| Bronze | small | 45 | 170 | 360 | 58 |
| Bronze | medium | 150 | 620 | 1390 | 41 |
| Bronze | large | 610 | 2400 | 5100 | 14 |
| Bronze | xlarge | 2100 | 7800 | 17100 | 3 |
| Silver | dm | 17 | 61 | 124 | 150 |
| Silver | small | 35 | 132 | 280 | 132 |
| Silver | medium | 118 | 480 | 1080 | 92 |
| Silver | large | 480 | 1880 | 4020 | 32 |
| Silver | xlarge | 1680 | 6100 | 13200 | 8 |
| Gold | dm | 13 | 48 | 98 | 350 |
| Gold | small | 28 | 101 | 216 | 310 |
| Gold | medium | 92 | 370 | 830 | 220 |
| Gold | large | 370 | 1460 | 3180 | 78 |
| Gold | xlarge | 1320 | 4850 | 10500 | 20 |
| Platinum | dm | 10 | 37 | 76 | 660 |
| Platinum | small | 22 | 79 | 168 | 580 |
| Platinum | medium | 72 | 288 | 650 | 410 |
| Platinum | large | 290 | 1140 | 2480 | 145 |
| Platinum | xlarge | 1040 | 3820 | 8200 | 38 |

Integrity baseline:

| Tier | Unauthorized key deliveries | Stale epoch accepts | Key audit completeness |
|---|---:|---:|---:|
| Bronze | 0 | 0 | 100.000% |
| Silver | 0 | 0 | 100.000% |
| Gold | 0 | 0 | 100.000% |
| Platinum | 0 | 0 | 100.000% |

### Methodology

Named SUT topology: `messenger-mls-key-service-with-identity-and-audit`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Key delivery latency starts at key-service receive and stops when delivery is visible to the target device queue.

Epoch rotation latency starts at commit proposal accept and stops when all active devices have a deliverable epoch update.

Missed epoch recovery starts at recovery request and stops when the device receives the minimal required update chain.

Unauthorized delivery and stale epoch acceptance are hard-fail counters.

Throughput cap is raised until p99, unauthorized delivery, stale epoch acceptance, or audit completeness breaches.

### Reproducibility

Primary command:

`BENCH_ID=PB-MLS-KEY-DELIVERY-PERF-2026-05-20 TIER=Gold SEED=94208001 ./benchmarks/mls-key-delivery-perf.sh`

k6 command:

`k6 run -e SUT_BASE_URL=https://msg-cell-01.dev.oyatie.local -e TIER=Gold -e SEED=94208001 benchmarks/mls-key-delivery-perf.k6.js`

Locust command:

`locust -f benchmarks/mls-key-delivery-perf.locust.py --headless --users 320 --spawn-rate 27 --run-time 1200s --host https://msg-cell-01.dev.oyatie.local`

Named seed values:

- `94208001` group-band seed.
- `94208002` device fanout seed.
- `94208003` rotation reason seed.
- `94208004` stale epoch seed.

### Failure Modes Detected

Key delivery p99 regression above group-band SLO.

Epoch rotation p99 regression above group-band SLO.

Throughput cap regression above 10 percent from baseline.

Unauthorized key delivery.

Stale epoch accepted.

Member remove does not revoke key access.

Missed epoch recovery returns an incomplete update chain.

Large group rotation starves DM key delivery.

Device update creates duplicate epoch.

Audit event missing for work-context key rotation.

Tenant mismatch accepted for key package fetch.

Credential expiry does not trigger rotation.

Fanout queue lag grows without backpressure.

Idempotency duplicate creates second commit.

RFC 9420 epoch metadata missing from response.

### Cross-References

- `specs/microservices/messenger.json`.
- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- RFC 9420 protocol behavior is represented by synthetic compatible fixtures in this benchmark, not by external service measurements.
- Service-owned `microservices/messenger/benchmarks/` remains untouched by this root corpus.

---

## 7. Ontology Query Performance

**benchmark_id:** `PB-ONTOLOGY-QUERY-PERF-2026-05-20`

**target_microservices:** ontology, tenancy, policy-cedar, observability, audit-chain

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0139-agentic-slo-gated-promotion, ADR-0142-crdt-portability-trait

### Benchmark Goal

Named target metric: `ontology_query_latency_ms`.

Named scaling metric: `ontology_query_throughput_per_second`.

Named SLO target: `SLO-ONTOLOGY-QUERY-P99`.

The SLO target is p99 ontology query latency per graph-depth band and per tenant-data-volume band.

The benchmark covers semantic entity reads, typed edge traversal, kinetic action receipts, and dynamic-state freshness joins.

The throughput cap is the highest query rate that preserves p99 latency, Cedar tenant isolation, freshness budgets, and zero cross-tenant result leaks.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

### Test Harness

Named load-generator topology: `ontology-depth-volume-cell-lab`.

Topology nodes:

- Bash prepares tenant graph fixtures, entity types, edge types, action receipts, and dynamic-state streams.
- k6 drives traversal, filter, projection, and action-receipt queries.
- Locust drives dashboard-style mixed reads and stale-state probes.
- The SUT includes ontology REST, ontology query worker, Postgres/RLS graph store, Cedar policy API, dynamic-state adapter, and audit-chain sink.
- Prometheus records query latency by graph depth, data volume, tenant, query shape, freshness lag, and cross-tenant denial counts.

### Test Workload

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

### Baseline Numbers

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

### Methodology

Named SUT topology: `ontology-rls-graph-store-with-dynamic-state-adapter`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Query latency starts at API receive and stops after result serialization.

Dynamic freshness lag is measured from source telemetry timestamp to query response.

Cross-tenant leak detection scans every returned row for tenant id mismatch.

Throughput cap is raised until p99 latency, freshness budget, or leak count breaches.

No outlier trimming is applied.

### Reproducibility

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

### Failure Modes Detected

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

### Cross-References

- `specs/microservices/ontology.json`.
- `registry/knowledge-graph-kinetic.json`.
- `registry/knowledge-graph-dynamic.json`.
- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0704-k8s-port-live-apex.md`.
- Service-owned `microservices/ontology/benchmarks/` remains untouched by this root corpus.

---

## 8. Workflow Engine Orchestration Cost

**benchmark_id:** `PB-WORKFLOW-ENGINE-ORCHESTRATION-COST-2026-05-20`

**target_microservices:** workflow, workflow-engine, ontology, policy-cedar, audit-chain

**status:** BaselineRecorded | **date:** 2026-05-20 | **owner:** ops-sre-performance

**related_oyatie_adrs:** ADR-0003-audit-chain-and-evidence-emission, ADR-0007-cedar-authorization-policy-and-persona-tier, ADR-0035-workflow-engine-state-machine-and-dag-hybrid, ADR-0139-agentic-slo-gated-promotion

### Benchmark Goal

Named target metric: `workflow_step_transition_latency_ms`.

Named throughput metric: `workflow_durable_transitions_per_second`.

Named compensation metric: `workflow_saga_compensation_latency_ms`.

Named SLO target: `SLO-WORKFLOW-ORCHESTRATION-P99`.

The SLO target is p99 durable step-transition latency per workflow shape and p99 saga-compensation latency after injected failure.

The benchmark finds durable-function throughput cap while preserving idempotency, audit emission, replay determinism, and zero duplicate side effects.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

### Test Harness

Named load-generator topology: `workflow-durable-saga-cell-lab`.

Topology nodes:

- Bash prepares workflow definitions, version pins, ontology action fixtures, Cedar permits, and failure-injection switches.
- k6 starts workflow instances and injects deterministic step failures.
- Locust polls timelines, triggers retries, reads replay state, and validates compensation completion.
- The SUT includes workflow REST, workflow worker, durable state store, ontology adapter, Cedar policy API, audit-chain sink, and external-effect stub.
- Prometheus records transition latency, instance throughput, replay latency, compensation latency, duplicate side effects, and audit completeness.

### Test Workload

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

### Baseline Numbers

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

### Methodology

Named SUT topology: `durable-state-worker-pool-with-policy-and-audit`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Transition latency starts when the worker claims a step and stops when durable state and audit evidence are committed.

Instance start latency starts at REST receive and stops when the instance is visible to workers.

Compensation latency starts at injected failure detection and stops when all declared compensations are durable.

Replay probe latency measures deterministic reconstruction without performing side effects.

Throughput cap is raised until p99 transition latency, duplicate side effects, or audit completeness breach.

### Reproducibility

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

### Failure Modes Detected

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

### Cross-References

- `specs/microservices/workflow.json`.
- `specs/microservices/ontology.json`.
- `docs/standards/saga-compensation-policy.md`.
- `docs/decisions/ADR-0709-general-live-apex.md`.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Service-owned `microservices/workflow-engine/benchmarks/` remains untouched by this root corpus.
