---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-004
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0028
  - ADR-0263
  - ADR-0276
companion_docs:
  - microservices/feature-flags/backfill-replay.md
  - microservices/feature-flags/runbooks/killswitch-engaged.md
  - microservices/feature-flags/incident-response.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Runbook: Audit Replay

## A. Trigger conditions

- Cell outage caused a gap in the sealed audit event chain.
- Compliance officer requests audit evidence for a specific time range.
- QSA audit requires full flag lifecycle history for a tenant.
- DSAR (Data Subject Access Request) requires portability export of flag evaluation history.
- Regulator (FedRAMP, HIPAA auditor, EU GDPR authority) requests audit trail.
- Audit chain integrity check fails: `oya audit verify` returns non-zero exit code.

## B. Pre-checks (≤5 minutes)

1. Verify the gap window:
   ```bash
   oya audit verify --tenant <tenant_id> \
     --start <start_ts> \
     --end <end_ts>
   # Exit 0: chain intact. Exit 1: gap detected; output shows gap intervals.
   ```
2. Check WAL archive availability (PITR up to 30 days):
   ```bash
   oya postgres wal-archive-status --microservice feature-flags --cell <cell_id>
   ```
3. Confirm requester authorization (Cedar `auditor-scope.cedar`):
   - Compliance officer: `audit_window_valid: true` must be set.
   - QSA: time-bounded access grant from platform admin.
   - Regulator: warrant ID required.
4. Check ClickHouse cold tier for the time range:
   ```bash
   oya clickhouse query "SELECT count(*) FROM feature_flags_audit_events WHERE timestamp BETWEEN '<start>' AND '<end>' AND tenant_id = '<tenant_id>'"
   ```

## C. Procedure

### Step 1 — Verify chain integrity (≤5 minutes)

```bash
oya audit verify \
  --tenant <tenant_id> \
  --start <start_ts> \
  --end <end_ts> \
  --microservice feature-flags
```

If intact (exit 0): proceed to export (Step 3). If gap (exit 1): proceed to WAL replay (Step 2).

### Step 2 — WAL replay to fill gap (≤30 minutes)

```bash
# Identify WAL LSN for the gap start
oya postgres wal-lsn-at-time --cell <cell_id> --time <gap_start_ts>

# Replay WAL from that LSN
oya feature-flags audit-replay \
  --from-lsn <lsn> \
  --to-lsn <end_lsn> \
  --tenant <tenant_id>
```

Replayed events are tagged `replayed: true`; sealed with the same key as the original. Chain re-verification:

```bash
oya audit verify --tenant <tenant_id> --start <start_ts> --end <end_ts>
# Must return exit 0
```

### Step 3 — Export for compliance or DSAR (≤15 minutes)

```bash
# DSAR portability export (per ADR-0276)
oya feature-flags dsar-export \
  --tenant-id <tenant_id> \
  --format json \
  --output /secure-export/<tenant_id>-flags-audit.json

# Encrypt with tenant encryption-key BYOK (ADR-0251 §D-10) or platform DEK
oya encrypt --input /secure-export/<tenant_id>-flags-audit.json \
  --key-ref "${openbao:secret/<tenant_id>/feature-flags/dsar-export-key}"
```

### Step 4 — Deliver to requester via secure channel

- Compliance officer / QSA: deliver via encrypted file share.
- Regulator: per ADR-0251 regulator-access procedure; deliver under warrant number.
- DSAR requester: deliver via tenant GDPR Art. 20 export surface (not directly).

Timing: regulators with 72h notification deadlines (GDPR breach, KR-PIPA) must receive export within the deadline. Check `compliance.md §pack-overlay-roster` for applicable timing.

### Step 5 — Log delivery (≤5 minutes)

```bash
oya audit emit --event-class AuditExportDelivered \
  --tenant <tenant_id> \
  --requester-type <compliance_officer|qsa|regulator|dsar> \
  --warrant-id <warrant_id_if_applicable> \
  --delivery-timestamp $(date -u +%Y-%m-%dT%H:%M:%SZ)
```

## D. Verification

- `oya audit verify` returns exit 0 for the requested time range.
- Export file contains expected event count (cross-check with ClickHouse count from pre-check).
- `AuditExportDelivered` event emitted and sealed.

## E. Rollback

Audit exports are read-only; no rollback needed. If export was delivered in error (wrong tenant):
1. Revoke delivery access.
2. Notify platform security: `axis-governance-security@oyatie.internal`.
3. Log `AuditExportDeliveredInError` event.

## F. Post-incident

- If chain gap was caused by a cell outage: add to post-mortem; audit continuity is a SEV-2 issue.
- If WAL archive was unavailable: check backup retention policy (30 days); extend if regulatory requirements demand longer.

## G. References

- `backfill-replay.md` — detailed backfill procedures.
- `policy/auditor-scope.cedar` — Cedar policy for audit access.
- ADR-0028 — sealed audit chain doctrine.
- ADR-0263 — observability emission contract.
- ADR-0276 — backup portability / DSAR.
