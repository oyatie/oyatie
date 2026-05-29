---
doc_class: ADRIndex
microservice: messenger
date: 2026-05-17
owner_team: axis-messenger + council-privacy + ops-security
doc_status: published
---

# messenger µservice — service-scoped ADRs

This directory holds ADRs that govern the `messenger` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/messenger/PRD.md`, in `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md`, or in a policy / runbook / threat-model artifact under `microservices/messenger/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-MSGR-0001](./ADR-MSGR-0001-huddles-placement.md) | Huddles (voice + video signaling) lives as a messenger bounded context, not a sibling µservice | Accepted | 2026-05-17 | PRD Open Question 2 (voice/video signaling placement) |
| [ADR-MSGR-0002](./ADR-MSGR-0002-e2e-personal-dm-key-escrow.md) | E2E key escrow tier-split — Personal-DM has no admin escrow ever; Professional-channel supports tenant-admin escrow under Cedar legal-hold policy | Accepted | 2026-05-17 | PRD Open Question 5 (E2E personal-DM key escrow) |
| [ADR-MSGR-0003](./ADR-MSGR-0003-search-backend-selection.md) | Message search backend — Meilisearch 0.10.0 LTS primary; Tantivy embedded fallback for single-cell deployments | Accepted | 2026-05-17 | PRD Open Question 1 (search backend selection — derived gap from catalog references) |
| [ADR-MSGR-0004](./ADR-MSGR-0004-federation-posture.md) | Federation posture — OFF by default; tenant-opt-in via Cedar policy + admin gate; Matrix Client-Server r0.6+ supported, XMPP refused, Personal-DM tier never federated | Accepted | 2026-05-17 | PRD Open Question 3 (Slack/Teams federation security review owner — extended to full federation posture) |

## Authoring conventions

- ADR ID format: `ADR-MSGR-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs (e.g., `ADR-MAIL-0001` referenced from `ADR-MSGR-0002` as paired privacy posture). Cross-µservice citations are encouraged where the decisions are genuinely paired.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-MSGR-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #4 (self-observability emission posture: one tenant or per-pack) | Resolved | resolved in IP-007 per PRD; per-pack emission with per-tenant tags — no ADR needed |

All five PRD Open Questions are now either closed by an ADR in this directory or resolved in-phase. Future ADRs land here with sequential `ADR-MSGR-XXXX` IDs.
