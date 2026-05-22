---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-006-feed-timeline-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, feed-cache-test, feed-render-slo]
---

# IP-006: Feed-timeline bounded context

## A. Problem
A social product without reliable feed materialization cannot match X, Instagram, TikTok, Threads, Bluesky, or Mastodon expectations.

## B. Approach
Implement the cataloged feed-timeline kernel and Valkey adapter plus planned domain/usecase/worker/rest/sdk/app layers. Provide chronological and governed ranked modes while preserving content policy, minor protection, and tenant/context isolation.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-feed-timeline-kernel.yaml` and `catalog/oya-social-feed-timeline-adapter-valkey.yaml` | Existing anchors. |
| `src/crates/oya-social-feed-timeline-{kernel,domain,usecase,adapter-postgres,adapter-valkey,worker,rest,sdk,app}/` | Planned family named by PRD/IP. |
| `decisions/ADR-SOC-0001-feed-ranking-algorithm.md` | Ranking decision source. |
| `slos/feed-render-latency.openslo.yaml` | Feed latency SLO. |

## D. Ordered implementation steps
1. Define `FeedEntry`, `RankingSignal`, `FanoutPlan`, and `RankSnapshot`.
2. Implement chronological feed first, with ranked mode behind capability/policy controls.
3. Add Valkey cache adapter and rebuild worker.
4. Apply Cedar visibility, block/mute, content policy, and minor-protection filters before render.
5. Add tests for hot/cold fanout, cache miss, deleted/tombstoned posts, and blocked authors.
6. Add latency tests for top-50 feed rendering.
7. Wire feed cache rebuild runbook and dashboard evidence.

## E. Acceptance
- `cargo nextest run -p oya-social-feed-timeline-kernel` passes.
- `cargo nextest run -p oya-social-feed-timeline-adapter-valkey` passes.
- `slos/feed-render-latency.openslo.yaml` resolves.
- `cargo run -p oya-dev-cli -- gate validate content-policy --microservice social` passes.
- `runbooks/feed-cache-rebuild.md` covers rebuild and degradation behavior.

## F. Evidence
- PRD FR-07 and performance table: `PRD.md`.
- Decision: `decisions/ADR-SOC-0001-feed-ranking-algorithm.md`.
- Policy: `policy/content-policy.cedar`, `policy/minor-protection.cedar`.
- Dashboard: `dashboards/feed-experience.json`.

## G. Counterpart comparison
X and Threads set broadcast feed expectations; Bluesky and Mastodon set user-control and federation expectations; TikTok and Instagram set visual discovery pressure. Oyatie's foundation feed should be explainable, policy-filtered, and SLO-gated rather than an engagement-only For-You clone.

## H. Foundation delivery expansion
- Deliverable detail: feed entries carry source post, author, relation state, visibility, ranking reason, and policy filter result.
- Deliverable detail: chronological mode ships first and ranked mode remains policy-controlled.
- Deliverable detail: cache keys include tenant, context, viewer, feed mode, policy version, and pagination window.
- Deliverable detail: fanout workers handle post publish, delete, moderation, block/mute, and follow graph changes.
- Deliverable detail: tombstone handling removes deleted or moderated posts without corrupting pagination.
- Deliverable detail: ranking snapshots record inputs safe enough for audit review.
- Deliverable detail: dashboard metrics include render latency, cache hit, suppression, and rebuild counts.
- Deliverable detail: Slack channel timelines are comparison pressure for ordered, moderated activity feeds.

## I. Acceptance expansion
- Acceptance detail: chronological tests must preserve event order and pagination stability.
- Acceptance detail: ranked mode tests must expose ranking reason and policy control.
- Acceptance detail: block/mute tests must remove hidden authors before render.
- Acceptance detail: minor-protection tests must suppress unsafe recommendations.
- Acceptance detail: cache rebuild tests must recover from corrupted or stale Valkey entries.
- Acceptance detail: feed SLO resolution must include top-50 render latency.
- Acceptance detail: runbook coverage must include rebuild and degraded cache behavior.
- Acceptance detail: Slack, X, Threads, TikTok, and Bluesky comparisons must be tied to feed behavior and policy evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for feed kernel and Valkey adapter.
- Evidence detail: capture content-policy gate output.
- Evidence detail: capture SLO resolution for feed render latency.
- Evidence detail: cite `ADR-SOC-0001-feed-ranking-algorithm.md`.
- Evidence detail: cite `dashboards/feed-experience.json`.
- Evidence detail: cite `policy/minor-protection.cedar` and `policy/content-policy.cedar`.
- Evidence detail: cite Slack as activity-feed/channel timeline pressure for predictable ordering and moderation.
