---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: docs
runbook_id: RB-doc-version-restore-corruption
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-docs
severity_applicable: [Sev-1, Sev-2]
related_failure_modes: [FM-03]
doc_status: published
---

# Runbook — Document version restore (corruption / PITR)

## When this runbook fires

- Tenant reports "I can't read my document; it returns an error."
- Postgres version-history integrity scan detects Merkle root mismatch on revert attempt.
- S3 object integrity check flags content-blob hash mismatch.
- Tenant-requested point-in-time-restore (rare; e.g., after mass-erroneous-write).
- Region failover failed and DR region also down (extremely rare).
- Audit-chain seal continuity break detected on integrity scan.

## Symptoms

- Document reads return integrity-mismatch error.
- Tenant cannot open the document.
- Version-history listing shows gaps or version-sha mismatches.
- CRDT op-log replay produces a state that diverges from the persisted snapshot.

## Probable causes

1. Disk-level corruption (rare; underlying block-storage).
2. Logical corruption (failed migration, mis-applied compaction, mis-applied tenant-DEK rotation).
3. Partial-CRDT-compaction state (worker crashed mid-compaction; per ADR-DOCS-0001 compaction).
4. Region outage during write; partial seal write.
5. Malicious admin action (mass-overwrite past 2-person-rule bypass — should be impossible per `policy/editor-isolation.md` Invariant 7).

## Triage (within 30 min)

1. Acknowledge page; declare Sev-1 if multi-tenant or Sev-2 if single-tenant.
2. Activate incident channel.
3. Determine scope:
   - Whole pack? Single tenant? Single document? Single version?
4. Identify last-known-good version snapshot:
   ```bash
   oya docs version list --tenant <tenant> --document <doc-id> --status verified
   ```
5. Check CRDT op-log retention: how far back can we replay?
   ```bash
   kubectl exec -n docs postgres-primary -- psql -c "SELECT version_sha, op_log_compacted_at FROM document_versions WHERE document_id = '<d>' ORDER BY sealed_at DESC LIMIT 10;"
   ```
6. Notify council-privacy if any data-loss expected (regulator notification timeline may engage).

## Mitigation steps

### Step 1 — Identify restore point

Determine RPO target:
- Default: most recent verified version snapshot + CRDT op-log replay up to crash-point.
- Tenant-requested: specific version_sha.

### Step 2 — Approve via 2-person rule

```bash
oya docs restore approve --tenant <t> --document <d> --to-version-sha <sha> \
  --approver-1 <ops-sre-id> --approver-2 <ops-security-id> \
  --audit-reason "RB-doc-version-restore-corruption-<unique-id>"
```

(Audit-chain emit; OpenBao JIT elevation required.)

### Step 3 — Replay CRDT op stream

```bash
oya docs collab replay --tenant <t> --document <d> \
  --from-version-sha <baseline-sha> --to-version-sha <target-sha>
```

This replays the persisted op-log through the pinned Loro version into a fresh snapshot, then compares against the previously emitted projection (AC-02 byte-equality invariant).

### Step 4 — Restore content blobs from S3 versioning / Object Lock

If content blob corrupted:
```bash
oya docs s3-restore --tenant <t> --document <d> --version-sha <sha>
```

S3 Object Lock retains prior versions for legal-hold blobs; non-held blobs retained per lifecycle policy.

### Step 5 — Validate restore

```bash
oya docs restore validate --tenant <t> --document <d> --to-version-sha <sha> \
  --checks "audit-chain-continuity,version-merkle,crdt-byte-equality,per-block-acl"
```

Validation includes:
- Audit-chain seal continuity (Merkle root matches expected).
- Version Merkle chain unbroken.
- CRDT op-replay byte-equality (AC-02 invariant).
- Per-tenant RLS still active.
- Per-block ACL still applied.
- Tenant-DEK access still works.

### Step 6 — Cut over (with tenant notification)

```bash
oya docs restore cutover --tenant <t> --document <d> --target-version-sha <sha> \
  --notify-tenant --audit-reason "RB-doc-version-restore-corruption-<id>"
```

WebSocket gateway expires affected leases; clients reconnect to latest version.

### Step 7 — Replay subsequent ops (if recoverable + intended)

If the CRDT op log covers a post-restore-point window AND the tenant wants those edits re-applied:

```bash
oya docs collab replay --tenant <t> --document <d> --from-version-sha <restored> --to head --apply-to-current
```

Replay emits audit-chain seal for each replayed op (idempotent on op-id).

### Step 8 — Notify regulator if data-loss scope crossed threshold

Per `incident-response.md` regulator-notification timelines:
- GDPR Art. 33: 72h notification.
- KR PIPA: 24h + 72h.
- HIPAA: 60d.

## Recovery validation

| Check | Target |
|---|---|
| Postgres primary up + replicas synced | yes |
| Audit-chain seal continuity | unbroken |
| Version Merkle chain | unbroken |
| CRDT byte-equality (AC-02) | green |
| Per-block ACL applied | yes |
| Tenant smoke-test passes | yes |
| RLS active | yes |
| Tenant-DEK rotation status | unchanged |

## Post-incident review

- What caused the corruption?
- Was the version-Merkle integrity verification working?
- Did 2-person-rule prevent unauthorised restore?
- Update threat-model.md if a new corruption vector discovered.
- Update CRDT compaction cadence if op-log was insufficient.

## Drills

- Quarterly: simulated version corruption + PITR drill in staging.
- Annual: full cross-region failover-then-restore drill.

## References

- `failure-modes.md` FM-03.
- `multi-region.md`.
- `incident-response.md`.
- ADR-DOCS-0001 (Loro CRDT; op-log replay).
- ADR-0028 (Bominal): audit-chain Merkle.
- Postgres PITR documentation.
- Patroni HA documentation.
- S3 Object Lock documentation.
