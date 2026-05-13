---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P06
title: Workspace Axis (14 surfaces — Mail, Calendar, Drive, Meet, Chat, +9 more)
status: stub
purpose: Ship Axis 2 — Workspace / Productivity Suite added 2026-05-09. Google Workspace / Naver Works / Microsoft 365 / AWS Productivity class.
---

# M03-P06 — Workspace Axis 14 Surfaces

## Purpose
Per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §1 (Axis 2, added 2026-05-09) and [`../../../../../docs/SPEC.md`](../../../../../docs/SPEC.md) §4 (14 surface rows).

## Acceptance
- All 14 [`../../../../../docs/SPEC.md`](../../../../../docs/SPEC.md) §4 rows ship at `stable` (preview for translate).
- RFC compliance: 5321 (SMTP), 3501 (IMAP), 8620 (JMAP), 4791 (CalDAV), 6352 (CardDAV).
- Yrs CRDT state-vector ≥ 2-version compatibility on docs.
- Per-object KMS-shred on drive; per-tenant SFU placement on meet.
- OpenAPI sources at `contracts/openapi/workspace/`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Mail + Calendar (SMTP/IMAP/JMAP/CalDAV) | stub | [`IP-001-mail-calendar.md`](IP-001-mail-calendar.md) |
| IP-002 | Docs + Sheets + Slides + Sites (Yrs CRDT) | stub | [`IP-002-docs-sheets-slides-sites.md`](IP-002-docs-sheets-slides-sites.md) |
| IP-003 | Drive + KMS-shred | stub | [`IP-003-drive-kms-shred.md`](IP-003-drive-kms-shred.md) |
| IP-004 | Meet + Chat + Recordings | stub | [`IP-004-meet-chat-recordings.md`](IP-004-meet-chat-recordings.md) |
| IP-005 | Forms + Address-Book + Tasks + Notes + Translate | stub | [`IP-005-forms-address-tasks-notes-translate.md`](IP-005-forms-address-tasks-notes-translate.md) |

## Estimated parallelism
5 agents in parallel; one per IP / surface cluster.

## Symbols-touched
`crates/oya-workspace-{mail,calendar,docs,sheets,slides,drive,meet,chat,forms,sites,tasks,notes,translate,recordings,address-book}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P06 complete: 14 Workspace surfaces stable; RFC compliance verified" -i critical -k "M03,P06,workspace,axis-2,complete"
```
