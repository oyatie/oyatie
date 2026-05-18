---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-010-notifications-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + axis-messenger
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location]
---

# IP-010: notifications BC (kernel → domain → usecase → adapter-postgres + adapter-redis + worker + websocket + sdk + app)

## Intent

Author the `notifications` BC: real-time WebSocket delivery + digest worker;
per-recipient idempotent processing; backpressure-coalesced under burst;
cross-µservice messenger-bridge (notification of social mention surfaces in
messenger inbox per ADR-SOC successor-IP); per-tenant pack-aware throttle.

## ChangeSet boundary

`notifications` BC end-to-end.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-notifications-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-notifications-domain/src/{notification,digest_bucket,realtime_frame,backpressure}.rs` | create |
| `src/crates/oya-social-notifications-usecase/src/{emit,dispatch,coalesce}.rs` | create |
| `src/crates/oya-social-notifications-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-notifications-adapter-redis/src/queue.rs` | create — Redis Streams for fanout |
| `src/crates/oya-social-notifications-worker/src/{dispatcher,digest_builder,messenger_bridge}.rs` | create |
| `tests/notifications_fanout_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-notifications-kernel
cargo nextest run -p oya-social-notifications-domain
```

## Test Plan

- Per-recipient idempotency: same notification event → single delivery.
- Backpressure coalesce: burst notifications collapsed into digest within 250ms window.
- AC-06 E2E: notification fanout to 10k followers ≤ 2s p99.
- Cross-µservice: social mention surfaces in messenger inbox via Workflow event bridge.
- Per-tenant pack-aware throttle: trial-tier ≤ 100 notifications/min/account.

## Halt Conditions

- Notification fanout backlog > 500k (FM-11) → runbook activates; surge scaling.

## Next IP

[`IP-011-content-moderation-bc.md`](IP-011-content-moderation-bc.md)
