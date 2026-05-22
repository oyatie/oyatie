---
doc_class: PerformanceBenchmark
benchmark_id: PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20
target_microservices:
  - drive
  - tenancy
  - policy-cedar
  - audit-chain
  - object-storage
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

# Drive Upload Download Throughput Benchmark

## Benchmark Goal

Named target metric: `drive_object_transfer_throughput_mib_per_second`.

Named latency metric: `drive_object_first_byte_latency_ms`.

Named SLO target: `SLO-DRV-TRANSFER-P99`.

The SLO target is p99 upload commit latency and p99 download first-byte latency per file-size band and per tier.

The benchmark finds the transfer throughput cap that keeps object integrity verification, tenant quota checks, Cedar checks, and audit-chain emission complete.

Baseline numbers are recorded synthetic lab baselines for the named topology and seed set; they are not a production availability or vendor-comparison claim.

## Test Harness

Named load-generator topology: `drive-band-matrix-cell-lab`.

Topology nodes:

- One bash fixture loader creates tenants, folders, quota classes, and sparse file payloads.
- k6 drives signed-upload create, chunk PUT, commit, metadata read, signed-download create, and ranged GET.
- Locust drives long-tail mixed downloads and concurrent resumable uploads.
- The SUT includes drive REST, drive worker, tenancy, Cedar, audit-chain, object-storage adapter, and malware-scan stub.
- Prometheus collects throughput, p50, p95, p99, checksum error rate, and quota-throttle counters.

```bash
#!/usr/bin/env bash
set -euo pipefail

BENCH_ID="${BENCH_ID:-PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20}"
SUT_BASE_URL="${SUT_BASE_URL:-https://drive-cell-01.dev.oyatie.local}"
PROM_URL="${PROM_URL:-http://prometheus.oya-observability.svc:9090}"
TENANT_PREFIX="${TENANT_PREFIX:-bench-drive}"
SEED="${SEED:-94203001}"
TIER="${TIER:-Bronze}"
OUTPUT_DIR="${OUTPUT_DIR:-benchmarks/out/drive}"
WARMUP_SECONDS="${WARMUP_SECONDS:-300}"
MEASURE_SECONDS="${MEASURE_SECONDS:-900}"

mkdir -p "${OUTPUT_DIR}/payloads"

case "${TIER}" in
  Bronze)
    VUS=80
    UPLOAD_RATE=120
    DOWNLOAD_RATE=220
    ;;
  Silver)
    VUS=160
    UPLOAD_RATE=260
    DOWNLOAD_RATE=540
    ;;
  Gold)
    VUS=280
    UPLOAD_RATE=520
    DOWNLOAD_RATE=980
    ;;
  Platinum)
    VUS=520
    UPLOAD_RATE=920
    DOWNLOAD_RATE=1750
    ;;
  *)
    echo "unknown tier: ${TIER}" >&2
    exit 64
    ;;
esac

payload_file() {
  local band="$1"
  local mib="$2"
  local path="${OUTPUT_DIR}/payloads/${band}-${mib}MiB.bin"
  if [ ! -f "${path}" ]; then
    dd if=/dev/urandom of="${path}" bs=1M count="${mib}" status=none
  fi
  printf "%s" "${path}"
}

SMALL_FILE="$(payload_file small 1)"
MEDIUM_FILE="$(payload_file medium 16)"
LARGE_FILE="$(payload_file large 128)"
XLARGE_FILE="$(payload_file xlarge 512)"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/reset" \
  -H "content-type: application/json" \
  -d "{\"bench_id\":\"${BENCH_ID}\",\"tenant_prefix\":\"${TENANT_PREFIX}\",\"seed\":${SEED}}" \
  > "${OUTPUT_DIR}/reset-${TIER}.json"

curl -fsS -X POST "${SUT_BASE_URL}/internal/bench/drive/fixture" \
  -H "content-type: application/json" \
  -d "{
    \"bench_id\":\"${BENCH_ID}\",
    \"tenant_prefix\":\"${TENANT_PREFIX}\",
    \"tier\":\"${TIER}\",
    \"tenant_count\":64,
    \"folder_count_per_tenant\":128,
    \"file_bands\":[\"small\",\"medium\",\"large\",\"xlarge\"],
    \"distribution\":\"zipfian-hot-folder-with-uniform-band-sweep\",
    \"seed\":${SEED}
  }" \
  > "${OUTPUT_DIR}/fixture-${TIER}.json"

k6 run \
  -e SUT_BASE_URL="${SUT_BASE_URL}" \
  -e BENCH_ID="${BENCH_ID}" \
  -e TIER="${TIER}" \
  -e SEED="${SEED}" \
  -e TENANT_PREFIX="${TENANT_PREFIX}" \
  -e UPLOAD_RATE="${UPLOAD_RATE}" \
  -e DOWNLOAD_RATE="${DOWNLOAD_RATE}" \
  -e WARMUP_SECONDS="${WARMUP_SECONDS}" \
  -e MEASURE_SECONDS="${MEASURE_SECONDS}" \
  -e SMALL_FILE="${SMALL_FILE}" \
  -e MEDIUM_FILE="${MEDIUM_FILE}" \
  -e LARGE_FILE="${LARGE_FILE}" \
  -e XLARGE_FILE="${XLARGE_FILE}" \
  -o "json=${OUTPUT_DIR}/k6-${TIER}.json" \
  benchmarks/drive-upload-download-throughput.k6.js

locust \
  -f benchmarks/drive-upload-download-throughput.locust.py \
  --headless \
  --users "${VUS}" \
  --spawn-rate "$(( VUS / 12 + 1 ))" \
  --run-time "$(( WARMUP_SECONDS + MEASURE_SECONDS ))s" \
  --host "${SUT_BASE_URL}" \
  --csv "${OUTPUT_DIR}/locust-${TIER}" \
  --html "${OUTPUT_DIR}/locust-${TIER}.html"

for band in small medium large xlarge; do
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_drive_upload_commit_latency_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",band=\"${band}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/upload-p99-${TIER}-${band}.json"
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=histogram_quantile(0.99,sum(rate(oya_drive_download_ttfb_ms_bucket{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",band=\"${band}\"}[15m])) by (le))" \
    > "${OUTPUT_DIR}/download-p99-${TIER}-${band}.json"
  curl -fsS --get "${PROM_URL}/api/v1/query" \
    --data-urlencode "query=sum(rate(oya_drive_transfer_bytes_total{bench_id=\"${BENCH_ID}\",tier=\"${TIER}\",band=\"${band}\"}[15m])) / 1048576" \
    > "${OUTPUT_DIR}/throughput-mib-${TIER}-${band}.json"
done

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_drive_checksum_mismatch_total{bench_id=\"${BENCH_ID}\"}[15m]))" \
  > "${OUTPUT_DIR}/checksum-mismatch-${TIER}.json"

curl -fsS --get "${PROM_URL}/api/v1/query" \
  --data-urlencode "query=sum(rate(oya_audit_chain_events_total{bench_id=\"${BENCH_ID}\",microservice=\"drive\"}[15m]))" \
  > "${OUTPUT_DIR}/audit-events-${TIER}.json"

jq -n \
  --arg bench_id "${BENCH_ID}" \
  --arg tier "${TIER}" \
  --slurpfile fixture "${OUTPUT_DIR}/fixture-${TIER}.json" \
  --slurpfile checksum "${OUTPUT_DIR}/checksum-mismatch-${TIER}.json" \
  --slurpfile audit "${OUTPUT_DIR}/audit-events-${TIER}.json" \
  '{bench_id:$bench_id,tier:$tier,fixture:$fixture[0],checksum_mismatch:$checksum[0],audit_events:$audit[0]}' \
  > "${OUTPUT_DIR}/summary-${TIER}.json"

echo "drive benchmark complete: ${OUTPUT_DIR}/summary-${TIER}.json"
```

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

export const uploadCommitLatency = new Trend('drive_upload_commit_latency_ms', true);
export const downloadTtfb = new Trend('drive_download_ttfb_ms', true);
export const transferMiB = new Counter('drive_transfer_mib');
export const checksumFailures = new Rate('drive_checksum_failures');
export const quotaThrottles = new Counter('drive_quota_throttles');

const baseUrl = __ENV.SUT_BASE_URL;
const benchId = __ENV.BENCH_ID || 'PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20';
const tier = __ENV.TIER || 'Bronze';
const seed = Number(__ENV.SEED || 94203001);
const tenantPrefix = __ENV.TENANT_PREFIX || 'bench-drive';
const uploadRate = Number(__ENV.UPLOAD_RATE || 120);
const downloadRate = Number(__ENV.DOWNLOAD_RATE || 220);
const warmupSeconds = Number(__ENV.WARMUP_SECONDS || 300);
const measureSeconds = Number(__ENV.MEASURE_SECONDS || 900);

export const options = {
  scenarios: {
    upload_matrix: {
      executor: 'constant-arrival-rate',
      rate: uploadRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(40, Math.floor(uploadRate / 4)),
      maxVUs: Math.max(200, uploadRate),
      exec: 'uploadObject',
    },
    download_matrix: {
      executor: 'constant-arrival-rate',
      rate: downloadRate,
      timeUnit: '1s',
      duration: `${warmupSeconds + measureSeconds}s`,
      preAllocatedVUs: Math.max(40, Math.floor(downloadRate / 8)),
      maxVUs: Math.max(300, downloadRate),
      exec: 'downloadObject',
    },
  },
  thresholds: {
    drive_upload_commit_latency_ms: ['p(99)<3000'],
    drive_download_ttfb_ms: ['p(99)<600'],
    drive_checksum_failures: ['rate==0'],
  },
};

function bandForIteration(iteration) {
  const v = (iteration + seed) % 100;
  if (v < 42) return { band: 'small', bytes: 1048576, chunks: 1 };
  if (v < 72) return { band: 'medium', bytes: 16777216, chunks: 4 };
  if (v < 94) return { band: 'large', bytes: 134217728, chunks: 16 };
  return { band: 'xlarge', bytes: 536870912, chunks: 64 };
}

function headers(extra = {}) {
  return Object.assign({
    'content-type': 'application/json',
    'x-oya-benchmark-id': benchId,
    'x-oya-tier': tier,
  }, extra);
}

export function uploadObject() {
  const band = bandForIteration(__ITER);
  const tenantId = `${tenantPrefix}-${(__VU + __ITER) % 64}`;
  const folderId = `folder-${(__ITER + seed) % 128}`;
  const create = http.post(`${baseUrl}/v1/drive/uploads`, JSON.stringify({
    tenant_id: tenantId,
    folder_id: folderId,
    object_name: `obj-${tier}-${__VU}-${__ITER}-${band.band}.bin`,
    declared_size_bytes: band.bytes,
    checksum_algorithm: 'sha256',
    band: band.band,
    idempotency_key: `upload-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers(), tags: { tier, band: band.band } });
  const created = check(create, {
    'upload session created': (r) => r.status === 201,
    'has upload id': (r) => !!r.json('upload_id'),
  });
  if (!created) {
    if (create.status === 429) quotaThrottles.add(1, { tier, band: band.band });
    return;
  }
  const uploadId = create.json('upload_id');
  const chunkBytes = Math.floor(band.bytes / band.chunks);
  for (let idx = 0; idx < band.chunks; idx += 1) {
    const chunk = 'x'.repeat(Math.min(chunkBytes, 262144));
    const put = http.put(`${baseUrl}/v1/drive/uploads/${uploadId}/chunks/${idx}`, chunk, {
      headers: headers({ 'content-type': 'application/octet-stream', 'x-oya-chunk-size': String(chunkBytes) }),
      tags: { tier, band: band.band, chunk: String(idx) },
    });
    check(put, { 'chunk accepted': (r) => r.status === 200 || r.status === 202 });
  }
  const start = Date.now();
  const commit = http.post(`${baseUrl}/v1/drive/uploads/${uploadId}/commit`, JSON.stringify({
    checksum: `synthetic-sha256-${seed}-${__VU}-${__ITER}`,
    expected_chunks: band.chunks,
  }), { headers: headers(), tags: { tier, band: band.band } });
  const ok = check(commit, {
    'commit accepted': (r) => r.status === 200 || r.status === 201,
    'commit has audit id': (r) => !!r.json('audit_event_id'),
    'commit has object id': (r) => !!r.json('object_id'),
  });
  uploadCommitLatency.add(Number(commit.json('commit_latency_ms') || (Date.now() - start)), { tier, band: band.band });
  checksumFailures.add(!ok, { tier, band: band.band });
  if (ok) transferMiB.add(band.bytes / 1048576, { tier, band: band.band, direction: 'upload' });
}

export function downloadObject() {
  const band = bandForIteration(__ITER);
  const tenantId = `${tenantPrefix}-${(__VU + seed + __ITER) % 64}`;
  const objectId = `seeded-${tenantId}-${band.band}-${(__ITER + seed) % 50000}`;
  const signed = http.post(`${baseUrl}/v1/drive/downloads`, JSON.stringify({
    tenant_id: tenantId,
    object_id: objectId,
    purpose: 'benchmark-read',
    idempotency_key: `download-${seed}-${__VU}-${__ITER}`,
  }), { headers: headers(), tags: { tier, band: band.band } });
  if (!check(signed, { 'signed download created': (r) => r.status === 201 || r.status === 200 })) return;
  const url = signed.json('download_url');
  const start = Date.now();
  const res = http.get(url, { headers: { 'range': 'bytes=0-1048575' }, tags: { tier, band: band.band } });
  const ok = check(res, {
    'range returned': (r) => r.status === 206 || r.status === 200,
    'has checksum header': (r) => !!r.headers['X-Oya-Checksum'],
  });
  downloadTtfb.add(Number(res.timings.waiting || (Date.now() - start)), { tier, band: band.band });
  if (ok) transferMiB.add(Math.min(band.bytes, 1048576) / 1048576, { tier, band: band.band, direction: 'download' });
  sleep(0.01);
}
```

```python
from __future__ import annotations

import json
import os
import random
import time
from locust import HttpUser, between, events, task

BENCH_ID = "PB-DRV-UPLOAD-DOWNLOAD-THROUGHPUT-2026-05-20"
SEED = int(os.getenv("SEED", "94203001"))
TENANT_PREFIX = os.getenv("TENANT_PREFIX", "bench-drive")
TIER = os.getenv("TIER", "Bronze")

random.seed(SEED)


class DriveMixedTransferUser(HttpUser):
    wait_time = between(0.02, 0.20)

    def on_start(self):
        self.tenant_id = f"{TENANT_PREFIX}-{random.randint(0, 63)}"
        self.headers = {
            "content-type": "application/json",
            "x-oya-benchmark-id": BENCH_ID,
            "x-oya-tier": TIER,
        }

    def _band(self):
        roll = random.randint(0, 99)
        if roll < 40:
            return "small", 1 * 1024 * 1024
        if roll < 70:
            return "medium", 16 * 1024 * 1024
        if roll < 94:
            return "large", 128 * 1024 * 1024
        return "xlarge", 512 * 1024 * 1024

    @task(38)
    def ranged_download(self):
        band, size_bytes = self._band()
        object_id = f"seeded-{self.tenant_id}-{band}-{random.randint(1, 50000)}"
        start = time.perf_counter()
        with self.client.post(
            "/v1/drive/downloads",
            data=json.dumps({
                "tenant_id": self.tenant_id,
                "object_id": object_id,
                "purpose": "benchmark-ranged-download",
                "idempotency_key": f"locust-download-{SEED}-{object_id}-{random.randint(1, 999999)}",
            }),
            headers=self.headers,
            name="/v1/drive/downloads",
            catch_response=True,
        ) as signed:
            if signed.status_code not in (200, 201):
                signed.failure(f"signed download failed {signed.status_code}")
                return
            signed.success()
            url = signed.json().get("download_url")
        with self.client.get(
            url,
            headers={"range": "bytes=0-1048575", "x-oya-benchmark-id": BENCH_ID},
            name="/signed/drive/range",
            catch_response=True,
        ) as response:
            elapsed_ms = (time.perf_counter() - start) * 1000
            if response.status_code not in (200, 206):
                response.failure(f"range failed {response.status_code}")
                return
            events.request.fire(request_type="CHECK", name=f"download_ttfb_{band}_ms", response_time=elapsed_ms, response_length=0)
            response.success()

    @task(32)
    def metadata_read(self):
        band, _ = self._band()
        object_id = f"seeded-{self.tenant_id}-{band}-{random.randint(1, 50000)}"
        with self.client.get(
            f"/v1/drive/objects/{object_id}/metadata?tenant_id={self.tenant_id}",
            headers=self.headers,
            name="/v1/drive/objects/:id/metadata",
            catch_response=True,
        ) as response:
            if response.status_code == 200 and "checksum" in response.text:
                response.success()
            elif response.status_code == 404:
                response.success()
            else:
                response.failure(f"metadata read failed {response.status_code}")

    @task(20)
    def resumable_upload_probe(self):
        band, size_bytes = self._band()
        payload = {
            "tenant_id": self.tenant_id,
            "folder_id": f"folder-{random.randint(1, 128)}",
            "object_name": f"locust-{TIER}-{band}-{random.randint(1, 999999)}.bin",
            "declared_size_bytes": size_bytes,
            "checksum_algorithm": "sha256",
            "band": band,
            "idempotency_key": f"locust-upload-{SEED}-{random.randint(1, 999999)}",
        }
        with self.client.post(
            "/v1/drive/uploads",
            data=json.dumps(payload),
            headers=self.headers,
            name="/v1/drive/uploads",
            catch_response=True,
        ) as response:
            if response.status_code in (200, 201, 202, 429):
                response.success()
            else:
                response.failure(f"upload probe failed {response.status_code}")

    @task(10)
    def quota_probe(self):
        with self.client.get(
            f"/v1/drive/quota?tenant_id={self.tenant_id}",
            headers=self.headers,
            name="/v1/drive/quota",
            catch_response=True,
        ) as response:
            if response.status_code == 200 and "remaining_bytes" in response.text:
                response.success()
            else:
                response.failure(f"quota probe failed {response.status_code}")
```

## Test Workload

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

## Baseline Numbers

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

## Comparison vs Named Vendors

Named vendors: Google Drive, Dropbox, Box, Microsoft OneDrive, Amazon S3.

Google Drive-class comparison: user-facing collaborative object storage with folder ACLs.

Dropbox-class comparison: resumable upload and sync-style large object transfer.

Box-class comparison: enterprise compliance metadata and auditability.

OneDrive-class comparison: identity-integrated tenant file sharing.

Amazon S3-class comparison: object-storage first-byte and throughput mechanics.

Oyatie differentiator measured here: upload/download SLOs remain inside tier budget while Cedar, tenant quotas, and audit-chain evidence stay enabled.

Vendor parity guard: this document does not assert public vendor p99 or throughput numbers; it names vendor categories to shape comparable future external tests.

## Methodology

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

## Reproducibility

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

## Failure Modes Detected

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

## Cross-References

- `docs/SLO-CATALOG.md`.
- `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md`.
- `docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md`.
- `docs/decisions/ADR-0009-cell-architecture-per-tenant-per-region.md`.
- `docs/decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md`.
- `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- `microservices/drive/` remains service-owned and is intentionally not modified by this corpus.
