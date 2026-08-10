---
doc_class: Runbook
title: Import pipeline failure
microservice: notes
severity: "Sev-2 (data-loss risk during import) / Sev-3 (transient failure)"
status: Accepted
owner_team: axis-notes + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/notes/decisions/ADR-NOTES-0006-portable-export-and-import-format.md
  - microservices/notes/failure-modes.md (F-IP-01..F-IP-03)
  - microservices/notes/PRD.md (FR-15)
doc_status: published
---

# Runbook: Import pipeline failure

## When

Triggers:

1. `oya_notes_import_pipeline_failure_total > 0 over 1h` (any source-format pipeline crash).
2. User reports "import stuck" or "import produced wrong notes".
3. Per-format adapter (`-adapter-obsidian`, `-adapter-enex`, `-adapter-apple-notes`, `-adapter-onenote`, `-adapter-notion`, `-adapter-bear`) emits internal error.
4. ENEX with suspected payload triggers sandbox-quarantine.

## Severity Decision Tree

```
Partial state left in user's vault (some notes imported, then crash)?
  YES → Sev-2 (data-loss / consistency risk)
  NO  → Sev-3
```

## Sev-2 — Data-Loss / Consistency Risk

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page axis-notes oncall | observability | t+0 |
| 2 | Identify affected import jobs (job_id + user_id + tenant_id) | oncall | t+15m |
| 3 | For each affected job: query `import_job` table — what checkpoint was reached? | oncall | t+30m |
| 4 | If checkpoint exists: option-A resume from checkpoint; option-B rollback to pre-import state | user-decision via UX |
| 5 | If no checkpoint: rollback partial-state by deleting imported notes (per `import_job.created_note_ids`) | oncall + worker | t+60m |
| 6 | User comms: in-product banner explaining rollback + retry affordance | gateway | t+60m |
| 7 | Audit-chain `ImportJobRolledBack{job_id, user_id, tenant_id, reason}` (Professional-tier) | audit-chain | t+60m |
| 8 | Root-cause analysis | axis-notes + ops-security | t+24h |
| 9 | Post-mortem within 5 business days | axis-notes | t+5d |

## Sev-3 — Transient Failure

| Step | Action | Owner |
|---|---|---|
| 1 | Acknowledge alert; check whether transient or systemic | oncall |
| 2 | Retry import via job-replay (worker resumes from last checkpoint) | worker |
| 3 | If retry succeeds → close incident | oncall |
| 4 | If retry fails twice → escalate to Sev-2 |

## Suspected-Malicious Payload

| Step | Action | Owner |
|---|---|---|
| 1 | Quarantine the import file in `s3://notes-import-quarantine/<job_id>/` | sandbox worker |
| 2 | Page ops-security | observability |
| 3 | Examine payload (sandboxed) for known XSS / RCE patterns | ops-security |
| 4 | If malicious: forensic capture; tenant-admin notification; refuse import | ops-security |
| 5 | If benign (false positive): release; resume import | ops-security |

## Per-Format Failure Patterns

### Obsidian vault

| Pattern | Recovery |
|---|---|
| Vault > 100k notes (oversized) | suggest split-import; chunk by `.obsidian/` workspace tabs |
| Vault contains symbolic-link cycles | adapter detects + breaks; emits warning |
| Vault contains binary files (non-attachment) | adapter ignores; emits warning |
| Vault `[[wikilinks]]` reference unresolved | imported as dangling; UX surfaces at end |

### Evernote ENEX

| Pattern | Recovery |
|---|---|
| ENEX with embedded HTML script tags | sanitised at parse; script tags stripped + warning emitted |
| ENEX with > 10MB inline image | extracted to attachments via drive-µservice; embed-ref created |
| ENEX with non-standard XML extensions | best-effort parse; unknown elements logged |

### Apple Notes archive

| Pattern | Recovery |
|---|---|
| Locked notes (encrypted with Apple-side password) | skipped; UX requests user-supplied key (out-of-scope at minimum-shippable-tier) |
| Notes with hand-drawn ink | rasterised to PNG attachment; reference inserted |
| Notes with table elements | converted to GFM table |

### OneNote .one + .onepkg

| Pattern | Recovery |
|---|---|
| OneNote inking | rasterised to PNG attachment |
| OneNote audio | uploaded to drive-µservice; embed-ref |
| OneNote section/page hierarchy | section → notebook; page → note |

### Notion Markdown export

| Pattern | Recovery |
|---|---|
| Notion database (CSV) | imported as a notebook with one note per row; database properties → frontmatter |
| Notion gallery view | imported as flat notebook; warning emitted |
| Notion synced blocks | first-occurrence kept; subsequent occurrences linked via `[[wikilink]]` |

### Bear .bearbk

| Pattern | Recovery |
|---|---|
| Bear hashtags | preserved as tags |
| Bear nested hashtags | preserved via slash separator `parent/child` |

## Personal-Tier-Specific Note

Per ADR-NOTES-0001, Personal-tier import:

- Server worker decrypts NOTHING; the import is **client-side** (SDK runs the parser locally + encrypts each imported note before submitting ciphertext).
- This runbook covers Professional-tier server-side imports primarily; Personal-tier import failures are SDK-side and surface to the user via SDK error.

## Failure Modes (Generic)

| Failure | Recovery |
|---|---|
| Worker OOM on Enterprise-sized vault | chunk + checkpoint per 1k notes |
| Worker crash mid-import | resume from checkpoint; idempotent inserts via note_id derivation |
| Network timeout during attachment upload to drive | retry with exponential backoff |
| Duplicate note_id collision (rare) | suffix `-imported-<n>` per ADR-NOTES-0006 §9 |

## Metrics

- `oya_notes_import_pipeline_failure_total{source_format, reason}` — alarm at > 0 over 1h.
- `oya_notes_import_pipeline_duration_seconds{source_format}` — duration histogram.
- `oya_notes_import_pipeline_quarantine_total` — payload-safety proxy.
- `oya_notes_import_pipeline_resumed_total` — resilience proxy.

## Pack Overlays

| Pack | Notes |
|---|---|
| pack-eu | GDPR Art. 20 — imports preserve data lineage; audit-chain seal records origin |
| pack-us-healthcare | HIPAA §164.502 — imports of PHI require attestation that source is HIPAA-eligible |
| pack-kr | KR PIPA Art. 17 — cross-tool data flow recorded |

## References

- ADR-NOTES-0006 (portable formats).
- `microservices/notes/failure-modes.md` F-IP-01..F-IP-03.
- `microservices/notes/PRD.md` FR-15.
- `microservices/notes/threat-model.md` T-I-11.
- OWASP ASVS v4 §5.1 (input validation).
