---
doc_class: ADRIndex
microservice: network
date: 2026-05-17
owner_team: axis-network + council-privacy + ops-security + ops-compliance
doc_status: published
---

# network µservice — service-scoped ADRs

This directory holds ADRs that govern the `network` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/network/PRD.md`, in `microservices/network/PHASE-01-NETWORK-FOUNDATION.md`, or in a policy / runbook / threat-model / DPIA artifact under `microservices/network/`.

The `network` µservice is the Professional-tier social-graph + identity µservice per parallel ADR-0126. Several decisions are paired with sibling µservice ADRs:

- `ADR-NET-0001` (storage) ↔ `ADR-SOC-0002` (sibling follow-graph storage; pattern aligned, consistency differs)
- `ADR-NET-0002` (recommender bounds) ↔ `ADR-SOC-0003` (sibling content-moderation bounds; both EU AI Act high-risk; different Annex III subclause)
- `ADR-NET-0003` (InMail bridge) ↔ `ADR-MSGR-0004` (sibling messenger Professional-tier surface)
- `ADR-NET-0005` (endorsement-chain integrity) ↔ audit-chain µservice ADRs
- `ADR-NET-0006` (profile portability + export) ↔ `ADR-SOC` profile-export pattern

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-NET-0001](./ADR-NET-0001-professional-graph-storage.md) | Professional-graph storage — Postgres adjacency-list primary with stronger consistency for endorsement-chain integrity; graph-database adapter (Dgraph / JanusGraph) future-pluggable | Accepted | 2026-05-17 | derived gap from capacity-model.md + ADR-NET-0005 integrity requirements |
| [ADR-NET-0002](./ADR-NET-0002-recommender-ai-act-eeoc-bounds.md) | Recommender + recruiter-stub + jobs-ranker + endorsement-aggregation bounds — EU AI Act Annex III §4 HIGH-RISK; Arts. 9-15 + 27 + 50 + 72 + 73; GDPR Art. 22 + Art. 25 + Art. 35; EEOC UGESP + Title VII + ADA + ADEA; NYC LL144; CA AB-331; CO SB 24-205; UK Equality Act + ICO ADM | Accepted | 2026-05-17 | PRD Outcome 4 + employment-context regulatory exposure |
| [ADR-NET-0003](./ADR-NET-0003-inmail-bridge-to-messenger.md) | InMail bridge to messenger — Professional-tier-only bridge; never federates to Personal-tier DM; throughput + rate-limit + spam-classifier; paired with messenger ADR-MSGR-0004 pattern | Accepted | 2026-05-17 | PRD FR-14 + InMail rate-budget + PCI-09 invariant |
| [ADR-NET-0004](./ADR-NET-0004-jobs-handoff-to-ats.md) | Jobs-handoff to ATS µservice — clean boundary between network µservice (posting surface) and Tier-G ATS µservice (pipeline management); contract-versioned event handoff with dual-version window | Accepted | 2026-05-17 | PRD FR-34 + Tier-G ATS roadmap |
| [ADR-NET-0005](./ADR-NET-0005-endorsement-chain-integrity.md) | Endorsement-chain integrity — Merkle-style chain via audit-chain µservice; per-endorser Ed25519 signature; revocation tombstone semantics | Accepted | 2026-05-17 | PRD Outcome 4 + audit-chain ADR-0028 pairing |
| [ADR-NET-0006](./ADR-NET-0006-profile-portability-and-export.md) | Profile portability + export — vCard 4.0 (RFC 6350) + JSON Resume + GDPR Art. 20 portable-JSON; per-pack redaction overlay; DSR cascade alignment | Accepted | 2026-05-17 | GDPR Art. 20 + DPDPA 2023 portability requirements |

## Authoring conventions

- ADR ID format: `ADR-NET-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-NNNN` at repo root) and sibling µservice ADRs (e.g., `ADR-SOC-0002` referenced from `ADR-NET-0001` as paired storage pattern). Cross-µservice citations are encouraged where the decisions are genuinely paired.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-NET-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 (Recruiter-stub activation strategy: closed-tenant-set vs open) | open | ADR-NET follow-up after M03; depends on EU AI Act notified-body engagement |
| #2 (Services-marketplace-stub fate: keep stubbed vs activate) | open | ADR-NET follow-up after M04 |
| #3 (Learning-stub fate: keep stubbed vs activate vs separate µservice) | open | ADR-NET follow-up after M05 |
| #4 (Self-observability emission posture) | Resolved | per-pack emission with per-tenant tags — resolved in IP-014 |
| #5 (Verified-handle global uniqueness vs per-tenant) | open | ADR-NET follow-up |
| #6 (Federation: ActivityPub + AT Protocol; Professional-tier-only) | open | ADR-NET follow-up; out-of-scope P01 |
| #7 (Salary-insights data sourcing strategy) | open | ADR-NET follow-up; aggregate-only invariant locked in P01 |

Future ADRs land here with sequential `ADR-NET-XXXX` IDs.
