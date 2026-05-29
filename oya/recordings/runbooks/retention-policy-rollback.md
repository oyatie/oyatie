---
doc_class: Runbook
title: Retention policy rollback (purge worker error or misapplied policy)
microservice: recordings
severity: "Sev-1 if data lost; Sev-2 for policy-misapplied recoverable cases"
status: Accepted
owner_team: ops-compliance + axis-recordings + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md
  - microservices/recordings/policy/data-residency.md
doc_status: published
---

# Runbook: Retention policy rollback

## Purpose

Roll back a wrongly-applied retention policy or a botched purge before
irreversible KMS-shred. After KMS-shred, the data is unrecoverable per
ADR-RECORDINGS-0002 + SEC 17a-4 + HIPAA secure-disposal requirements.

## Symptoms

- Tenant reports recordings missing that should have been retained.
- Audit-chain shows `RetentionPolicyApplied` events on rows that should not
  have been purgeable.
- `retention-policy-correctness` SLO breach (load-bearing 100 % invariant).

## Diagnosis

1. Pull the affected `recording_id`s from audit-chain.
2. Confirm whether `KmsShredExecuted` events have fired for the same scope
   (irreversible past this point).
3. Check whether `LegalHoldEngaged` was present at purge-time (must never
   purge against a hold — Sev-1 if so).
4. Check policy state at purge-time: was the wrong `RetentionPolicy.purge_after_epoch_seconds`
   value applied?

## Procedure

### Case A — KMS-shred has NOT yet executed (recoverable)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page ops-compliance + axis-recordings | on-call | immediate |
| 2 | Engage emergency legal-hold over the affected scope per `runbooks/legal-hold-court-order-receipt.md` | ops-compliance | ≤ 5 min |
| 3 | Verify retention worker + KMS-shred worker have aborted in-flight operations against scope | ops-sre | ≤ 5 min |
| 4 | Restore S3 versioned objects per S3 versioning (objects are soft-deleted; recover via `s3 list-object-versions`) | ops-sre | ≤ 30 min |
| 5 | Re-write retention policy row with corrected `purge_after_epoch_seconds` | ops-compliance | ≤ 5 min |
| 6 | Verify content_hash matches pre-incident audit-chain | server | ≤ 5 min |
| 7 | Release emergency legal-hold | ops-compliance | ≤ 5 min |

### Case B — KMS-shred HAS executed (UNRECOVERABLE)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Declare Sev-1; page council-privacy + GTM customer-success | on-call | immediate |
| 2 | Audit-chain dump of every affected row + KMS-shred event | ops-sre | ≤ 30 min |
| 3 | Customer notification per pack breach-notification rules (GDPR Art. 33 72h; HIPAA HITECH 60d; KR PIPA 72h+5d) | council-privacy | ≤ 24h |
| 4 | Post-mortem within 5 business days; root-cause + action items | axis-recordings | ≤ 5 BD |
| 5 | If load-bearing SLO breach: pause all retention purges across all packs until preventive control is shipped | ops-compliance | ≤ 1h |

## Verification

- `oya recordings retention-policy show --recording-id <id>` returns the
  corrected policy.
- Audit-chain has full event trail of rollback + restore.
- Customer signs receipt of recovered recordings (Case A) or breach notice
  (Case B).

## Postmortem Triggers

- Any Case B occurrence (Sev-1 load-bearing breach).
- Any Case A occurrence affecting > 1 % of tenant base.
- Any retention purge that executed against a recording with
  `legal_hold_engaged == true` (load-bearing breach — must never happen).

## Preventive Controls

- Pessimistic Postgres advisory-lock between retention worker + KMS-shred
  worker + legal-hold engagement (per ADR-RECORDINGS-0002).
- Per-tenant retention-purge soft-grace period (7 days) before hard purge.
- `RetentionPurgePending` event emitted 7 days before purge; tenant can
  veto.
- Load-bearing SLO `retention-policy-correctness` + CI lane.

## References

- ADR-RECORDINGS-0002.
- `runbooks/legal-hold-court-order-receipt.md`.
- `slos/retention-policy-correctness.openslo.yaml`.
- GDPR Art. 33/34; HIPAA HITECH 13402; KR PIPA Arts. 34/34-2; DPDPA 2023.
