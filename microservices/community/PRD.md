---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-community
microservice: community
status: Accepted
sales_segment: shared-substrate
tier: shared
milestone_first_ship: M02-shared-substrate
bominal_source: [ADR-0208]
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/connect-unbundle.json]
date: 2026-05-17
owner_team: axis-community
doc_status: published
---

# PRD-community: Org-wide Community Surface (Announcements + Q&A + KB + Discussion)

## Purpose

The `community` µservice is oyatie's org-wide community surface. Per parallel-session ADR-0126 (Connect unbundle), the legacy Bominal Connect product split into two sibling µservices: `messenger` (real-time chat + DMs) and `community` (this µservice: org-wide announcements, Q&A, knowledge-base articles, threaded discussion forums, voting, moderation, search).

This µservice is **shared substrate** consumed by every product (Workflow Studio, Foundry, Ontology consumers, Workflow-Engine clients) and exposed directly to tenants for their internal community surface. It is the competitive parity surface for Atlassian Community, Microsoft Yammer/Viva Engage, Salesforce Community Cloud, Discourse, Slack channels (read-only mode), and Stack Overflow Teams.

Inherits Bominal ADR-0208 dual-context posture (B2B tier + B2C tier) per `feedback_bominal_inheritance_precedence.md`.

## Tenant Value

- **Tenant Outcome 1 — One canonical community surface per org.** No second product purchase; community is shared substrate with the same SLOs and the same audit-chain that the rest of oyatie ships with.
- **Tenant Outcome 2 — Q&A graph wins, not chat threads.** Stack Overflow-grade voted Q&A with accepted-answer semantics, surfaced in search, with cross-product `ontology` links to the entity types being discussed.
- **Tenant Outcome 3 — Knowledge-base articles are first-class.** Long-form curated content with attachment store (S3-backed), revisions, and tenant-controlled publication review. Replaces Confluence-class workflows for org knowledge.
- **Tenant Outcome 4 — Moderation is auditable.** Every moderation action emits an audit-chain record (Merkle / Ed25519); spam / abuse / impersonation flows route through `foundry-guardrails` for policy enforcement.
- **Tenant Outcome 5 — Cross-product mention-resolution.** Mentions resolve against `messenger` (chat), `ontology` (entity types), and the tenant's directory via `tenancy`. Mentions never reach across tenant boundaries.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant member | to publish an announcement to a community space | I reach the whole org or a scoped audience | post-store | Must |
| FR-02 | tenant member | to ask a question and accept the best answer | knowledge accumulates in a voted graph rather than chat scrollback | post-store, thread-tree, voting-engine | Must |
| FR-03 | tenant member | to publish a knowledge-base article with attachments | long-form curated content lives next to community threads | kb-article-store | Must |
| FR-04 | tenant member | to reply in a threaded discussion forum | conversation depth is preserved without flattening | thread-tree | Must |
| FR-05 | tenant member | to upvote / downvote posts and answers | best content rises algorithmically; vote p99 ≤ 100 ms | voting-engine | Must |
| FR-06 | tenant moderator | to hide / lock / pin / move / merge posts | community health is maintainable | moderation-queue | Must |
| FR-07 | tenant member | to search across announcements + Q&A + KB articles | discovery is one entry point; search p99 ≤ 500 ms | search-index | Must |
| FR-08 | tenant member | to subscribe to a space / thread / tag | I receive notifications when content I care about changes | post-store, search-index | Must |
| FR-09 | tenant operator | to view per-space usage + moderation metrics | I can plan capacity, validate engagement, meet contractual SLAs | post-store, moderation-queue | Must |
| FR-10 | foundry-guardrails | to consume `PostCreated` and `PostEdited` events for spam / abuse classification | I can take a moderation action without polling | post-store | Must |
| FR-11 | ontology | to expose cross-product links inside KB articles + posts | a community thread can deep-link to an entity instance | post-store, kb-article-store | Must |
| FR-12 | messenger | to resolve `@user` mentions inside posts via `community` mention-resolution | both surfaces share a single identity registry | post-store | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Feed render (per-space) | ≤80 ms | ≤300 ms | ≤800 ms | Redis hot-feed cache; warm hit on read |
| Search query (cross-space, ranked) | ≤120 ms | ≤500 ms | ≤1.2 s | Elasticsearch / Tantivy multi-index query |
| Vote cast (idempotent) | ≤25 ms | ≤100 ms | ≤300 ms | Redis-buffered + Postgres flush; conflict-free counter |
| Post create | ≤80 ms | ≤250 ms | ≤700 ms | Postgres insert + async fan-out to search + Redis |
| Post edit | ≤80 ms | ≤250 ms | ≤700 ms | revision append; search reindex async |
| KB article publish | ≤200 ms | ≤500 ms | ≤1.5 s | Postgres insert + S3 attachment uploads (resumable) |
| Moderation action (hide/lock/pin) | ≤80 ms | ≤200 ms | ≤500 ms | Postgres update + audit-chain seal |
| Threaded reply render (1 000 nodes) | ≤100 ms | ≤350 ms | ≤900 ms | materialised path + lazy load |

### Security

- All writes are authenticated via `tenancy`-issued JWT; tenant boundary enforced at every layer (RLS in Postgres, multi-index isolation in Elasticsearch, per-tenant Redis key prefix, per-tenant S3 prefix in KB attachment store).
- Every moderation action and vote-cast event emits an audit-chain record (Merkle / Ed25519 per ADR-0028).
- KB article attachments are antivirus-scanned (ClamAV inline) before publication; oversize / malicious uploads bounce at the adapter-s3 layer.
- Cross-tenant mention-resolution is forbidden at the Cedar policy layer (see `policy/tenant-scope.cedar`).
- Mass-spam abuse path: `foundry-guardrails` classifier emits `PostShouldHide` events that the moderation-queue worker consumes; defence-in-depth is per-tenant rate-limit (post create ≤ 60 / min / member; vote ≤ 600 / min / member).

### Audit + Compliance

- Append-only audit log: every `PostCreated`, `PostEdited`, `PostDeleted`, `VoteCast`, `ModerationActioned`, `KBArticlePublished` event is sealed within 1 s.
- Section 230 + similar safe-harbor stance in `compliance.md`: oyatie is a provider; tenants are publishers; moderation is good-faith.
- Per-tenant retention: announcements 7 y default; Q&A indefinite; KB articles indefinite (revisions sealed); moderation actions 7 y.
- HIPAA: when pack-us-healthcare is active, PHI in community posts is opt-in only via tenant-side warning + Cedar entitlement.

### Availability + SLO

- Availability target: 99.95 % monthly for read paths; 99.9 % monthly for write paths.
- RTO ≤ 15 min; RPO ≤ 30 s (Postgres WAL + Redis AOF).
- Search index rebuild SLO ≤ 60 min for 10⁷ documents (drill quarterly).

### Data residency

- Posts, KB articles, votes, and moderation records inherit the tenant's `jurisdiction_code` per ADR-0117. Postgres + Elasticsearch + Redis + S3 are all per-region; cross-region replication is opt-in.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename), layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-search`, `adapter-s3`, `adapter-moderation-bridge`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `post-store` | `oya-community-post-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Author + edit + delete posts; thread parent links; mention resolution; revision history | `Post`, `Author`, `Mention`, `Revision`, `SpaceRef` |
| `thread-tree` | `oya-community-thread-tree-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Threaded reply structure (materialised path); lazy-load children | `Thread`, `Node`, `Path`, `Depth` |
| `voting-engine` | `oya-community-voting-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Upvote / downvote; accepted-answer; conflict-free counter | `Vote`, `Tally`, `Acceptance` |
| `moderation-queue` | `oya-community-moderation-queue-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-moderation-bridge,worker,sdk}` | Moderator actions; flag triage; spam classifier consumption | `Flag`, `Action`, `QueueItem`, `ModeratorVerdict` |
| `kb-article-store` | `oya-community-kb-article-store-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,sdk,app}` | Long-form curated articles; attachment store; revision pinning | `Article`, `Attachment`, `Revision`, `PublicationState` |
| `search-index` | `oya-community-search-index-{kernel,domain,usecase,api,adapter,adapter-search,worker,sdk}` | Cross-BC search; ranking; tag taxonomy | `Document`, `Index`, `Tag`, `RankSignal` |

Naming justification — `post-store`:

```
NAME: oya-community-post-store-<layer>
JUSTIFICATION:
  L1 prefix: oya (workspace)
  L2 domain: community (this microservice)
  L3 BC: post-store (post authorship aggregate)
  L4 layer: kernel|domain|usecase|api|adapter|adapter-postgres|rest|worker|sdk|app
  Conforms: BNF v4.1, 13-layer enum ADR-0105
```

## Cross-µservice Consumption

| Consumed | Purpose |
|---|---|
| `tenancy` | JWT issuance; per-tenant scope claims |
| `audit-chain` | Append-only sealing of post/vote/moderation events |
| `ontology` | Cross-product entity links inside posts + KB articles |
| `messenger` | `@user` mention-resolution shared identity registry |
| `foundry-guardrails` | Spam / abuse / impersonation classification |
| `observability` | SLO authoring; burn-rate gating; promotion eligibility |

## Competitor Parity

See `competitor-parity-matrix.md`. Targets: Atlassian Community + Microsoft Yammer/Viva Engage + Salesforce Community Cloud + Discourse + Slack channel-feed + Stack Overflow Teams.

## Deferrals

- Live-stream / video-post hosting: out of scope for M02; defer to a later "community-media" sibling.
- AI-generated answer synthesis: depends on `foundry-runtime`; defer to M03 cross-product integration.
- Federated communities (multi-tenant cross-org): defer to M04 federation initiative.

## Out-of-Scope

- Real-time DMs and ephemeral chat → `messenger`.
- Identity directory + SSO → `tenancy`.
- Entity-type modelling → `ontology`.
