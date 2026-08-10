---
doc_class: Runbook
title: Recording storage degraded (S3 outage during active meetings)
microservice: meet
severity: "Sev-2 (degradation) / Sev-1 (pack-us-financial or pack-us-healthcare)"
status: Accepted
owner_team: ops-sre-reliability + axis-meet
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-03)
  - comms/meet/dashboards/recording-pipeline.json
  - microservices/meet/slos/recording-start-latency.openslo.yaml
  - microservices/meet/policy/data-residency.md
doc_status: published
---

# Runbook: Recording storage degraded (meet)

## Trigger

Any of:
- `oya_meet_recording_upload_failure_rate` > 5 % for ≥ 2 min.
- S3 health check failing in pack region.
- ffmpeg worker local-disk-buffer > 80 % capacity (≤ 1h capacity).
- Recording manifest write failures > 5 % for ≥ 2 min (Postgres recording-manifest table).

## Severity

- Default: Sev-2.
- Sev-1 if:
  - pack-us-financial: SEC 17a-4(f) WORM compliance at risk.
  - pack-us-healthcare: HIPAA recording-retention obligation at risk.
  - pack-eu investment firm: MiFID II 5-7y retention at risk.
  - Sustained > 30 min.

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect `dashboards/recording-pipeline.json`: confirm S3 outage scope | ≤ 2 min |
| 2 | ffmpeg recording workers buffer to local PV (≤ 1h capacity); active recordings continue | ≤ immediate |
| 3 | Surface in-meeting host banner: "Recording is paused due to a service incident; your meeting can continue, but please consider concluding if compliance recording is required" | ≤ 5 min |
| 4 | If DR-pair pack: failover S3 to DR replica; resume uploads | ≤ 10 min |
| 5 | Suspend new recording starts (Cedar forbid temporarily set via emergency policy push) | ≤ 5 min |
| 6 | Notify pack-us-financial supervisors of recording integrity event (regulatory chain per `incident-response.md`) | ≤ 30 min (if Sev-1) |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| S3 endpoint outage in pack region | S3 health-check fails | cloud-iac status; OCI Object Storage status page |
| KMS outage blocks SSE-KMS | KMS health-check fails | cloud-secrets engagement |
| Tenant-DEK envelope key lost | per-tenant DEK fetch fails | cloud-secrets escalation; potential Sev-1 |
| Postgres recording-manifest outage | manifest write failures | FM-08 procedure |
| Local-disk buffer fills (worker pod disk full) | PV usage > 80 % | scale ffmpeg pods + larger PV; emergency |
| S3 Object Lock policy violation | Object Lock errors | check Object Lock + tenant retention floor configuration |

## DR Failover Procedure (Sev-1)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm DR-pair S3 replica lag < 5 min | ≤ 5 min |
| 2 | Switch ffmpeg worker upload endpoint to DR S3 bucket (config push) | ≤ 5 min |
| 3 | Verify uploads succeed to DR bucket | ≤ 5 min |
| 4 | When primary S3 recovers: drain local-disk-buffered recordings to primary; reconcile manifest | ≤ 1h |
| 5 | Verify content_hash integrity post-recovery; audit-chain re-seal if needed | ≤ 1h |

## Local-Disk Buffer Limits

Each ffmpeg recording worker has ≤ 1h of recording buffer on local PV:
- 1 active recording at 2 Mbps = ~ 900 MB / hour
- 4 concurrent recordings per pod × 1h = ~ 3.6 GB
- 16 ffmpeg pods × 3.6 GB = ~ 60 GB total buffer

If outage exceeds 1h cumulative across all active recordings, oldest buffered recording lost; surface as Sev-1 + tenant comms.

## DSR Retention-Conflict Handling

If during outage a DSR erasure request arrives:
1. DSR cascade waits for storage recovery; does NOT delete buffered recordings.
2. Subject is notified: "Your erasure request is acknowledged; deletion will complete within 30 days per GDPR Art. 17; current storage outage may extend timeline."
3. On recovery: standard DSR cascade per `policy/data-residency.md` §DSR.

## Calendar Binding Refresh (FM-17 cross-reference)

If during outage, calendar binding is stale:
1. Re-emit `CalendarEventUpdated` Workflow consumer.
2. Verify meet-link in calendar event matches current meeting-room URL.
3. If mismatch: cache-bust + reactivate binding.

## Event Replay (cross-reference to backfill-replay.md)

After outage recovery, run `meet replay-events --tenant <t> --from <outage-start> --to <recovery>` to reconcile any missed Workflow events.

## Recovery Verification

- `oya_meet_recording_upload_failure_rate` back to < 0.1 % for ≥ 15 min.
- Local-disk buffer drained.
- Recording manifest writes succeeding.
- Audit-chain seals current for all completed recordings.

## Postmortem Triggers

- Root-cause identified within 5 business days.
- If S3 vendor outage: engage cloud-iac for SLA claim.
- If pack-us-financial: supervisor postmortem within 1 business day (SEC 17a-4 expectation).
- If pack-us-healthcare: HHS OCR breach review if PHI compromised.

## References

- SEC Rule 17a-4(f); FINRA Rule 4511; HIPAA 45 CFR §164.312(c)(1); MiFID II Art. 16(7); KR PIPA Art. 21.
- ADR-MEET-0002 (recording pipeline).
- `microservices/meet/threat-model.md` T-T-01; T-D-04.
- OCI Object Storage status page.
- `comms/messenger/runbooks/attachment-restore.md` (analogous pattern).
