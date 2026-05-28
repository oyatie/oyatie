---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-009-trending-topics-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, trend-poisoning-test, content-policy-test]
---

# IP-009: Trending-topics bounded context

## A. Problem
Trending surfaces are useful but vulnerable to poisoning, policy bypass, and minor-safety failures.

## B. Approach
Implement the planned trending-topics crate family named by PRD/IP. Compute per-tenant and per-pack trend windows from policy-visible hashtag/entity signals, exclude unsafe or hidden content, and emit audit evidence for suppression and ranking decisions.

## C. Deliverables
| Artifact | Role |
|---|---|
| `src/crates/oya-community-social-trending-topics-{kernel,domain,usecase,api,adapter-postgres,adapter-valkey,worker,sdk}/` | Planned family named by PRD/IP. |
| `runbooks/trending-topic-poisoning.md` | Abuse response runbook. |
| `policy/content-policy.cedar` and `policy/abuse-defence.cedar` | Trend eligibility controls. |
| `dashboards/abuse-defence-outcomes.json` | Abuse outcome evidence. |

## D. Ordered implementation steps
1. Define trend window, rank, eligibility, and suppression-reason types.
2. Aggregate visible hashtag/entity emissions per tenant and pack.
3. Add poisoning and coordinated behavior dampening hooks.
4. Apply minor-protection and content-policy filters.
5. Store rank snapshots for audit and appeal review.
6. Test spike, bot, blocked-content, and deleted-content cases.
7. Wire trend poisoning runbook and dashboard metrics.

## E. Acceptance
- `cargo nextest run -p oya-community-social-trending-topics-kernel` passes.
- Trend eligibility tests respect `policy/content-policy.cedar`.
- Abuse dampening tests respect `policy/abuse-defence.cedar`.
- `runbooks/trending-topic-poisoning.md` remains linked.
- Trend outputs omit non-visible and minor-restricted posts.

## F. Evidence
- PRD FR-10 and moderation requirements: `PRD.md`.
- Policies: `policy/content-policy.cedar`, `policy/abuse-defence.cedar`, `policy/minor-protection.cedar`.
- Runbook: `runbooks/trending-topic-poisoning.md`.
- Feature matrix: `feature-parity-matrix-2026-05-20.md`.

## G. Counterpart comparison
X, TikTok, Instagram, Threads, and LinkedIn all have discovery/trending pressure; Mastodon is weaker here by design. Oyatie should provide trends only when the surface remains policy-filtered, abuse-resistant, and transparent enough for DSA-style review.

## H. Foundation delivery expansion
- Deliverable detail: trend records include window, tenant, pack, corpus, score, suppression reason, and audit correlation.
- Deliverable detail: input signals come from visible hashtag/entity events only.
- Deliverable detail: abuse dampening handles bot bursts, coordinated repeats, and sudden low-trust spikes.
- Deliverable detail: minor-protection filters prevent unsafe trends from reaching protected users.
- Deliverable detail: DSA transparency exports include suppression basis and appealable moderation links.
- Deliverable detail: rank snapshots are stored for review without exposing private post content.
- Deliverable detail: dashboard metrics include candidate count, suppressed count, bot dampening, and appeal outcomes.
- Deliverable detail: Slack channel trend/digest behavior is community counterpart pressure for safe discovery.

## I. Acceptance expansion
- Acceptance detail: trend tests must exclude deleted, moderated, blocked, and non-visible source posts.
- Acceptance detail: poisoning tests must dampen coordinated low-trust spikes.
- Acceptance detail: minor-protection tests must alter trend visibility by viewer class.
- Acceptance detail: snapshot tests must preserve audit reason without raw restricted content.
- Acceptance detail: DSA fixtures must include suppression and appeal references.
- Acceptance detail: runbook coverage must include freeze, suppress, rollback, and report export.
- Acceptance detail: dashboard JSON must validate for abuse outcome metrics.
- Acceptance detail: Slack, X, TikTok, and LinkedIn comparisons must map to discovery and moderation evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for trending topics kernel.
- Evidence detail: capture abuse-defence and content-policy fixture output.
- Evidence detail: capture dashboard JSON validation for abuse outcomes.
- Evidence detail: cite `policy/abuse-defence.cedar` and `policy/content-policy.cedar`.
- Evidence detail: cite `dashboards/abuse-defence-outcomes.json`.
- Evidence detail: cite `feature-parity-matrix-2026-05-20.md`.
- Evidence detail: cite Slack as community discovery pressure where channel trend/digest surfaces need moderation.
