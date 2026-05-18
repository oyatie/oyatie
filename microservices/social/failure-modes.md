---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-social
deciders: ops-sre-reliability, axis-social, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/social/threat-model.md
  - microservices/social/dpia.md
  - microservices/social/policy/dual-context-isolation.md
  - microservices/social/incident-response.md
  - microservices/social/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting social
doc_status: published
---

# Failure-Mode Catalog (social µservice)

## Purpose

Enumerate failure scenarios on-call must handle, detection signals, mitigation, RCA path, RTO target, and the owning runbook. Cross-referenced from `incident-response.md` for severity classification.

## Index

Each entry: FM-ID, trigger, detection, tenant impact, severity, immediate mitigation, RTO, recovery runbook, postmortem owner.

## FM-01: Feed-render storm

| Field | Value |
|---|---|
| Trigger | Viral post causes mass concurrent feed-pulls; mobile app push triggers cascade |
| Detection | `social_feed_render_requests_per_sec` > 10× baseline for ≥ 1 min; feed-cache hit rate drops < 70% |
| Tenant impact | Feed latency spike; possible 503 on cache miss |
| Severity | Sev-2 (multi-tenant degradation) — Sev-1 if persistent > 30 min |
| Immediate mitigation | HPA scale-up; widen feed-cache TTL; enable per-tenant feed-render rate limit; serve cached chronological fallback |
| RTO | ≤ 10 min |
| Runbook | `runbooks/feed-cache-rebuild.md` |
| Postmortem | axis-social |

## FM-02: Postgres post-store outage (primary failure)

| Field | Value |
|---|---|
| Trigger | OOM / crash / disk-full on primary; replication lag exceeds threshold |
| Detection | `social_post_store_primary_alive == 0` for ≥ 1 min |
| Tenant impact | Post-create fails (writes blocked); reads degrade to replica |
| Severity | Sev-1 (multi-tenant write blocked) |
| Immediate mitigation | Failover primary → replica; re-route writes; rebuild standby |
| RTO | ≤ 5 min failover; ≤ 30 min full recovery |
| Runbook | `runbooks/postgres-primary-failover.md` (in cell µservice; social references) |
| Postmortem | ops-sre-reliability + axis-social |

## FM-03: Redis feed-cache corruption

| Field | Value |
|---|---|
| Trigger | Replication split-brain; AOF corruption; mass eviction event |
| Detection | `social_feed_cache_inconsistency_total` > 0; feed-render p99 spike with cache-miss > 50% sustained |
| Tenant impact | Feed display stale or missing; latency spike |
| Severity | Sev-2 |
| Immediate mitigation | Rebuild feed cache from latest posts; flush stale entries; pause fanout-on-write briefly |
| RTO | ≤ 30 min |
| Runbook | `runbooks/feed-cache-rebuild.md` |
| Postmortem | axis-social |

## FM-04: Media-store outage (S3 unavailable)

| Field | Value |
|---|---|
| Trigger | S3 endpoint outage in pack region |
| Detection | `social_media_upload_failure_rate` > 5 % for ≥ 2 min |
| Tenant impact | Media uploads queued; reads degraded (cached previews work; original blob fetch fails) |
| Severity | Sev-2 |
| Immediate mitigation | DR-pair failover where available; surface backlog visibility to tenants |
| RTO | provider-dependent; ≤ 1 h DR; ≤ 4 h single-region |
| Runbook | `runbooks/media-transcode-degraded.md` (Slice B; same shape as messenger attachment-restore) |
| Postmortem | ops-sre-reliability + cloud-secrets |

## FM-05: Follow-graph corruption (Postgres adjacency-list ≠ audit-chain authoritative replay)

| Field | Value |
|---|---|
| Trigger | Direct Postgres mutation bypassing service; replication conflict; backup-restore inconsistency |
| Detection | Periodic graph-drift detector: `social_follow_graph_drift_total` > 0 |
| Tenant impact | Inconsistent follow / unfollow visibility |
| Severity | Sev-1 (relationship-data integrity risk; possible privacy implication) |
| Immediate mitigation | Quarantine affected accounts; re-derive follow-graph from audit-chain authoritative replay; ops-security engaged |
| RTO | ≤ 30 min per affected account |
| Runbook | `runbooks/follow-graph-corruption.md` |
| Postmortem | ops-security + axis-social |

## FM-06: Search index lag (Meilisearch indexer falling behind)

| Field | Value |
|---|---|
| Trigger | Ingest spike; indexer worker CPU saturation; bad query plan |
| Detection | `social_search_indexer_lag_seconds` > 60s sustained |
| Tenant impact | Recent posts / profiles not surfaced in search; UX appears stale |
| Severity | Sev-3 (search is best-effort by design; live-fallback to Postgres ILIKE) |
| Immediate mitigation | Scale indexer workers; enable Postgres-ILIKE fallback path; pause low-priority tenants' indexing |
| RTO | ≤ 1 h |
| Runbook | `runbooks/search-index-rebuild.md` (similar to messenger; not authored in social Slice-A; defaulted via failure-modes pointer) |
| Postmortem | axis-social |

## FM-07: Cross-tenant post leak (RLS misconfig)

| Field | Value |
|---|---|
| Trigger | Helm config change disables RLS; live mutation bypass; bug in app code skips RLS GUC |
| Detection | `oya-check-postgres-rls-coverage` lane fails OR `social_cross_tenant_leak_detector_total` > 0 |
| Tenant impact | Potential cross-tenant data exposure |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Auto-rollback to last green Helm; isolate cluster; declare breach-suspect; engage ops-security; GDPR Art. 33 clock starts |
| RTO | ≤ 5 min auto-rollback; investigation days |
| Runbook | `runbooks/cross-tenant-leak-investigation.md` (in governance; social references) |
| Postmortem | ops-security + axis-social |

## FM-08: Mention storm (one post @-mentions thousands)

| Field | Value |
|---|---|
| Trigger | Power-user mentions many handles in a single post; bot script floods mentions |
| Detection | `social_mention_fanout_queue_depth` > 100k |
| Tenant impact | Notification fanout backlog; mention-resolution latency spike |
| Severity | Sev-3 (degraded; auto-recover) — Sev-2 if persistent |
| Immediate mitigation | Per-post mention cap (default 50); throttle per-tenant; drop low-priority notification reps |
| RTO | ≤ 15 min |
| Runbook | `runbooks/mention-storm-throttle.md` |
| Postmortem | axis-social |

## FM-09: Media malware (scanner positive)

| Field | Value |
|---|---|
| Trigger | OPSWAT / ClamAV verdict positive on uploaded blob |
| Detection | `social_media_malware_detected_total` > 0 |
| Tenant impact | Blob quarantined; sender notified; viewers see "media removed" placeholder |
| Severity | Sev-2 (if isolated) — Sev-1 (if pattern suggests organised attack) |
| Immediate mitigation | Quarantine bucket holds blob; never copies to production; audit-chain seal of detection; tenant security-admin notified |
| RTO | immediate (detection is preventative) |
| Runbook | `runbooks/media-malware-quarantine.md` (Slice B; same shape as messenger attachment-malware) |
| Postmortem | ops-security + axis-social |

## FM-10: Cross-context routing violation (parallel ADR-0135 invariant breach)

| Field | Value |
|---|---|
| Trigger | Bug allows Personal post write into Professional context (or vice versa); LEAN lane regression |
| Detection | `social_context_switch_denied_attempt_total` > 0 OR LEAN lane fails on PR |
| Tenant impact | Privacy violation: personal data in professional audit scope (or vice versa) |
| Severity | Sev-1 (regulatory breach; GDPR / PIPA scope) |
| Immediate mitigation | Block service write path; quarantine affected entities; declare breach; engage council-privacy + ops-security |
| RTO | ≤ 15 min block; investigation days |
| Runbook | `runbooks/cross-context-violation.md` (Slice B; same shape as messenger) |
| Postmortem | council-privacy + ops-security + axis-social |

## FM-11: Notification fanout backlog

| Field | Value |
|---|---|
| Trigger | Celebrity account posts; cascading follower fanout; classifier-induced re-notification |
| Detection | `social_notification_fanout_queue_depth` > 500k sustained |
| Tenant impact | Notification delivery delayed (best-effort); no message loss |
| Severity | Sev-3 |
| Immediate mitigation | Scale notification workers; coalesce more aggressively into digest; pause low-priority push channel |
| RTO | ≤ 30 min |
| Runbook | `runbooks/abuse-report-backlog-drain.md` (Slice A; pattern applies to notification backlog with parameter swap) |
| Postmortem | axis-social |

## FM-12: Four-eyes disclosure mis-pairing (insider attempt)

| Field | Value |
|---|---|
| Trigger | Same principal attempts to satisfy both halves of four-eyes; programmatic spoofing of paired-approver-id |
| Detection | `social_four_eyes_pairing_violation_total` > 0 |
| Tenant impact | Disclosure attempt blocked; alerting on suspicious activity |
| Severity | Sev-1 (insider-malicious threat actor signal) |
| Immediate mitigation | Cedar evaluator denies; audit-chain seal of attempt; ops-security engaged |
| RTO | immediate (preventative) |
| Runbook | `runbooks/four-eyes-violation.md` (in governance; social references) |
| Postmortem | ops-security |

## FM-13: Cross-pack residency misroute

| Field | Value |
|---|---|
| Trigger | Bad Helm overlay deploys pack-X tenant into pack-Y cluster; Cedar pack-router bug |
| Detection | Periodic residency audit: `social_pack_residency_violation_total` > 0 |
| Tenant impact | Tenant data in wrong jurisdiction → regulatory breach |
| Severity | Sev-1 (GDPR / PIPA / HIPAA breach risk) |
| Immediate mitigation | Halt writes for affected tenant; migrate data back to correct pack; engage council-privacy |
| RTO | ≤ 30 min halt; migration may take hours |
| Runbook | `runbooks/pack-residency-recovery.md` (in governance; social references) |
| Postmortem | council-privacy + ops-security + axis-social |

## FM-14: Personal-tier federation leak attempt

| Field | Value |
|---|---|
| Trigger | Service operator or bug attempts to federate Personal-tier post |
| Detection | `social_personal_tier_federation_attempt_total` > 0 (compile-time invariant means this should be unreachable; runtime guard fires) |
| Tenant impact | Privacy breach signal; user trust at stake |
| Severity | Sev-1 (privacy violation) |
| Immediate mitigation | Operation denied at runtime; audit-chain seal; ops-security + council-privacy engaged; emergency LEAN-lane investigation (compile-time invariant should have prevented this) |
| RTO | immediate (preventative) |
| Runbook | `runbooks/federation-bridge-degraded.md` (Personal-tier-leak-attempt section) |
| Postmortem | council-privacy + ops-security |

## FM-15: Minor age-attestation pivot (minor-list leak)

| Field | Value |
|---|---|
| Trigger | Unauthorised read of `social_age_attestations` table OR DSR cascade leaks minor flag |
| Detection | Periodic access-audit of age-attestation table; non-entitled access detected |
| Tenant impact | Child-protection privacy breach |
| Severity | Sev-1 (especially regulatory: COPPA / GDPR Art. 8) |
| Immediate mitigation | Revoke unauthorised access; audit-chain seal; council-privacy + ops-security engaged; regulator notification per COPPA / GDPR Art. 8 |
| RTO | immediate (preventative) |
| Runbook | `runbooks/age-attestation-pivot.md` (Slice B) |
| Postmortem | council-privacy + ops-security + axis-social |

## FM-16: Moderation classifier drift (mass false-positive event)

| Field | Value |
|---|---|
| Trigger | Foundry-runtime classifier verdict-rate spikes on hate-speech / abuse / spam categories |
| Detection | `oya_social_moderation_verdict_total{verdict="abuse"|"hate-speech"|"spam"}` rate > 5× baseline for ≥ 10 min |
| Tenant impact | Mass content auto-hidden; tenant escalations; possible regulatory non-compliance |
| Severity | Sev-2 (drift) → Sev-1 (mass false-positive event with free-speech impact) |
| Immediate mitigation | Roll back classifier version; pause T2 auto-categorize per-pack via Cedar entitlement revoke; restore any messages auto-hidden by the rolled-back version (last 24h) |
| RTO | ≤ 15 min |
| Runbook | `runbooks/content-moderation-rollback.md` |
| Postmortem | axis-social + axis-foundry-runtime + ops-security + council-privacy |

## FM-17: Trending-topic poisoning (sybil-amplified hashtag)

| Field | Value |
|---|---|
| Trigger | Coordinated sybil amplification injects artificial trend |
| Detection | foundry-guardrails sybil detector signal + `social_trending_topic_anomaly_total` > 0 |
| Tenant impact | Trending-topic feed shows manipulated content; possible public-discourse harm |
| Severity | Sev-3 (auto-recover via sybil filter); Sev-2 if scale > tenant-scope |
| Immediate mitigation | Sybil-detector verdict applied; tenant-admin can pin / unpin; per-author influence cap in trending recompute |
| RTO | ≤ 30 min |
| Runbook | `runbooks/trending-topic-poisoning.md` |
| Postmortem | axis-social + ops-security + axis-foundry-guardrails |

## FM-18: Capacity exhaustion (per-cell limit hit)

| Field | Value |
|---|---|
| Trigger | Tenant growth or burst exceeds per-cell envelope; cardinality breach |
| Detection | `social_accounts_per_tenant_total` > 1M OR `social_active_connections_total` > 5M per cell |
| Tenant impact | New accounts rejected for affected tenant; new connections rejected |
| Severity | Sev-3 |
| Immediate mitigation | Shard tenant to new cell; raise per-tenant cap (with FinOps approval); notify gtm-customer-success |
| RTO | ≤ 4 h shard migration |
| Runbook | `runbooks/cell-shard-migration.md` (in cell µservice; social references) |
| Postmortem | ops-sre-reliability + axis-social |

## FM-19: Ontology lookup failure breaks mention-resolution

| Field | Value |
|---|---|
| Trigger | Ontology µservice outage |
| Detection | `social_mention_resolution_failure_rate` > 5 % for ≥ 2 min |
| Tenant impact | @mentions display as raw text; no notification fanout |
| Severity | Sev-3 (graceful degradation; mention still posts as text) |
| Immediate mitigation | Cache last known ontology graph; degrade to raw-text mode; reconcile when ontology returns |
| RTO | ≤ 30 min |
| Runbook | `runbooks/ontology-degraded-mode.md` (in ontology µservice; social references) |
| Postmortem | axis-social + axis-ontology |

## FM-20: Federation peer compromise (untrusted peer ingestion)

| Field | Value |
|---|---|
| Trigger | Federation peer is compromised; mass-spam or malicious-content arrives via inbox |
| Detection | `social_federation_peer_spam_rate` > threshold; HTTP Signature verification anomalies |
| Tenant impact | Federated content shows spam / malicious payloads |
| Severity | Sev-2 |
| Immediate mitigation | Remove peer from allowlist; quarantine ingested content from peer; engage ops-security |
| RTO | ≤ 30 min (peer removal); investigation hours |
| Runbook | `runbooks/federation-bridge-degraded.md` |
| Postmortem | ops-security + axis-social |
