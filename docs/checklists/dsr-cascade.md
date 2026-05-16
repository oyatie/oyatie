---
doc_status: published
---

# Checklist: DSR Cascade Execution

> **When:** A Data Subject Request (export / delete / restrict) or consent withdrawal is received per [PRIVACY-PROGRAM.md §2.2.9](../PRIVACY-PROGRAM.md).
> **Owner:** `council-privacy` DSR operator.
> **SLA:** 30d preview / 14d stable / 7d GA per [SLO-CATALOG.md §1](../SLO-CATALOG.md).
> **Validator:** DSR queue dashboard + per-request proof-of-erasure record in audit chain.

---

## 0. Intake

1. ☐ **DSR received** via `oya admin privacy dsr submit` OR tenant-side request
2. ☐ **Subject identity verified** per pack identity-provider (e.g. KR 본인확인서비스 / EU eIDAS / US Login.gov)
3. ☐ **Request type classified** — export / delete / restrict / portability / object-to-processing
4. ☐ **Lawful basis check** — confirm the request is honorable (some classes may have legal-obligation retention)
5. ☐ **Audit-chain emission** — DSR-received event recorded
6. ☐ **Tenant-admin notified** per regional pack notification convention

## 1. Cascade scope identification

7. ☐ **Per-axis impact map** generated:
   - SaaS — workflow runs, plugin invocations, tenant API records
   - Workspace — Mail, Calendar, Doc, Drive, Meet recording, Chat, Forms, Sites, Tasks, Notes, Address Book
   - Vertical — per-vertical regulated records
   - Foundry — RAG cache, agent step traces, capability invocation records
   - Cloud — tenant compute, storage objects, network logs
   - Search — per-tenant private index entries; cross-tenant index entries (per consent)
   - Ads + Analytics — impression / click / conversion records keyed to user; analytics warehouse rows
8. ☐ **Per-store list** identified (Postgres / object store / search index / vector index / KMS / audit chain / event topic / Redis / Kafka backlog / cold archive)
9. ☐ **Per-pack residency check** — verify cascade hits all regions where data may have replicated

## 2. Cascade execution

10. ☐ **Authoritative tenant store** — record marked `pending_dsr` immediately; cascade deadline = SLA start
11. ☐ **Search index** — delete all per-tenant private + cross-tenant entries derived from affected records; emit `EVT-SEARCH-DSR-DELETE`
12. ☐ **Ads attribution store** — purge impression/click/conversion records keyed by user; preserve aggregate-only counters per k-anonymity rule
13. ☐ **Analytics warehouse** — purge per-user rows; preserve aggregated facts only after re-aggregation removes affected user
14. ☐ **Agent runtime context** — purge RAG caches, tool-call traces, agent memory keyed to user
15. ☐ **Audit chain** — emit `EVT-DSR-FULFILLED` with cascade proof (chain is append-only; prior records are NOT deleted but pointers annotated)
16. ☐ **Cloud storage** — block-storage shred (cryptographic erasure of DEK) for any per-record encrypted blobs
17. ☐ **Backup tier** — schedule purge from cold archive next backup-rotation cycle (per retention policy)
18. ☐ **Per-pack store-side cascade** — KR-pack 신용정보 cascade if applicable; EU-pack GDPR cascade; US-pack CCPA cascade

## 3. Verification

19. ☐ **Proof-of-erasure record** generated per affected store
20. ☐ **Per-store integrity check** — re-query for affected records returns empty
21. ☐ **Audit-chain verification** — `oya admin privacy dsr verify <request-id>` passes
22. ☐ **Per-DSR audit trail** complete — receipt → verification → cascade-events → proof-of-erasure
23. ☐ **Per-tenant trust-portal** entry updated

## 4. Notification + closure

24. ☐ **Subject notified** per pack notification convention with proof-of-erasure summary
25. ☐ **Tenant-admin notified** of completion
26. ☐ **DSR queue dashboard** entry closed
27. ☐ **Per-quarter DSR stats report** updated

## 5. Anti-patterns

- Skipping per-store integrity check — never
- Not purging RAG caches — RAG retains semantic copies even if record is deleted
- Not running per-pack-specific cascade — KR-pack 신용정보 has separate cascade vs general PIPA
- Hard-deleting audit-chain entries — audit-chain is append-only; emit a deletion-evidence record instead
- Skipping backup-tier purge — backups retain personal data; must be scheduled

## 6. References
- [PRIVACY-PROGRAM.md §2.2.9](../PRIVACY-PROGRAM.md) DSR cascade
- KR PIPA Art 39-7
- GDPR Art 17
- CCPA / CPRA
- ADR-0003 audit chain
- ADR-0008-data-use-boundary
