---
doc_class: PerformanceBenchmark
benchmark_id: PB-MSG-THROUGHPUT-LATENCY-2026-05-20
target_microservices:
  - messenger
  - tenancy
  - policy-cedar
  - audit-chain
status: BaselineRecorded
date: 2026-05-20
owner: ops-sre-performance
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0044-service-mesh-istio-ambient-and-envoy-gateway
  - ADR-0139-agentic-slo-gated-promotion
---

# Messenger Throughput And Latency Benchmark

## Benchmark Goal

Named target metric: `messenger_message_delivery_latency_ms`.

Named throughput metric: `messenger_messages_delivered_per_second`.

Named SLO target: `SLO-MSG-DELIVERY-P99`.

The SLO target is p99 message delivery latency at or below 250 ms for Bronze, 200 ms for Silver, 160 ms for Gold, and 120 ms for Platinum during the steady-state window.

The throughput cap is the highest delivered-message rate that preserves delivery completeness at 99.99 percent and audit emission completeness at 100 percent.

This benchmark covers the send path, fanout path, receipt path, tenant-policy path, and audit emission path for direct messages, group messages, and large channels.

The comparison unit is one accepted message that reaches every intended recipient and produces the expected delivery receipts.

Baseline numbers below are recorded synthetic lab baselines for the named topology and seed set; they are not production claims.

## Test Harness

Named load-generator topology: `msg-four-tier-cell-lab`.

Topology nodes:

- One k6 coordinator generates HTTP send and read-receipt load.
- Three Locust workers generate WebSocket receive and acknowledgement pressure.
- One bash harness prepares tenants, channels, payloads, and extracts Prometheus snapshots.
- One cell-local SUT runs messenger REST, realtime gateway, tenancy, Cedar policy, audit-chain, and object storage stubs.
- Time is synchronized by cell NTP; samples with clock skew above 5 ms are rejected.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-MSG-THROUGHPUT-LATENCY-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://msg-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-msg}"
SEED="${SEED:-94202001}"
TIER="${TIER:-Bronze}"
CHANNEL_COUNT="${CHANNEL_COUNT:-64}"
DIRECT_USERS="${DIRECT_USERS:-800}"
GROUP_USERS="${GROUP_USERS:-2400}"
CHANNEL_USERS="${CHANNEL_USERS:-12000}"
PAYLOAD_BYTES="${PAYLOAD_BYTES:-1024}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/messenger}"

mkdir -p "${OUTPUT_DIR}"

case "${TIER}" in
  Bronze)
    VUS=120
    RATE=1200
    EXPECTED_P99_MS=250
    ;;
  Silver)
    VUS=220
    RATE=2600
    EXPECTED_P99_MS=200
    ;;
  Gold)
    VUS=420
    RATE=5400
    EXPECTED_P99_MS=160
    ;;
  Platinum)
    VUS=760
    RATE=9800
    EXPECTED_P99_MS=120
    ;;
  *)
    echo "unknown tier: ${TIER}" >&2
    exit 64
    ;;
esac

printf '{"bench_id":"%s","tier":"%s","seed":%s}\n' "${BENCH_ID}" "${TIER}" "${SEED}" > "${OUTPUT_DIR}/run.jsonl"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/reset" \
  -H "content-type: application/json" \
  -d "{\"bench_id\":\"${BENCH_ID}\",\"tenant_prefix\":\"${TENANT_PREFIX}\",\"seed\":${SEED}}" \
  >> "${OUTPUT_DIR}/run.jsonl"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/messenger/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"channel_count\":${CHANNEL_COUNT},
    \"direct_users\":${DIRECT_USERS},
    \"group_users\":${GROUP_USERS},
    \"channel_users\":${CHANNEL_USERS},
    \"payload_bytes\":${PAYLOAD_BYTES},
    \"distribution\":\"zipfian-with-burst-overlay\",
    \"seed\":${SEED}
  }" \
  >> "${OUTPUT_DIR}/fixture.json"

echo "warmup_seconds=${WARMUP_SECONDS}" | tee "${OUTPUT_DIR}/warmup.txt"
k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TARGET_RATE="${RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -e PAYLOAD_BYTES="${PAYLOAD_BYTES}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/messenger-throughput-and-latency.k6.js

locust \
  -f benchmarks/messenger-throughput-and-latency.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 10 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_messenger_delivery_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m])) by (le))" \
  > "${OUTPUT_DIR}/prom-p99-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_messenger_messages_delivered_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\"}[15m]))" \
  > "${OUTPUT_DIR}/prom-throughput-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_audit_chain_events_total{bench_id=\"${BENCH_ID}\",microservice=\"messenger\"}[15m])) / sum(rate(oya_messenger_messages_accepted_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/prom-audit-ratio-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --arg expected_p99_ms "${EXPECTED_P99_MS}" \
  --slurpfile p99 "${OUTPUT_DIR}/prom-p99-${TIER}.json" \
  --slurpfile throughput "${OUTPUT_DIR}/prom-throughput-${TIER}.json" \
  --slurpfile audit "${OUTPUT_DIR}/prom-audit-ratio-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,expected_p99_ms:($expected_p99_ms|tonumber),p99:$p99[0],throughput:$throughput[0],audit_ratio:$audit[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "messenger benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

export const deliveryLatency = new Trend('messenger_delivery_latency_ms', true);
export const receiptLatency = new Trend('messenger_receipt_latency_ms', true);
export const sendFailures = new Rate('messenger_send_failures');
export const delivered = new Counter('messenger_delivered_messages');
export const auditTagged = new Counter('messenger_audit_tagged_messages');

const baseUrl = __ENV.SUT_BASE_URL;
const tier = __ENV.TIER || 'Bronze';
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-msg';
const payloadBytes = Number(__ENV.PAYLOAD_BYTES || 1024);
const seed = Number(__ENV.SEED || 94202001);
const targetRate = Number(__ENV.TARGET_RATE || 1200);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    send_mix: {
      executor: 'constant-arrival-rate',
      rate: targetRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(50, Math.floor(targetRate / 20)),
      maxVUs: Math.max(200, Math.floor(targetRate / 5)),
      exec: 'sendMessage',
    },
    read_receipts: {
      executor: 'ramping-vus',
      stages: [
        { duration: `${Math.floor(warmupSeconds / 2)}s`, target: Math.max(10, Math.floor(targetRate / 100)) },
        { duration: `${warmupSeconds + measureSeconds}s`, target: Math.max(20, Math.floor(targetRate / 60)) },
        { duration: '30s', target: 0 },
      ],
      exec: 'receiptLoop',
    },
  },
  thresholds: {
    messenger_delivery_latency_ms: ['p(99)<300'],
    messenger_receipt_latency_ms: ['p(99)<350'],
    messenger_send_failures: ['rate<0.001'],
  },
};

function pickShape(iteration) {
  const v = (iteration + seed) % 100;
  if (v < 58) return 'direct_dm';
  if (v < 86) return 'group_thread';
  if (v < 96) return 'large_channel';
  return 'burst_channel';
}

function payload(shape) {
  const body = 'm'.repeat(payloadBytes);
  return JSON.stringify({
    tenant_id: `${tenantPrefix}-${__VU % 32}`,
    client_msg_id: `msg-${seed}-${__VU}-${__ITER}`,
    shape,
    body,
    idempotency_key: `idem-${seed}-${__VU}-${__ITER}`,
    requested_receipts: true,
    audit_probe: true,
    tier,
  });
}

export function sendMessage() {
  const shape = pickShape(__ITER);
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/messenger/messages`, payload(shape), {
    headers: {
      'content-type': 'application/json',
      'x-oya-benchmark-id': 'PB-MSG-THROUGHPUT-LATENCY-2026-05-20',
      'x-oya-tier': tier,
    },
    tags: { shape, tier },
  });
  const ok = check(res, {
    'accepted': (r) => r.status === 202 || r.status === 201,
    'has delivery id': (r) => !!r.json('delivery_id'),
    'has audit id': (r) => !!r.json('audit_event_id'),
  });
  sendFailures.add(!ok);
  if (ok) {
    const ms = Number(res.json('delivery_latency_ms') || (Date.now() - started));
    deliveryLatency.add(ms, { shape, tier });
    delivered.add(1, { shape, tier });
    auditTagged.add(1, { shape, tier });
  }
}

export function receiptLoop() {
  const tenant = `${tenantPrefix}-${__VU % 32}`;
  const url = `${baseUrl.replace('https://', 'wss://')}/v1/messenger/ws?tenant_id=${tenant}&tier=${tier}`;
  ws.connect(url, {}, function (socket) {
    socket.on('message', function (data) {
      const event = JSON.parse(data);
      if (event.type === 'delivery') {
        const now = Date.now();
        receiptLatency.add(now - event.sent_at_ms, { tier, shape: event.shape || 'unknown' });
        socket.send(JSON.stringify({
          type: 'ack',
          delivery_id: event.delivery_id,
          received_at_ms: now,
          idempotency_key: `ack-${event.delivery_id}`,
        }));
      }
    });
    socket.setTimeout(function () {
      socket.close();
    }, 1000);
  });
  sleep(1);
}
```

```python
from __future__ import annotations

import json
import os
import random
import time
from gevent import sleep
from locust import HttpUser, between, events, task

BENCH_ID = "PB-MSG-THROUGHPUT-LATENCY-2026-05-20"
SEED = int(os.getenv("SEED", "94202001"))
PAYLOAD_BYTES = int(os.getenv("PAYLOAD_BYTES", "1024"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-msg")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class MessengerRecipientUser(HttpUser):
    wait_time = between(0.01, 0.08)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 31)}"
        self.user_id = f"user-{random.randint(1, 12000)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
            "x-oya-recipient-user": self.user_id,
        }

    @task(45)
    def poll_direct_inbox(self):
        start = time.perf_counter()
        with self.client.get(
            f"/v1/messenger/inbox/{self.user_id}",
            headers=self.headers,
            name="/v1/messenger/inbox/:user_id",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code != 200:
                response.failure(f"inbox read failed {response.status_code}")
                return
            body = response.json()
            if body.get("stale") is True:
                response.failure("inbox read returned stale=true")
                return
            events.request.fire(request_type="CHECK", name="inbox_fresh_ms", response_time=elapsed_ms, response_length=0)
            response.success()

    @task(25)
    def ack_pending_delivery(self):
        delivery_id = f"synthetic-{SEED}-{self.user_id}-{random.randint(1, 100000)}"
        payload = {
            "delivery_id": delivery_id,
            "tenant_id": self.tenant_id,
            "received_at_ms": int(time.time() * 1000),
            "client_clock_skew_ms": random.randint(-3, 3),
            "idempotency_key": f"ack-{delivery_id}",
        }
        with self.client.post(
            "/v1/messenger/receipts",
            data=json.dumps(payload),
            headers=self.headers,
            name="/v1/messenger/receipts",
            catch_response=True,
        ) as response:
            if response.status_code not in (200, 202, 409):
                response.failure(f"receipt rejected {response.status_code}")
            else:
                response.success()

    @task(20)
    def read_channel_cursor(self):
        channel_id = f"channel-{random.randint(1, 64)}"
        cursor = random.randint(1, 1000000)
        with self.client.get(
            f"/v1/messenger/channels/{channel_id}/messages?cursor={cursor}&limit=50",
            headers=self.headers,
            name="/v1/messenger/channels/:channel/messages",
            catch_response=True,
        ) as response:
            if response.status_code != 200:
                response.failure(f"channel cursor failed {response.status_code}")
                return
            if "messages" not in response.json():
                response.failure("channel cursor missing messages")
                return
            response.success()

    @task(10)
    def verify_audit_probe(self):
        probe_id = f"probe-{SEED}-{random.randint(1, 250000)}"
        with self.client.get(
            f"/internal/bench/audit-probes/{probe_id}",
            headers=self.headers,
            name="/internal/bench/audit-probes/:probe_id",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 404):
                response.success()
            else:
                response.failure(f"audit probe lookup failed {response.status_code}")
        sleep(0.02)
```

## Test Workload

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

## Baseline Numbers

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

## Comparison vs Named Vendors

Named vendors: Slack, Microsoft Teams, Google Chat, Discord, Matrix Synapse.

Comparison scope is category-level messenger delivery behavior, not hidden vendor internals.

Slack-class comparison: channel fanout latency and unread cursor freshness.

Microsoft Teams-class comparison: tenant-scoped enterprise compliance and audit receipts.

Google Chat-class comparison: workspace identity integration and cross-device delivery.

Discord-class comparison: large channel burst fanout and WebSocket stability.

Matrix Synapse-class comparison: federated delivery overhead and durable room state.

Oyatie differentiator measured here: every work-context message records audit evidence without dropping p99 below the tier SLO.

Vendor parity guard: no claim is made that Oyatie is faster than a named vendor without an external vendor test run using the same workload.

The benchmark records whether Oyatie stays inside its own SLO while preserving tenant isolation, Cedar policy evaluation, and audit-chain emission.

## Methodology

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

## Reproducibility

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

Required environment variables:

- `SUT_BASE_URL`.
- `PROM_URL`.
- `TIER`.
- `SEED`.
- `TENANT_PREFIX`.

## Failure Modes Detected

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

## Cross-References

- `specs/microservices/messenger.json`.
- `docs/SLO-CATALOG.md`.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0009-cell-architecture-per-tenant-per-region.md`.
- `docs/decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- `microservices/messenger/` remains the service-owned doc suite and is intentionally not modified by this corpus.
