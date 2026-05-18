---
doc_class: Runbook
title: Mailbox restore (point-in-time recovery from S3 + Postgres WAL)
microservice: mail
severity: "Sev-2 (single-mailbox loss) / Sev-1 (multi-mailbox or tenant-wide)"
status: Accepted
owner_team: axis-mail + ops-sre-reliability + ops-backup-restore
date: 2026-05-17
related_artifacts:
  - microservices/mail/failure-modes.md (FM-MB-01 mailbox data loss, FM-MB-03 retention misfire)
  - microservices/mail/capacity-model.md (§"Mailbox restore time budget")
  - microservices/mail/multi-region.md
  - microservices/mail/contracts/openapi.yaml §"/v1/admin/mailboxes/{id}/restore"
  - ADR-0117 data residency
  - ADR-0133 cross-tenant mail-server pattern
doc_status: published
---

# Runbook: Mailbox restore from backup

## Purpose

Restore a mailbox (or a tenant's full mail surface) to a chosen point-in-time. RPO ≤ 5 min (sync-replicated Postgres WAL + per-region S3 versioning). RTO ≤ 15 min for a single 5 GB mailbox per PRD §"Performance" Mailbox restore.

Three restore classes:
- **Class A — Mailbox PITR**: a specific mailbox restored to a specific timestamp (most common; user accidentally deleted a folder, retention engine misfired, ransomware on user device replicated into IMAP).
- **Class B — Tenant-wide PITR**: rare; tenant-wide ransomware, schema-corruption, retention-engine catastrophic misfire, ops-error.
- **Class C — Cell-wide PITR**: catastrophic; multi-tenant cell needs PITR (last-resort; tenant SCC required when crossing residency).

## Trigger

| Trigger | Class | Severity |
|---|---|---|
| User support ticket: "my Sent folder is gone" | A | Sev-3 |
| Compliance officer: legal-hold scope's underlying mailboxes were tampered with | A | Sev-2 |
| Tenant admin: "all employees' inboxes are wrong since 14:00 UTC" | B | Sev-1 |
| Internal: retention-engine deleted past statutory floor (anti-pattern; should never happen) | A or B | Sev-1 |
| Internal: ransomware indicator on a cell (encrypted-blob hash drift; `oya_mail_mailbox_blob_integrity_violation_total > 0`) | B or C | Sev-1 |

## Pre-checks

| # | Check | Command |
|---|---|---|
| 1 | Identify mailbox + target PITR timestamp + jurisdiction (residency lock) | tenant admin or compliance officer request; verify with audit-chain |
| 2 | Confirm restore is permitted: dual-context invariant — Personal-context restore requires user consent; Professional-context requires tenant admin + compliance officer co-sign per `policy/dual-context-isolation.md` §FM-DCI | per request |
| 3 | Confirm Postgres WAL retention covers the PITR timestamp | `kubectl exec -n mail <pg-primary> -- pg_waldump --start-time="<ts>"` returns non-empty |
| 4 | Confirm S3 versioning has objects at the PITR timestamp | `aws s3api list-object-versions --bucket oya-mail-<pack> --prefix tenants/<t>/mailboxes/<m>/ --query 'Versions[?LastModified <= \`<ts>\`]'` |
| 5 | Confirm any active legal-hold IS preserved across the restore (hold-before-purge invariant from `policy/dual-context-isolation.md` Invariant DCI-04) | `oya-mail-cli legal-hold list --tenant=<t> --scope=mailbox:<m>` returns current holds |
| 6 | Confirm cell capacity for restore workload | `oya_mail_restore_job_in_flight` < 4 (per-cell concurrency limit) |
| 7 | Pack residency: restore region matches origin region (no cross-pack restore without tenant SCC + audit-emit) | per `multi-region.md` |

## Steps — Class A (single mailbox PITR)

| Step | Action | Time |
|---|---|---|
| 1 | Open Sev incident if applicable; assign IC + restore-engineer | ≤ 5 min |
| 2 | Open ChangeSet: `oya vcs claim --agent <id> --intent "mailbox-restore:<t>:<m>:<pitr>" --paths "microservices/mail/evidence/restore/<job-id>/**"` | ≤ 2 min |
| 3 | Confirm pre-checks 1-7 above | ≤ 5 min |
| 4 | Suspend live writes to the target mailbox: `oya-mail-cli mailbox quiesce --mailbox=<m> --reason="<rfc>"`. IMAP/JMAP/SMTP-submission for this mailbox returns `503 try later`; SMTP-inbound queues deliveries with `421 4.7.0 try later` per RFC 5321 §4.5.4. Quiesce TTL: 30 min (auto-resume). | ≤ 1 min |
| 5 | Snapshot current state for forensics (audit-chained): `oya-mail-cli mailbox snapshot --mailbox=<m> --tag=pre-restore-<job-id>`. Snapshot is a Postgres logical dump + S3 versioned objects pinned at NOW; retained 30 days. | ≤ 5 min |
| 6 | Initiate restore: `oya-mail-cli mailbox restore --mailbox=<m> --to-timestamp=<ts> --job-id=<id>`. Pipeline: <br>  a. PITR Postgres WAL replay to a fresh staging Postgres into `mailboxes_restore_<job-id>` schema <br>  b. S3 versioned object fetch at <ts> for every MIME blob <br>  c. DEK envelope decrypted via tenant KMS (audit-emit) <br>  d. Integrity verify: blob SHA-256 matches Postgres metadata <br>  e. Restore validation: `oya-mail-cli mailbox restore-validate --job=<id>` confirms message count, thread topology, folder structure | ≤ 10 min for 5 GB |
| 7 | Swap: `oya-mail-cli mailbox restore-swap --job=<id>`. Pipeline: <br>  a. Atomic transaction: rename `mailboxes` → `mailboxes_replaced_<job-id>`, rename `mailboxes_restore_<job-id>` → `mailboxes` (Postgres ALTER SCHEMA RENAME) <br>  b. Update mailbox metadata pointer to restored Postgres rows <br>  c. S3 already addressable (CAS by hash; restored hashes are the originals) <br>  d. Emit `MailMailboxRestored{mailbox_id, restored_to, restored_at, job_id, approver_ids, scope}` Ed25519-sealed audit event | ≤ 2 min |
| 8 | Resume mailbox: `oya-mail-cli mailbox unquiesce --mailbox=<m>`. IMAP/JMAP/SMTP traffic resumes. | ≤ 1 min |
| 9 | Tenant + user notification: per `policy/dual-context-isolation.md` audit-trail row, restore is audit-chained; user/admin receives notification email with restore-job summary + retention class confirmation. | ≤ 30 min |
| 10 | Decommission `mailboxes_replaced_<job-id>` schema after T+30d (kept for review window). | T+30d worker |
| 11 | `oya vcs done` ChangeSet; evidence at `microservices/mail/evidence/restore/<job-id>.json`. | ≤ 2 min |

## Steps — Class B (tenant-wide PITR)

Same as Class A applied per-mailbox in the tenant's scope, with:

- Pre-step: tenant admin + ops-onboarding co-sign required (4-eyes).
- Parallelism: up to 8 concurrent mailbox restores per cell (capacity per `capacity-model.md`); rest queued.
- Total time: bounded by tenant mailbox count × 10 min / 8-parallelism, plus 30 min overhead for cell quiesce/unquiesce.
- Cross-cell coordination if tenant spans multiple cells (per-tenant sharding key).

## Steps — Class C (cell-wide PITR)

Last-resort; engage ExecSponsor + ops-SRE-IC. Requires:

- All tenant SCCs verified for in-cell restore.
- DR pair available (per `multi-region.md`); failover to DR while origin cell restores.
- Full incident postmortem within 24h.
- Regulatory notification per pack (GDPR Art. 33; KR PIPA Art. 34; HIPAA §164.410 if PHI exposure during restore).

## Hold-before-restore invariant

Per `policy/dual-context-isolation.md` Invariant DCI-04 — if the restored mailbox is under active legal-hold:
- The restore MUST preserve all message IDs present at the hold-engagement timestamp.
- If PITR target is BEFORE hold-engagement, the restore MAY reduce held content; this requires four-eyes compliance officer co-sign (PITR effectively "rolls back" the hold scope).
- If PITR target is AFTER hold-engagement, no scope shift; standard restore.

## Verification

After restore completes:
- Mailbox message count at PITR timestamp matches expected (from pre-incident state per backup metadata).
- IMAP/JMAP/REST returns correct folder + thread topology.
- DKIM + audit-chain seals on every restored message verified by `oya-mail-cli mailbox audit-verify --mailbox=<m>`.
- Tenant retention policy still applies; legal-hold scope preserved.
- `oya_mail_mailbox_restore_p99_seconds` ≤ SLO target.
- User can authenticate + read mailbox (smoke test).
- Audit-chain `MailMailboxRestored` event sealed + visible in tenant audit log.
- IF Class B/C: tenant comms sent; regulator notifications per pack.

## Post-incident updates

- If FM-MB-01 (data loss): root-cause why backup recovery was needed; is the upstream code path safer post-fix?
- If retention engine misfire (FM-MB-03): refine retention-policy worker test coverage; tighten `retention-floor-conformance` lane.
- If ransomware indicator (FM-MB-04): engage ops-security; consider per-tenant immutable-storage tier (S3 Object-Lock or WORM-compliant cold-tier).
- Postmortem within 5 business days (Sev-2) or 24h (Sev-1).
- Backup integrity drill quarterly per `policy/data-residency.md`.

## References

- RFC 5321 §4.5.4 (Retry Strategies; 421 temp-fail)
- ADR-0117 data residency
- ADR-0133 cross-tenant mail-server pattern
- Postgres PITR docs — `https://www.postgresql.org/docs/current/continuous-archiving.html`
- AWS S3 versioning + Object Lock — `https://docs.aws.amazon.com/AmazonS3/latest/userguide/Versioning.html`
- Velero (Kubernetes backup tool) — `https://velero.io`
- GDPR Art. 33 (breach notification); KR PIPA Art. 34; HIPAA §164.410
- `microservices/mail/failure-modes.md` FM-MB-01..04
- `microservices/mail/multi-region.md`
- `microservices/mail/policy/dual-context-isolation.md` Invariant DCI-04
