---
doc_class: ADRIndex
microservice: anonymous
date: 2026-05-17
owner_team: axis-anonymous + council-architecture + council-privacy + ops-security + general-counsel
doc_status: published
---

# anonymous µservice — service-scoped ADRs

This directory holds ADRs that govern the `anonymous` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one architecture-level decision whose answer the PRD (`microservices/anonymous/PRD.md`) references but does not formally fix. These are the seven decisions that anchor the µservice's seven privacy invariants (I1–I7) plus the affinity-cluster k-anonymity floor decision (Sweeney 2002).

## Index

| ID | Title | Status | Date | Closes (PRD invariant + FR refs) |
|---|---|---|---|---|
| [ADR-ANON-0001](./ADR-ANON-0001-cryptographic-blinding-protocol.md) | Cryptographic-blinding protocol — BBS+ over BLS12-381 with rust-bls 0.5 (Camenisch-Lysyanskaya commitment) under FIPS 140-3 Level 3 air-gapped HSM ceremony | Accepted | 2026-05-17 | I1; PRD FR-01, FR-02; `policy/affinity-attestation-verification.md` |
| [ADR-ANON-0002](./ADR-ANON-0002-affinity-attestation-verification.md) | Affinity-attestation verification — BBS+ selective-disclosure primary path; OIDC + blinding-proxy fallback for legacy IdPs | Accepted | 2026-05-17 | I2; PRD FR-16, FR-22, FR-23; `policy/affinity-attestation-verification.md` |
| [ADR-ANON-0003](./ADR-ANON-0003-legal-process-disclosure-workflow.md) | Legal-process disclosure workflow — court-order receipt + dual-control + 14-day notice (or gag-order) + audit-chain seal + transparency-report | Accepted | 2026-05-17 | I7; PRD FR-17, FR-18; `policy/legal-process-disclosure.cedar`; `runbooks/legal-process-court-order-receipt.md` |
| [ADR-ANON-0004](./ADR-ANON-0004-retention-and-deletion-policy.md) | Retention + deletion policy — 30-day default, 30/60/90-day tenant-selectable tiers; hard-delete with audit-chain tombstone within p99 ≤ 5s | Accepted | 2026-05-17 | I3; PRD FR-13, FR-15; `slos/hard-delete-propagation-correctness.openslo.yaml` |
| [ADR-ANON-0005](./ADR-ANON-0005-abuse-classifier-bounds.md) | Abuse-classifier bounds — EU AI Act limited-risk (not Annex III); Art. 50 transparency obligations apply; GDPR Art. 22 not triggered (anonymous users) | Accepted | 2026-05-17 | PRD FR-19, FR-20, FR-27; `capabilities/T1-assist.yaml`, `capabilities/T2-auto.yaml` |
| [ADR-ANON-0006](./ADR-ANON-0006-federation-refusal-and-anti-pattern-anchoring.md) | Federation refusal — ActivityPub / AT Proto / Matrix / XMPP REFUSED forever; anchored against Secret / 4chan / Whisper anti-patterns | Accepted | 2026-05-17 | I5; PRD FR-26 |
| [ADR-ANON-0007](./ADR-ANON-0007-affinity-cluster-design.md) | Affinity-cluster design — k=50 geo / k=20 employer / k=10 small-employer fallback; hierarchical region merge (Sweeney 2002 k-anonymity) | Accepted | 2026-05-17 | I2 + I7; PRD FR-16; `runbooks/geo-affinity-cluster-rebalance.md` |

## Authoring conventions

- ADR ID format: `ADR-ANON-XXXX` (4-digit, scope-prefixed) per ADR-0131.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 grouped Positive/Negative/Operational/Regulatory; CRITICAL: every Consequences section addresses how the decision preserves I1–I7), References (named industry source).
- Lifecycle per ADR-0131: `Proposed → Accepted → (Superseded by ADR-ANON-NNNN | Deprecated)`. Never delete; supersede.
- Cross-µservice citations encouraged (e.g., ADR-COMM-0001 inherited for moderation chain-of-responsibility; ADR-MSGR-0002 inherited for MLS DM).

## Invariant traceability matrix

| Invariant | Closed by ADR | Enforcement mechanism |
|---|---|---|
| I1 (no user_id↔post_id correlation) | ADR-ANON-0001 + ADR-ANON-0006 | cryptographic blinding + federation refusal |
| I2 (affinity-not-identity) | ADR-ANON-0002 + ADR-ANON-0007 | BBS+ selective-disclosure + k-anonymity floor |
| I3 (short retention + hard-delete) | ADR-ANON-0004 | retention worker + tombstone seal SLO |
| I4 (no 3rd-party trackers) | (cross-cutting; enforced by build-time LEAN lane) | dependency lint |
| I5 (federation refused) | ADR-ANON-0006 | structural refusal (no BC; no helm toggle) |
| I6 (E2E DM via MLS) | (inherited from ADR-MSGR-0002) | MLS RFC 9420 |
| I7 (legal-process dual-control + transparency) | ADR-ANON-0003 | Cedar policy fragment + workflow-engine state machine |
