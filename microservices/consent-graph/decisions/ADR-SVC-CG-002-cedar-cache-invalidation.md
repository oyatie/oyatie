---
id: ADR-SVC-CG-002
title: "Consent Cedar cache invalidation is revocation-led and fail-closed"
status: Accepted
date: 2026-05-18
microservice: consent-graph
related_oyatie_adrs:
  - ADR-0003
  - ADR-0214
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0258
  - ADR-0263
decision_owner: axis-consent-graph + axis-policy-gateway
---

# ADR-SVC-CG-002: Consent Cedar cache invalidation is revocation-led and fail-closed

## Context

- The named architectural pressure is `revocation-freshness-over-cache-hit-rate`.
- Consent-graph is the source of authorization facts for data-sharing agreements.
- ADR-0243 makes Cedar the universal gate for policy decisions.
- ADR-0214 makes cross-tenant visibility conditional on real-time consent state.
- ADR-0244 makes tenant scoping a universal primitive.
- ADR-0251 requires compliance-pack cell boundaries to be explicit and testable.
- Prior incident class `revoked-agreement-cache-hit` allowed a grantee read after grantor revocation.
- Prior incident class `policy-bundle-stale-after-pack-update` evaluated a pre-amendment Cedar policy.
- Prior incident class `cross-region-cache-fanout-gap` invalidated one cell but left a peer cell hot.
- Prior incident class `negative-cache-replay` cached a deny and masked a later lawful grant.
- Consent revocation is legally sensitive under GDPR Art. 7(3), GDPR Art. 17, HIPAA §164.312(a)(1), KR PIPA Art. 37, and SOC 2 CC6.6.
- Grantor tenants expect revocation to stop downstream access quickly.
- Grantee tenants need deterministic failure modes rather than stale grants.
- Data subjects need evidence that withdrawal of consent is effective.
- The cache exists to keep Cedar hot-path checks below 1 ms p99.
- The cache must never become a second source of truth.
- The cache must carry policy bundle version, agreement version, and consent epoch.
- The cache must support same-region and cross-region invalidation.
- The cache must survive transient Apache Pulsar delivery lag.
- The cache must fail closed if freshness evidence is missing.
- The cache must avoid unbounded cardinality by hashing tenant pairs.
- The cache must expose metrics that prove revocation freshness.
- The cache must be implementable without privileged database reads in request handlers.
- The cache must support intern-buildability from this ADR.

## Decision

- We choose `revocation-led Cedar cache invalidation`.
- The named pattern is `versioned decision cache with monotonic consent epoch`.
- The cache key includes `tenant_pair_hash`.
- The cache key includes `agreement_id`.
- The cache key includes `subject_hash`.
- The cache key includes `sharing_mode`.
- The cache key includes `cedar_policy_bundle_version`.
- The cache key includes `consent_epoch`.
- The cache value stores allow or deny.
- The cache value stores `evaluated_at`.
- The cache value stores `expires_at`.
- The cache value stores `source_event_id`.
- The cache value stores `agreement_state`.
- The cache value stores `pack_id`.
- We use in-process LRU cache for sub-millisecond same-pod reads.
- We use Valkey 8.0 as the regional shared cache.
- We use Apache Pulsar 3.3.x for invalidation fanout.
- In-process cache TTL is 250 ms for allow.
- In-process cache TTL is 2 seconds for deny.
- Valkey allow TTL is 30 seconds.
- Valkey deny TTL is 10 seconds.
- Revocation invalidation bypasses TTL.
- Policy bundle invalidation bypasses TTL.
- Compliance-pack overlay invalidation bypasses TTL.
- Same-pod revocation invalidation target is 10 ms p99.
- Same-region revocation invalidation target is 500 ms p99.
- Cross-region revocation invalidation target is 1 second p99.
- A request handler must verify cache epoch against the revocation freshness table when cached allow age exceeds 200 ms.
- Missing freshness evidence fails closed.
- Pulsar topic `persistent://consent-graph/{pack_id}/cedar-cache-invalidations` carries invalidation events.
- Pulsar topic retention is 7 days.
- Pulsar subscription type is key-shared by `agreement_id`.
- Cache invalidation messages use RFC 8785 canonical JSON.
- Cache invalidation messages are Ed25519-signed by `consent-graph.policy-writer`.
- Cedar action `consent-graph.cache.invalidate` gates invalidation publication.
- Cedar action `consent-graph.enforcement.evaluate` gates policy evaluation.
- Cedar action `consent-graph.cache.replay` gates replay of retained invalidation messages.
- Cedar action `consent-graph.cache.inspect` gates operator introspection.
- Operators cannot force an allow by writing cache data.
- Operators can only delete cache data through the replayable invalidation API.
- We do not cache Cedar principal attributes directly.
- We cache only derived decision metadata and opaque subject hashes.

## Alternatives Considered

### Cache with fixed short TTL only

- Pro: simple implementation.
- Pro: no fanout bus required.
- Pro: tolerates missed invalidation messages.
- Con: stale grants remain possible for the TTL duration.
- Con: lowering TTL enough to satisfy revocation freshness destroys cache value.
- Con: cannot prove GDPR Art. 7(3) withdrawal effectiveness.
- Con: policy-bundle changes still wait for expiry.
- Tradeoff: operational simplicity but unacceptable freshness ambiguity.
- Rejected.

### No cache and evaluate Cedar on every request

- Pro: freshest possible decision.
- Pro: fewer cache-invalidation bugs.
- Pro: simpler evidence story.
- Con: Cedar hot path p99 misses ADR-0243 1 ms budget at high load.
- Con: cross-tenant visibility requests amplify policy CPU.
- Con: outage in policy-bundle store becomes request outage.
- Con: does not protect request handlers during revocation bursts.
- Tradeoff: correctness is strong but performance and availability are weak.
- Rejected.

### Valkey-only shared cache

- Pro: one cache layer to invalidate.
- Pro: easier inspection.
- Pro: avoids process-local staleness.
- Con: adds network hop to every authorization.
- Con: Valkey outage becomes direct request-path failure.
- Con: p99 cannot stay below 1 ms under cross-zone traffic.
- Tradeoff: simpler coherence but too much hot-path latency.
- Rejected.

### Pulsar invalidation without epoch checks

- Pro: faster common path.
- Pro: less database lookup traffic.
- Pro: simpler request handler.
- Con: missed or delayed messages produce stale allows.
- Con: cannot distinguish late fanout from successful revocation.
- Con: chaos testing cannot prove fail-closed semantics.
- Tradeoff: lower latency but insufficient safety.
- Rejected.

### Write-through policy database cache

- Pro: source-of-truth and cache update atomically.
- Pro: no external bus for same-region changes.
- Pro: easier one-cell implementation.
- Con: cross-region cells still need fanout.
- Con: request handlers become coupled to policy database topology.
- Con: replay after outage is harder than Pulsar retention.
- Tradeoff: good local coherence but poor multi-cell recovery.
- Rejected.

## Consequences

- Positive: revocation wins over cache hit rate.
- Positive: stale allow windows are bounded and measured.
- Positive: cache invalidation is replayable from Pulsar retention.
- Positive: cache values carry policy bundle and consent epoch.
- Positive: request handlers can fail closed when freshness proof is absent.
- Positive: dashboards can prove same-region and cross-region freshness.
- Negative: every allow cache hit older than 200 ms may require a freshness check.
- Negative: Pulsar, Valkey, and in-process cache make the system more complex.
- Negative: cache tuning is now part of compliance posture.
- Negative: bad invalidation storms can increase authorization latency.
- Neutral: deny caching is allowed but shorter than allow caching.
- Neutral: future Cedar partial-evaluation optimizations can keep this cache shape.
- Follow-up work: implement `IP-017-consent-revocation-cache-ledger`.
- Follow-up work: add replay runbook for missed invalidation windows.
- Follow-up work: add policy-bundle hash to every authorization decision event.
- Follow-up work: add quarterly revocation-freshness game day.

## Implementation Notes

- Data shape `CedarDecisionCacheKeyV1` contains `tenant_pair_hash`.
- Data shape `CedarDecisionCacheKeyV1` contains `agreement_id`.
- Data shape `CedarDecisionCacheKeyV1` contains `subject_hash`.
- Data shape `CedarDecisionCacheKeyV1` contains `sharing_mode`.
- Data shape `CedarDecisionCacheKeyV1` contains `cedar_policy_bundle_version`.
- Data shape `CedarDecisionCacheKeyV1` contains `consent_epoch`.
- Data shape `CedarDecisionCacheValueV1` contains `decision`.
- Data shape `CedarDecisionCacheValueV1` contains `evaluated_at`.
- Data shape `CedarDecisionCacheValueV1` contains `expires_at`.
- Data shape `CedarDecisionCacheValueV1` contains `source_event_id`.
- Data shape `CedarDecisionCacheValueV1` contains `agreement_state`.
- Data shape `CedarDecisionCacheValueV1` contains `pack_id`.
- Data shape `CacheInvalidationV1` contains `invalidation_id`.
- Data shape `CacheInvalidationV1` contains `agreement_id`.
- Data shape `CacheInvalidationV1` contains `tenant_pair_hash`.
- Data shape `CacheInvalidationV1` contains `subject_hash`.
- Data shape `CacheInvalidationV1` contains `new_consent_epoch`.
- Data shape `CacheInvalidationV1` contains `reason`.
- Data shape `CacheInvalidationV1` contains `cedar_policy_bundle_version`.
- Data shape `CacheInvalidationV1` contains `issued_at`.
- Data shape `CacheInvalidationV1` contains `signature`.
- Reason enum includes `grant_created`.
- Reason enum includes `grant_scope_changed`.
- Reason enum includes `grant_revoked`.
- Reason enum includes `grant_expired`.
- Reason enum includes `policy_bundle_rotated`.
- Reason enum includes `pack_overlay_changed`.
- API endpoint `POST /v1/internal/cedar-cache/invalidate` publishes invalidation.
- API endpoint `POST /v1/internal/cedar-cache/replay` replays retained invalidations.
- API endpoint `GET /v1/internal/cedar-cache/freshness/{agreement_id}` returns epoch and lag.
- API endpoint `POST /v1/agreements/{agreement_id}/evaluate` evaluates Cedar for a sharing decision.
- API endpoint `POST /v1/agreements/{agreement_id}/revoke` increments consent epoch and publishes invalidation.
- API endpoint `GET /v1/internal/cedar-cache/inspect/{agreement_id}` is operator-read only.
- Valkey key prefix is `cg:cedar:v1:{pack_id}:{tenant_pair_hash}:{agreement_id}:{subject_hash}`.
- In-process cache uses TinyLFU admission and 50,000-entry max per pod.
- Valkey max memory policy is `allkeys-lfu`.
- Valkey persistence is disabled for decision values.
- Pulsar retention is the replay source, not Valkey persistence.
- Cache invalidation producer is idempotent on `invalidation_id`.
- Cache invalidation consumer commits after local and Valkey deletion both succeed.
- Failed deletion retries with exponential backoff for 30 seconds.
- Failed deletion after 30 seconds marks agreement fail-closed.
- Cedar principal for invalidation is `Oyatie::Principal::Service("consent-graph.policy-writer")`.
- Cedar principal for evaluation is `Oyatie::Principal::Service("consent-graph.api")`.
- Cedar principal for replay is `Oyatie::Principal::Service("consent-graph.cache-replayer")`.
- Cedar principal for inspection is `Oyatie::Principal::Service("consent-graph.sre-oncall")`.
- Cedar resource for invalidation is `ConsentGraph::CacheKey`.
- Example permit: principal `consent-graph.policy-writer`, action `consent-graph.cache.invalidate`, resource `ConsentGraph::CacheKey::"dsa_01HY"`, context `{reason:"grant_revoked", pack_id:"gdpr-eu", new_consent_epoch:42}`.
- Example permit: principal `consent-graph.api`, action `consent-graph.enforcement.evaluate`, resource `ConsentGraph::Agreement::"dsa_01HY"`, context `{cache_epoch:42, revocation_epoch:42, cache_age_ms:73}`.
- Example forbid: principal `consent-graph.api`, same action, context `{cache_epoch:41, revocation_epoch:42, cache_age_ms:73}`.
- Example forbid: principal `consent-graph.sre-oncall`, action `consent-graph.cache.invalidate`, resource `ConsentGraph::CacheKey::"dsa_01HY"`, context `{reason:"operator_override"}`.
- Audit event `ConsentGraphCacheInvalidationPublished` emits on publication.
- Audit event `ConsentGraphCacheInvalidationApplied` emits on consumer success.
- Audit event `ConsentGraphCacheFreshnessMiss` emits on fail-closed request.
- Audit event `ConsentGraphCacheReplayStarted` emits on replay.
- Metric `oya_consent_graph_cache_hit_ratio` tracks hit ratio by decision kind.
- Metric `oya_consent_graph_revocation_propagation_ms` tracks propagation.
- Metric `oya_consent_graph_cache_freshness_miss_total` tracks fail-closed misses.
- Metric `oya_consent_graph_invalidation_replay_total` tracks replay events.
- SLO `consent-graph-revocation-freshness.openslo.yaml` sets same-region p99 <= 500 ms.
- SLO `consent-graph-cross-region-revocation.openslo.yaml` sets cross-region p99 <= 1 second.
- SLO `consent-graph-cedar-eval.openslo.yaml` sets cached evaluation p99 <= 1 ms.
- Failure mode `pulsar_lag_over_budget` fails closed for affected agreements.
- Failure mode `valkey_unavailable` bypasses Valkey and checks source epoch.
- Failure mode `source_epoch_unavailable` fails closed.
- Failure mode `signature_invalid` drops invalidation and opens Sev-1.
- Failure mode `policy_bundle_hash_mismatch` fails closed.

## Verification

- Test `cache_key_contains_policy_bundle_version` verifies key composition.
- Test `cache_key_contains_consent_epoch` verifies epoch composition.
- Test `revocation_invalidates_in_process_cache` expects same-pod <= 10 ms.
- Test `revocation_invalidates_valkey_cache` expects same-region <= 500 ms.
- Test `cross_region_revocation_invalidates_peer_cell` expects <= 1 second.
- Test `stale_epoch_allow_fails_closed` verifies fail-closed behavior.
- Test `missing_freshness_record_fails_closed` verifies source outage behavior.
- Test `negative_cache_expires_before_allow_cache` verifies deny TTL.
- Test `policy_bundle_rotation_flushes_cache` verifies policy update behavior.
- Test `operator_cannot_write_allow_cache` verifies no privileged bypass.
- Test `cache_replay_idempotent` verifies replay safety.
- Test `cache_invalidation_signature_required` verifies Ed25519 signature.
- Metric `oya_consent_graph_revocation_propagation_ms` must meet p99 <= 500 ms same-region.
- Metric `oya_consent_graph_revocation_propagation_ms` must meet p99 <= 1 second cross-region.
- Metric `oya_consent_graph_cache_freshness_miss_total` must page above 10 per 5 minutes.
- Dashboard `consent-graph-revocation-freshness.json` shows propagation percentiles.
- Dashboard `consent-graph-cedar-cache.json` shows hit ratio, TTL age, and fail-closed count.
- Dashboard `consent-graph-policy-bundle-rollout.json` shows bundle hash per cell.
- CI check `consent-cache-schema` validates canonical JSON fixtures.
- CI check `consent-cache-cedar-actions` validates Cedar action coverage.
- CI check `consent-cache-no-principal-attributes` rejects raw principal caching.
- CI check `consent-cache-revocation-freshness-load` runs 50,000 revocations.
- CI check `oya-governance-observability-emission --microservice consent-graph` verifies telemetry.
- Chaos test drops Pulsar messages and requires replay success.
- Chaos test stalls Valkey and requires fail-closed evaluation.
- Chaos test rotates Cedar bundle during revocation storm.
- Quarterly game day proves revocation freshness evidence export.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0214: Cross-tenant real-time visibility.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- Cedar policy language documentation.
- Apache Pulsar 3.3.x documentation.
- Valkey 8.0 documentation.
- RFC 8032: Ed25519 signatures.
- RFC 8785: JSON Canonicalization Scheme.
- GDPR Art. 7(3) and Art. 17.
- HIPAA 45 CFR §164.312(a)(1).
- KR PIPA Art. 37.
- SOC 2 CC6.6 and CC7.2.
- Google Zanzibar consistency notes.
- AWS IAM policy cache invalidation operational guidance.
