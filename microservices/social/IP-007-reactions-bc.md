---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-007-reactions-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, reaction-counter-test, audit-event-test]
---

# IP-007: Reactions bounded context

## A. Problem
Reactions are high-volume engagement writes that must not corrupt counts, leak blocked content, or bypass audit and moderation rules.

## B. Approach
Implement the planned reactions kernel/domain/usecase/adapters/worker/sdk family named by PRD and IP. Use Valkey buffering plus Postgres durable records, with per-user uniqueness and tombstone-aware counter repair.

## C. Deliverables
| Artifact | Role |
|---|---|
| `src/crates/oya-community-social-reactions-{kernel,domain,usecase,api,adapter-postgres,adapter-valkey,worker,sdk}/` | Planned family named by PRD/IP. |
| `contracts/asyncapi/social-events.yaml` | `ReactionAdded` event source. |
| `policy/content-policy.cedar` | Visibility and moderation guard. |
| `runbooks/feed-cache-rebuild.md` | Counter/feed repair closure. |

## D. Ordered implementation steps
1. Define bounded reaction vocabulary, reaction record, and tally types.
2. Enforce one active reaction per user/post/reaction-kind where product rules require it.
3. Add write-through or buffered counter strategy with idempotency keys.
4. Deny reactions to unavailable, blocked, or policy-hidden posts.
5. Emit audit and feed invalidation events.
6. Test concurrent add/remove, duplicate writes, and counter rebuild.
7. Wire metrics used by feed and moderation dashboards.

## E. Acceptance
- `cargo nextest run -p oya-community-social-reactions-kernel` passes.
- `cargo nextest run -p oya-community-social-reactions-adapter-valkey` passes.
- Reaction writes respect `policy/content-policy.cedar`.
- AsyncAPI event validation passes for reaction events.
- Counter repair evidence links to `runbooks/feed-cache-rebuild.md`.

## F. Evidence
- PRD FR-06 and Workflow event requirements: `PRD.md`.
- Contracts: `contracts/asyncapi/social-events.yaml`.
- Policy: `policy/content-policy.cedar`, `policy/public-read.cedar`.

## G. Counterpart comparison
Instagram, TikTok, X, Threads, Bluesky, Mastodon, and LinkedIn all support lightweight engagement. Oyatie should match expected reaction ergonomics while keeping counters tenant-scoped, auditable, and repairable.

## H. Foundation delivery expansion
- Deliverable detail: reaction vocabulary defines allowed kinds, per-post uniqueness, actor, target, and tombstone state.
- Deliverable detail: counter updates carry idempotency keys and repair sequence numbers.
- Deliverable detail: Valkey buffering is optional acceleration; Postgres remains durable truth.
- Deliverable detail: policy checks deny reactions to blocked, deleted, hidden, moderated, or minor-restricted posts.
- Deliverable detail: reaction events invalidate feed and notification projections.
- Deliverable detail: repair worker recomputes counts from durable reaction records.
- Deliverable detail: audit logs include actor, target, policy result, and visible counter delta.
- Deliverable detail: Slack emoji reactions are a direct counterpart for community/channel engagement expectations.

## I. Acceptance expansion
- Acceptance detail: duplicate reaction tests must prove stable idempotent results.
- Acceptance detail: concurrent add/remove tests must preserve final counter correctness.
- Acceptance detail: blocked and hidden post tests must deny reactions before counter mutation.
- Acceptance detail: repair tests must rebuild counters after simulated Valkey loss.
- Acceptance detail: AsyncAPI validation must pass for reaction-added and reaction-removed events.
- Acceptance detail: feed invalidation tests must update affected timelines.
- Acceptance detail: public-read policy tests must redact restricted counter detail where required.
- Acceptance detail: Slack, LinkedIn, X, and Instagram comparisons must map to engagement and counter evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for reactions kernel and Valkey adapter.
- Evidence detail: capture content-policy fixture output.
- Evidence detail: capture AsyncAPI validation for reaction events.
- Evidence detail: cite `contracts/asyncapi/social-events.yaml`.
- Evidence detail: cite `policy/content-policy.cedar` and `policy/public-read.cedar`.
- Evidence detail: cite `runbooks/feed-cache-rebuild.md` for counter/feed repair.
- Evidence detail: cite Slack as the approved counterpart name for emoji-style community reactions.
