---
doc_class: Benchmark
microservice: incident-management
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0263, ADR-0251]
doc_status: published
---

# Benchmarks — oyatie incident-management vs PagerDuty vs Opsgenie vs Squadcast vs Rootly vs FireHydrant vs incident.io

Workloads measured: (a) page-delivery latency, (b) on-call resolution latency, (c) escalation policy depth supported, (d) AI-triage accuracy, (e) annual TCO at 500 responders + 50 000 incidents/year + 7-y post-mortem retention.

Hardware (oyatie on-prem paid tier): 9× incident API pods (16 vCPU AMD EPYC 9354P, 64 GiB DDR5, 500 GiB NVMe each), 6× paging-router pods, PostgreSQL 16.6 cluster, SeaweedFS-S3 for post-mortems. Multi-provider SMS (Twilio + Bandwidth + Plivo).

## Workload (a) — page-delivery latency (alert ingest → first SMS/voice/Slack delivered to on-call)

| Engine | p50 (s) | p99 (s) | p99.9 (s) |
|---|---:|---:|---:|
| oyatie incident-management (paid, Twilio only) | 4.2 | 9.8 | 14.5 |
| oyatie incident-management (paid, Twilio + Bandwidth + Plivo parallel) | 2.4 | 5.6 | 8.4 |
| PagerDuty | 3.8 | 9.4 | 14.0 |
| Opsgenie | 4.4 | 10.2 | 15.8 |
| Squadcast | 4.0 | 9.6 | 14.4 |
| Rootly | 4.6 | 10.8 | 16.2 |
| FireHydrant | 5.2 | 11.6 | 17.4 |
| incident.io | 3.6 | 8.8 | 13.2 |

Reading: oyatie paid leads the field via multi-provider parallel routing (the fastest of three SMS providers wins). PagerDuty + incident.io are competitive. The mid-pack tools (Opsgenie, Squadcast, Rootly, FireHydrant) lag by 1-2 s p99.

## Workload (b) — on-call resolution latency ("who's on-call right now?")

| Engine | p99 (ms) |
|---|---:|
| oyatie incident-management (paid) | 80 |
| oyatie incident-management (paid, 3-AZ Postgres) | 40 |
| PagerDuty | 180 |
| Opsgenie | 220 |
| Squadcast | 280 |
| Rootly | 240 |
| FireHydrant | 320 |
| incident.io | 200 |

Reading: oyatie paid leads. The on-call resolution query is a critical path for any tool that integrates incident-management (e.g. observability calling "who do I page?") — sub-50 ms is the paid standard. PagerDuty's multi-tenant query queue causes their tail to be slower.

## Workload (c) — escalation policy depth supported

| Engine | Max levels | Conditionals supported | Cross-team fan-out |
|---|---:|---|---|
| oyatie (paid) | 8 | Severity + service + business-hours | Yes |
| oyatie (paid) | 100 | Severity + service + business-hours + metadata + Cedar policy | Yes |
| PagerDuty | 10 (Business tier) | Severity + service + business-hours | Yes |
| Opsgenie | 10 (Enterprise) | Severity + service + tag | Yes |
| Squadcast | 8 | Severity + service | Yes |
| Rootly | 12 | Severity + service + condition | Yes |
| FireHydrant | 10 | Severity + service | Yes |
| incident.io | 12 | Severity + service + condition | Yes |

Reading: oyatie paid's escalation policies are the deepest in the field (100 levels with Cedar-policy conditionals). The Cedar integration allows policies like "escalate to VP-Engineering only if customer_impact > $50k OR data-loss-classification = true".

## Workload (d) — AI-triage classification accuracy (test set: 1 000 alerts, classify to service + likely-cause)

| Engine | Service classification accuracy | Cause classification accuracy |
|---|---:|---:|
| oyatie (paid, no AI) | n/a | n/a |
| oyatie (paid, Llama-3.1-70B local) | 91.4 % | 83.7 % |
| PagerDuty (PagerDuty Copilot) | 89.2 % | 82.1 % |
| Opsgenie (basic AI) | 78.5 % | 69.4 % |
| Squadcast | 72.3 % | 63.5 % |
| Rootly | 86.7 % | 78.4 % |
| FireHydrant | 74.6 % | 67.2 % |
| incident.io | 88.4 % | 81.5 % |

Reading: oyatie paid leads on both classification axes. The fine-tuning on tenant-specific prior incidents (paid tenant_class feature) provides the edge over PagerDuty Copilot and incident.io's AI which are tenant-agnostic.

## Workload (e) — annual TCO at 500 responders + 50 000 incidents/year

| Platform | Hardware (USD) | Licence/per-responder (USD) | SMS/voice (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie incident-management (paid on-prem) | 380 000 | 0 | 60 000 (multi-provider SMS + voice at 50k incidents × ~ 4 pages avg) | 248 000 (2 SRE × 0.4 FTE) | 688 000 |
| PagerDuty (Business tier, 500 responders) | 0 | 240 000 ($40/responder/mo × 500 × 12) | 0 (included up to fair-use) | 124 000 (1 SRE × 0.2 FTE) | 364 000 |
| Opsgenie (Enterprise, 500 responders) | 0 | 174 000 ($29/responder/mo × 500 × 12) | 0 | 124 000 | 298 000 |
| Squadcast (Enterprise, 500 responders) | 0 | 174 000 | 0 | 124 000 | 298 000 |
| Rootly (Business, 500 responders) | 0 | 282 000 ($47/responder/mo × 500 × 12) | 0 | 124 000 | 406 000 |
| FireHydrant (Enterprise, 500 responders) | 0 | 240 000 | 0 | 124 000 | 364 000 |
| incident.io (Business, 500 responders) | 0 | 270 000 ($45/responder/mo × 500 × 12) | 0 | 124 000 | 394 000 |

Reading: Opsgenie + Squadcast lead on TCO at ~ 300k USD/yr. PagerDuty + FireHydrant are mid-pack. Rootly + incident.io are premium-priced for their AI features. oyatie paid is the highest TCO due to self-hosted hardware + ops; the value proposition is sovereign-pack residency + pack-resident paging providers + Cedar-policy escalation, which NO competitor offers.

For tenants who don't need sovereign-pack features, Opsgenie or Squadcast is hard to beat on TCO. For tenants who DO (KR-PIPA-Finance, CSAP, EU NIS2), oyatie paid is the only option.

Caveats:
- Per-responder pricing assumes no negotiation; enterprise contracts commonly receive 20-30 % discount.
- SMS/voice cost in cloud comparators is "included" up to fair-use limits; at 50 k incidents × 4 pages × 500 responders = 100 000 SMS/voice messages, you may exceed fair-use and pay overage.
- Ops cost includes paging-router + on-call scheduler + PostgreSQL lifecycle (0.8 FTE total) — the real hidden cost of self-hosted.

## Workload (f) — sovereign-pack feature parity (oyatie-exclusive)

"Route SEV-1 pages to Kakao Talk Bizmessage + KT 070 voice for KR-PIPA-Finance tenant with pack-resident paging providers + dual-control admin."

| Engine | Support |
|---|---|
| oyatie incident-management (paid) | Yes (per ADR-0251 + KR pack) |
| PagerDuty | No (US-cloud only) |
| Opsgenie | No |
| Squadcast | Limited (basic Korean SMS via Twilio Korea) |
| Rootly | No |
| FireHydrant | No |
| incident.io | No |

This is the categorical differentiator. KR-PIPA-Finance + FSC-regulated tenants choose oyatie paid specifically for the pack-resident paging providers + dual-control + FSC regulator pre-notification automation.

## Reproducibility

Benchmark harness at `benchmarks/incident-management/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks incident-management \
    --workload 50k-incidents-sustained-1h \
    --tenant-class paid \
    --output ./results.json
```

Results at `benchmarks/results/incident-management/<date>.csv`, re-run weekly in CI.
