---
doc_class: FailureModes
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-forms
doc_status: published
---

# Forms — Failure Modes & Recovery

This document enumerates known failure modes, their detection, and the runbook that owns recovery. Sev levels follow standard ops conventions.

## Top-level matrix

| ID | Failure | Detection signal | Sev | Recovery runbook |
|---|---|---|---|---|
| FM-01 | Spam flood from botnet | `oya_forms_submission_total{tenant=*}` spike > 10x baseline | 2 | `runbooks/spam-flood-throttle.md` |
| FM-02 | hCaptcha provider outage | `oya_forms_captcha_provider_up == 0` | 2 | `runbooks/captcha-degraded.md` |
| FM-03 | Postgres response-store corruption | Audit-chain replay fails ⇒ `oya_forms_response_chain_integrity_failed_total > 0` | 1 | `runbooks/response-store-corruption.md` |
| FM-04 | Export pipeline backed up | `oya_forms_export_queue_depth > 10k` ≥ 10min | 3 | `runbooks/export-pipeline-failure.md` |
| FM-05 | Embed iframe CSP misfire (parent flagged unsafe-inline) | `oya_forms_embed_csp_violation_total > 0` | 2 | `runbooks/embed-iframe-csp-incident.md` |
| FM-06 | PII leak from response export | `oya_forms_export_pii_unredacted_total > 0` | 1 (P0) | `runbooks/pii-leak-incident-p0.md` |
| FM-07 | AI-form-build emits bad output | `oya_forms_ai_build_schema_invalid_total` rate > 30% | 3 | `runbooks/ai-form-build-rollback.md` |
| FM-08 | Citus shard skew | `oya_forms_citus_shard_size_bytes{shard=*}` max/min > 5 | 3 | Cell migration per ADR-0164 |
| FM-09 | Redis OOM | Redis `used_memory / maxmemory > 0.95` | 2 | Scale Redis cluster |
| FM-10 | Webhook DLQ depth growing | `oya_forms_webhook_dlq_depth > 1k` ≥ 30min | 3 | Tenant notify + replay |
| FM-11 | Bulk-distribute mail throttled by mail µservice | `oya_forms_bulk_distribute_throttle_total > 0` | 3 | Per-tenant back-pressure + queue depth alert |
| FM-12 | E-signature CA outage | `oya_forms_esign_ca_up == 0` | 2 | Tenant notification + provider failover |
| FM-13 | Form-builder WASM bundle corrupted at CDN | `oya_forms_builder_wasm_hash_mismatch_total > 0` | 2 | CDN invalidate + re-publish |
| FM-14 | DSR cascade fails on a response | `oya_forms_dsr_cascade_failure_total > 0` | 2 | DSR retry + tenant notify |
| FM-15 | Cross-pack write attempt | `oya_forms_cross_pack_write_attempt_total > 0` | 1 | Immediate investigation; data-residency probe |

## Cross-failure cascades

- **FM-03 → FM-14**: Postgres corruption can cause DSR cascade to fail. DSR runner is idempotent; replay after restore.
- **FM-09 → FM-01 amplification**: Redis OOM erodes rate-limit; spam flood amplifies. Both runbooks engage simultaneously.
- **FM-02 → FM-01**: captcha outage opens flood; multi-provider fallback bounds blast radius.
- **FM-07 → tenant trust**: AI-form-build emitting bad output erodes T2 acceptance rate; cf. AI-Act Art. 72 post-market monitoring.

## Verification

- Quarterly chaos drill induces FM-03 + FM-09 + FM-15 (each once).
- `oya-forms-failure-mode-coverage` CI lane asserts every FM has a runbook reference (no orphan failures).

## References

- All runbooks under `runbooks/`.
- `threat-model.md`.
- `incident-response.md`.
- Google SRE Workbook chapter 15 (Postmortem culture).
