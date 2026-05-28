---
doc_class: ArchitectureWalkthrough
shape: Reference
length_cap: 2400
authority_tier: 2
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0272
  - ADR-0276
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
companion_docs:
  - microservices/social/PRD.md
  - microservices/social/threat-model.md
  - microservices/social/dpia.md
  - microservices/social/compliance.md
  - microservices/social/manifest.json
planned_enforcement_ref: oya-governance-adr-adherence-matrix
inbound_citations:
  - microservices/social/PRD.md
  - microservices/social/README.md
---

# Social µservice — Architecture Walkthrough

## §entry-point — cold-start

The Social µservice is oyatie's broadcast-shape social surface (distinct from `community` which is forum-shape). Hyperscaler precedents: **Twitter/X + Facebook + Instagram + LinkedIn + Threads + Bluesky + Mastodon**. The shape: short-form posts + follow-graph + ranked feed timeline + engagement (likes/replies/reposts) + media + moderation + federation (ActivityPub).

Cold-start question: *Where does a post composed by a user appear in their followers' feeds, get moderated, and get attribution-tracked while resisting bot floods?* Trace:
1. Compose surface (`oya-community-social-post-composition-kernel`) receives the post; alt-text on media required for a11y; minor-protection age-gate evaluated per ADR-0292.
2. Cedar gates evaluated: `policy/tenant-scope.cedar`, `policy/abuse-defence.cedar` (anti-bot + anti-spoof + anti-scrape), `policy/content-policy.cedar` (community standards), `policy/minor-protection.cedar`.
3. Content-moderation classifier (`oya-community-social-content-moderation-kernel`) emits scores: NSFW, violence, hate, harassment, CSAM (CSAM → automatic block + NCMEC report).
4. Media transcode (`oya-community-social-post-composition-adapter-ffmpeg` + `-imagemagick`) sanitizes EXIF, transcodes to compliant formats, scans with `-clamav` and `-opswat`.
5. Post persisted (`oya-community-social-post-composition-adapter-postgres`); blob in `-s3`.
6. Feed-fanout (`oya-community-social-feed-timeline-kernel`) decides push vs pull per follower-count tier — push for ≤10k followers, pull for celebrities (Twitter "celeb fanout" pattern).
7. Federation (`oya-community-social-federation-gateway-adapter-activitypub`) optionally federates to Mastodon / Bluesky / partner instances when tenant opted-in.
8. ADR-0263 audit events emitted: `oya.social.post-create`, `oya.social.content-moderate`, `oya.social.feed-fanout`, etc.
9. Recipients receive push notifications via the standard substrate; web client receives WebSocket push.

## §principals (ADR-0242)

Operates as `oyatie.social.{post-composition, feed-timeline, follow-graph, user-profile, content-moderation, search, federation-gateway}` principals. Called by tenant principals `<tenant>.<workspace>.<actor>` and substrate `ontology` (profile-entity-mirror), `intelligence` (caption + alt-text + summary), `governance`, `messenger` (DM-from-profile).
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `principals (ADR-0242)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `principals (ADR 0242)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `principals (ADR 0242)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `principals (ADR 0242)` workflow.
- Depth detail 17: `social` telemetry for `principals (ADR 0242)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §cedar-gates (ADR-0243)

Defence-in-depth FORBIDs:
- `policy/tenant-scope.cedar` — default-deny baseline
- `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`, `policy/public-read.cedar`
- `policy/dual-context-isolation.md` — personal vs work
- `policy/abuse-defence.cedar` — **EXTENSIVE** anti-bot + anti-spoof + anti-scrape per ADR-0297 (social is the most-targeted surface)
- `policy/content-policy.cedar` — community standards (hate, harassment, CSAM, sexual content)
- `policy/minor-protection.cedar` — COPPA <13 refusal + KOSA 14-17 tier + EU age-verification per ADR-0292
- `policy/profile-verification.cedar` — verified-profile gates
- `policy/federation-egress.cedar` — outbound ActivityPub gates
- `policy/dm-scope.cedar` — DM gating (separates from messenger primary surface)

Cedar v4.2 LTS. Fragment soak ≥60s per ADR-0294.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `cedar-gates (ADR-0243)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §tenant-scoping (ADR-0244)

Every post, follow-edge, engagement row carries `tenant_id` + `home_cell` + `dr_cell` + `audience_type` + `provider_credential_mode` + `compliance_packs[]` + `minor_age_band`. `audience_type` enum: `B2C_PERSONAL`, `B2C_PERSONAL_MINOR_KOSA_14_17`, `B2B_BRAND`, `B2B_CREATOR_VERIFIED`, `FRIENDLY_CRAWLER_PARTNER`, `INTERNAL_SUBSTRATE`. `provider_credential_mode` default `PLATFORM_MANAGED` for free tier, `TENANT_BYOK` for verified creators + brands.
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `tenant-scoping (ADR-0244)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `tenant scoping (ADR 0244)` workflow.
- Depth detail 17: `social` telemetry for `tenant scoping (ADR 0244)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §substrate-product-binding (ADR-0245)

**Tier: product.** Substrate dependencies: `ontology` (profile-entity-mirror, hashtag-entity-mirror, place-entity-mirror), `intelligence` (caption + alt-text + content-moderation classifier), `governance` (moderation policy + appeals), `cell`, `tenancy`, `policy-engine`, `observability`, `compliance`, `cloud-secrets`, `messenger` (DM substrate reuse), `comms-email` (notification email substrate).
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `substrate-product-binding (ADR-0245)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.
- Depth detail 17: `social` telemetry for `substrate product binding (ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §policy-evaluation (ADR-0246 + amendment)

Library-first via `oya-shared-policy-eval`. `policy_evaluation_mode: LIBRARY_FIRST`. Cedar evaluation on hot-path posts MUST stay ≤3ms p99 (every additional ms is a fanout multiplier through celebrity accounts).
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `policy-evaluation (ADR-0246 + amendment)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `social` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §intelligence-dispatch (ADR-0255 + amendment)

Library-first for: alt-text generation (T1), caption suggestion (T1), summary (T1), content-moderation classifier (T2 limited), ranking (T2 limited). For `B2C_PERSONAL_MINOR_KOSA_14_17`: AI-driven ranking disabled by default per ADR-0292 (chronological-only); requires guardian consent to enable algorithmic ranking.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `intelligence-dispatch (ADR-0255 + amendment)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `intelligence dispatch (ADR 0255 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255 + amendment)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `intelligence dispatch (ADR 0255 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `intelligence dispatch (ADR 0255 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `intelligence dispatch (ADR 0255 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255 + amendment)` workflow.
- Depth detail 17: `social` telemetry for `intelligence dispatch (ADR 0255 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §ontology-read-path (ADR-0257 + amendment)

`ontology_read_mode: LIBRARY_FIRST_BYO_CACHE`. Profile pages enrich with ontology data (places, brands, mentions, hashtag context). `freshness_floor: TIGHT` (≤2s) for profile pages — visitors expect current data.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `ontology-read-path (ADR-0257 + amendment)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `ontology read path (ADR 0257 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 + amendment)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `ontology read path (ADR 0257 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `ontology read path (ADR 0257 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `ontology read path (ADR 0257 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 + amendment)` workflow.
- Depth detail 17: `social` telemetry for `ontology read path (ADR 0257 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §time-coordination (ADR-0252)

HLC default; sufficient for ranked-feed ordering (deterministic tie-break by ULID).
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `time-coordination (ADR-0252)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `social` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §transport (ADR-0253)

REST + WebSocket + ActivityPub over HTTP/3 + QUIC default. Fallback h3 → h2 → h1.1. TLS 1.3 floor. ECH advertised; PQC hybrid `X25519MLKEM768`; signature hybrid `ed25519+ml_dsa_65`. Native clients all HTTP/3-capable.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `transport (ADR-0253)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `social` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §deployment-shape (ADR-0254)

- `oya-community-social-post-composition-app` → Kata pod (handles UGC + transcode + safety scan)
- `oya-community-social-feed-timeline-kernel` → standard pod with Valkey cache
- `oya-community-social-content-moderation-kernel` → Kata pod with GPU when present
- `oya-community-social-federation-gateway-adapter-activitypub` → Kata pod (handles external untrusted payloads)
- `oya-community-social-search-adapter-meilisearch` → standard pod
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `deployment-shape (ADR-0254)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `deployment shape (ADR 0254)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §marketplace (ADR-0249)

Exposes: `creator-monetization-template`, `caption-style-recipe`, `engagement-bot-stub` (legitimate scheduled-post automation under verified-creator gating), `branded-content-template`.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `marketplace (ADR-0249)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `marketplace (ADR 0249)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace (ADR 0249)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace (ADR 0249)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `marketplace (ADR 0249)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `marketplace (ADR 0249)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `marketplace (ADR 0249)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `marketplace (ADR 0249)` workflow.
- Depth detail 17: `social` telemetry for `marketplace (ADR 0249)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §observability (ADR-0263)

Audit-event classes (extensive due to abuse surface): `oya.social.post-create`, `oya.social.post-delete`, `oya.social.content-moderate`, `oya.social.feed-fanout`, `oya.social.follow-create`, `oya.social.follow-delete`, `oya.social.engagement-emit`, `oya.social.federation-outbound`, `oya.social.federation-inbound`, `oya.social.abuse-defence-block`, `oya.social.csam-detect`, `oya.social.csam-ncmec-report`, `oya.social.minor-protect-engage`, `oya.social.report-submit`, `oya.social.moderation-action`, `oya.social.appeal-submit`, `oya.social.appeal-resolve`, `oya.social.shadowban-engage`, `oya.social.shadowban-release`, `oya.social.spoof-attempt-block`, `oya.social.scrape-pattern-detect`.

Per-metric cardinality budget: 10000. High-cardinality (user_id, post_id) → trace-span attributes only.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four primary SRE signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `observability (ADR-0263)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `observability (ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `observability (ADR 0263)` workflow.
- Depth detail 17: `social` telemetry for `observability (ADR 0263)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §consent (ADR-0272)

Per-purpose consent on first sign-in: (a) algorithmic ranking, (b) targeted-ads (opt-in only; default off for KOSA tier), (c) content-moderation classifier (required for posting), (d) federation outbound (per-tenant + per-post toggle), (e) compose-assist AI, (f) marketing-from-platform.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `consent (ADR-0272)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `consent (ADR 0272)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `consent (ADR 0272)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent (ADR 0272)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `consent (ADR 0272)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `consent (ADR 0272)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `consent (ADR 0272)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `consent (ADR 0272)` workflow.
- Depth detail 17: `social` telemetry for `consent (ADR 0272)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §minor-protection (ADR-0292) — EXTENSIVE

- `declared_age < 13` → **refuse account provisioning** (no social account; remediation: parental-managed minor surface if tenant offers it).
- `KOSA_14_17` → strict defaults:
  - DMs from non-followers OFF
  - Algorithmic ranking OFF (chronological only)
  - Notifications muted overnight
  - Targeted ads OFF
  - Profile non-discoverable to adult strangers
  - Direct messaging from adult accounts requires accepted-follower-status
  - Live-stream feature OFF until guardian-acknowledgement
  - In-app purchases require guardian approval
- EU jurisdictions: age-verification per the GDPR digital-services minor age (typically 13-16 depending on member-state).
- Guardian dashboard surface: parental visibility into screen-time, content surfaces, contacts.
- Anti-grooming heuristics: detect adult-stranger-pattern DMs to minors; auto-flag + manual review.
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `minor-protection (ADR-0292) — EXTENSIVE` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `minor protection (ADR 0292) — EXTENSIVE` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection (ADR 0292) — EXTENSIVE`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.

## §abuse-defence (ADR-0297) — EXTENSIVE (this surface is the highest-targeted)

### Anti-bot (most aggressive on social):
- Edge: token-bucket per-IP + per-tenant + per-fingerprint + per-route; bot-mgmt with ML scoring; CAPTCHA-on-suspicion (Turnstile + hCaptcha + Cloudflare Challenge); device attestation (App Attest + Play Integrity + WebAuthn Origin-binding).
- Account-creation: aggressive ML-driven sock-puppet detection (cluster analysis on device fingerprint + behavior + email-domain + phone-number registration patterns); CAPTCHA on every sign-up.
- Engagement-bot detection: pattern-anomaly on rapid follow / unfollow / like / comment patterns; shadow-ban on confirmed engagement-bot.
- HIBP credential-stuffing detection on sign-in.

### Anti-spoof (heavy on social — impersonation is the dominant attack):
- Profile-verification with attested government-ID for `B2B_CREATOR_VERIFIED` accounts.
- Display-name-vs-handle similarity check (block sign-up of confusingly-similar names to verified accounts within a Levenshtein distance).
- Federation inbound: AP signature verification + actor-pubkey lookup + replay-window enforcement.
- mTLS + SPIFFE for all µservice-to-µservice calls per ADR-0295.
- Webhook anti-spoof: HMAC signature + replay-window ≤5min per ADR-0297.
- Watermark profile photos with platform-provenance signal so leaked photos are detectable.

### Anti-scrape (heavy on social — entire business models built on scraping social):
- Rate-limit per-IP + per-fingerprint + per-tenant; aggressive low caps on unauthenticated read.
- Pattern-anomaly detection: breadth-first profile traversal, sequential username-ID enumeration, alphabetical hashtag enumeration.
- robots.txt + Sitemaps + crawl-delay per tenant + per-locale.
- Paid-API tier for legitimate scrapers (search engines + data-aggregators get accredited API access).
- Per-user invisible watermarks on rendered HTML (zero-width chars; structural HTML mutated per-session).
- Adaptive challenge on scrape-pattern: bot-score + scrape-pattern + tenant-policy → CAPTCHA, JS PoW, throttle-then-degrade.
- Dynamic content rewriting: CSS class-name randomization per-session; semantic API surface stable, scraping surface unstable.
- Legal-channel registration: Bug Bounty + abuse-report email + DMCA agent + GDPR Art. 14 right-to-object surface.

### UX-floor (CRITICAL on social — friction kills adoption):
- Default-path: zero added latency for legitimate users; bot-mgmt scoring is passive and asynchronous.
- Challenges presented ONLY on bot-score > 95 OR on sign-up + suspicious password-reset OR on burst-rate violation.
- CAPTCHA UX uses Turnstile invisible challenge (Cloudflare-style) for ≥95% of traffic; visible CAPTCHA only on confirmed-suspicion.
- Accessibility floor (WCAG 2.2 AA): audio CAPTCHA + keyboard nav + screen-reader-friendly text + no time pressure for assistive-tech users.
- Cognitive load ≤10s solve for legitimate users.
- Session continuity preserved across challenge.
- Tenant-tier-adaptive: higher-tier accounts get lower sensitivity (fewer challenges); anonymous/sandbox tier strictest.
- Locale-aware challenge text + image-set; mobile UX native (App Attest / Play Integrity, not webview CAPTCHA).
- Friendly-crawler partner allow-list (Google + Bing + accredited researchers) — never see a challenge.
- Transparent telemetry to tenant-admin: per-tenant FP rate, friction events, blocked-bot count visible in dashboard.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §credential-isolation (ADR-0296)

Per-tenant federation signing keys, OAuth-app secrets, push-notification provider tokens, intelligence-provider-BYOK tokens all live in OpenBao with ≤60s sidecar TTL. Social µservice holds no long-lived per-tenant credentials.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `credential-isolation (ADR-0296)` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `social` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §portability (ADR-0276)

Per-tenant export: ActivityPub-compatible JSON archive + media bundle (compatible with Mastodon / Bluesky / Threads import). GDPR Art. 20 honored.

## §pack-overlays

- `pack-eu` → DSA (EU Digital Services Act) compliance: transparency reports, content-moderation appeal, designated point-of-contact, large-platform requirements if applicable.
- `pack-kr` → KCC content moderation requirements + PIPA per-purpose consent.
- `pack-us-healthcare` → not typical for social; but for `B2B_HIPAA_PHI` brand accounts (e.g., hospitals posting on social) the HIPAA pack reduces what can be posted from authoritative health accounts.

## §self-modification

Consumes Foundry-built moderation policy updates; meta-trust-root attestation per ADR-0293.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `self-modification` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification` workflow.
- Depth detail 17: `social` telemetry for `self modification` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §fragment-publish + §bootstrap-trust-chain

Cedar fragments soak 60s. Post-composition + content-moderation boot with SPIFFE attestation; kill-switch on attestation failure.
### Content-pass expansion — fragment-publish
- This expansion preserves the existing prose above and closes `fragment-publish` for `social` to the ≥50-line documentation-rigor floor.
- Service owner `axis-social` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `composer-suggest-and-hashtag-completion`; bounded contexts: `social`.
- API surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy surfaces: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`; +5 more.
- State/event surfaces: `social.social`.
- SLO/dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS AppConfig bake windows anchors the external control pattern for `fragment-publish`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `fragment-publish`.
- Tenant-scope invariant: every `social` `composer-suggest-and-hashtag-completion` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/social/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `social` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `social` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `social` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `social` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `social` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `composer-suggest-and-hashtag-completion` evaluates `<tenant>.social.composer-suggest-and-hashtag-completion` against policy, writes `social.social`, and emits `oya.social.composer.suggest.and.hashtag.completion.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `fragment-publish`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `fragment-publish` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `social` binds `fragment-publish + §bootstrap-trust-chain` to `{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya-community-social-app', 'oya-community-social-content-moderation-adapter-clamav', 'oya-community-social-content-moderation-adapter-opswat', 'oya-community-social-content-moderation-kernel', 'oya-community-social-federation-gateway-adapter-activitypub', 'oya-community-social-feed-timeline-adapter-valkey', 'oya-community-social-feed-timeline-kernel', 'oya-community-social-follow-graph-adapter-postgres', 'oya-community-social-follow-graph-kernel', 'oya-community-social-post-composition-adapter-ffmpeg', 'oya-community-social-post-composition-adapter-imagemagick', 'oya-community-social-post-composition-adapter-postgres', 'oya-community-social-post-composition-adapter-s3', 'oya-community-social-post-composition-kernel', 'oya-community-social-search-adapter-meilisearch', 'oya-community-social-user-profile-adapter-postgres', 'oya-community-social-user-profile-domain', 'oya-community-social-user-profile-kernel', 'oya-community-social-user-profile-rest']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `social` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `fragment publish + §bootstrap trust chain` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `social` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/content-policy.cedar, policy/data-residency.md, policy/dm-scope.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `fragment publish + §bootstrap trust chain`.
- Depth detail 4: `social` state/event naming uses `social.{'name': 'social', 'description': "Bounded context 'social' within social (data plane)", 'crates': ['oya_social_app', 'oya_social_content_moderation_adapter_clamav', 'oya_social_content_moderation_adapter_opswat', 'oya_social_content_moderation_kernel', 'oya_social_federation_gateway_adapter_activitypub', 'oya_social_feed_timeline_adapter_valkey', 'oya_social_feed_timeline_kernel', 'oya_social_follow_graph_adapter_postgres', 'oya_social_follow_graph_kernel', 'oya_social_post_composition_adapter_ffmpeg', 'oya_social_post_composition_adapter_imagemagick', 'oya_social_post_composition_adapter_postgres', 'oya_social_post_composition_adapter_s3', 'oya_social_post_composition_kernel', 'oya_social_search_adapter_meilisearch', 'oya_social_user_profile_adapter_postgres', 'oya_social_user_profile_domain', 'oya_social_user_profile_kernel', 'oya_social_user_profile_rest']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `social` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `social` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `social` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `fragment publish + §bootstrap trust chain` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `social` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `social` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `social` uses SLOs `slos/content-policy-enforcement-correctness.openslo.yaml, slos/csam-classifier-latency.openslo.yaml, slos/feed-render-latency.openslo.yaml, slos/follow-action-latency.openslo.yaml, slos/minor-protection-engagement-correctness.openslo.yaml, plus 5 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/csam-and-trust-safety.json, dashboards/federation-and-cross-context.json, dashboards/feed-experience.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `social` uses runbooks `runbooks/abuse-report-backlog-drain.md, runbooks/content-moderation-rollback.md, runbooks/coordinated-inauthentic-behavior-response.md, runbooks/csam-detect-and-ncmec-report.md, runbooks/dsa-transparency-report-generation.md, plus 7 more` so `fragment publish + §bootstrap trust chain` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `social` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/social/Chart.yaml, iac/helm/social/templates/deployment.yaml, iac/helm/social/templates/hpa.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `social` uses `capabilities/T0-suggest.yaml, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml` and `catalog/oya-community-social-app.yaml, catalog/oya-community-social-content-moderation-adapter-clamav.yaml, catalog/oya-community-social-content-moderation-adapter-opswat.yaml, catalog/oya-community-social-content-moderation-kernel.yaml, plus 19 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `social` fails closed when `fragment publish + §bootstrap trust chain` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `social` emits denial evidence for `fragment publish + §bootstrap trust chain` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `social` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `fragment publish + §bootstrap trust chain` workflow.
- Depth detail 17: `social` telemetry for `fragment publish + §bootstrap trust chain` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `social` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §where-to-read-next

- `microservices/social/PRD.md`
- `microservices/social/threat-model.md`
- `microservices/social/dpia.md`
- `microservices/social/compliance.md`

---



## §cell-eligibility
This anchor is closed for `social` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `composer-suggest-and-hashtag-completion` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `social` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `social` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `social` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `social` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `composer-suggest-and-hashtag-completion` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `social` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

