---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-014-notifications-and-creator-analytics-bc
status: pending
owner: axis-shorts + axis-messenger
depends_on: [IP-008, IP-009]
---

# IP-014: notifications + creator-analytics BC end-to-end

## Intent

- `notifications` BC: real-time WebSocket + digest worker; per-recipient idempotent; backpressure-coalesced. Celebrity-tier (>1M followers) sharded fanout.
- `creator-analytics` BC: per-creator dashboards — watch-time, audience demographics, posting cadence, audience growth. k-anonymity ≥ 10 on demographic slices.

## ChangeSet boundary

10 + 8 = 18 crates.

## Concrete File Targets

Key entities: `Notification`, `DigestBucket`, `RealtimeFrame`, `CreatorMetric`, `AudienceSlice`, `PostingCadence`, `AudienceGrowth`.

Ports: `NotificationSink`, `DigestBatcher`, `AnalyticsAggregator`, `KAnonymityEnforcer`.

## Acceptance Gates

```bash
cargo build -p oya-shorts-notifications-worker
cargo build -p oya-shorts-creator-analytics-worker
cargo nextest run -p oya-shorts-notifications-{kernel,domain,usecase,adapter-postgres,adapter-redis}
cargo nextest run -p oya-shorts-creator-analytics-{kernel,domain,usecase,adapter-postgres,adapter-redis}
```

E2E: 10k follower fanout ≤ 2s p99; creator analytics dashboard with k≥10 enforcement; suppressed slices for low-cardinality.

## Halt Conditions

- k-anonymity threshold violated — Sev-2.

## Next IP

[`IP-015-drm-and-hg-shorts-registration.md`](IP-015-drm-and-hg-shorts-registration.md)

## References

- PRD FR-21, FR-22.
- `threat-model.md` T-L-14.
