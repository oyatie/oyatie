---
doc_class: Phase
template_id: TPL-PHASE
phase_id: PHASE-01-community-substrate
microservice: community
status: Accepted
milestone: M02-shared-substrate
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/connect-unbundle.json]
date: 2026-05-17
owner_team: axis-community
doc_status: published
---

# PHASE-01 — Community substrate (org-wide announcements + Q&A + KB + discussion)

## Intent

Ship the M02 community substrate: post-store, thread-tree, voting-engine, moderation-queue, kb-article-store, and search-index. End-state: a tenant can publish announcements, ask + answer questions, write KB articles, vote, moderate, and search across the surface — with full audit-chain coverage, full SLO authoring, and full IaC.

## Scope

In-scope:
- 6 BCs × layer set per ADR-0105.
- Layer-A substrate: Postgres (post-store + voting-engine + moderation-queue + kb-article-store), Elasticsearch (search-index), Redis (hot-feed cache), S3 (KB attachment store).
- IPs IP-001 through IP-015 covered in this Phase.
- All policy (Cedar fragments), runbooks, dashboards, capabilities, contracts.

Out of scope:
- Live-stream / video-post BC.
- AI answer synthesis (defer to M03 foundry integration).
- Federated communities (defer to M04).

## Sequence

1. **IP-001 Postgres post-store + Citus shard plan** — adopt the Citus-on-Postgres pattern; per-tenant `tenant_id` distribution column; RLS on every table; WAL retention 7 d.
2. **IP-002 post-store kernel + domain** — types: `Post`, `Author`, `Mention`, `Revision`, `SpaceRef`.
3. **IP-003 post-store usecase + api** — `PostAuthored`, `PostEdited`, `PostDeleted` commands + queries; spec'd contract surface.
4. **IP-004 post-store adapter-postgres + rest + worker + sdk + app** — full crate set landed.
5. **IP-005 thread-tree (materialised path)** — kernel + domain + usecase + adapter-postgres; benchmark 10⁴-node tree at p99 ≤ 350 ms.
6. **IP-006 voting-engine** — Redis-buffered counter; per-tenant tally; idempotent vote cast.
7. **IP-007 moderation-queue** — flag triage; moderator verdict; audit-chain seal per action.
8. **IP-008 kb-article-store + S3 attachment adapter** — resumable multipart upload; ClamAV inline scan.
9. **IP-009 search-index (Elasticsearch)** — per-tenant index; reindex pipeline; tag taxonomy.
10. **IP-010 foundry-guardrails moderation bridge adapter** — consumes `PostCreated` events; emits `PostShouldHide`.
11. **IP-011 cedar policy fragments** — tenant-scope + ci-scope + auditor-scope + public-read.
12. **IP-012 OpenSLO manifests + Grafana dashboards** — feed-throughput, vote-rate, moderation-queue-depth, search-latency.
13. **IP-013 oya-vcs promotion-readiness wiring** — community µservice fronts a promotion gate identical to observability.
14. **IP-014 hyperscaler maturity gate HG-COMMUNITY** — claim parity matrix vs. Atlassian + Viva + Salesforce + Discourse + Stack Overflow Teams.
15. **IP-015 capacity + cost + chaos drill** — quarterly load drill at 10× nominal; failure-modes.md coverage.

## Acceptance

- `cargo test -p oya-community-*` green across all 6 BCs.
- All OpenSLO manifests authored at `microservices/community/slos/*.openslo.yaml`.
- Promotion-eligibility verdict for the community µservice is `GREEN` for `(community, source_sha, dev → staging)` and `(community, source_sha, staging → production)` for 7 consecutive cycles.
- Quarterly chaos drill: search index rebuild from cold completes in ≤ 60 min for 10⁷ docs.
- Quarterly chaos drill: moderation-queue clear completes in ≤ 30 min for 10⁵ flagged items.
- Cedar fragment coverage: every action in `community.proto` has a corresponding `permit` or explicit `forbid`.
- Audit-chain seal latency p99 ≤ 1 s.
- Multi-region pack-kr overlay green.

## Risks

| Risk | Mitigation |
|---|---|
| Search index rebuild storm under traffic spike | Per-tenant rebuild scheduler with token-bucket; staggered rebuild windows |
| Vote race + double-count | Redis Lua script for atomic increment + idempotency key per (member, post) |
| Moderation queue OOM under flag storm | Per-tenant queue depth cap; overflow to S3 cold queue with worker drain |
| KB attachment S3 outage | Cross-region replication + retry queue + degraded mode (show without attachment) |
| Mass-spam abuse against new tenant | foundry-guardrails-driven per-member post rate-limit + new-member cooldown |
| Cross-tenant mention leakage | Cedar policy + RLS belt-and-braces; deny-by-default at fragment layer |

## Owners

- Pillar lead: axis-community
- Reviewers: ops-security, council-architecture, council-privacy, axis-observability
