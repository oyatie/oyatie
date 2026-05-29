---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-010-notifications-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, notification-fanout-slo, websocket-smoke]
---

# IP-010: Notifications bounded context

## A. Problem
Mentions, reactions, follows, moderation, appeals, and digest delivery require idempotent notification fanout with backpressure and policy filtering.

## B. Approach
Implement the planned notifications crate family with Postgres durability, Valkey fanout queues, worker coalescing, websocket delivery, and digest generation. Every notification is scoped by tenant, principal, context, and visibility policy.

## C. Deliverables
| Artifact | Role |
|---|---|
| `src/crates/oya-community-social-notifications-{kernel,domain,usecase,api,adapter-postgres,adapter-valkey,worker,sdk,app}/` | Planned family named by PRD/IP. |
| `contracts/asyncapi/social-events.yaml` | Notification-triggering event source. |
| `slos/notification-fanout-latency.openslo.yaml` | Fanout SLO. |
| `runbooks/mention-storm-throttle.md` | Burst-control runbook. |

## D. Ordered implementation steps
1. Define notification, digest bucket, realtime frame, and idempotency key types.
2. Implement policy-filtered recipient resolution.
3. Add Valkey queueing with retry and coalescing.
4. Add websocket frame delivery and digest worker paths.
5. Test duplicate events, blocked recipients, muted minors, and backpressure.
6. Emit audit and SLO metrics for delivered, delayed, suppressed, and failed notifications.
7. storm throttling runbooks and dashboards.

## E. Acceptance
- `cargo nextest run -p oya-community-social-notifications-kernel` passes.
- `slos/notification-fanout-latency.openslo.yaml` resolves.
- Fanout tests prove idempotency for repeated events.
- Notification policy tests respect `policy/dm-scope.cedar` and `policy/minor-protection.cedar`.
- WebSocket smoke tests complete without cross-tenant delivery.

## F. Evidence
- PRD FR-16 and notification performance targets: `PRD.md`.
- Contracts: `contracts/asyncapi/social-events.yaml`.
- Policies: `policy/dm-scope.cedar`, `policy/minor-protection.cedar`.
- SLO: `slos/notification-fanout-latency.openslo.yaml`.

## G. Counterpart comparison
X, Instagram, TikTok, Snapchat, Threads, and Bluesky all create strong realtime notification expectations. Oyatie must match responsiveness while making suppression, coalescing, and minor-protection behavior explicit and auditable.

## H. Foundation delivery expansion
- Deliverable detail: notification records include trigger event, recipient, channel, context, policy result, and idempotency key.
- Deliverable detail: digest buckets preserve coalescing reason and delay target.
- Deliverable detail: websocket frames include only policy-visible fields.
- Deliverable detail: Valkey queue entries carry retry count, backoff, tenant, and audit correlation.
- Deliverable detail: suppression evidence records muted, blocked, minor-protected, hidden, and rate-limited outcomes.
- Deliverable detail: storm controls coalesce mention bursts and fanout spikes.
- Deliverable detail: dashboard metrics include delivered, delayed, failed, suppressed, and retried counts.
- Deliverable detail: Slack notification and channel mention behavior is a direct counterpart for realtime fanout expectations.

## I. Acceptance expansion
- Acceptance detail: duplicate event tests must prove idempotent notification creation.
- Acceptance detail: blocked/muted tests must suppress delivery before websocket or digest output.
- Acceptance detail: minor-protection tests must prove default-silent or restricted delivery where required.
- Acceptance detail: backpressure tests must use retry and dead-letter paths.
- Acceptance detail: websocket smoke tests must prove cross-tenant frames are impossible.
- Acceptance detail: SLO resolution must include notification fanout latency.
- Acceptance detail: storm runbook must cover throttle, drain, and replay.
- Acceptance detail: Slack, X, Instagram, and Snapchat comparisons must map to realtime notification behavior and suppression evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for notifications kernel and worker.
- Evidence detail: capture AsyncAPI validation for trigger events.
- Evidence detail: capture DM-scope and minor-protection policy tests.
- Evidence detail: cite `slos/notification-fanout-latency.openslo.yaml` if present.
- Evidence detail: cite `policy/dm-scope.cedar` and `policy/minor-protection.cedar`.
- Evidence detail: cite `runbooks/mention-storm-throttle.md` if present.
- Evidence detail: cite Slack as notification and channel-mention counterpart pressure.
