---
doc_class: Runbook
title: Response-store corruption (Postgres + Citus + audit-chain integrity)
microservice: forms
severity: "Sev-1"
status: Accepted
owner_team: ops-sre-reliability + axis-forms + axis-foundry-evidence + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/forms/threat-model.md §"T-T-04" + §"T-I-04"
  - microservices/forms/failure-modes.md FM-03
  - microservices/forms/backfill-replay.md
doc_status: published
---

# Runbook: Response-store corruption (Sev-1)

## Purpose

Response-store corruption (Postgres row-level / Citus shard / audit-chain seal mismatch) is a P0 incident: tenant data integrity is the product's load-bearing claim. This runbook contains, diagnoses, recovers, and re-seals.

## Trigger

ONE of:

1. **`oya_forms_response_chain_integrity_failed_total > 0`** — Ed25519 seal mismatch on replay.
2. **Postgres reports row-level corruption** (e.g., `pg_amcheck` failure; index corruption; toast tear).
3. **Citus shard checksum mismatch** between primary + replica.
4. **Tenant reports**: response that "definitely existed" missing from response list.
5. **Backup restore validation fails**: restored row-count diverges from pre-failure manifest.

## Severity

Sev-1. Tenant data integrity is the product's brand promise.

## Impact

- Tenant cannot trust response export (regulatory audit risk for GDPR / HIPAA / PIPA).
- DSR cascade may be incomplete.
- Audit-chain seal chain broken (regulatory finding).
- Cross-µservice consumers (sheets-bridge, workflow-trigger) may receive divergent state.

## Pre-checks

1. Determine scope: single row / single shard / pack-wide?
2. `kubectl -n forms exec postgres-primary -- pg_amcheck -d forms` — index integrity.
3. `kubectl -n forms exec postgres-primary -- psql -d forms -c "SELECT * FROM forms_audit_chain_verify();"` — chain integrity.
4. Last clean backup timestamp: `cargo run -p oya-dev-cli -- forms backup-status --pack <pack>`.
5. Determine root cause: hardware (rare on OCI) / Postgres bug / application bug / malicious tamper?

## Recovery Path A — Single-row corruption (no chain breakage)

Cause: isolated row toast tear OR application-bug write.

| Step | Action |
|---|---|
| 1 | Identify the row: `SELECT * FROM forms_responses WHERE response_id = '<id>'`. |
| 2 | Compare against audit-chain seal: `forms_audit_chain_verify_row('<id>')`. |
| 3 | If row matches seal: row is canonical; suspect downstream consumer drift. |
| 4 | If row diverges from seal: restore from last clean backup OR audit-chain replay. |
| 5 | Re-seal restored row. |
| 6 | Tenant notification per regulatory: GDPR Art. 33 may apply. |

## Recovery Path B — Single-shard corruption

| Step | Action |
|---|---|
| 1 | Drain the shard: `cargo run -p oya-dev-cli -- forms drain-shard --shard <n>`. |
| 2 | Promote shard replica to primary. |
| 3 | Restore the corrupted shard from latest WAL ship + last clean backup. |
| 4 | Replay audit-chain to verify integrity. |
| 5 | Reattach shard. |
| 6 | Tenant notification per pack regulatory (PIPC / EU DPA / ANPD as applicable). |

## Recovery Path C — Pack-wide corruption / chain breakage

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage all relevant teams + council-privacy + council-legal-compliance. |
| 2 | Quarantine pack: stop writes; reads served from read-only replica. |
| 3 | Identify break point in audit-chain: `forms_audit_chain_find_break --pack <pack>`. |
| 4 | Restore from last clean backup pre-break. |
| 5 | Replay submissions from Kafka submission-event topic (28d retention) post-break. |
| 6 | Per-submission audit-chain re-seal; new chain root sealed by signing-authority. |
| 7 | Validate tenant analytics + DSR cascade + workflow-trigger consistency. |
| 8 | Per-pack regulatory notification (GDPR Art. 33 ≤ 72h; HIPAA §164.404 ≤ 60d; PIPA Art. 34 ≤ 72h). |
| 9 | Tenant comms with timeline + remediation. |
| 10 | Public postmortem within 5 business days. |

## Recovery Path D — Malicious tamper suspected

| Step | Action |
|---|---|
| 1 | Engage ops-security; preserve evidence. |
| 2 | Quarantine pack; lock all credentials. |
| 3 | Forensic snapshot of Postgres + Valkey + audit-chain WAL. |
| 4 | Identify tamper vector: insider / external / supply-chain? |
| 5 | Per-pack regulatory notification (criminal complaint where applicable). |
| 6 | Restore as Path C. |
| 7 | Post-incident: ADR for control gap; pen-test successor-IP. |

## Verification

After recovery:
- `forms_audit_chain_verify()` reports clean across full chain.
- `pg_amcheck` reports clean indexes.
- Row-count matches pre-incident manifest (modulo legitimate writes during incident).
- Tenant export passes integrity check.
- DSR cascade resumes successfully.

## Post-incident updates

- Postmortem within 5 business days; public if Sev-1.
- Backup cadence review (increase if RPO too high).
- Audit-chain key rotation if compromise suspected.
- Per-pack DPO successor-IP.

## References

- `threat-model.md` T-T-04, T-I-04.
- `failure-modes.md` FM-03.
- `backfill-replay.md` §"Audit-chain integrity check".
- ADR-0028 audit-chain Ed25519 seal.
- ADR-FORMS-0003 PII column encryption.
- GDPR Art. 33; HIPAA §164.404; KR PIPA Art. 34; LGPD Art. 48.
- Postgres documentation — `postgresql.org/docs/16/wal.html`.
- Citus documentation — `docs.citusdata.com/`.
