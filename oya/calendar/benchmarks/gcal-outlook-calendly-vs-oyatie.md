---
doc_class: Benchmark
microservice: calendar
benchmark_date: 2026-05-20
related_adrs: [ADR-CAL-001, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie calendar vs Google Calendar vs Outlook Calendar vs Calendly vs Doodle

Workloads measured: (a) FREEBUSY query latency, (b) recurrence expansion throughput, (c) cross-tenant grant evaluation, (d) ICS import + recurrence-bomb defense, (e) TZDB refresh propagation, (f) annual TCO for 10k-user enterprise.

Hardware (oyatie paid on-prem): 8× calendar-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL Citus 13.0 (3 shards × 2 replicas), Valkey 7.4 cluster (5 nodes), Kafka 3.8 (5-broker), Radicale 3.2 CalDAV server.

Comparators: Google Calendar Workspace Business. Microsoft Outlook Calendar (M365 E3). Calendly Pro Teams. Doodle Business.

## Workload (a) — FREEBUSY query latency (single calendar, 7-day window, ~ 80 events)

| Platform | p99 (ms) cached | p99 (ms) cold | Cache TTL |
|---|---:|---:|---|
| oyatie calendar (paid, busy_only) | 18 | 48 | 60 s (default), 10 s (high-risk pack) |
| oyatie calendar (paid, limited_details) | 24 | 62 | 60 s |
| oyatie calendar (paid) | 12 | 28 | 60 s |
| Google Calendar (FreeBusy API) | ~ 35 | ~ 120 | Google internal |
| Outlook Calendar (EWS FreeBusyService) | ~ 80 | ~ 240 | M365 internal |
| Calendly (single user calendar) | ~ 600 (full page render) | ~ 1 800 | None |
| Doodle (poll query) | ~ 480 | ~ 1 400 | Limited |

Reading: oyatie's typed `FreebusyQuery` + four disclosure modes lets the policy evaluation happen in O(1) per requester. Cache hit ratio is dominated by `policy_hash` stability (per ADR-CAL-001 § Decision).

## Workload (b) — Recurrence expansion throughput (RRULE `FREQ=DAILY;COUNT=365`)

| Platform | Instances/sec | Cap enforced? |
|---|---:|---|
| oyatie calendar (paid) | 18 000 | Yes (10 000 per query) |
| oyatie calendar (paid) | 42 000 | Yes |
| Google Calendar (recurring event expansion) | ~ 24 000 | Limited (recurrence limit varies) |
| Outlook Calendar | ~ 18 000 | Limited |
| Calendly (no recurrence; single-shot bookings) | N/A | N/A |
| Doodle (no recurrence) | N/A | N/A |

Reading: oyatie + Google have similar raw expansion throughput. The differentiator is oyatie's hard cap (10k instances per query) which is enforced at the parser layer — Google's limit is less explicit + sometimes manifests as silently truncated results.

## Workload (c) — Cross-tenant FREEBUSY grant evaluation (50k active grants in tenant)

| Platform | p95 grant lookup (ms) | Grants supported per calendar |
|---|---:|---:|
| oyatie calendar (paid) | 4.2 | Unlimited |
| oyatie calendar (paid) | 2.8 | Unlimited |
| Google Calendar (shared calendars) | ~ 18 | Unlimited (but shared model differs) |
| Outlook Calendar (delegated permissions) | ~ 28 | Limited per-delegation patterns |
| Calendly (no grant model; share-link is per-link not per-grantee) | N/A | N/A |
| Doodle (no grant model) | N/A | N/A |

Reading: oyatie's typed `FreebusyGrant` model with per-grantee Cedar evaluation is fastest. Google/Outlook share calendars at a coarser granularity (calendar-level sharing) but don't support per-grantee disclosure modes.

## Workload (d) — ICS import 1 MB file with recurrence-bomb defense

| Platform | Normal 1 MB ICS import p99 (s) | Recurrence-bomb rejection time (ms) |
|---|---:|---:|
| oyatie calendar (paid) | 1.8 | 240 (rejected before expansion attempts) |
| oyatie calendar (paid) | 1.2 | 180 |
| Google Calendar | ~ 2.4 | Unknown (Google appears to truncate silently) |
| Outlook Calendar | ~ 3.6 | Unknown |
| Calendly | N/A (Calendly doesn't import ICS) | N/A |
| Doodle | N/A | N/A |

Reading: oyatie's pre-import recurrence expansion estimate (per ADR-CAL-001 § Decision) rejects recurrence bombs before they consume CPU. Outlook + Google occasionally silently truncate, which is worse for security audits.

## Workload (e) — TZDB refresh propagation (IANA 2026a → 2026b update affecting 10k events)

| Platform | Wall-clock (s) | Past occurrences preserved? |
|---|---:|---|
| oyatie calendar (paid) | 18 | Yes (per ADR-CAL-001 § Decision) |
| oyatie calendar (paid) | 11 | Yes |
| Google Calendar | ~ 60 (Google rolls TZDB automatically; tenant has no control) | Limited |
| Outlook Calendar | ~ 120 | Limited (timezone re-mapping can shift past events) |
| Calendly | N/A (no TZDB control) | Limited |
| Doodle | N/A | Limited |

Reading: oyatie's per-occurrence `tzdb_version` pinning (per ADR-CAL-001 § Decision) means past events NEVER shift. Outlook's behavior is the worst — TZDB updates have historically caused past events to "move" by an hour in some timezones (notably Brazil + Lebanon DST changes).

## Workload (f) — Annual TCO for 10k-user enterprise (50k events/year/user + 1k room resources + 10k scheduling-link bookings)

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie calendar (paid self-hosted) | 305 000 | 0 | 248 000 (2 SRE × 0.4 FTE) | 553 000 |
| oyatie calendar (paid) | 580 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 952 000 |
| Google Calendar (Workspace Business Standard portion ~ $4/user/mo) | 0 | 480 000 | 124 000 | 604 000 |
| Outlook Calendar (M365 E3 portion ~ $3/user/mo) | 0 | 360 000 | 124 000 | 484 000 |
| Calendly Pro Teams ($16/user/mo) | 0 | 1 920 000 | 124 000 | 2 044 000 |
| Doodle Business ($14.95/user/mo) | 0 | 1 794 000 | 124 000 | 1 918 000 |

Reading: Calendly + Doodle are surprisingly expensive at 10k seats because they price per-user even for those who only schedule occasionally. oyatie integrates the Calendly + Doodle functionality natively; no separate licensing.

## Caveats

- Calendly + Doodle are point solutions (scheduling links + group polls); they're not full calendar platforms. Direct TCO comparison is uneven.
- Hardware costs amortize over 5+ years.
- Google + Microsoft offer calendar as part of their bundles; pure calendar comparison is harder.
- FREEBUSY query latency depends heavily on cache warmth + recurrence complexity.

## Reproducibility

The benchmark harness lives at `benchmarks/calendarbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks calendar \
    --workload 10k-users-50k-events-yr \
    --tenant-class paid \
    --comparators gcal,outlook,calendly,doodle \
    --include-recurrence-bomb-suite \
    --output ./benchmark-results.json
```

Comparator runs require valid Google Workspace + M365 + Calendly + Doodle business trials. Results live at `benchmarks/results/calendar/<date>.csv` and are re-run quarterly.
