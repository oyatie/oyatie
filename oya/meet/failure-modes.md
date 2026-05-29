---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-meet
deciders: ops-sre-reliability, axis-meet, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/meet/threat-model.md
  - microservices/meet/dpia.md
  - microservices/meet/incident-response.md
  - microservices/meet/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting meet
doc_status: published
---

# Failure-Mode Catalog (meet µservice)

## Purpose

Enumerate failure scenarios on-call must handle, detection signals, mitigation, RCA path, RTO target, and the owning runbook. Cross-referenced from `incident-response.md` for severity classification.

## Index

Each entry: FM-ID, trigger, detection, tenant impact, severity, immediate mitigation, RTO, recovery runbook, postmortem owner.

## FM-01: LiveKit SFU degraded (packet-loss / MOS / CPU)

| Field | Value |
|---|---|
| Trigger | LiveKit pod CPU > 90% sustained; packet-loss > 3% p95; mean MOS < 3.5; symmetric-NAT cascade; codec mismatch; bad upstream release |
| Detection | `oya_meet_media_packet_loss_pct` p95 > 3% for ≥ 5 min; `oya_meet_media_mos` mean < 3.5 for ≥ 5 min; LiveKit CPU > 90% |
| Tenant impact | Audio/video glitches; participants reconnect; new joins slow |
| Severity | Sev-2 default; Sev-1 if sustained > 15 min OR if pack-us-healthcare/financial recording session degrades |
| Immediate mitigation | HPA scale-up; cordon hot pod; disable simulcast (audio-only fallback); scale coturn; helm rollback if CVE |
| RTO | ≤ 10 min |
| Runbook | `runbooks/sfu-degraded.md` |
| Postmortem | axis-meet + ops-sre-reliability |

## FM-02: coturn TURN saturation

| Field | Value |
|---|---|
| Trigger | TURN allocation rate exceeds capacity; bandwidth > 70% provisioned; symmetric-NAT cascade |
| Detection | `coturn_traffic_bytes_total` rate vs cluster cap; ICE relay-candidate selection > 50% (suggests TURN-only fallback) |
| Tenant impact | Participants behind symmetric NAT cannot establish media |
| Severity | Sev-2 |
| Immediate mitigation | Scale coturn cluster (HPA); verify external_ip reachable; rotate auth secret if compromise suspected |
| RTO | ≤ 10 min |
| Runbook | `runbooks/coturn-key-rotation.md` |
| Postmortem | ops-sre-reliability |

## FM-03: Recording storage degraded (S3 outage during active meetings)

| Field | Value |
|---|---|
| Trigger | S3 endpoint outage in pack region |
| Detection | `oya_meet_recording_upload_failure_rate` > 5% for ≥ 2 min |
| Tenant impact | New recording uploads queued; ongoing recordings buffer to local disk; reads degraded |
| Severity | Sev-2; Sev-1 if pack-us-financial (SEC 17a-4 WORM impacted) or pack-us-healthcare |
| Immediate mitigation | Local-disk buffer at ffmpeg worker (≤ 1h capacity); retry on S3 recovery; surface backlog to hosts; DR-pair failover where available |
| RTO | provider-dependent; ≤ 1 h DR; ≤ 4 h single-region |
| Runbook | `runbooks/recording-storage-degraded.md` |
| Postmortem | ops-sre-reliability + axis-meet |

## FM-04: Transcription classifier rollback (Whisper model drift)

| Field | Value |
|---|---|
| Trigger | Whisper-large model upgrade introduces regression in BLEU / chrF++ vs baseline set; tenant-reported caption-quality complaints |
| Detection | `oya_meet_transcription_quality_score` regression > 5% vs prior 7-day baseline; tenant escalations |
| Tenant impact | Live captions degrade; transcripts less accurate |
| Severity | Sev-2 |
| Immediate mitigation | Roll back to prior Whisper model version; verify model SHA against pinned registry; ops-sre-reliability + axis-foundry-runtime engaged |
| RTO | ≤ 15 min |
| Runbook | `runbooks/transcription-classifier-rollback.md` |
| Postmortem | axis-meet + axis-foundry-runtime + council-privacy |

## FM-05: Lobby bypass incident

| Field | Value |
|---|---|
| Trigger | Bug or attack allows guest to enter meeting without host approval |
| Detection | `oya_meet_lobby_bypass_attempt_total` > 0; tenant escalation; ops-security alert |
| Tenant impact | Unauthorized participant in meeting; potential privacy breach |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Block meet-rest token-redemption path; force-disconnect bypass participant; engage ops-security; declare breach-suspect; GDPR Art. 33 clock starts if EU data subjects |
| RTO | ≤ 5 min block; investigation days |
| Runbook | `runbooks/lobby-bypass-incident.md` |
| Postmortem | ops-security + axis-meet |

## FM-06: Live caption stalled / Whisper GPU pool exhaustion

| Field | Value |
|---|---|
| Trigger | Whisper streaming pool depth > 5 sustained; GPU node failure; model load error |
| Detection | `oya_meet_live_caption_lag_seconds` p99 > 1.0 s for ≥ 2 min; GPU pool depth alerting |
| Tenant impact | Live captions stall; transcripts may catch up post-meeting |
| Severity | Sev-2 |
| Immediate mitigation | Burst-pool GPU spin-up; downgrade Whisper-large → Whisper-medium; if no GPUs available, degrade to "captions unavailable" with banner; surface to host |
| RTO | ≤ 15 min |
| Runbook | `runbooks/live-caption-stalled.md` |
| Postmortem | axis-meet + axis-foundry-runtime |

## FM-07: Webinar overload (>10k attendee fan-out throttled)

| Field | Value |
|---|---|
| Trigger | Webinar exceeds 10k interactive cap or 100k broadcast cap; WHIP/HLS mesh saturated |
| Detection | `oya_meet_webinar_attendees_active` > 10k OR HLS edge cache miss rate > 50% |
| Tenant impact | New attendees rejected; existing attendees may experience lag |
| Severity | Sev-2 |
| Immediate mitigation | Force WHIP/HLS broadcast mode; throttle interactive-tier join; scale SRS RTMP egress; raise per-tenant cap with FinOps approval; ops-sre-reliability engaged |
| RTO | ≤ 15 min throttle activation; ≤ 1h scale-out |
| Runbook | `runbooks/webinar-overload-throttle.md` |
| Postmortem | ops-sre-reliability + axis-meet |

## FM-08: Postgres meeting-store outage (primary failure)

| Field | Value |
|---|---|
| Trigger | OOM / crash / disk-full on primary; replication lag exceeds threshold |
| Detection | `oya_meet_meeting_store_primary_alive == 0` for ≥ 1 min |
| Tenant impact | Meeting-create fails (writes blocked); reads degrade to replica |
| Severity | Sev-1 (multi-tenant write blocked) |
| Immediate mitigation | Failover primary → replica; re-route writes; rebuild standby |
| RTO | ≤ 5 min failover; ≤ 30 min full recovery |
| Runbook | `runbooks/postgres-primary-failover.md` (in cell µservice; meet references) |
| Postmortem | ops-sre-reliability + axis-meet |

## FM-09: Cross-tenant recording leak (RLS misconfig)

| Field | Value |
|---|---|
| Trigger | Helm config change disables RLS; live mutation bypass; bug skips RLS GUC |
| Detection | `oya-check-postgres-rls-coverage` lane fails OR `oya_meet_cross_tenant_leak_detector_total` > 0 |
| Tenant impact | Potential cross-tenant recording exposure |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Auto-rollback to last green Helm; isolate cluster; declare breach-suspect; engage ops-security; GDPR Art. 33 clock starts |
| RTO | ≤ 5 min auto-rollback; investigation days |
| Runbook | `runbooks/cross-tenant-leak-investigation.md` (in governance; meet references) |
| Postmortem | ops-security + axis-meet |

## FM-10: Cross-pack residency misroute (pack-eu recording in pack-us cluster)

| Field | Value |
|---|---|
| Trigger | Bad Helm overlay deploys pack-X tenant into pack-Y cluster; Cedar pack-router bug |
| Detection | Periodic residency audit: `oya_meet_pack_residency_violation_total` > 0 |
| Tenant impact | Tenant data in wrong jurisdiction → regulatory breach |
| Severity | Sev-1 (GDPR / PIPA / HIPAA breach risk) |
| Immediate mitigation | Halt writes for affected tenant; migrate data back to correct pack; engage council-privacy |
| RTO | ≤ 30 min halt; migration may take hours |
| Runbook | `runbooks/pack-residency-recovery.md` |
| Postmortem | council-privacy + ops-security + axis-meet |

## FM-11: E2E mode decrypt attempt (server-side)

| Field | Value |
|---|---|
| Trigger | Service operator attempts to decrypt E2E-mode meeting body server-side |
| Detection | `oya_meet_e2e_admin_decrypt_attempt_total` > 0 |
| Tenant impact | Privacy breach signal |
| Severity | Sev-1 (privacy violation) |
| Immediate mitigation | Operation denied by Cedar; audit-chain seal; ops-security + council-privacy engaged |
| RTO | immediate (preventative) |
| Runbook | `runbooks/e2e-decrypt-attempt.md` |
| Postmortem | council-privacy + ops-security |

## FM-12: ffmpeg recording worker CVE escape (gVisor sandbox breach attempt)

| Field | Value |
|---|---|
| Trigger | ffmpeg media-parser CVE; attacker uploads crafted media; sandbox-escape attempt |
| Detection | gVisor escape alert; unusual syscall pattern; pod-restart pattern matching crash |
| Tenant impact | Worker pod isolated; recording job retried; potential data corruption on affected job |
| Severity | Sev-2; Sev-1 if escape confirmed |
| Immediate mitigation | Quarantine affected pod; rotate ffmpeg image to pinned-CVE-free version; gVisor policy review; ops-security engaged |
| RTO | ≤ 10 min pod-rotation; ≤ 4h image-rotation cluster-wide |
| Runbook | `runbooks/ffmpeg-gvisor-escape.md` |
| Postmortem | ops-security + axis-meet |

## FM-13: RTMP egress to unauthorized destination

| Field | Value |
|---|---|
| Trigger | Misconfigured egress destination; attacker swaps target endpoint |
| Detection | `oya_meet_rtmp_egress_destination` not in tenant allow-list; outbound DNS for non-allow-listed host |
| Tenant impact | Recording streamed to unauthorized destination → privacy breach |
| Severity | Sev-1 |
| Immediate mitigation | Egress NetworkPolicy + DNS allow-list refuses unauthorized destination; SRS stream terminated; audit-chain seal; ops-security engaged |
| RTO | immediate (preventative) |
| Runbook | `runbooks/webinar-overload-throttle.md` §"Egress allow-list violation" |
| Postmortem | ops-security + axis-meet |

## FM-14: Recording legal-hold conflict (retention floor vs erasure request)

| Field | Value |
|---|---|
| Trigger | DSR erasure request meets HIPAA 6y / SEC 17a-4 3-7y retention floor |
| Detection | DSR cascade detects retention-bound conflict |
| Tenant impact | Erasure cannot complete fully; data subject notified of retention basis |
| Severity | Sev-3 |
| Immediate mitigation | Redact identifiers (face-blur, voice-mask) but preserve body in access-restricted form; notify subject + DPO |
| RTO | ≤ 30 days SLA (GDPR Art. 17) |
| Runbook | `runbooks/recording-storage-degraded.md` §"DSR retention conflict" |
| Postmortem | council-privacy + ops-compliance |

## FM-15: Capacity exhaustion (per-cell limit hit)

| Field | Value |
|---|---|
| Trigger | Tenant growth or burst exceeds per-cell envelope; cardinality breach |
| Detection | `oya_meet_meetings_concurrent_total` > 50k OR `oya_meet_participants_concurrent_total` > 500k per cell |
| Tenant impact | New meetings rejected; new joins rejected for affected tenant |
| Severity | Sev-3 |
| Immediate mitigation | Shard tenant to new cell; raise per-tenant cap (with FinOps approval); notify gtm-customer-success |
| RTO | ≤ 4 h shard migration |
| Runbook | `runbooks/cell-shard-migration.md` |
| Postmortem | ops-sre-reliability + axis-meet |

## FM-16: Ontology lookup failure breaks participant directory

| Field | Value |
|---|---|
| Trigger | Ontology µservice outage |
| Detection | `oya_meet_participant_resolution_failure_rate` > 5% for ≥ 2 min |
| Tenant impact | Display names show as raw user_ref; no Person-card hover |
| Severity | Sev-3 (graceful degradation) |
| Immediate mitigation | Cache last known ontology graph; degrade to raw-ref mode; reconcile when ontology returns |
| RTO | ≤ 30 min |
| Runbook | `runbooks/ontology-degraded-mode.md` (in ontology µservice; meet references) |
| Postmortem | axis-meet + axis-ontology |

## FM-17: Calendar binding stale (calendar µservice latency)

| Field | Value |
|---|---|
| Trigger | Calendar event updated but meet-link binding stale |
| Detection | `oya_meet_calendar_binding_stale_total` > 0 |
| Tenant impact | Attendees use outdated meet-link from invite |
| Severity | Sev-3 |
| Immediate mitigation | Reactivate binding; re-emit `CalendarEventUpdated` Workflow consumer; cache-bust |
| RTO | ≤ 15 min |
| Runbook | `runbooks/recording-storage-degraded.md` §"Calendar binding refresh" |
| Postmortem | axis-meet + axis-calendar |
