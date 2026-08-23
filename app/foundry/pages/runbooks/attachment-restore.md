---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: docs
runbook_id: RB-attachment-restore
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-docs
severity_applicable: [Sev-2, Sev-1]
related_failure_modes: [FM-05]
doc_status: published
---

# Runbook — Attachment restore (S3 object recovery)

## When this runbook fires

- `docs_attachment_integrity_failure_total > 0`.
- Tenant reports "I can't download my attachment; the link returns 404 or 5xx."
- S3-side delete event detected on a held attachment (should be Object-Lock-blocked).
- Per-doc attachment count drops unexpectedly (compared to last verified backup).

## Symptoms

- Attachment download returns 404 / 500.
- Per-doc attachment listing shows orphaned references (DB rows without S3 objects).
- ClamAV / OPSWAT scan history shows attachments that were green now flagged corrupted.

## Probable causes

1. S3 object integrity drift (rare; bit-rot or block-storage corruption).
2. Accidental delete by tenant operator (non-held; held should be Object-Lock-blocked).
3. S3 region failover incomplete; attachment present in source region but not in DR.
4. Tenant-initiated bulk-delete that included attachments not properly scoped.
5. Forensic-related quarantine (post-malware-scan disposition).

## Triage (within 15 min)

1. Acknowledge page; declare Sev-2 (or Sev-1 if held attachment lost).
2. Identify affected attachments:
   ```bash
   oya docs attachment list --tenant <t> --document <d> --status missing
   ```
3. Check S3 version history for the affected object:
   ```bash
   aws s3api list-object-versions --bucket <pack-bucket> --prefix tenants/<tenant>/docs/<doc-id>/attachments/<attachment-id>
   ```
4. Verify Object Lock status for held attachments:
   ```bash
   aws s3api get-object-retention --bucket <pack-bucket> --key tenants/<tenant>/docs/<doc-id>/attachments/<attachment-id>
   ```
5. Notify council-privacy if legal-hold attachment was lost (data-loss notification timeline engages).

## Mitigation steps

### Step 1 — Restore from S3 version history (if available)

```bash
oya docs attachment restore --tenant <t> --document <d> --attachment <a> --version-id <s3-version-id>
```

This restores the prior S3 version, validates its integrity hash matches the Postgres-recorded hash, and emits an audit-chain seal for the restore.

### Step 2 — If version exhausted: restore from cold-tier backup

```bash
oya docs attachment cold-restore --tenant <t> --document <d> --attachment <a> --as-of "<iso-timestamp>"
```

Cold-tier backup retention: 30d hot + 12mo cold per `multi-region.md`; pack-us-healthcare 6y.

### Step 3 — Verify integrity

```bash
oya docs attachment verify --tenant <t> --document <d> --attachment <a> --checks "integrity-hash,clamav,acl"
```

Validation:
- SHA-256 matches Postgres-recorded hash.
- ClamAV / OPSWAT scan green.
- Per-block ACL applied to attachment reference.

### Step 4 — Re-scan attachment

If the original scan was green but restored attachment was corrupted, re-scan to confirm restored object is clean:
```bash
oya docs attachment rescan --tenant <t> --document <d> --attachment <a>
```

### Step 5 — Tenant notification

```bash
oya docs notify tenant --tenant <t> --kind attachment_restored --document <d> --attachment <a>
```

### Step 6 — If attachment-list mismatch is broader: re-enumerate

```bash
oya docs attachment reconcile --tenant <t> --document <d>
```

Compare Postgres-recorded attachments vs S3 prefix listing; emit reconciliation report.

### Step 7 — If legal-hold attachment was lost: regulator notification

Per `incident-response.md`. Held attachments must NOT be lost (Object Lock blocks); if lost, this is a critical compliance gap requiring root-cause + remediation evidence to auditors.

## Recovery validation

| Check | Target |
|---|---|
| Restored attachment SHA-256 matches Postgres record | yes |
| ClamAV / OPSWAT scan green | yes |
| Per-block ACL applied | yes |
| Audit-chain seal emitted for restore | yes |
| Tenant can download | yes |

## Post-incident review

- How did the attachment go missing?
- Was Object Lock properly applied for held attachments?
- Was S3 version retention sufficient?
- Update threat-model.md FM-05 mitigation if needed.
- If accidental tenant-delete: review delete-confirmation UI for held attachments.

## Drills

- Quarterly: simulated attachment corruption + restore drill in staging.
- Annual: cross-region attachment failover drill.

## References

- `failure-modes.md` FM-05.
- `multi-region.md`.
- `incident-response.md`.
- ADR-DOCS-0004 (per-block ACL; attachment-block linkage).
- ADR-0028 (audit-chain).
- S3 Object Lock + Versioning documentation.
- ClamAV documentation; OPSWAT MetaDefender documentation (pack-us-healthcare).
