---
doc_class: ADRIndex
microservice: mail
date: 2026-05-17
owner_team: axis-mail + council-privacy
doc_status: published
---

# mail µservice — service-scoped ADRs

This directory holds ADRs that govern the `mail` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/mail/PRD.md`, in `microservices/mail/PHASE-01-MAIL-DISSOLUTION-FROM-CONNECT.md`, or in a capability / runbook / DPIA artifact under `microservices/mail/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-MAIL-0001](./ADR-MAIL-0001-personal-mail-key-recovery.md) | Personal-pillar mail E2E key recovery — user-held-only default + opt-in Shamir 3-of-5 trustee escrow | Accepted | 2026-05-17 | PRD Open Question 4 (personal-pillar key escrow opt-in for M03) |
| [ADR-MAIL-0002](./ADR-MAIL-0002-backend-tenant-class-workload-policy.md) | Mail-server backend per tenant_class and workload profile — Postfix+Dovecot for high-volume paid workloads; Stalwart for demo_trial and standard paid workloads; both behind oya-mail-* port traits | Accepted | 2026-05-17 | PRD Open Question 4 (gap-fill — Stalwart vs Postfix+Dovecot backend selection) |
| [ADR-MAIL-0003](./ADR-MAIL-0003-sdk-launch-order.md) | SDK launch order — native JMAP for Swift, JMAP-jam wrapper for TypeScript, IMAP4rev2 as fallback after JMAP feature-parity | Accepted | 2026-05-17 | PRD Open Question 1 (JMAP vs IMAP priority) + Open Question 5 (SDK ship sequence) |
| [ADR-MAIL-0004](./ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md) | Spam + phishing + DLP classifier EU AI Act scope — Annex III-exempt by default; tenant-opt-in conformity assessment when scoped to employment / HR mail | Accepted | 2026-05-17 | Derived gap from `capabilities/T1-assist.yaml` `T1-mail-smart-classifier.eu_ai_act_classification` |

## Authoring conventions

- ADR ID format: `ADR-MAIL-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) but the inverse is rare; repo-root ADRs change cross-µservice rules and don't cite single-service ADRs unless they're being explicitly promoted to cross-cutting scope.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-MAIL-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #2 (per-tenant SMTP IP pool sizing + warmup) | Open | targeting ops-deliverability ADR; not blocking M03 |
| #3 (search-index backend: Tantivy vs Elasticsearch) | Open | derived from PHASE-01 IP-009; resolved per `microservices/mail/PHASE-01-MAIL-DISSOLUTION-FROM-CONNECT.md` IP-009 in favour of Tantivy first; documented in IP but not yet promoted to an ADR |
| #6 (mail-to-Workflow extraction default consent posture) | Open | targeting council-privacy + axis-workflow joint ADR |

These remain in `microservices/mail/PRD.md` §"Open Questions"; future ADRs land here with sequential IDs.
