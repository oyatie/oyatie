---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-recordings
related_adrs: [ADR-0130, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0004]
doc_status: published
---

# Failure Modes & Mitigations: recordings µservice

| FM-ID | Failure mode | Impact | Detection | Mitigation | Runbook |
|---|---|---|---|---|---|
| FM-01 | Postgres primary outage | recording-list + metadata writes block | health-check + replica lag alert | DR pair failover; RTO ≤ 5 min | `runbooks/retention-policy-rollback.md` (related) |
| FM-02 | S3 hot tier outage | playback fails; ingest fails | health-check + S3 4xx/5xx rate | DR-pair S3 endpoint flip; degraded read-only mode from cold tier with 12h restore SLA | `runbooks/playback-cdn-cache-cascade.md` |
| FM-03 | CDN cache cascade collapse | playback origin overloaded | cache-hit-rate < 70 % alert | engage origin shielding; degraded HLS-low-bitrate-only | `runbooks/playback-cdn-cache-cascade.md` |
| FM-04 | Whisper GPU pool exhausted | transcription queue grows | queue depth > 60 min alert | Whisper-medium fallback; per-tenant priority lane | `runbooks/transcript-pipeline-degraded-whisper.md` |
| FM-05 | pyannote diarization model crash | diarization queue grows | health-check fail | fall back to single-speaker mode; chapter-marker auto-via-diarization disabled | `runbooks/transcript-pipeline-degraded-whisper.md` |
| FM-06 | ffmpeg transcode pipeline failure | encode queue grows; playback ladder incomplete | encode failure rate > 5 % | gVisor sandbox restart; per-tenant tenant-priority shed | `runbooks/transcode-pipeline-failure.md` |
| FM-07 | Redaction overlay corruption (un-redact attack) | redacted content visible | overlay rows mismatch audit-chain seal | engage `runbooks/redaction-overlay-corruption.md`; emergency overlay re-apply; audit | `runbooks/redaction-overlay-corruption.md` |
| FM-08 | Retention purge runs against held recording | spoliation + court-order violation | load-bearing SLO breach + Sev-1 | pessimistic-lock on hold table; **load-bearing 100 % invariant — must never happen** | `runbooks/retention-policy-rollback.md` |
| FM-09 | Legal-hold engagement lag > 1s | court-order violation risk | load-bearing SLO breach + Sev-1 | direct DB write path + bypass batch worker | `runbooks/legal-hold-court-order-receipt.md` |
| FM-10 | eDiscovery export Merkle seal verification fails | counsel rejects bundle | bundle-sign error | re-export with audit-chain rewind; engage ops-security | `runbooks/ediscovery-export.md` |
| FM-11 | Share-link HMAC secret leak | unauthorised playback at scale | anomaly detection on signed-URL usage | OpenBao rotate; revoke all outstanding share-links; engage tenant | `runbooks/watermark-key-rotation.md` (related rotation procedure) |
| FM-12 | Watermark key rotation lag | leaked recording cannot be attributed | per-viewer watermark missing | engage `runbooks/watermark-key-rotation.md`; emergency watermark re-stamp | `runbooks/watermark-key-rotation.md` |
| FM-13 | Meilisearch index corruption | search returns wrong / no results | search consistency check | rebuild from transcript Workflow event replay | (see capacity model) |
| FM-14 | OPSWAT/ClamAV scan miss (false negative) | malware ingested | post-ingest scan-update detection | quarantine + alert + re-scan | (security-side) |
| FM-15 | Producer µservice ingest contract version skew | ingest refused | contract version mismatch alert | contract-version negotiation + fallback shim | (see ingest-contract spec) |
| FM-16 | Cross-pack recording leak | residency breach | weekly residency audit | refuse + alert; engage council-privacy | (see `policy/data-residency.md`) |
| FM-17 | Cedar policy evaluator stalls | every action blocks | Cedar p99 > 50ms alert | restart Cedar pods; per-tenant rate limit on heavy-policy calls | (see authority-cohesion gate) |
| FM-18 | foundry-runtime gVisor sandbox escape | transcoder host compromise | gVisor + ABI-watcher alert | quarantine host; engage ops-security; pin to last-known-safe sandbox | (security-side) |
| FM-19 | KMS-shred fails on retention expiry | retention not actually shredded | KMS-shred-failure rate > 0 | retry + alert; engage cloud-secrets | (see `runbooks/retention-policy-rollback.md`) |
| FM-20 | Pandoc CVE triggered by malicious transcript | sandbox escape | gVisor alert | pinned LTS + quarterly CVE review | (security-side) |

## Failure-Mode Risk Matrix

| Probability | Low impact | Medium impact | High impact |
|---|---|---|---|
| **High** | FM-04 (Whisper queue), FM-13 (Meilisearch corruption) | FM-06 (transcode), FM-15 (ingest contract skew) | — |
| **Medium** | FM-03 (CDN), FM-12 (watermark), FM-14 (scan miss), FM-19 (KMS-shred) | FM-11 (HMAC leak), FM-17 (Cedar stall) | FM-07 (redaction corruption), FM-10 (export Merkle), FM-16 (cross-pack leak) |
| **Low** | FM-01 (Postgres), FM-02 (S3), FM-05 (pyannote), FM-20 (Pandoc CVE) | — | FM-08 (retention vs hold) — **load-bearing**, FM-09 (legal-hold lag) — **load-bearing**, FM-18 (gVisor escape) |

## References

- ADR-RECORDINGS-0001..0007.
- `runbooks/*.md`.
- `threat-model.md`.
