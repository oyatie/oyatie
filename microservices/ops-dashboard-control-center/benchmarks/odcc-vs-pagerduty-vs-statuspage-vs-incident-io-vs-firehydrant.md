---
doc_class: Benchmark
microservice: ops-dashboard-control-center
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0243, ADR-0131]
doc_status: published
---

# Benchmarks — oyatie ODCC vs PagerDuty / Atlassian Statuspage / incident.io / FireHydrant / Rootly / ServiceNow ITSM

Workloads measured: (a) operator-action p99 latency (incident declare → audit-seal-acked), (b) deployment rollback p99 latency, (c) evidence-pack export throughput, (d) step-up-auth round-trip latency, (e) cluster-health panel render time, (f) annual TCO at 50 operators × 100 cells × 5000 incidents/year.

Hardware (oyatie paid): 16× console-frontend + 8× console-API + 12× Postgres + 6× Valkey × 3 regions.

Comparators measured against published vendor benchmarks (PagerDuty engineering blog, Atlassian whitepaper, incident.io public reliability reports) + our integration-test cells.

## Workload (a) — operator-action p99 latency (incident declare → audit-seal-acked)

| Platform | p50 (ms) | p99 (ms) | Notes |
|---|---:|---:|---|
| oyatie ODCC paid | 52 | 138 | step-up freshness + Cedar gate + audit-chain emit + Postgres write |
| PagerDuty Incident.io v2 (incident declare) | 250 | 650 | API roundtrip + persist; no built-in step-up |
| incident.io | 180 | 480 | API + Slack webhook; no Cedar; step-up via separate vendor |
| FireHydrant | 220 | 520 | API + webhook; no Cedar; step-up = manual |
| Rootly | 200 | 510 | similar to FireHydrant |
| ServiceNow ITSM (incident creation) | 350 | 1 200 | enterprise-market system; rich UI; slower API |

Reading: oyatie ODCC paid leads on p99 latency. The latency budget is consumed by step-up freshness check + Cedar gate + audit-chain emit + Postgres write; competitors lack audit-chain + step-up so they're faster on the raw write but pay later in audit reconciliation cost.

PRD target: operator-action p99 ≤ 140 ms; achieved.

## Workload (b) — deployment rollback p99 latency (rollback command → traffic shifted)

| Platform | Rollback p99 (s) | Notes |
|---|---:|---|
| oyatie ODCC paid (rapid-traffic-shift to 100%) | 8 s | command → CDN/SLB config update → propagation |
| PagerDuty Rundeck integration (rollback runbook) | 25 s | runbook execution time |
| incident.io with native deploy integration | 15 s | webhook + integration adapter |
| FireHydrant with native deploy integration | 18 s | similar |
| ServiceNow change-mgmt + ITSM rollback | 90 s | ITSM workflow approval gate |
| Manual `kubectl rollout undo` | 8-30 s | depends on cluster + propagation; no audit; no Cedar |

Reading: oyatie ODCC paid is comparable to native `kubectl rollout undo` BUT carries audit-chain emission + Cedar gate + step-up + idempotency key — these are not free in raw `kubectl`.

PRD target: deployment rollback p99 ≤ 10 s; achieved.

## Workload (c) — evidence-pack export throughput

| Platform | Pack size | Time | Notes |
|---|---:|---:|---|
| oyatie ODCC paid (HSM-signed + L1+L2 notarized) | 100 GB | 180 s | sustained 555 MB/s; FIPS-140-2 Level 3 HSM partition |
| PagerDuty audit-export API | 50 GB max | 600 s | software signing only; no notarization; no FIPS-140-2 |
| Atlassian Statuspage incident-history-export | 10 GB max | 60 s | CSV only; no signing; no notarization |
| ServiceNow audit-table-export | 100 GB | 1 800 s | enterprise; slow; no notarization |
| Splunk Enterprise audit-search-export | 200 GB | 900 s | search-time export; software signing; no L2 anchor |

Reading: oyatie ODCC's bottleneck is the HSM partition's signing throughput (FIPS-mandated single-stream signing). Competitors are either faster (Splunk via parallel search) or slower (ServiceNow's workflow approval gate) but NONE have L2-anchor notarization for tamper-evident long-term archive.

## Workload (d) — step-up-auth round-trip latency (WebAuthn challenge → response → freshness updated)

| Platform | p99 (ms) | Notes |
|---|---:|---|
| oyatie ODCC paid (WebAuthn Yubikey 5 FIPS) | 780 | challenge gen + user tap + sig verify + Postgres write |
| oyatie ODCC paid (WebAuthn Touch ID) | 620 | platform authenticator faster than security key |
| Okta Verify push notification | 5 000 | user must unlock phone + tap; perception is slow |
| Duo Security push | 4 500 | similar to Okta |
| Microsoft Authenticator | 6 000 | similar |
| Manual TOTP code entry | 18 000 | user types 6-digit code |

Reading: WebAuthn dominates push notifications + TOTP. The 780 ms p99 includes the user's physical Yubikey tap (latency dominated by human reaction time ~ 200 ms + actual NFC/USB round-trip ~ 50 ms + cryptographic verify ~ 30 ms + Postgres write ~ 10 ms).

## Workload (e) — cluster-health panel render time (≤ 5000 cells in view, multi-region)

| Platform | p99 (ms) | Notes |
|---|---:|---|
| oyatie ODCC paid | 280 | server-side filter via Cedar + Postgres + Valkey cache |
| Grafana cluster overview (custom dashboard) | 800 | Prometheus + PromQL; no Cedar |
| DataDog Infrastructure Overview | 600 | proprietary backend |
| Splunk IT Service Intelligence | 1 200 | search-time computation |
| New Relic Infrastructure | 700 | proprietary backend |
| Honeycomb (with custom dashboard) | 900 | column store; slower for fan-out reads |

Reading: ODCC paid leads on p99. The Cedar pre-filter narrows the result set before render; Grafana/DataDog/Splunk render all data + paint with client-side filter (slower).

## Workload (f) — annual TCO at 50 operators × 100 cells × 5000 incidents/year

| Platform | Per-operator/year | Total at 50 operators | Notes |
|---|---:|---:|---|
| oyatie ODCC paid (on-prem; cell-cost) | n/a | $540 000 (cell-cost; operators free) | Flat-cell |
| PagerDuty Business ($41/user/mo) | $492 | $24 600 | per-user; no audit-chain |
| PagerDuty Digital Operations Plus ($59/user/mo) | $708 | $35 400 | per-user; more features; still no audit-chain |
| incident.io Pro ($25/responder/mo) | $300 | $15 000 | per-responder; chat-ops focused |
| FireHydrant Pro ($20/responder/mo) | $240 | $12 000 | per-responder |
| Rootly Standard ($24/responder/mo) | $288 | $14 400 | per-responder |
| ServiceNow ITSM Pro ($100+/user/mo, negotiated) | $1 200+ | $60 000+ | enterprise; rich workflow |
| Atlassian Statuspage Business ($99/page/mo) | n/a | $1 188 (per-page; not per-user) | comms-only |

Reading: at 50 operators, oyatie's cell-cost is higher than per-user SaaS. The cross-over math:

- ODCC paid cell-cost is fixed at $540k/year (paid tier).
- PagerDuty Business at $492/user/year breaks even at ~ 1 100 operators per cell.
- At ≥ 1 100 operators per cell, oyatie ODCC paid is cheaper than PagerDuty Business.
- The non-cost differentiators (audit-chain, FIPS-HSM signing, L2 notarization, Cedar gate, sovereign-pack overlay, follow-the-sun cross-region) are the value drivers for enterprise + sovereign cells.

## What competitors don't have

- **HSM-signed + L1/L2-notarized evidence packs** for tamper-evident long-term archive (SOC2 + FedRAMP 3PAO want this).
- **Cedar universal gate** on every operator action (per ADR-0243); competitors are role-based, not policy-engine.
- **Per-pack sovereign overlay** (KR-PIPA, FedRAMP High, EU-GDPR statutory-clock); competitors are jurisdiction-agnostic.
- **2-person Tier-3 step-up** with WebAuthn enforcement; competitors are SaaS MFA only.
- **Cross-tenant audit-chain mirroring** (per IP-006); competitors silo audit per-customer.

## Reproducibility

Benchmark harness at `benchmarks/odccbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks odcc \
    --workload operator-action-latency \
    --tenant-class oyatie-paid \
    --duration 30m \
    --operators 50 \
    --cells 100 \
    --output ./benchmark-results.json
```

For step-up-auth latency (requires physical WebAuthn authenticator):

```sh
cargo run -p oya-dev-cli -- benchmarks odcc \
    --workload step-up-auth-latency \
    --authenticator yubikey-5-fips \
    --iterations 100 \
    --output ./step-up-results.json
```

## Source citations

- PagerDuty Engineering blog 2024-Q3 on incident-API latency targets.
- Atlassian Statuspage technical whitepaper 2024 on export bandwidth.
- incident.io public reliability disclosures 2025-Q1.
- ServiceNow ITSM Performance Benchmarking Guide 2024.
- WebAuthn FIDO2 L2 specification (W3C 2022) for cryptographic round-trip floor.
