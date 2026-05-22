---
doc_class: Benchmark
microservice: connect
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0243, ADR-0131]
doc_status: published
---

# Benchmarks — oyatie connect vs Slack Connect / Microsoft Teams External Access / Discord servers / Matrix federation / Mattermost Boards-and-Channels

Workloads measured: (a) cross-tenant message p99 latency, (b) federation handshake duration, (c) presence-sync p99, (d) MLS group rekey latency, (e) audit-mirror lag, (f) annual TCO at 5 federated peers × 5 channels × 1 000 users.

Hardware (oyatie paid): 16× channel-bridge + 12× Postgres + 6× NATS × 3 regions.

Comparators measured against published platform docs (Slack Engineering blog on Connect, Microsoft Teams external-access whitepaper, Matrix federation tests).

## Workload (a) — cross-tenant message p99 latency (within same region)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie connect paid | 48 | 138 |
| Slack Connect (Premium tier) | 85 | 280 |
| Microsoft Teams External Access | 120 | 380 |
| Discord (server-to-server, no native federation; using bots-as-relay) | 95 | 240 |
| Matrix (homeserver federation, Synapse) | 280 | 850 |
| Mattermost (channel-mirroring) | 150 | 320 |

Reading: oyatie paid leads. Slack Connect is close. Matrix federation pays the federation-batch + signature-verify cost.

PRD target: cross-tenant message p99 ≤ 140 ms; achieved.

## Workload (b) — federation handshake duration

| Platform | Handshake duration |
|---|---:|
| oyatie connect | 35 s (bilateral request + accept + MLS group setup) |
| Slack Connect | "Up to 24 hours" (per Slack docs; depends on admin signoff) |
| Microsoft Teams External Access | "Up to 7 days" (per Microsoft docs; varies by tenant policy) |
| Matrix federation | ~ 10 s (auto-federated; no admin signoff needed) |
| Mattermost trust | manual; ~ 1 day with admin coordination |

Reading: Matrix's auto-federation is fastest but lowest-trust. oyatie's 35 s reflects active admin signoff (which is the right tradeoff for B2B trust).

## Workload (c) — presence sync p99 (cross-tenant user state)

| Platform | p99 (ms) |
|---|---:|
| oyatie connect (real-time at paid) | 180 |
| Slack Connect | 800 (per Slack engineering) |
| Microsoft Teams External Access | 1 200 |
| Matrix (per server homepage) | 600 |
| Discord servers | n/a (no cross-server presence) |

Reading: oyatie's real-time presence via signed presence-token streams over NATS leads.

## Workload (d) — MLS group rekey latency (member added; 100-member group)

| Platform | Rekey duration |
|---|---:|
| oyatie connect (MLS RFC 9420) | 28 ms (libgroup-mls 0.4) |
| Slack Connect (E2EE Slack Connect, in preview) | n/a (not GA at benchmark time) |
| Microsoft Teams (E2EE 1:1 only; no group) | n/a (group E2EE not GA) |
| Matrix (Megolm group rekey) | 320 ms |
| Signal protocol group (Pairwise N²) | 1 200 ms (for 100 members, N² fan-out) |

Reading: MLS scales sublinearly; Matrix's Megolm is acceptable; Signal pairwise is the worst-case for groups (the reason MLS was designed).

## Workload (e) — cross-tenant audit-mirror lag

| Platform | Audit-mirror lag p99 |
|---|---:|
| oyatie connect | 220 ms |
| Slack Connect (admin audit log, cross-org) | 30 s (per Slack docs; audit log near-real-time but not sub-second) |
| Microsoft Teams (audit log) | 60 s |
| Matrix federation (per-server audit) | ~ 5 s |

Reading: oyatie's audit-mirror runs on the same NATS substrate as messages; sub-second is the design point.

## Workload (f) — annual TCO at 5 federated peers × 5 channels × 1 000 users

| Platform | Per-user/year | Total at 1 000 users | Notes |
|---|---:|---:|---|
| oyatie connect paid (on-prem; included in tenancy + per-user at $0) | n/a | $380 000 (cell-cost; users free) | Flat-cell |
| Slack Business+ ($12.50/u/mo) | $150 | $150 000 | Per-user; Slack Connect included |
| Slack Enterprise Grid ($18-24/u/mo, negotiated) | $216 | $216 000 | Per-user; Connect included |
| Microsoft Teams (M365 Business Premium, $22/u/mo) | $264 | $264 000 | Per-user; external access included |
| Discord Nitro Business (n/a; Discord is consumer-grade for B2B) | n/a | n/a | Not designed for B2B |
| Matrix (self-hosted Synapse, ops at ~ 2 FTE) | n/a | ~ $500 000 (ops + infrastructure) | Self-managed |
| Mattermost Cloud Enterprise ($15/u/mo, negotiated) | $180 | $180 000 | Per-user |

Reading: at 1 000 users, oyatie's cell-cost is higher than per-user models. Crossover at ~ 3 000-5 000 users (per cell).

## Reproducibility

Benchmark harness at `benchmarks/connectbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks connect \
    --workload cross-tenant-message-latency \
    --tenant-class oyatie-paid \
    --duration 30m \
    --peers 5 \
    --output ./benchmark-results.json
```
