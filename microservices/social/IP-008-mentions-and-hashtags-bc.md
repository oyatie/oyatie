---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-008-mentions-and-hashtags-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, mention-policy-test, notification-handoff-test]
---

# IP-008: Mentions and hashtags bounded contexts

## A. Problem
Mentions and hashtags power discovery, notifications, search, abuse controls, and cross-product handoff. They must resolve safely against tenant/context and block/mute policy.

## B. Approach
Implement the planned mention and hashtag crate families already named by PRD/IP. Mention parsing resolves targets through Ontology/profile ports and emits notification handoffs; hashtag parsing feeds search and trending without leaking restricted posts.

## C. Deliverables
| Artifact | Role |
|---|---|
| `src/crates/oya-social-mentions-{kernel,domain,usecase,api,adapter,worker,sdk}/` | Planned mention family named by PRD/IP. |
| `src/crates/oya-social-hashtags-{kernel,domain,usecase,api,adapter-postgres,worker,sdk}/` | Planned hashtag family named by PRD/IP. |
| `contracts/asyncapi/social-events.yaml` | Mention and hashtag event source. |
| `policy/content-policy.cedar` and `policy/dm-scope.cedar` | Visibility and notification guards. |

## D. Ordered implementation steps
1. Define mention target, hashtag token, corpus, and fanout-plan types.
2. Parse posts after content normalization and before publication.
3. Resolve mention targets with tenant/context and block/mute filters.
4. Emit notification events with idempotency keys.
5. Add hashtag corpus writes for search/trending.
6. Test invisible/blocked/minor targets and duplicate hashtags.
7. Wire mention-storm throttling and moderation signals.

## E. Acceptance
- `cargo nextest run -p oya-social-mentions-kernel` passes.
- `cargo nextest run -p oya-social-hashtags-kernel` passes.
- Notification fanout respects `policy/dm-scope.cedar`.
- `runbooks/mention-storm-throttle.md` covers burst handling.
- AsyncAPI mention/hashtag events validate.

## F. Evidence
- PRD FR-08, FR-09, FR-25, FR-26: `PRD.md`.
- Contracts: `contracts/asyncapi/social-events.yaml`.
- Policies: `policy/content-policy.cedar`, `policy/dm-scope.cedar`, `policy/minor-protection.cedar`.
- Runbook: `runbooks/mention-storm-throttle.md`.

## G. Counterpart comparison
X, Instagram, TikTok, Threads, Bluesky, Mastodon, and LinkedIn all use mentions and hashtags for discovery. Oyatie's counterpart requirement is familiar behavior plus tenant-safe resolution, minor-protection defaults, and audit-backed notification handoff.

## H. Foundation delivery expansion
- Deliverable detail: mention parsing resolves handles after normalization and context policy checks.
- Deliverable detail: hashtag parsing records canonical token, display token, locale, corpus, and source post.
- Deliverable detail: notification fanout excludes blocked, muted, minor-restricted, and hidden targets.
- Deliverable detail: hashtag corpus writes exclude deleted or moderated posts.
- Deliverable detail: duplicate tokens collapse deterministically for indexing and notification.
- Deliverable detail: mention-storm throttling records dropped, coalesced, and delayed fanout counts.
- Deliverable detail: search/trending handoff uses events, not direct crate imports.
- Deliverable detail: Slack @mentions and channel tags are direct community counterpart pressure.

## I. Acceptance expansion
- Acceptance detail: mention parser tests must handle Unicode, case folding, punctuation, and duplicate mentions.
- Acceptance detail: invisible target tests must suppress notification and avoid existence leaks.
- Acceptance detail: hashtag tests must normalize display and canonical tokens without losing locale intent.
- Acceptance detail: minor-protection tests must suppress unsafe contact pathways.
- Acceptance detail: AsyncAPI validation must pass for mention and hashtag events.
- Acceptance detail: storm runbook must include throttle, replay, and notification repair behavior.
- Acceptance detail: search/trending integration tests must consume events rather than direct imports.
- Acceptance detail: Slack, X, Instagram, and Mastodon comparisons must map to mention/tag behavior and safety evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for mentions and hashtags kernels.
- Evidence detail: capture AsyncAPI validation for mention/hashtag events.
- Evidence detail: capture minor-protection or DM-scope policy fixture output.
- Evidence detail: cite `policy/dm-scope.cedar` and `policy/minor-protection.cedar`.
- Evidence detail: cite `runbooks/mention-storm-throttle.md` if present, or record it as a required gap.
- Evidence detail: cite `contracts/asyncapi/social-events.yaml`.
- Evidence detail: cite Slack as the approved counterpart for @mention/channel-tag moderation pressure.
