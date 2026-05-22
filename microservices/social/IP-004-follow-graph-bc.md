---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-004-follow-graph-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, graph-policy-test, follow-latency-slo]
---

# IP-004: Follow-graph bounded context

## A. Problem
Feeds, notifications, mentions, blocks, mutes, and minor-protection defaults all depend on a correct directed relationship graph.

## B. Approach
Implement the cataloged follow-graph kernel and Postgres adapter plus planned domain/usecase/worker/sdk layers. Store follow, block, mute, and derived friend edges with tenant/context boundaries and audit evidence.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-follow-graph-kernel.yaml` and `catalog/oya-social-follow-graph-adapter-postgres.yaml` | Existing anchors. |
| `src/crates/oya-social-follow-graph-{kernel,domain,usecase,adapter-postgres,worker,sdk}/` | Planned family named by PRD/IP. |
| `slos/follow-action-latency.openslo.yaml` | Follow mutation SLO. |
| `runbooks/follow-graph-corruption.md` | Repair and rollback runbook. |

## D. Ordered implementation steps
1. Define follow, block, mute, and friend-derivation types.
2. Implement graph invariants for self-follow refusal, block precedence, and context isolation.
3. Add Postgres adjacency-list adapter with tenant-scoped indexes.
4. Emit graph-change events to feed, notification, and audit consumers.
5. Add corruption detection and replay tests.
6. Add latency tests for follow/unfollow and block/mute actions.
7. Wire runbook evidence for graph rebuild.

## E. Acceptance
- `cargo nextest run -p oya-social-follow-graph-kernel` passes.
- `cargo nextest run -p oya-social-follow-graph-adapter-postgres` passes.
- `cargo run -p oya-dev-cli -- gate validate data-residency --microservice social` passes.
- `slos/follow-action-latency.openslo.yaml` resolves.
- `runbooks/follow-graph-corruption.md` covers rebuild and audit reconciliation.

## F. Evidence
- PRD FR-02 and graph port table: `PRD.md`.
- Catalog: `catalog/oya-social-follow-graph-*.yaml`.
- Policy: `policy/tenant-scope.cedar`, `policy/minor-protection.cedar`.
- SLO: `slos/follow-action-latency.openslo.yaml`.

## G. Counterpart comparison
X, Instagram, TikTok, Bluesky, Mastodon, Threads, and LinkedIn all rely on follow or connection graphs. Oyatie's graph must meet expected UX while making block/mute/minor-protection and tenant/context constraints auditable.

## H. Foundation delivery expansion
- Deliverable detail: graph types include follow, unfollow, block, unblock, mute, unmute, friend, and edge tombstone records.
- Deliverable detail: block precedence outranks follow and recommendation surfaces.
- Deliverable detail: graph adapter indexes tenant, context, source, target, edge type, and tombstone status.
- Deliverable detail: mutation events feed notification, timeline, search, audit, and abuse consumers.
- Deliverable detail: rebuild worker can replay graph-change events into derived projections.
- Deliverable detail: minor-protection rules can suppress follow suggestions and contact pathways.
- Deliverable detail: corruption detection reports missing inverse, impossible edge, and stale projection cases.
- Deliverable detail: Slack channel membership and community moderation are comparison pressure for relationship boundaries.

## I. Acceptance expansion
- Acceptance detail: self-follow tests must reject direct and normalized self-target variants.
- Acceptance detail: block/mute tests must prove feed and notification effects.
- Acceptance detail: RLS tests must reject cross-tenant graph reads and writes.
- Acceptance detail: replay tests must rebuild derived friend edges from append-only events.
- Acceptance detail: latency SLO tests must include follow and block operations.
- Acceptance detail: corruption runbook must cover detect, freeze, rebuild, and reconcile steps.
- Acceptance detail: event contracts must validate for graph-change emissions.
- Acceptance detail: Slack, LinkedIn, X, and Mastodon comparisons must map to relation/permission behavior.

## J. Evidence expansion
- Evidence detail: capture nextest output for graph kernel and adapter crates.
- Evidence detail: capture data-residency or tenant-scope gate output.
- Evidence detail: capture AsyncAPI validation for graph-change events.
- Evidence detail: cite `policy/minor-protection.cedar` for protected relation behavior.
- Evidence detail: cite `catalog/oya-social-follow-graph-*.yaml` for crate anchors.
- Evidence detail: cite `runbooks/follow-graph-corruption.md` if present, or record it as a required runbook gap.
- Evidence detail: cite Slack as relationship-surface pressure where membership and moderation constrain social graph access.
