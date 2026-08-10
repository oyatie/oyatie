---
doc_class: ADRIndex
microservice: social
date: 2026-05-17
owner_team: axis-social + council-privacy + ops-security
doc_status: published
---

# social µservice — service-scoped ADRs

This directory holds ADRs that govern the `social` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/social/PRD.md`, in `microservices/social/PHASE-01-SOCIAL-FOUNDATION.md`, or in a policy / runbook / threat-model artifact under `microservices/social/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-SOC-0001](./ADR-SOC-0001-feed-ranking-algorithm.md) | Feed-ranking algorithm — hybrid chronological-first + heuristic-algorithmic in P01; ML-driven ranking scheduled-for-distinct-tracked-work to P03 with EU AI Act high-risk obligations | Accepted | 2026-05-17 | derived gap from PRD §"Out-of-scope"; ranking model is high-risk per EU AI Act Annex III §1(a) |
| [ADR-SOC-0002](./ADR-SOC-0002-follow-graph-storage.md) | Follow-graph storage — Postgres adjacency-list primary; graph-database adapter (Dgraph / JanusGraph) future-pluggable | Accepted | 2026-05-17 | derived gap from capacity-model.md; storage choice critical for follow-action p99 ≤ 50ms |
| [ADR-SOC-0003](./ADR-SOC-0003-content-moderation-classifier-bounds.md) | Content-moderation classifier bounds — EU AI Act high-risk classification confirmed; Art. 9-15 + Art. 50 obligations operative; alignment with messenger ADR-MSGR + mail ADR-MAIL spam-classifier pattern | Accepted | 2026-05-17 | derived gap from compliance.md + capabilities/T2-auto.yaml; EU AI Act 2024/1689 |
| [ADR-SOC-0004](./ADR-SOC-0004-federation-posture.md) | Federation posture — ActivityPub OFF by default; tenant-opt-in per-tenant for Professional-tier only; Personal-tier NEVER federates (compile-time invariant DCI-08); align with messenger ADR-MSGR-0004 | Accepted | 2026-05-17 | PRD Open Question 2 + DCI-08 invariant |
| [ADR-SOC-0005](./ADR-SOC-0005-dual-context-feed-isolation.md) | Dual-context feed isolation — Personal pillar feed vs Professional pillar feed; same dual-context-isolation pattern as messenger/mail; data-model invariant per parallel ADR-0238 | Accepted | 2026-05-17 | parallel ADR-0238 inheritance + threat-model T-I-07 |
| [ADR-SOC-0006](./ADR-SOC-0006-media-transcode-and-storage.md) | Media transcode + storage — ImageMagick 7.1 LTS for image variants; ffmpeg 7.x LTS for HLS video; S3 with per-tenant prefix isolation + KMS SSE; CDN tier (Cloudflare R2); per-pack data-residency | Accepted | 2026-05-17 | derived gap from PRD §"Performance" + threat-model T-E-05 |

## Authoring conventions

- ADR ID format: `ADR-SOC-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs (e.g., `ADR-MSGR-0004` referenced from `ADR-SOC-0004` as paired federation posture). Cross-µservice citations are encouraged where the decisions are genuinely paired.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-SOC-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 (Ranking-model openness: closed-weights vs published-weights) | open | ADR-SOC successor-IP; depends on EU AI Act notified-body engagement |
| #2 (Federation: AT Protocol in addition to ActivityPub) | open | ADR-SOC successor-IP after federation minimum-shippable-tier (ActivityPub) ships |
| #3 (Ads-substrate-stub fate: keep interface-only-pending-impl vs delete vs activate) | open | ADR-SOC successor-IP after M03 |
| #4 (Self-observability emission posture) | Resolved | per-pack emission with per-tenant tags — resolved in IP-014 |
| #5 (Verified-handle uniqueness scope) | open | ADR-SOC successor-IP |

Future ADRs land here with sequential `ADR-SOC-XXXX` IDs.
