---
doc_class: FailureModes
title: Failure Modes + Effects Analysis (FMEA)
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-shorts + ops-security
deciders: ops-sre-reliability, axis-shorts, council-architecture, ops-security, council-privacy
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/shorts/threat-model.md
  - microservices/shorts/incident-response.md
  - microservices/shorts/runbooks/
review_cadence: quarterly + post-incident
doc_status: published
---

# Failure Modes + Effects Analysis (shorts µservice)

## Purpose

Enumerate every load-bearing failure mode, its detection signal, blast-radius, and runbook pointer. Anchors burn-rule design in `capacity-model.md` + `incident-response.md`.

## Severity Legend

- **SEV-1**: hero-product down / security invariant breach / regulatory clock running
- **SEV-2**: user-visible degradation; reversible
- **SEV-3**: internal-only; user-invisible
- **SEV-4**: tracking-only

## Failure Modes

| FM | Description | Detection | Severity | Blast radius | Runbook |
|---|---|---|---|---|---|
| FM-01 | Feed cache cold-restart | `oya_shorts_feed_cache_hit_ratio` < 70 % | Sev-2 | per-pack viewers; up to 30 min latency degradation | runbooks/feed-cache-rebuild.md |
| FM-02 | Postgres primary failure | `up{job="shorts-postgres-primary"} == 0` | Sev-1 | per-pack videos write-blocked ≤ 5 min during failover | cell/runbooks/postgres-primary-failover.md |
| FM-03 | Redis split-brain | sentinel quorum lost | Sev-1 | feed cache + watch-position + likes counters; rebuild | runbooks/feed-cache-rebuild.md |
| FM-04 | Transcode queue backup | `oya_shorts_transcode_queue_depth` > 1000 sustained 5min | Sev-2 | upload→playable latency degrades; affects new uploads | runbooks/transcode-queue-backup.md |
| FM-05 | ffmpeg worker CVE → RCE | Trivy/Grype CVE scan + worker SBOM | Sev-1 | quarantine; rebuild from LTS | runbooks/transcode-queue-backup.md (sandbox section) |
| FM-06 | CDN POP failure (single POP) | Cloudflare health-check failure | Sev-3 | latency for affected region; auto-failover to nearest healthy POP | runbooks/cdn-cache-invalidation-cascade.md |
| FM-07 | CDN cache invalidation cascade (mass-takedown) | `oya_shorts_cdn_invalidation_rate` spike | Sev-2 | global purge storm; latency for cold reads | runbooks/cdn-cache-invalidation-cascade.md |
| FM-08 | Copyright-claim storm (claimant bulk-files 10k claims) | `oya_shorts_copyright_claim_filings_per_sec` > rate limit | Sev-2 | claim worker queue depth; potential mass auto-hide false-positives | runbooks/copyright-claim-storm-throttle.md |
| FM-09 | Moderation classifier false-positive event | per-verdict drift rate > 5x baseline | Sev-1 | mass content auto-hides; EU AI Act Art. 73 evaluation | runbooks/moderation-classifier-rollback.md |
| FM-10 | Cross-context routing (Personal short → Professional feed) | `oya_shorts_dual_context_denied_total` > 0 | Sev-1 | data-model invariant breach; regulatory clock | (cordon + investigate; incident-response Sev-1) |
| FM-11 | Personal-tier federation attempt | `oya_shorts_personal_tier_federation_attempt_total` > 0 | Sev-1 | DCI-08 invariant breach | (cordon; investigate compile-time bypass) |
| FM-12 | Search-index lag during ingest spike | `oya_shorts_search_index_lag_seconds` > 60s | Sev-3 | search staleness; fallback to Postgres ILIKE | (degrade gracefully; backpressure) |
| FM-13 | Pack residency violation (cross-pack replication) | `oya_shorts_pack_residency_violation_total` > 0 | Sev-1 | GDPR/PIPA/LGPD breach clock | (cordon cross-pack; restore source-pack) |
| FM-14 | Minor-protection bypass attempt | `oya_shorts_minor_protection_bypass_attempt_total` > 0 | Sev-1 | child-protection breach; COPPA/GDPR Art. 8/KR 청소년 보호법 clock | runbooks/age-gate-bypass-incident.md |
| FM-15 | DRM key rotation failure | `oya_shorts_drm_key_rotation_failure_total` > 0 | Sev-1 | DRM-protected content playback degradation; potential key compromise | runbooks/drm-key-rotation.md |
| FM-16 | Notification fanout storm (celebrity post) | `oya_shorts_notification_fanout_lag_seconds` > 60s | Sev-2 | delayed delivery; coalesce digests; backpressure | (autoscale + coalesce; see capacity-model) |
| FM-17 | Fingerprint matcher latency tail | `oya_shorts_fingerprint_match_duration_p95` > 2s | Sev-3 | publication delays; pre-publication slowness | (autoscale; shard repartition) |
| FM-18 | Auto-caption ASR backlog | `oya_shorts_caption_generation_backlog` > 1000 jobs | Sev-3 | caption appears late; chronological fallback | (autoscale; foundry-runtime scaling) |
| FM-19 | Audio-track licensing-metadata mismatch | per-track license-status sanity check fails | Sev-2 | per-track usage may be unlicensed; risk of DMCA | (cordon affected tracks; ops-legal review) |
| FM-20 | Watch-time tampering (artificial inflation) | foundry-guardrails sybil detector trips | Sev-3 | ranking-signal pollution; affects discovery | (cap + flag; per-(viewer, video) idempotency) |
| FM-21 | Trending sound poisoning | trending-sound sybil detector trips | Sev-3 | spurious trending sounds; affects discovery | runbooks/trending-poisoning.md §sound-poisoning (sibling runbook in the same `microservices/shorts/runbooks/` directory; mitigation: cap + flag the offending sound + temporary trending-list withholding pending ops-trust review) |
| FM-22 | S3 quarantine bucket overflow | bucket capacity > 80% | Sev-3 | scan-pending blocked; uploads stall | (autoscale; purge clean-passed quarantined blobs) |
| FM-23 | Meilisearch shard saturation | `meilisearch_shard_size_bytes` > limit | Sev-3 | indexing stalls; search degraded | (shard split; rebuild) |
| FM-24 | OPSWAT scan SaaS outage | scan endpoint unreachable | Sev-2 | uploads stall in quarantine; fallback to ClamAV self-hosted | (failover to ClamAV; vendor escalation) |
| FM-25 | Foundry-runtime classifier endpoint outage | classifier endpoint unreachable | Sev-2 | moderation queue grows; fallback to manual review at threshold | runbooks/moderation-classifier-rollback.md (rollback to prior version path) |
| FM-26 | Federation peer revocation (peer leaves allowlist mid-flight) | peer-allowlist mismatch | Sev-3 | federation egress blocked to that peer; in-flight activities held | (verify allowlist; engage federation-gateway) |
| FM-27 | DRM license issuance overload | `oya_shorts_drm_license_request_rate` > capacity | Sev-2 | playback degradation for DRM-protected content | runbooks/drm-key-rotation.md (HA scaling section) |
| FM-28 | Audio fingerprint corpus poisoning | per-licensor namespace mismatch | Sev-1 | spurious copyright claims at scale | (cordon affected namespace; ops-legal review) |
| FM-29 | Eviction cascade (Redis eviction pressure) | Redis eviction-rate > 1% | Sev-2 | feed-cache freshness degraded; affects ranking | (memory shard scale-out) |
| FM-30 | DMCA designated-agent unavailability | ops-legal DMCA agent on PTO + no backup | Sev-3 | counter-notice processing delays; risks Safe Harbor | (backup agent designation; ops-legal rotation) |
| FM-31 | Audit-chain seal failure (sealing endpoint down) | `oya_audit_chain_seal_failure_total` > 0 | Sev-1 | state transitions unsealed; non-repudiation breach | audit-chain µservice runbook (cross-µservice) |
| FM-32 | Per-tenant DEK rotation failure (OpenBao) | OpenBao alert | Sev-1 | Professional video bodies un-decryptable until resolved | cloud-secrets µservice runbook |
| FM-33 | Cell boundary violation (per-tenant cell escape) | cell µservice alert | Sev-1 | tenant isolation breach risk | cell µservice runbook |
| FM-34 | Ontology cache lag (Person/Sound/Hashtag resolution stale) | `oya_ontology_resolution_lag_seconds` > 60s | Sev-3 | mention/sound attribution staleness | ontology µservice runbook |
| FM-35 | Workflow event-bus backpressure (AMQP queue depth) | event-bus depth > 10k | Sev-2 | downstream µservice lag (audit-chain, observability) | workflow-engine runbook |

## Cascade Hazards

| Origin | Cascade target | Mitigation |
|---|---|---|
| FM-02 Postgres primary | FM-03 Redis split-brain (cache invalidates) | failover scripted; cache rebuild lazy |
| FM-04 Transcode backup | FM-07 CDN invalidation cascade (mass purge on bulk publish post-thaw) | gradual thaw + rate-limit publish |
| FM-09 Classifier false-positive event | FM-08 Copyright-claim storm (creators may file false counter-storm) | manual moderator throttle; classifier-rollback fast path |
| FM-15 DRM key rotation failure | FM-27 DRM license overload (clients retry storm) | exponential backoff in client SDK |
| FM-25 Foundry classifier outage | FM-09 mass auto-hide cascading false-positive on rebuild | rollback to prior version with golden-set verification |

## References

- `microservices/shorts/threat-model.md`.
- `microservices/shorts/incident-response.md`.
- `microservices/shorts/runbooks/`.
- `microservices/shorts/capacity-model.md`.
- `microservices/social/failure-modes.md` (sibling pattern).
- NIST SP 800-61 Rev. 2.
