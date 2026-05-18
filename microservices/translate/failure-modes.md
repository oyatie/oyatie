---
doc_class: FailureModes
title: Failure Modes catalog (FMEA)
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-translate
related_adrs: [ADR-0139, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0004, ADR-TRANSLATE-0005, ADR-TRANSLATE-0006]
related_artifacts:
  - microservices/translate/threat-model.md
  - microservices/translate/capacity-model.md
  - microservices/translate/multi-region.md
  - microservices/translate/incident-response.md
  - microservices/translate/runbooks/
review_cadence: quarterly + post-incident
doc_status: published
---

# Failure Modes (FMEA) — translate µservice

## Format

Each Failure Mode = (id, summary, severity, detection signal, runbook, mitigation, recovery RTO).

## FM Catalog

### Translate-router + engine adapters

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-01 | DeepL upstream 5xx surge | Sev-2 | `oya_translate_engine_error_rate{vendor="deepl"}` > 5 % rolling 5 m | `runbooks/mt-engine-degraded-shed.md` | ≤ 60 s (router demote) |
| FM-02 | Anthropic rate-limit (HTTP 429) sustained | Sev-3 | `oya_translate_engine_429_total{vendor="anthropic"}` > tenant quota | `runbooks/mt-engine-degraded-shed.md` | ≤ 30 s (router shed) |
| FM-03 | Google Cloud Translation EU-region outage | Sev-2 | `oya_translate_engine_availability_rolling_15m{vendor="google",region="europe-west3"}` < 99 % | `runbooks/mt-engine-degraded-shed.md` | ≤ 90 s (router failover to alternate EU engine) |
| FM-04 | All external vendors unavailable simultaneously | Sev-1 | aggregate vendor availability < 50 % | `runbooks/mt-engine-degraded-shed.md` + `incident-response.md` | ≤ 5 min (in-house only fallback) |
| FM-05 | In-house MT (foundry-runtime) cold-start latency spike | Sev-2 | `oya_translate_in_house_p99` > 1 s sustained 5 m | `runbooks/mt-engine-degraded-shed.md` | ≤ 60 s (scale up foundry-runtime) |
| FM-06 | Engine response shape anomaly (vendor format change) | Sev-2 | `oya_translate_response_shape_anomaly_total` > 0 | `runbooks/mt-engine-degraded-shed.md` + adapter version pin | ≤ 30 min (pin previous adapter version) |
| FM-07 | Router decision latency > p99 budget (5 ms) | Sev-3 | `oya_translate_router_decision_p99_ms` > 5 | profile router; capacity scale | ≤ 60 s |
| FM-08 | Per-tenant token-bucket exhausted (cascade) | Sev-3 | `oya_translate_token_bucket_denials_total` > tenant alert threshold | `runbooks/mt-engine-degraded-shed.md` §"per-tenant cascade" | ≤ 60 s |

### Translation Memory

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-10 | Postgres TM table corruption (block-checksum mismatch) | Sev-1 | Postgres logs `invalid page header` | `runbooks/tm-corruption-restore.md` | ≤ 60 min (PITR restore) |
| FM-11 | Meilisearch TM index corruption (segment lookup fails) | Sev-2 | `oya_translate_tm_leverage_error_rate` > 1 % | `runbooks/tm-corruption-restore.md` §"meilisearch reindex" | ≤ 30 min (re-index from Postgres) |
| FM-12 | TM leverage match latency > p99 (80 ms) | Sev-3 | `oya_translate_tm_leverage_p99_ms` > 80 | scale Meilisearch; per-tenant cache warm | ≤ 5 min |
| FM-13 | Cross-tenant TM leverage event (RLS violation) | Sev-1 | `oya_translate_tm_cross_tenant_match_total` > 0 (canary) | immediate halt + `incident-response.md` §"tenant-isolation breach" | ≤ 0 (block at policy layer) |
| FM-14 | TM bulk-export failure | Sev-3 | bulk-job stuck > 5 m | `runbooks/tm-corruption-restore.md` §"export retry" | ≤ 15 min |

### Quality Estimation + Language Detection

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-20 | QE model regression (verdict-correctness drops) | Sev-2 | `oya_translate_qe_eval_pass_rate` < 0.99 vs golden set | `runbooks/quality-estimation-rollback.md` | ≤ 30 min (rollback to previous model version) |
| FM-21 | QE score skew (model over-reports quality) | Sev-2 | offline regression detected by golden eval-set | `runbooks/quality-estimation-rollback.md` | ≤ 30 min |
| FM-22 | LangDetect model returns wrong language family | Sev-3 | `oya_translate_langdetect_eval_pass_rate` < 0.95 | `runbooks/quality-estimation-rollback.md` §"langdetect rollback" | ≤ 30 min |
| FM-23 | QE EU AI Act disclosure suppression (event not emitted) | Sev-2 | `oya_translate_eu_ai_act_disclosure_emit_ratio` < 1.0 | `incident-response.md` + per-tenant notify | ≤ 60 min |

### Real-Time Stream

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-30 | Caption-stream chunk latency exceeds p99 budget (400 ms) | Sev-2 | `oya_translate_stream_chunk_p99_ms` > 400 | `runbooks/real-time-caption-stream-stall.md` | ≤ 60 s (scale stream workers) |
| FM-31 | Stream session drops (WS disconnect cascade) | Sev-2 | `oya_translate_stream_session_drop_total` > baseline | `runbooks/real-time-caption-stream-stall.md` §"WS recovery" | ≤ 30 s (auto-reconnect with replay) |
| FM-32 | Correction-replay queue grows unbounded | Sev-2 | `oya_translate_stream_replay_queue_depth` > 100 per session | `runbooks/real-time-caption-stream-stall.md` §"replay drain" | ≤ 60 s |
| FM-33 | STT source (meet) drops audio chunks | Sev-3 (translate; meet owns root cause) | upstream meet signal | meet runbook + translate continues with last good source | per meet |

### Document Translation

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-40 | Pandoc parse error on incoming DOCX | Sev-3 | `oya_translate_doc_parse_error_total{format="docx"}` > 0 | `runbooks/document-round-trip-corruption.md` | ≤ 15 min (per-doc) |
| FM-41 | LibreOffice round-trip drops formatting (fidelity regression) | Sev-3 | `oya_translate_doc_fidelity_score` < tier-bound | `runbooks/document-round-trip-corruption.md` | ≤ 30 min |
| FM-42 | gVisor sandbox process crash (malicious doc) | Sev-2 | `oya_translate_doc_sandbox_crash_total` > 0 | `runbooks/document-round-trip-corruption.md` §"sandbox crash" + threat-model T-06 | ≤ 5 min (quarantine doc + replay sandbox) |
| FM-43 | gVisor seccomp violation (sandbox attempted forbidden syscall) | Sev-1 | `oya_translate_doc_seccomp_violation_total` > 0 | `incident-response.md` + adapter pin | ≤ 5 min |
| FM-44 | Doc translate latency exceeds 10-page p95 (8 s) | Sev-3 | `oya_translate_doc_10page_p95_s` > 8 | scale doc workers | ≤ 5 min |
| FM-45 | Document-output S3 write failure | Sev-2 | `oya_translate_doc_s3_write_error_total` > 0 | check OCI Object Storage; retry; circuit-break | ≤ 30 min |

### Bulk Translate

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-50 | Bulk-job queue depth grows beyond 5× baseline | Sev-3 | `oya_translate_bulk_queue_depth` > 5× | scale workers; capacity-model.md | ≤ 10 min |
| FM-51 | Bulk-job per-chunk failure cascade (XLIFF malformed) | Sev-3 | `oya_translate_bulk_chunk_error_rate` > 5 % | `runbooks/document-round-trip-corruption.md` | ≤ 15 min |
| FM-52 | Bulk-job 10 k-segment XLIFF p95 > 60 s | Sev-3 | `oya_translate_bulk_10k_xliff_p95_s` > 60 | scale workers | ≤ 10 min |
| FM-53 | Bulk-job S3 storage quota exhausted | Sev-2 | `oya_translate_bulk_s3_quota_remaining_pct` < 5 % | extend OCI bucket quota | ≤ 30 min |

### Termbase + Glossary

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-60 | Termbase entry conflict (two terms map to incompatible target) | Sev-3 | `oya_translate_termbase_conflict_total` > 0 | `runbooks/glossary-conflict-resolution.md` | ≤ 30 min (human resolve) |
| FM-61 | TBX import schema validation fail | Sev-4 | `oya_translate_termbase_import_error_total` | reject + tenant notify | per-import |
| FM-62 | Termbase enforcement bypass (MT output ignores glossary) | Sev-3 | `oya_translate_termbase_enforce_miss_rate` > 5 % | engine prompt rewrite + retry; `runbooks/glossary-conflict-resolution.md` | ≤ 60 min |

### Data Residency

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-70 | Sovereign tenant content routed to non-resident engine (R-02 realised) | **Sev-1 (P0)** | `oya_translate_residency_violation_total` > 0 (canary; ANY value triggers) | `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` | ≤ 0 (block at decide); if observed: ≤ 5 min halt all egress |
| FM-71 | Per-pack engine whitelist drift (mismatched with `policy/data-residency.md`) | Sev-1 | `oya_translate_residency_whitelist_hash` ≠ canonical | auto-rollback Helm + 2-person reconfigure | ≤ 15 min |
| FM-72 | Cross-pack TM leverage (Tenant A pack-kr query matches Tenant B pack-eu TM unit) | Sev-1 | `oya_translate_tm_cross_pack_match_total` > 0 (canary) | `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` | ≤ 0 (block at Cedar policy) |
| FM-73 | CN-stub pack accidentally activated (external vendor enabled) | Sev-1 | `oya_translate_external_vendor_in_pack_cn_total` > 0 | immediate halt; rollback overlay | ≤ 5 min |

### Credentials + Cross-cutting

| # | Summary | Severity | Detection signal | Runbook | RTO |
|---|---|---|---|---|---|
| FM-80 | Vendor credential resolution failure (OpenBao agent down) | Sev-2 | `oya_translate_credential_resolve_error_rate` > 5 % | check OpenBao agent; `cloud-secrets` µservice runbook | ≤ 10 min |
| FM-81 | Vendor credential leak detected (logs/error/git) | Sev-1 | `oya_translate_credential_leak_canary_total` > 0 | `incident-response.md` + emergency rotation per cloud-secrets runbook | ≤ 5 min (rotate) |
| FM-82 | Audit-chain emission failure (NATS unreachable) | Sev-2 | `oya_translate_audit_emit_error_rate` > 1 % | check audit-chain NATS + replay buffer | ≤ 10 min |
| FM-83 | Mimir scrape failure (translate self-SLI down) | Sev-3 | absent `oya_translate_*` metrics > 5 m | check observability ingest; restart OTel collector | ≤ 15 min |
| FM-84 | Ed25519 envelope sign failure (KMS unreachable) | Sev-2 | `oya_translate_envelope_sign_error_total` > 0 | check KMS; failover | ≤ 10 min |

## Cross-Failure Recovery

- Router decision is stateless; restart-safe.
- TM + termbase reads are RLS-isolated; per-pack failover safe.
- Bulk-jobs persisted in Redis + S3; replay on worker restart.
- Stream sessions reconnect with last-correction replay per ADR-TRANSLATE-0006.

## Verification

- `tests/integration/chaos/` directory contains targeted fault-injection per FM.
- Quarterly chaos drill exercises FM-04 (all vendors out) + FM-13 (cross-tenant) + FM-70 (residency violation; HARD blocker).

## References

- `microservices/translate/threat-model.md`.
- `microservices/translate/multi-region.md`.
- `microservices/translate/capacity-model.md`.
- `microservices/translate/incident-response.md`.
- `microservices/translate/runbooks/` (all 7).
- ADR-TRANSLATE-0001 (engine fallback).
- ADR-TRANSLATE-0004 (residency).
- ADR-0139 (SLO-gated promotion).
