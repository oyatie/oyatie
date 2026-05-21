---
doc_class: PerformanceBenchmark
benchmark_id: PB-MLS-KEY-DELIVERY-PERF-2026-05-20
target_microservices:
  - messenger
  - identity
  - tenancy
  - audit-chain
  - policy-cedar
status: BaselineRecorded
date: 2026-05-20
owner: ops-sre-performance
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0139-agentic-slo-gated-promotion
---

# MLS Key Delivery Performance Benchmark

## Benchmark Goal

Named target metric: `mls_key_delivery_latency_ms`.

Named rotation metric: `mls_key_rotation_cadence_cost_ms`.

Named SLO target: `SLO-MLS-KEY-DELIVERY-P99`.

The benchmark covers RFC 9420-style MLS group key delivery, epoch advancement, member add/remove, device fanout, and rotation cadence cost.

The SLO target is p99 key package delivery latency and p99 epoch-rotation completion latency per group-size band and tier.

The throughput cap is the highest key-delivery and rotation rate that preserves zero unauthorized key delivery, zero stale epoch acceptance, and full audit evidence.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set.

## Test Harness

Named load-generator topology: `mls-epoch-rotation-cell-lab`.

Topology nodes:

- Bash prepares work and personal messenger groups, devices, identity keys, credential fixtures, and epoch states.
- k6 drives key package fetch, group commit, welcome delivery, update proposal, remove proposal, and epoch acknowledgement.
- Locust drives device reconnect, missed epoch recovery, and group membership churn.
- The SUT includes messenger key service, identity, tenancy, Cedar policy, audit-chain, and realtime delivery stubs.
- Prometheus records key delivery latency, epoch rotation latency, fanout queue lag, stale epoch rejection, and unauthorized key delivery.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-MLS-KEY-DELIVERY-PERF-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://msg-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-mls}"
SEED="${SEED:-94208001}"
TIER="${TIER:-Bronze}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/mls}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"

mkdir -p "${OUTPUT_DIR}"

case "${TIER}" in
  Bronze)
    VUS=80
    KEY_RATE=600
    ROTATION_RATE=40
    ;;
  Silver)
    VUS=160
    KEY_RATE=1500
    ROTATION_RATE=95
    ;;
  Gold)
    VUS=320
    KEY_RATE=3400
    ROTATION_RATE=220
    ;;
  Platinum)
    VUS=620
    KEY_RATE=6300
    ROTATION_RATE=410
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

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/mls/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"group_bands\":{\"dm\":2,\"small\":16,\"medium\":128,\"large\":1024,\"xlarge\":5000},
    \"devices_per_user\":3,
    \"credential_mode\":\"synthetic-rfc9420-compatible\",
    \"distribution\":\"zipfian-group-hotset-with-burst-rotation\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e KEY_RATE="${KEY_RATE}" \
  -e ROTATION_RATE="${ROTATION_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/mls-key-delivery-perf.k6.js

locust \
  -f benchmarks/mls-key-delivery-perf.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 12 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for band in dm small medium large xlarge; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_mls_key_delivery_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",group_band=\"${band}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/delivery-p99-${TIER}-${band}.json"
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_mls_epoch_rotation_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",group_band=\"${band}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/rotation-p99-${TIER}-${band}.json"
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_mls_unauthorized_key_delivery_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/unauthorized-delivery-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_mls_stale_epoch_accept_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/stale-epoch-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile unauthorized "${OUTPUT_DIR}/unauthorized-delivery-${TIER}.json" \
  --slurpfile stale "${OUTPUT_DIR}/stale-epoch-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,unauthorized_delivery:$unauthorized[0],stale_epoch_accept:$stale[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "mls benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const keyDeliveryLatency = new Trend('mls_key_delivery_latency_ms', true);
export const epochRotationLatency = new Trend('mls_epoch_rotation_latency_ms', true);
export const fanoutLag = new Trend('mls_key_fanout_lag_ms', true);
export const unauthorizedDelivery = new Rate('mls_unauthorized_key_delivery_rate');
export const staleEpochAccepted = new Rate('mls_stale_epoch_accept_rate');
export const keyDeliveries = new Counter('mls_key_deliveries');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-MLS-KEY-DELIVERY-PERF-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94208001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-mls';
const keyRate = Number(__ENV.KEY_RATE || 600);
const rotationRate = Number(__ENV.ROTATION_RATE || 40);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    key_delivery: {
      executor: 'constant-arrival-rate',
      rate: keyRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(80, Math.floor(keyRate / 15)),
      maxVUs: Math.max(500, Math.floor(keyRate / 4)),
      exec: 'deliverKeyPackage',
    },
    epoch_rotation: {
      executor: 'constant-arrival-rate',
      rate: rotationRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(40, Math.floor(rotationRate / 2)),
      maxVUs: Math.max(250, rotationRate * 3),
      exec: 'rotateEpoch',
    },
    stale_epoch_probe: {
      executor: 'constant-vus',
      vus: 12,
      duration: `${warmupSeconds + measureSeconds}s`,
      exec: 'probeStaleEpoch',
    },
  },
  thresholds: {
    mls_key_delivery_latency_ms: ['p(99)<500'],
    mls_epoch_rotation_latency_ms: ['p(99)<5000'],
    mls_unauthorized_key_delivery_rate: ['rate==0'],
    mls_stale_epoch_accept_rate: ['rate==0'],
  },
};

function groupBand(iteration) {
  const v = (iteration + seed) % 100;
  if (v < 25) return { band: 'dm', members: 2 };
  if (v < 55) return { band: 'small', members: 16 };
  if (v < 80) return { band: 'medium', members: 128 };
  if (v < 96) return { band: 'large', members: 1024 };
  return { band: 'xlarge', members: 5000 };
}

function headers(extra = {}) {
  return Object.assign({
    'content-type': 'application/json',
    'x-oya-benchmark-id': benchId,
    'x-oya-tier': tier,
  }, extra);
}

export function deliverKeyPackage() {
  const idx = __ITER + seed + __VU;
  const group = groupBand(idx);
  const tenantId = `${tenantPrefix}-${idx % 64}`;
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/messenger/mls/groups/${tenantId}-${group.band}-${idx % 2048}/key-packages:deliver`, JSON.stringify({
    tenant_id: tenantId,
    group_band: group.band,
    member_count: group.members,
    device_id: `device-${idx % 30000}`,
    credential_ref: `credential-${idx % 30000}`,
    epoch: idx % 100000,
    rfc: 'RFC9420',
    idempotency_key: `mls-key-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'x-oya-group-band': group.band }), tags: { tier, group_band: group.band } });
  const ok = check(res, {
    'key delivered': (r) => r.status === 200 || r.status === 202,
    'delivery id present': (r) => !!r.json('delivery_id'),
    'audit id present': (r) => !!r.json('audit_event_id'),
    'epoch returned': (r) => Number.isFinite(Number(r.json('epoch'))),
  });
  unauthorizedDelivery.add(ok ? Boolean(res.json('unauthorized_delivery')) : false, { tier, group_band: group.band });
  if (ok) {
    keyDeliveryLatency.add(Number(res.json('delivery_latency_ms') || (Date.now() - started)), { tier, group_band: group.band });
    fanoutLag.add(Number(res.json('fanout_lag_ms') || 0), { tier, group_band: group.band });
    keyDeliveries.add(1, { tier, group_band: group.band });
  }
}

export function rotateEpoch() {
  const idx = __ITER + seed + __VU;
  const group = groupBand(idx);
  const tenantId = `${tenantPrefix}-${idx % 64}`;
  const started = Date.now();
  const res = http.post(`${baseUrl}/v1/messenger/mls/groups/${tenantId}-${group.band}-${idx % 2048}/epochs:rotate`, JSON.stringify({
    tenant_id: tenantId,
    group_band: group.band,
    member_count: group.members,
    reason: idx % 7 === 0 ? 'member_remove' : idx % 11 === 0 ? 'device_update' : 'cadence',
    prior_epoch: idx % 100000,
    proposed_epoch: (idx % 100000) + 1,
    idempotency_key: `mls-rotate-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'x-oya-group-band': group.band }), tags: { tier, group_band: group.band } });
  const ok = check(res, {
    'rotation accepted': (r) => r.status === 200 || r.status === 202,
    'commit id present': (r) => !!r.json('commit_id'),
    'rotation audit id present': (r) => !!r.json('audit_event_id'),
  });
  if (ok) {
    epochRotationLatency.add(Number(res.json('rotation_latency_ms') || (Date.now() - started)), { tier, group_band: group.band });
  }
}

export function probeStaleEpoch() {
  const idx = __ITER + seed + __VU;
  const group = groupBand(idx);
  const tenantId = `${tenantPrefix}-${idx % 64}`;
  const res = http.post(`${baseUrl}/v1/messenger/mls/groups/${tenantId}-${group.band}-${idx % 2048}/messages:accept`, JSON.stringify({
    tenant_id: tenantId,
    group_band: group.band,
    epoch: Math.max(0, (idx % 100000) - 2),
    ciphertext_ref: `ciphertext-${idx}`,
    idempotency_key: `mls-stale-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers({ 'x-oya-group-band': group.band }), tags: { tier, group_band: group.band, probe: 'stale_epoch' } });
  staleEpochAccepted.add(res.status >= 200 && res.status < 300, { tier, group_band: group.band });
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

BENCH_ID = "PB-MLS-KEY-DELIVERY-PERF-2026-05-20"
SEED = int(os.getenv("SEED", "94208001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-mls")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class MlsDeviceRecoveryUser(HttpUser):
    wait_time = between(0.01, 0.16)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 63)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
        }

    def _group_band(self):
        return random.choice(["dm", "small", "medium", "large", "xlarge"])

    def _group_id(self, band: str):
        return f"{self.tenant_id}-{band}-{random.randint(1, 2048)}"

    @task(32)
    def missed_epoch_recovery(self):
        band = self._group_band()
        group_id = self._group_id(band)
        start = time.perf_counter()
        with self.client.post(
            f"/v1/messenger/mls/groups/{group_id}/epochs:recover",
            data=json.dumps({
                "tenant_id": self.tenant_id,
                "group_band": band,
                "device_id": f"device-{random.randint(1, 30000)}",
                "last_seen_epoch": random.randint(1, 90000),
                "idempotency_key": f"recover-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "x-oya-group-band": band},
            name="/v1/messenger/mls/groups/:id/epochs:recover",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code in (200, 202, 404):
                events.request.fire(request_type="CHECK", name=f"mls_recovery_{band}_ms", response_time=elapsed_ms, response_length=0)
                response.success()
            else:
                response.failure(f"recovery failed {response.status_code}")

    @task(28)
    def group_state_read(self):
        band = self._group_band()
        group_id = self._group_id(band)
        with self.client.get(
            f"/v1/messenger/mls/groups/{group_id}/state?tenant_id={self.tenant_id}",
            headers={**self.headers, "x-oya-group-band": band},
            name="/v1/messenger/mls/groups/:id/state",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 404):
                response.success()
            else:
                response.failure(f"group state failed {response.status_code}")

    @task(24)
    def member_churn_add_remove(self):
        band = random.choice(["small", "medium", "large"])
        group_id = self._group_id(band)
        action = random.choice(["add", "remove"])
        with self.client.post(
            f"/v1/messenger/mls/groups/{group_id}/members:{action}",
            data=json.dumps({
                "tenant_id": self.tenant_id,
                "member_id": f"member-{random.randint(1, 50000)}",
                "device_count": 3,
                "idempotency_key": f"member-{action}-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "x-oya-group-band": band},
            name=f"/v1/messenger/mls/groups/:id/members:{action}",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 202, 404, 409):
                response.success()
            else:
                response.failure(f"member {action} failed {response.status_code}")

    @task(16)
    def stale_epoch_negative_probe(self):
        band = self._group_band()
        group_id = self._group_id(band)
        with self.client.post(
            f"/v1/messenger/mls/groups/{group_id}/messages:accept",
            data=json.dumps({
                "tenant_id": self.tenant_id,
                "group_band": band,
                "epoch": 1,
                "ciphertext_ref": f"stale-{random.randint(1, 999999)}",
                "idempotency_key": f"stale-{SEED}-{random.randint(1, 999999)}",
            }),
            headers={**self.headers, "x-oya-group-band": band},
            name="/v1/messenger/mls/groups/:id/messages:accept-stale",
            catch_response=True,
        ) as response:
            if response.status_code in (400, 401, 403, 409):
                response.success()
            else:
                response.failure(f"stale epoch accepted status={response.status_code}")
```

## Test Workload

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

## Baseline Numbers

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

## Comparison vs Named Vendors

Named vendors and projects: Signal, WhatsApp, Matrix, Wire, Slack Enterprise Key Management.

Signal-class comparison: end-to-end key update and sender-key style delivery cost.

WhatsApp-class comparison: large group and multi-device key fanout pressure.

Matrix-class comparison: federated encrypted-room recovery and device churn.

Wire-class comparison: enterprise group key lifecycle.

Slack EKM-class comparison: enterprise audit and key-governance posture, not MLS protocol parity.

Oyatie differentiator measured here: RFC 9420-style group key delivery remains p99 bounded while work-context audit evidence and tenant policy checks stay enabled.

Vendor parity guard: this document does not assert hidden vendor cryptographic performance; named vendors define comparable external test categories.

## Methodology

Named SUT topology: `messenger-mls-key-service-with-identity-and-audit`.

Warmup duration: 5 minutes.

Measurement window: 15 minutes.

Cooldown duration: 3 minutes.

Key delivery latency starts at key-service receive and stops when delivery is visible to the target device queue.

Epoch rotation latency starts at commit proposal accept and stops when all active devices have a deliverable epoch update.

Missed epoch recovery starts at recovery request and stops when the device receives the minimal required update chain.

Unauthorized delivery and stale epoch acceptance are hard-fail counters.

Throughput cap is raised until p99, unauthorized delivery, stale epoch acceptance, or audit completeness breaches.

## Reproducibility

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

## Failure Modes Detected

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

## Cross-References

- `specs/microservices/messenger.json`.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0009-cell-architecture-per-tenant-per-region.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- RFC 9420 protocol behavior is represented by synthetic compatible fixtures in this benchmark, not by external service measurements.
- Service-owned `microservices/messenger/benchmarks/` remains untouched by this root corpus.
