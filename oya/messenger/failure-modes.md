---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-messenger
deciders: ops-sre-reliability, axis-messenger, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/messenger/threat-model.md
  - microservices/messenger/dpia.md
  - microservices/messenger/policy/dual-context-isolation.md
  - microservices/messenger/incident-response.md
  - microservices/messenger/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting messenger
doc_status: published
---

# Failure-Mode Catalog (messenger µservice)

## Purpose

Enumerate failure scenarios on-call must handle, detection signals, mitigation, RCA path, RTO target, and the owning runbook. Cross-referenced from `incident-response.md` for severity classification.

## Index

Each entry: FM-ID, trigger, detection, tenant impact, severity, immediate mitigation, RTO, recovery runbook, postmortem owner.

## FM-01: WebSocket gateway storm

| Field | Value |
|---|---|
| Trigger | Mass reconnection after network blip; mobile-network handoff cascade; client bug causes reconnect loop |
| Detection | `messenger_gateway_connection_attempts_per_sec` > 10× baseline for ≥ 1 min; gateway CPU > 90 % |
| Tenant impact | New connections slow / rejected; existing connections may drop |
| Severity | Sev-2 (multi-tenant degradation; auto-recover possible) — Sev-1 if persistent > 30 min |
| Immediate mitigation | HPA scale-up; enable per-tenant connection rate limit; enable jittered-backoff client SDK kill-switch |
| RTO | ≤ 10 min |
| Runbook | `runbooks/websocket-storm.md` |
| Postmortem | axis-messenger |

## FM-02: Postgres message-store outage (primary failure)

| Field | Value |
|---|---|
| Trigger | OOM / crash / disk-full on primary; replication lag exceeds threshold |
| Detection | `messenger_message_store_primary_alive == 0` for ≥ 1 min |
| Tenant impact | Message-send fails (writes blocked); reads degrade to replica |
| Severity | Sev-1 (multi-tenant write blocked) |
| Immediate mitigation | Failover primary → replica; re-route writes; rebuild standby |
| RTO | ≤ 5 min failover; ≤ 30 min full recovery |
| Runbook | `runbooks/postgres-primary-failover.md` (in cell µservice; messenger references) |
| Postmortem | ops-sre-reliability + axis-messenger |

## FM-03: Valkey presence corruption

| Field | Value |
|---|---|
| Trigger | Replication-split brain; AOF corruption; mass eviction event |
| Detection | `messenger_presence_inconsistency_total` > 0; presence stale for > 5 min globally |
| Tenant impact | Stale presence display (online users appear offline; presence transitions miss) |
| Severity | Sev-2 |
| Immediate mitigation | Rebuild presence from active WebSocket connections; flush stale entries |
| RTO | ≤ 15 min |
| Runbook | `runbooks/presence-rebuild.md` |
| Postmortem | axis-messenger |

## FM-04: Attachment-store outage (S3 unavailable)

| Field | Value |
|---|---|
| Trigger | S3 endpoint outage in pack region |
| Detection | `messenger_attachment_upload_failure_rate` > 5 % for ≥ 2 min |
| Tenant impact | Attachment uploads queued; reads degraded (cached previews work; original blob fetch fails) |
| Severity | Sev-2 |
| Immediate mitigation | DR-pair failover where available; surface backlog visibility to tenants |
| RTO | provider-dependent; ≤ 1 h DR; ≤ 4 h single-region |
| Runbook | `runbooks/attachment-restore.md` |
| Postmortem | ops-sre-reliability + cloud-secrets |

## FM-05: Channel ACL drift (Postgres ACL ≠ audit-chain authoritative replay)

| Field | Value |
|---|---|
| Trigger | Direct Postgres mutation bypassing service; replication conflict; backup-restore inconsistency |
| Detection | Periodic ACL-drift detector: `messenger_acl_drift_total` > 0 |
| Tenant impact | Potential over-permitted or under-permitted access for affected channel |
| Severity | Sev-1 (security risk) |
| Immediate mitigation | Quarantine affected channel; re-derive ACL from audit-chain authoritative replay; ops-security engaged |
| RTO | ≤ 30 min per affected channel |
| Runbook | `runbooks/channel-acl-drift.md` |
| Postmortem | ops-security + axis-messenger |

## FM-06: Search index lag (Tantivy / ES indexer falling behind)

| Field | Value |
|---|---|
| Trigger | Ingest spike; indexer worker CPU saturation; bad query plan |
| Detection | `messenger_search_indexer_lag_seconds` > 60s sustained |
| Tenant impact | Recent messages not surfaced in search; UX appears stale |
| Severity | Sev-3 (search is best-effort by design; live-fallback to Postgres LIKE) |
| Immediate mitigation | Scale indexer workers; enable Postgres-LIKE fallback path; pause low-priority tenants' indexing |
| RTO | ≤ 1 h |
| Runbook | `runbooks/search-index-rebuild.md` |
| Postmortem | axis-messenger |

## FM-07: Cross-tenant message leak (RLS misconfig)

| Field | Value |
|---|---|
| Trigger | Helm config change disables RLS; live mutation bypass; bug in app code skips RLS GUC |
| Detection | `oya-check-postgres-rls-coverage` lane fails OR `messenger_cross_tenant_leak_detector_total` > 0 |
| Tenant impact | Potential cross-tenant data exposure |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Auto-rollback to last green Helm; isolate cluster; declare breach-suspect; engage ops-security; GDPR Art. 33 clock starts |
| RTO | ≤ 5 min auto-rollback; investigation days |
| Runbook | `runbooks/cross-tenant-leak-investigation.md` (in governance; messenger references) |
| Postmortem | ops-security + axis-messenger |

## FM-08: Mention storm (one message @-mentions thousands)

| Field | Value |
|---|---|
| Trigger | Power-user mentions `@channel` in 10k-member channel; bot script floods mentions |
| Detection | `messenger_mention_fanout_queue_depth` > 100k |
| Tenant impact | Notification fanout backlog; mention-resolution latency spike |
| Severity | Sev-3 (degraded; auto-recover) — Sev-2 if persistent |
| Immediate mitigation | Per-message mention cap (default 50); throttle per-tenant; drop low-priority notification reps |
| RTO | ≤ 15 min |
| Runbook | `runbooks/mention-storm-throttle.md` |
| Postmortem | axis-messenger |

## FM-09: File-attachment malware (scanner positive)

| Field | Value |
|---|---|
| Trigger | OPSWAT / ClamAV verdict positive on uploaded blob |
| Detection | `messenger_attachment_malware_detected_total` > 0 |
| Tenant impact | Blob quarantined; sender notified; channel members see "attachment removed" placeholder |
| Severity | Sev-2 (if isolated) — Sev-1 (if pattern suggests organised attack) |
| Immediate mitigation | Quarantine bucket holds blob; never copies to production; audit-chain seal of detection; tenant security-admin notified |
| RTO | immediate (detection is preventative) |
| Runbook | `runbooks/attachment-malware-quarantine.md` |
| Postmortem | ops-security + axis-messenger |

## FM-10: Cross-context routing violation (parallel ADR-0238 invariant breach)

| Field | Value |
|---|---|
| Trigger | Bug allows Personal-DM write into Professional channel (or vice versa); LEAN lane regression |
| Detection | `messenger_context_switch_denied_attempt_total` > 0 OR LEAN lane fails on PR |
| Tenant impact | Privacy violation: personal data in professional audit scope (or vice versa) |
| Severity | Sev-1 (regulatory breach; GDPR / PIPA scope) |
| Immediate mitigation | Block service write path; quarantine affected entities; declare breach; engage council-privacy + ops-security |
| RTO | ≤ 15 min block; investigation days |
| Runbook | `runbooks/cross-context-violation.md` |
| Postmortem | council-privacy + ops-security + axis-messenger |

## FM-11: Read-receipt fanout backpressure storm

| Field | Value |
|---|---|
| Trigger | Mass read-receipt emission during peak hour; Valkey coalescer overwhelmed |
| Detection | `messenger_read_receipt_coalesce_window_breach_total` > 0 sustained |
| Tenant impact | Read-receipt updates lag (best-effort); no message loss |
| Severity | Sev-3 |
| Immediate mitigation | Widen coalesce window from 250ms to 1s; scale receipt-worker pods |
| RTO | ≤ 10 min |
| Runbook | `runbooks/presence-rebuild.md` §"Read-receipt path" |
| Postmortem | axis-messenger |

## FM-12: Four-eyes disclosure mis-pairing (insider attempt)

| Field | Value |
|---|---|
| Trigger | Same principal attempts to satisfy both halves of four-eyes; programmatic spoofing of paired-approver-id |
| Detection | `messenger_four_eyes_pairing_violation_total` > 0 |
| Tenant impact | Disclosure attempt blocked; alerting on suspicious activity |
| Severity | Sev-1 (insider-malicious threat actor signal) |
| Immediate mitigation | Cedar evaluator denies; audit-chain seal of attempt; ops-security engaged |
| RTO | immediate (preventative) |
| Runbook | `runbooks/four-eyes-violation.md` |
| Postmortem | ops-security |

## FM-13: Cross-pack residency misroute

| Field | Value |
|---|---|
| Trigger | Bad Helm overlay deploys pack-X tenant into pack-Y cluster; Cedar pack-router bug |
| Detection | Periodic residency audit: `messenger_pack_residency_violation_total` > 0 |
| Tenant impact | Tenant data in wrong jurisdiction → regulatory breach |
| Severity | Sev-1 (GDPR / PIPA / HIPAA breach risk) |
| Immediate mitigation | Halt writes for affected tenant; migrate data back to correct pack; engage council-privacy |
| RTO | ≤ 30 min halt; migration may take hours |
| Runbook | `runbooks/pack-residency-recovery.md` |
| Postmortem | council-privacy + ops-security + axis-messenger |

## FM-14: Personal-DM ciphertext key-escrow exposure attempt

| Field | Value |
|---|---|
| Trigger | Service operator attempts to query / decrypt personal-DM body server-side |
| Detection | `messenger_personal_dm_admin_decrypt_attempt_total` > 0 |
| Tenant impact | Privacy breach signal; user trust at stake |
| Severity | Sev-1 (privacy violation) |
| Immediate mitigation | Operation denied; audit-chain seal; ops-security + council-privacy engaged |
| RTO | immediate (preventative) |
| Runbook | `runbooks/personal-dm-admin-attempt.md` (referenced from cross-context-violation) |
| Postmortem | council-privacy + ops-security |

## FM-15: Capacity exhaustion (per-cell limit hit)

| Field | Value |
|---|---|
| Trigger | Tenant growth or burst exceeds per-cell envelope; cardinality breach |
| Detection | `messenger_channels_per_tenant_total` > 50k OR `messenger_active_websocket_connections_total` > 1M per cell |
| Tenant impact | New channels rejected for affected tenant; new connections rejected |
| Severity | Sev-3 |
| Immediate mitigation | Shard tenant to new cell; raise per-tenant cap (with FinOps approval); notify gtm-customer-success |
| RTO | ≤ 4 h shard migration |
| Runbook | `runbooks/cell-shard-migration.md` |
| Postmortem | ops-sre-reliability + axis-messenger |

## FM-16: Ontology lookup failure breaks mention-resolution

| Field | Value |
|---|---|
| Trigger | Ontology µservice outage |
| Detection | `messenger_mention_resolution_failure_rate` > 5 % for ≥ 2 min |
| Tenant impact | @mentions display as raw text; no notification fanout |
| Severity | Sev-3 (graceful degradation; mention still posts as text) |
| Immediate mitigation | Cache last known ontology graph; degrade to raw-text mode; reconcile when ontology returns |
| RTO | ≤ 30 min |
| Runbook | `runbooks/ontology-degraded-mode.md` (in ontology µservice; messenger references) |
| Postmortem | axis-messenger + axis-ontology |
