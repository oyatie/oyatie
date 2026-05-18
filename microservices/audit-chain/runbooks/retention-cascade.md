---
doc_class: Runbook
title: Retention cascade + DSR processing
microservice: audit-chain
severity: Sev-2 (operational) / Sev-1 (mass-delete anomaly)
status: Accepted
owner_team: council-privacy + axis-audit-chain
date: 2026-05-17
related_artifacts:
  - microservices/audit-chain/failure-modes.md (FM-11, FM-12)
  - microservices/audit-chain/policy/data-residency.md §"DSR Cascade"
  - microservices/audit-chain/policy/retention-matrix.yaml
  - microservices/audit-chain/incident-response.md
doc_status: published
---

# Runbook: Retention cascade + DSR processing

## Purpose

Operational procedures for retention-cascade-worker: scheduled retention sweeps, DSR cascade handling, backlog drain, and mass-delete anomaly response.

## Daily Scheduled Retention Sweep

### Cadence

Daily at 02:00 pack-local time (off-peak).

### Procedure

Automated; runs without human intervention. Each run:
1. Reads `policy/retention-matrix.yaml` (versioned in git).
2. For each `(tenant_partition, data_class)`:
   - Identify events older than `retention_window` (per matrix; e.g., HIPAA 6y, KR PIPA 3y).
   - Soft-delete (mark for redaction); preserve Merkle proof.
   - After `grace_window` (30d default), hard-delete payload.
3. Emit `RetentionApplied` event for each operation (sealed in chain).
4. Report summary to `oya_audit_chain_retention_apply_rate` metric.

### Verification

- `oya_audit_chain_retention_apply_rate` within expected baseline + 3σ.
- No anomaly alerts fire.
- Per-tenant retention conformance lane passes.

## Backlog Drain (FM-11) — Procedure

### Trigger

`oya_audit_chain_dsr_backlog_seconds > 30d` for ≥ 1h.

### Procedure

| Step | Action |
|---|---|
| 1 | Declare Sev-2; engage council-privacy + axis-audit-chain SME |
| 2 | Identify backlog cause: <br>  a. retention-cascade-worker under-scaled? Check replica count + CPU; <br>  b. tenant identifier mapping issue? Check DSR runner integration with `tenancy` µservice; <br>  c. Postgres index slow? Check query latency. |
| 3 | Scale up retention-cascade-worker replicas: `kubectl scale deployment retention-cascade-worker-<pack> --replicas=<higher>` |
| 4 | Verify drain progress: `oya_audit_chain_dsr_backlog_seconds` trending down |
| 5 | If tenant SLA already breached (e.g., 30d GDPR SLA elapsed): tenant comms + council-privacy + regulatory notification chain |
| 6 | Backlog drained: confirm `oya_audit_chain_dsr_backlog_seconds < 7d` (within SLA window) |

### Recovery

≤ 24h target.

### Notification

- Tenant: per `incident-response.md` Sev-2 template if SLA at risk.
- Regulator (per pack): if SLA breached.

## DSR Cascade — Procedure

### Trigger

`DataSubjectRequestRaised` event from `tenancy` µservice.

### Procedure (automated; documented for forensic walk-through)

| Step | Automated action |
|---|---|
| 1 | retention-cascade-worker receives event; extracts `(tenant_partition, subject_hash, dsr_id, request_type)` |
| 2 | Query Postgres + S3 index for all events containing `subject_hash` within the affected tenant_partition |
| 3 | For each matching event: <br>  a. mark for soft-redaction (set redaction-token in Postgres index); <br>  b. preserve Merkle leaf hash in chain (Bominal ADR-0028 §"Retention proof"); <br>  c. emit `RetentionApplied{event_id, dsr_id, mode=soft_delete, applied_at}` event (sealed in chain) |
| 4 | After 30d grace (regulator-mandated review window): hard-delete payload (set payload to opaque `<redacted>` marker); emit `RetentionApplied{event_id, dsr_id, mode=redact_payload}` |
| 5 | Send DSR-receipt-confirmation back to `tenancy` for tenant notification (within per-pack SLA: GDPR 30d, KR PIPA 10d, BR LGPD 15d) |

### Limitations

- Events in statutory retention window (HIPAA 6y for PHI; KR-FSS 5y for finance) are marked for redaction-at-retention-expiry rather than immediate; chain remains intact with the redaction-marker.
- Tenant + subject informed via DSR receipt with the locked-until date.

### Verification

- DSR receipt confirms within per-pack SLA.
- `RetentionApplied` event in chain audit-trail.
- Verifying the original event returns `verified: true` with `reason: payload_redacted` (chain integrity preserved; payload no longer present).

## Mass-Delete Anomaly (FM-12) — Procedure

### Trigger

`oya_audit_chain_retention_apply_rate` exceeds expected baseline + 3σ for ≥ 5min.

### Procedure

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage IC (ops-security primary) + council-privacy chair + axis-audit-chain SME + ExecSponsor | ≤ 5 min |
| 2 | HALT retention-cascade-worker: `kubectl scale deployment retention-cascade-worker-<pack> --replicas=0` | ≤ 2 min |
| 3 | Diagnose anomaly source: <br>  a. Did `policy/retention-matrix.yaml` change recently in git? Check signed-commit history + CODEOWNERS log; <br>  b. Was retention-cascade-worker passed an unexpected DSR cascade event? Check DSR audit log; <br>  c. Is there a code bug in the worker? Check recent deploys |
| 4 | If git change is the source: revert offending commit; engage ops-security for forensics (who/why) | ≤ 1h |
| 5 | If DSR-misfire is the source: verify the originating `DataSubjectRequestRaised` event; if forged, treat as breach (T-S-01 in threat-model) | ≤ 1h |
| 6 | If code bug: pin to known-good worker version | ≤ 1h |
| 7 | Restoration: payloads soft-deleted within grace window are recoverable from S3 WORM raw blobs (which are NOT deleted by retention soft-delete; only marked for hard-delete after grace). Restore via: `cargo run -p oya-dev-cli -- audit-chain restore-soft-deleted --pack <pack> --tenant-partition <partition> --since <ts>` (2-person rule) | ≤ 4h |
| 8 | Tenant notification per `incident-response.md` Sev-1 if customer-visible | ≤ 30 min |
| 9 | Postmortem within 5 business days |

### Verification

- Retention-apply rate returns to baseline.
- Restored events verify successfully.
- Postmortem identifies root cause + corrective action.

## References

- `microservices/audit-chain/policy/data-residency.md` §"DSR Cascade".
- `microservices/audit-chain/policy/retention-matrix.yaml`.
- `microservices/audit-chain/failure-modes.md` FM-11 + FM-12.
- `microservices/audit-chain/incident-response.md`.
- Bominal ADR-0028 §"Retention proof" + §"Right-to-erasure with chain preservation".
- GDPR Arts. 17 + recital 65 (retention for legal-claims defence).
- HIPAA §164.316(b)(2); KR PIPA Art. 36; BR LGPD Art. 18(V).
