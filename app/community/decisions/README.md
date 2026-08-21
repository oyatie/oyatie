---
doc_class: ADRIndex
microservice: community
date: 2026-05-17
owner_team: axis-community + council-architecture + council-privacy
doc_status: published
---

# community µservice — service-scoped ADRs

This directory holds ADRs that govern the `community` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `app/community/PRD.md`, in `app/community/PHASE-01-COMMUNITY-SUBSTRATE.md`, or in an IP / policy / threat-model artifact under `app/community/`. The PRD itself records three out-of-scope deferrals (live-stream media, AI answer synthesis, federation) which are intentionally **not** ADR-closable at M02 — they are reserved for the milestones noted in `PRD.md#Deferrals`.

The ADRs in this directory close five **architecture-level** decisions whose answers the PRD references in passing but does not formally fix: moderation pipeline composition, voting algorithm, KB versioning model, search backend selection, and discussion threading shape. These are the decisions the community substrate cannot ship without.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-COMM-0001](./ADR-COMM-0001-moderation-policy-pipeline-architecture.md) | Moderation policy pipeline — chain-of-responsibility composition (auto-classifier → human queue → appeal) with Cedar policy + audit-chain at every hop | Accepted | 2026-05-17 | PRD §"Security" (moderation flow) + IP-007 + IP-010 |
| [ADR-COMM-0002](./ADR-COMM-0002-voting-engine-tie-breaking-and-decay.md) | Voting engine ranking — Wilson lower-bound + Reddit-style logarithmic time decay; Hacker News fallback for low-vote regimes | Accepted | 2026-05-17 | PRD FR-05 + IP-006 |
| [ADR-COMM-0003](./ADR-COMM-0003-kb-article-versioning-and-fork-merge.md) | KB article versioning — Wikipedia-style immutable revision history with tenant-scoped editorial review; no branch/PR/merge | Accepted | 2026-05-17 | PRD FR-03 + IP-008 |
| [ADR-COMM-0004](./ADR-COMM-0004-content-search-backend.md) | Content search backend — Meilisearch 0.10.0 LTS primary; Tantivy embedded fallback (aligned with ADR-MSGR-0003) | Accepted | 2026-05-17 | PRD FR-07 + IP-009 (supersedes the Elasticsearch reference in IP-009 phrasing) |
| [ADR-COMM-0005](./ADR-COMM-0005-graph-of-discussions-and-replies.md) | Discussion threading — nested replies with materialised path + depth cap 6 (Reddit-style), with flat-render mode for Stack-Overflow Q&A surface | Accepted | 2026-05-17 | PRD FR-04 + IP-005 |

## Authoring conventions

- ADR ID format: `ADR-COMM-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts grouped Positive/Negative/Operational/Regulatory), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs (e.g., `ADR-MSGR-0003` referenced from `ADR-COMM-0004` as paired backend selection). Cross-µservice citations are encouraged where the decisions are genuinely paired.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-COMM-NNNN | Deprecated)`. Never delete; supersede.

## Open questions scheduled-for-distinct-tracked-work (not ADR-closable at M02)

| PRD Deferral | Status | Notes |
|---|---|---|
| Live-stream / video-post hosting | Deferred to community-media sibling | Out of M02 scope per PRD §"Deferrals"; no ADR until BC scope is fixed |
| AI-generated answer synthesis | Deferred to M03 | Depends on `foundry-runtime` integration; will land as `ADR-COMM-0006-foundry-runtime-ai-answer-synthesis.md` |
| Federated communities (cross-org) | Deferred to M04 | Will pair with `ADR-MSGR-0004` federation posture; placeholder ADR-COMM-0007 reserved |

All five M02-scope open questions are closed by an ADR in this directory. Future ADRs land here with sequential `ADR-COMM-XXXX` IDs.
