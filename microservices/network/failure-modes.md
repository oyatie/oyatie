---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-network
deciders: ops-sre-reliability, axis-network, ops-security, council-architecture, ops-compliance
related_adrs: [ADR-0117, ADR-0126, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/network/threat-model.md
  - microservices/network/dpia.md
  - microservices/network/policy/professional-context-isolation.md
  - microservices/network/incident-response.md
  - microservices/network/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting network
doc_status: published
---

# Failure-Mode Catalog (network µservice)

## Purpose

Enumerate failure scenarios on-call must handle, detection signals, mitigation, RCA path, RTO target, and the owning runbook. Cross-referenced from `incident-response.md` for severity classification.

## Index

Each entry: FM-ID, trigger, detection, tenant impact, severity, immediate mitigation, RTO, recovery runbook, postmortem owner.

## FM-01: Feed-render storm

| Field | Value |
|---|---|
| Trigger | Viral Professional post causes mass concurrent feed-pulls; mobile app push triggers cascade |
| Detection | `network_feed_render_requests_per_sec` > 10× baseline for ≥ 1 min; feed-cache hit rate drops < 70% |
| Tenant impact | Feed latency spike; possible 503 on cache miss |
| Severity | Sev-2 (multi-tenant degradation) — Sev-1 if persistent > 30 min |
| Immediate mitigation | HPA scale-up; widen feed-cache TTL; enable per-tenant feed-render rate limit; serve cached chronological fallback |
| RTO | ≤ 10 min |
| Runbook | `runbooks/feed-cache-rebuild.md` |
| Postmortem | axis-network |

## FM-02: Postgres post-store outage (primary failure)

| Field | Value |
|---|---|
| Trigger | OOM / crash / disk-full on primary; replication lag exceeds threshold |
| Detection | `network_post_store_primary_alive == 0` for ≥ 1 min |
| Tenant impact | Post-create + connection-action + endorsement-add fails (writes blocked); reads degrade to replica |
| Severity | Sev-1 (multi-tenant write blocked) |
| Immediate mitigation | Failover primary → replica; re-route writes; rebuild standby |
| RTO | ≤ 5 min failover; ≤ 30 min full recovery |
| Runbook | `runbooks/postgres-primary-failover.md` (in cell µservice; network references) |
| Postmortem | ops-sre-reliability + axis-network |

## FM-03: Redis feed-cache corruption

| Field | Value |
|---|---|
| Trigger | Replication split-brain; AOF corruption; mass eviction event |
| Detection | `network_feed_cache_inconsistency_total` > 0; feed-render p99 spike with cache-miss > 50% sustained |
| Tenant impact | Feed display stale or missing; latency spike |
| Severity | Sev-2 |
| Immediate mitigation | Rebuild feed cache from canonical Postgres; flush stale entries; pause fanout-on-write briefly |
| RTO | ≤ 30 min |
| Runbook | `runbooks/feed-cache-rebuild.md` |
| Postmortem | axis-network |

## FM-04: Media + document store outage (S3 unavailable)

| Field | Value |
|---|---|
| Trigger | S3 endpoint outage in pack region |
| Detection | `network_media_upload_failure_rate` > 5 % for ≥ 2 min |
| Tenant impact | Media / document uploads queued; reads degraded (cached previews work; original blob fetch fails) |
| Severity | Sev-2 |
| Immediate mitigation | DR-pair failover where available; surface backlog visibility to tenants |
| RTO | provider-dependent; ≤ 1 h DR; ≤ 4 h single-region |
| Runbook | `runbooks/feed-cache-rebuild.md` §"degraded media" section (paired) |
| Postmortem | ops-sre-reliability + cloud-secrets |

## FM-05: Connection-graph corruption (Postgres adjacency-list ≠ audit-chain authoritative replay)

| Field | Value |
|---|---|
| Trigger | Direct Postgres mutation bypassing service; replication conflict; backup-restore inconsistency |
| Detection | Periodic graph-drift detector: `network_connection_graph_drift_total` > 0 |
| Tenant impact | Inconsistent connection visibility; degree-of-separation miscalculation; possible privacy implication |
| Severity | Sev-1 (relationship-data integrity risk; possible privacy implication; Professional network trust at stake) |
| Immediate mitigation | Quarantine affected accounts; re-derive connection-graph from audit-chain authoritative replay; ops-security engaged |
| RTO | ≤ 30 min per affected account |
| Runbook | `runbooks/connection-graph-corruption.md` |
| Postmortem | ops-security + axis-network |

## FM-06: Search index lag (Meilisearch indexer falling behind)

| Field | Value |
|---|---|
| Trigger | Ingest spike; indexer worker CPU saturation; bad query plan; per-index sharding imbalance |
| Detection | `network_search_indexer_lag_seconds{index="..."}` > 60s sustained |
| Tenant impact | Recent profiles / posts / jobs not surfaced in search; UX appears stale |
| Severity | Sev-3 (search is best-effort by design; live-fallback to Postgres ILIKE on people index) |
| Immediate mitigation | Scale indexer workers per affected index; enable Postgres-ILIKE fallback path for people-search (most-searched surface); pause low-priority tenants' indexing |
| RTO | ≤ 1 h |
| Runbook | `runbooks/feed-cache-rebuild.md` §"search degraded" section (paired pattern) |
| Postmortem | axis-network |

## FM-07: Cross-tenant connection-graph leak (RLS misconfig)

| Field | Value |
|---|---|
| Trigger | Helm config change disables RLS; live mutation bypass; bug in app code skips RLS GUC |
| Detection | `oya-check-postgres-rls-coverage` lane fails OR `network_cross_tenant_leak_detector_total` > 0 |
| Tenant impact | Potential cross-tenant connection-graph or endorsement exposure |
| Severity | Sev-1 (security breach; B2B trust at stake) |
| Immediate mitigation | Auto-rollback to last green Helm; isolate cluster; declare breach-suspect; engage ops-security; GDPR Art. 33 clock starts |
| RTO | ≤ 5 min auto-rollback; investigation days |
| Runbook | `runbooks/connection-graph-corruption.md` §"cross-tenant" section |
| Postmortem | ops-security + axis-network |

## FM-08: Endorsement storm (one user endorses thousands)

| Field | Value |
|---|---|
| Trigger | Power-user or bot script endorses many connections in burst |
| Detection | `network_endorsement_add_per_user_per_minute` > threshold (default 100/min) |
| Tenant impact | Endorsement fanout backlog; endorsement-chain seal worker lag; notification storm |
| Severity | Sev-3 (degraded; auto-recover) — Sev-2 if persistent |
| Immediate mitigation | Per-user endorsement-rate cap; throttle endorsement-chain seal batch size up to absorb; drop low-priority notification reps |
| RTO | ≤ 15 min |
| Runbook | `runbooks/endorsement-storm-throttle.md` |
| Postmortem | axis-network |

## FM-09: Media malware (scanner positive)

| Field | Value |
|---|---|
| Trigger | OPSWAT / ClamAV verdict positive on uploaded blob (image / video / document) |
| Detection | `network_media_malware_detected_total` > 0 |
| Tenant impact | Blob quarantined; sender notified; viewers see "media removed" placeholder |
| Severity | Sev-2 (if isolated) — Sev-1 (if pattern suggests organised attack or weaponised document) |
| Immediate mitigation | Quarantine bucket holds blob; never copies to production; audit-chain seal of detection; tenant security-admin notified |
| RTO | immediate (detection is preventative) |
| Runbook | `runbooks/feed-cache-rebuild.md` §"media-malware quarantine" section (paired pattern) |
| Postmortem | ops-security + axis-network |

## FM-10: Cross-context routing violation (parallel ADR-0126 invariant breach)

| Field | Value |
|---|---|
| Trigger | Bug allows Personal post / Personal user to surface in `network` Professional context |
| Detection | `network_professional_context_violation_total` > 0 OR LEAN lane fails on PR |
| Tenant impact | Privacy violation: personal data in Professional B2B scope |
| Severity | Sev-1 (regulatory breach; GDPR / PIPA scope; B2B trust violation) |
| Immediate mitigation | Block service write path; quarantine affected entities; declare breach; engage council-privacy + ops-security |
| RTO | ≤ 15 min block; investigation days |
| Runbook | `runbooks/connection-graph-corruption.md` §"cross-context" section |
| Postmortem | council-privacy + ops-security + axis-network |

## FM-11: Notification fanout backlog

| Field | Value |
|---|---|
| Trigger | Celebrity Page or 300k+ connection user posts; cascading fanout; endorsement storm cascade |
| Detection | `network_notification_fanout_queue_depth` > 500k sustained |
| Tenant impact | Notification delivery delayed (best-effort); no message loss |
| Severity | Sev-3 |
| Immediate mitigation | Scale notification workers; coalesce more aggressively into digest; pause low-priority push channel |
| RTO | ≤ 30 min |
| Runbook | `runbooks/feed-cache-rebuild.md` §"notification backlog" section (paired pattern) |
| Postmortem | axis-network |

## FM-12: Four-eyes disclosure mis-pairing (insider attempt)

| Field | Value |
|---|---|
| Trigger | Same principal attempts to satisfy both halves of four-eyes; programmatic spoofing of paired-approver-id |
| Detection | `network_four_eyes_pairing_violation_total` > 0 |
| Tenant impact | Disclosure attempt blocked; alerting on suspicious activity |
| Severity | Sev-1 (insider-malicious threat actor signal) |
| Immediate mitigation | Cedar evaluator denies; audit-chain seal of attempt; ops-security engaged |
| RTO | immediate (preventative) |
| Runbook | `runbooks/connection-graph-corruption.md` §"four-eyes violation" section |
| Postmortem | ops-security |

## FM-13: Cross-pack residency misroute

| Field | Value |
|---|---|
| Trigger | Bad Helm overlay deploys pack-X tenant into pack-Y cluster; Cedar pack-router bug |
| Detection | Periodic residency audit: `network_pack_residency_violation_total` > 0 |
| Tenant impact | Tenant data in wrong jurisdiction → regulatory breach |
| Severity | Sev-1 (GDPR / PIPA / HIPAA breach risk) |
| Immediate mitigation | Halt writes for affected tenant; migrate data back to correct pack; engage council-privacy |
| RTO | ≤ 30 min halt; migration may take hours |
| Runbook | `runbooks/connection-graph-corruption.md` §"pack residency" section |
| Postmortem | council-privacy + ops-security + axis-network |

## FM-14: Endorsement-chain integrity compromise

| Field | Value |
|---|---|
| Trigger | Merkle root mismatch detected on audit-chain replay; per-endorser Ed25519 signature verification fails at scale; possible signing-key compromise |
| Detection | `network_endorsement_chain_integrity_failure_total` > 0; `network_endorsement_signature_verify_failure_rate` > 0.01% |
| Tenant impact | Endorsement display marked "integrity_under_verification"; tenant B2B trust impacted |
| Severity | Sev-1 (signing-chain compromise + identity-attestation reliability) |
| Immediate mitigation | Quarantine affected partition; re-derive chain from audit-chain replay; verify Ed25519 keystore (KMS audit) for compromise indicators |
| RTO | ≤ 30 min quarantine; verification + restore may take hours |
| Runbook | `runbooks/connection-graph-corruption.md` §"endorsement-chain integrity" section (paired) |
| Postmortem | ops-security + axis-network + axis-audit-chain |

## FM-15: Recruiter-stub bias-audit failure

| Field | Value |
|---|---|
| Trigger | EU AI Act + EEOC UGESP 4/5-rule statistical disparity ratio falls below 0.8 in production usage; classifier drift triggers bias regression |
| Detection | `network_recruiter_bias_audit_disparity_ratio{group="<protected_group>"}` < 0.8 sustained ≥ 1h |
| Tenant impact | Possible employment-decision impact for affected protected groups |
| Severity | Sev-1 (regulatory: EU AI Act Art. 73 serious-incident + NYC LL144 + EEOC + CA AB-331 + CO SB 24-205) |
| Immediate mitigation | Auto-rollback recruiter-stub to last-known-good version; pause for NYC + CA + CO tenants; engage ComplianceLead + Privacy Lead; affected-candidate notifications scheduled |
| RTO | ≤ 15 min rollback; investigation days; re-audit before redeployment |
| Runbook | `runbooks/recruiter-classifier-rollback.md` |
| Postmortem | ops-compliance + council-privacy + axis-network + axis-foundry-runtime |

## FM-16: Minor-account leak / pivot

| Field | Value |
|---|---|
| Trigger | Unauthorised read of minor-account profiles OR DSR cascade leaks minor flag |
| Detection | Periodic access-audit of minor flag; non-entitled access detected |
| Tenant impact | Child-protection privacy breach |
| Severity | Sev-1 (regulatory: COPPA / GDPR Art. 8 / KR 청소년 보호법) |
| Immediate mitigation | Revoke unauthorised access; audit-chain seal; council-privacy + ops-security engaged; regulator notification |
| RTO | immediate (preventative) |
| Runbook | `runbooks/connection-graph-corruption.md` §"minor leak" section (paired pattern) |
| Postmortem | council-privacy + ops-security + axis-network |

## FM-17: Recommender drift (PYMK / ranker mass false-positive)

| Field | Value |
|---|---|
| Trigger | Foundry-runtime classifier verdict-rate spikes on PYMK / ranker output distribution; demographic skew detected |
| Detection | `network_recommender_output_drift_total` > 5× baseline for ≥ 10 min OR bias-audit dashboard flags drift |
| Tenant impact | Mass low-quality suggestions; tenant escalations; possible regulatory non-compliance |
| Severity | Sev-2 (drift) → Sev-1 (mass discriminatory-impact event) |
| Immediate mitigation | Roll back ranker version; pause T2 auto-ranking per-pack via Cedar entitlement revoke; restore chronological default |
| RTO | ≤ 15 min |
| Runbook | `runbooks/recruiter-classifier-rollback.md` §"ranker fallback" section (paired pattern) |
| Postmortem | axis-network + axis-foundry-runtime + ops-security + council-privacy |

## FM-18: Trending-topic poisoning (sybil-amplified hashtag in Professional context)

| Field | Value |
|---|---|
| Trigger | Coordinated sybil amplification injects artificial Professional trend (e.g., fake skill / fake company campaign) |
| Detection | foundry-guardrails sybil detector signal + `network_trending_topic_anomaly_total` > 0 |
| Tenant impact | Trending-topic feed shows manipulated content |
| Severity | Sev-3 (auto-recover via sybil filter); Sev-2 if scale > tenant-scope |
| Immediate mitigation | Sybil-detector verdict applied; tenant-admin can pin / unpin; per-author influence cap in trending recompute |
| RTO | ≤ 30 min |
| Runbook | `runbooks/endorsement-storm-throttle.md` §"trending poisoning" section (paired) |
| Postmortem | axis-network + ops-security + axis-foundry-guardrails |

## FM-19: Jobs-handoff bridge to ATS degraded

| Field | Value |
|---|---|
| Trigger | ATS µservice (Tier G) unreachable; contract-version mismatch on event handoff |
| Detection | `network_ats_bridge_queue_depth` > 50k OR `network_ats_bridge_contract_mismatch_total` > 0 |
| Tenant impact | Job-postings queue; applicant referrals delayed; tenant ATS pipelines stall |
| Severity | Sev-2 |
| Immediate mitigation | Hold queue in Redis Streams; replay on ATS recovery; emit ContractMismatch event to ops-architecture if version drift |
| RTO | ≤ 30 min queue drain post-recovery |
| Runbook | `runbooks/jobs-handoff-ats-failure.md` |
| Postmortem | axis-network + axis-ats |

## FM-20: Capacity exhaustion (per-cell limit hit)

| Field | Value |
|---|---|
| Trigger | Tenant growth or burst exceeds per-cell envelope; cardinality breach |
| Detection | `network_accounts_per_tenant_total` > 1M OR `network_active_connections_total` > 5M per cell |
| Tenant impact | New accounts rejected for affected tenant; new connections rejected |
| Severity | Sev-3 |
| Immediate mitigation | Shard tenant to new cell; raise per-tenant cap (with FinOps approval); notify gtm-customer-success |
| RTO | ≤ 4 h shard migration |
| Runbook | `runbooks/feed-cache-rebuild.md` §"cell shard" section (cell µservice authoritative) |
| Postmortem | ops-sre-reliability + axis-network |

## FM-21: Ontology lookup failure breaks mention-resolution

| Field | Value |
|---|---|
| Trigger | Ontology µservice outage |
| Detection | `network_mention_resolution_failure_rate` > 5 % for ≥ 2 min |
| Tenant impact | @mentions of Person / Company / Skill display as raw text; no notification fanout |
| Severity | Sev-3 (graceful degradation; mention still posts as text) |
| Immediate mitigation | Cache last known ontology graph; degrade to raw-text mode; reconcile when ontology returns |
| RTO | ≤ 30 min |
| Runbook | `runbooks/connection-graph-corruption.md` §"ontology degraded" section (paired pattern) |
| Postmortem | axis-network + axis-ontology |

## FM-22: InMail-bridge to messenger degraded

| Field | Value |
|---|---|
| Trigger | messenger µservice unreachable or rate-limit refusing; per-tenant spam-classifier rejecting |
| Detection | `network_inmail_bridge_queue_depth` > 100k OR `network_inmail_send_failure_rate` > 5% |
| Tenant impact | InMail sends queue; users see "delivery pending" UI |
| Severity | Sev-2 |
| Immediate mitigation | Hold in Redis Streams; surface backlog UI; replay on messenger recovery; honor spam-classifier verdicts |
| RTO | ≤ 30 min queue drain |
| Runbook | `runbooks/inmail-fanout-degraded.md` |
| Postmortem | axis-network + axis-messenger |

## FM-23: Profile-export vCard 4.0 / JSON Resume corruption

| Field | Value |
|---|---|
| Trigger | Bug in vCard 4.0 emitter; JSON Resume schema drift; PII redactor mis-applies |
| Detection | `network_profile_export_corruption_total` > 0; user-reported corrupt downloads |
| Tenant impact | GDPR Art. 20 portability obligation degraded; tenant cannot rely on export integrity |
| Severity | Sev-2 (Art. 20 obligation degraded) |
| Immediate mitigation | Disable corrupt emitter path; revert to last-known-good; regenerate affected exports server-side; notify affected users |
| RTO | ≤ 1 h |
| Runbook | `runbooks/profile-export-vcard-corruption.md` |
| Postmortem | axis-network + council-privacy |
