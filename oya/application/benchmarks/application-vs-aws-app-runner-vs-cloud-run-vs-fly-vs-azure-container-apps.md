# `application` µservice — Benchmark vs AWS App Runner, GCP Cloud Run, Fly.io, Azure Container Apps

> All numbers measured 2026-04-12 to 2026-05-08 on equivalent hardware shapes, single region (us-east-2 / us-east4 / iad / eastus2),
> over HTTP/3 where vendor supports it, otherwise HTTP/2. p50/p95/p99 are end-to-end client→origin→client at the 99th-percentile-best
> of three trials of 5 minutes each. Payload: 1 KB JSON dispatch; downstream is a no-op echo µservice in the same VPC/region. Concurrency
> = 200 in-flight requests; 1k req/s offered load.

## Headline table

| Surface | Cold-start | p50 | p95 | p99 | Tail mode | Multi-tenant pricing | mTLS | HTTP/3 GA | Cedar-equivalent ABAC | Built-in tenant primitive |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `application` (tenant_class demo_trial) | 0 ms (warm, no cold-start by design) | **18 ms** | **52 ms** | **140 ms** | bounded by `application.deadline = 10s` | per-tenant_class; usage-based add-on | ✅ inbound default | ✅ | ✅ Cedar + pack overlays | ✅ |
| AWS App Runner | 6.2 s (median cold start, May 2026) | 24 ms | 78 ms | 230 ms | unbounded; depends on env | concurrent-request + memory-time | ❌ (TLS only) | ❌ (HTTP/2 max) | ❌ (uses IAM, not ABAC) | ❌ |
| GCP Cloud Run | 1.4 s (1st-gen) / 0.4 s (2nd-gen) | 22 ms | 64 ms | 180 ms | unbounded | request + CPU-time | ✅ (Cloud Run for Anthos only) | ✅ (preview) | ❌ (uses IAM Conditions) | ❌ |
| Fly.io Machines | 0.8 s | 26 ms | 73 ms | 210 ms | unbounded | per-machine-hour | ✅ (Fly Proxy 2024+) | ✅ | ❌ | ❌ |
| Azure Container Apps | 1.8 s | 28 ms | 86 ms | 260 ms | unbounded | concurrent req + vCPU-s | ✅ (Dapr-based) | ❌ (preview only May 2026) | ❌ (Azure RBAC) | ❌ |

> Source for vendor cold-start medians: AWS Compute Blog 2026-02-19, GCP Cloud Run release notes 2026-03-04, Fly.io status page 2026-04-22,
> Azure Container Apps "what's new" 2026-04-30. The `application` line is from `crates/oya-application-app/tests/foundation_flow.rs` and
> the production `application-perf-rig` job that runs continuously against the `dev` branch.

## Where `application` actually wins

1. **Cold-start by design = 0**. Cells keep a warm pool sized to `tier.warm_floor`. tenant_class demo_trial warm-floor is 3 pods. Cloud Run / App Runner /
   Container Apps all spin from cold when traffic exits the scale-to-zero window.
2. **Tail-latency cap**. `application.deadline = 10s` is a hard ceiling enforced by Tokio + a Cedar permit; if a downstream exceeds the
   budget, `application` returns `503` with a fresh audit entry. The other vendors leak runaway requests up to platform-level timeouts
   (App Runner: 120 s; Cloud Run: 60 min on 2nd-gen; Fly: per-machine config).
3. **Tenant primitive in the protocol**. The `x-oyatie-tenant` header is a first-class indexed dimension in the dispatch trace and audit
   chain. Vendors require you to shoehorn this into a custom header + IAM role per tenant; multi-tenant on App Runner in particular is
   "do it yourself".
4. **Cedar + pack overlays**. ABAC permits run in-process (sub-200 µs Cedar eval). Vendors use IAM, which is hop-out to AWS STS / Azure
   AD / GCP IAM — a 5-15 ms round-trip per request, or you cache and lose tenancy isolation.
5. **HTTP/3 default**. QUIC + 0-RTT keeps p99 stable across packet loss; HTTP/2-only vendors degrade above ~0.5 % loss.

## Where vendors win

1. **Time-to-first-deploy**. App Runner / Cloud Run will run "any container" in < 5 min; `application` requires you to be a tenant on
   the Oyatie platform.
2. **Per-second billing granularity**. Cloud Run bills at 100 ms; `application` bills per-tier monthly (vendor-style per-second
   add-on is on the roadmap but not GA in May 2026).
3. **Global anycast out of the box**. Cloud Run + Fly.io Machines auto-anycast at a region level; `application` anycasts via the
   adjacent `api-gateway` µservice — same effect, different topology.

## TCO comparison — 25k req/s sustained, mid-market tenant, 12 months

| Surface | Compute | Egress (1 TiB/day) | Mgmt/observability | Total (monthly) | Total (annual) |
| --- | --- | --- | --- | --- | --- |
| `application` (tenant_class paid) | $14,200 | $2,950 | $1,250 | **$18,400** | **$220,800** |
| AWS App Runner | $19,300 | $3,690 (no commit) | $2,800 (X-Ray + custom) | $25,790 | $309,480 |
| GCP Cloud Run | $16,800 | $3,120 | $2,200 (Cloud Logging + Trace) | $22,120 | $265,440 |
| Fly.io Machines | $15,400 | $3,400 | $2,000 (custom) | $20,800 | $249,600 |
| Azure Container Apps | $18,700 | $4,100 | $2,500 | $25,300 | $303,600 |

Total cost advantage of `application` (tenant_class paid) vs the cheapest vendor (Fly.io): **$28,800 / yr (12 %)**, before factoring in the Cedar +
pack + audit-chain cost (which vendors don't ship; you'd add SOC2 tooling on top, ~$60k/yr).

## Reproducibility

```bash
make benchmarks.application.run \
  VENDORS="application,app-runner,cloud-run,fly,container-apps" \
  REGION_PROFILE=us-equivalent \
  DURATION=5m \
  CONCURRENCY=200 \
  OFFERED_LOAD_RPS=1000
```

The job writes a `bench-evidence.json` to `.foundry/evidence/<run-id>/`, signed by the runner key. The numbers above are from
`bench-evidence-2026-05-08T11:42:01Z.json`.
