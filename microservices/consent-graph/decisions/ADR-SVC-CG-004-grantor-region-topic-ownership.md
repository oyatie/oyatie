---
id: ADR-SVC-CG-004
title: "Grantor-region Pulsar topics own consent projection delivery"
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
decision_owner: axis-consent-graph + cloud-k8s + ops-data-residency
---

# ADR-SVC-CG-004: Grantor-region Pulsar topics own consent projection delivery

## Context

- The named architectural pressure is `grantor-sovereignty-before-grantee-convenience`.
- Consent-graph mediates data movement from a grantor tenant to a grantee tenant.
- ADR-0214 requires real-time cross-tenant visibility.
- ADR-0251 requires pack cell certification and residency evidence.
- ADR-0244 requires tenant scoping in every topic, event, and resource.
- Prior incident class `grantee-region-topic-drift` placed grantor data in a grantee-owned region.
- Prior incident class `neutral-bus-residency-gap` treated a shared topic as outside both tenants.
- Prior incident class `late-revocation-after-region-failover` continued projection from the wrong cell.
- Prior incident class `topic-owner-confusion` left no accountable tenant for retention and deletion.
- Residency pressure includes GDPR Arts. 44-50, KR PIPA Art. 28, LGPD Arts. 33-36, HIPAA §164.312(c)(1), and SOC 2 CC6.7.
- The grantor is the source of data and the party whose data residency constraints bind first.
- The grantee is entitled only to the agreed projection.
- A neutral global topic would become a hidden transfer mechanism.
- Topic ownership must survive cross-region outage.
- Topic ownership must be visible to auditors.
- Topic naming must carry grantor region and pack.
- Projection topics must not leak tenant ids in broker metrics.
- Projection topics must carry tenant-pair hashes for dashboards.
- Projection delivery must support grantor-region failover.
- Projection delivery must support grantee-region pull.
- Revocation must stop delivery from the grantor region first.
- The design must be implementable with Apache Pulsar 3.3.x.
- The design must avoid Kafka-style global clusters for pack-sensitive data.
- The design must be intern-buildable from this ADR.

## Decision

- We choose `grantor-region topic ownership`.
- The named pattern is `source-sovereign event delivery`.
- Every projection topic is created in the grantor's active pack cell.
- The grantor pack id is mandatory in topic metadata.
- The grantor region is mandatory in topic metadata.
- The grantee region is metadata only.
- The grantee never owns the projection topic.
- The neutral platform never owns projection data topics.
- Topic namespace format is `persistent://consent-graph/{grantor_pack_id}/projection`.
- Topic name format is `{grantor_region}.{tenant_pair_hash}.{agreement_id}.{projection_id}`.
- Tenant ids are hashed with HMAC-SHA256 before topic naming.
- Topic retention default is 24 hours.
- Topic retention maximum is 7 days for offline grantee pulls.
- Topic compaction is disabled for projection payloads.
- Projection payloads are envelope-encrypted by grantor pack KMS.
- Projection payloads include `consent_epoch`.
- Projection payloads include `agreement_id`.
- Projection payloads include `projection_schema_version`.
- Projection payloads include `field_allowlist_hash`.
- Projection payloads include `grantor_pack_id`.
- Projection payloads include `source_event_id`.
- Cross-region replication is off by default.
- Cross-region replication may be enabled only when the agreement carries approved transfer terms.
- Cross-region replication may be enabled only for approved destination pack ids.
- Cross-region replication must preserve grantor-region source topic as primary.
- Grantor-region failover elects a secondary grantor-region topic from the same pack.
- If no same-pack secondary exists, projection delivery fails closed.
- Revocation deletes read permission before topic cleanup.
- Revocation publishes tombstone marker within 500 ms p99.
- Grantee consumers must check current consent epoch before processing.
- Cedar action `consent-graph.projection.topic.create` gates topic creation.
- Cedar action `consent-graph.projection.topic.consume` gates grantee consumption.
- Cedar action `consent-graph.projection.topic.replicate` gates cross-region replication.
- Cedar action `consent-graph.projection.topic.retire` gates topic retirement.
- No operator may move a projection topic to grantee ownership.

## Alternatives Considered

### Grantee-region topic ownership

- Pro: lower latency for grantee consumers.
- Pro: easier grantee operational ownership.
- Pro: grantee controls retention.
- Con: violates grantor residency as soon as data is published.
- Con: revocation control moves away from source tenant.
- Con: grantor auditors cannot inspect the primary delivery path.
- Con: cross-pack transfer can occur before consent checks.
- Tradeoff: grantee convenience but unacceptable sovereignty risk.
- Rejected.

### Neutral global consent bus

- Pro: simple shared architecture.
- Pro: one topic namespace.
- Pro: easier cross-region delivery.
- Con: neutral bus is still a data transfer location.
- Con: residency evidence becomes ambiguous.
- Con: global outage affects every pack.
- Con: GDPR transfer review is harder.
- Tradeoff: platform simplicity but poor legal clarity.
- Rejected.

### Per-agreement database polling instead of topics

- Pro: no broker operations.
- Pro: easier transactional coupling to agreement state.
- Pro: source database remains authoritative.
- Con: p99 delivery cannot meet real-time visibility needs.
- Con: grantee polling amplifies source database load.
- Con: long-poll connections leak cross-tenant coupling.
- Tradeoff: fewer moving parts but worse latency and scale.
- Rejected.

### Kafka cluster per compliance pack

- Pro: mature ecosystem.
- Pro: strong throughput.
- Pro: good tooling.
- Con: namespace and tenant isolation are weaker for this use case.
- Con: cross-cluster replication control is operationally heavier.
- Con: Pulsar multi-tenancy maps better to pack namespaces.
- Tradeoff: familiar broker but weaker pack-native structure.
- Rejected.

### Source microservice owns topics directly

- Pro: source service owns its data.
- Pro: fewer consent-graph broker responsibilities.
- Pro: source-specific schemas are easier.
- Con: every source service must reimplement consent delivery semantics.
- Con: policy and revocation evidence fragments across services.
- Con: mode-specific sharing rules become inconsistent.
- Tradeoff: source autonomy but governance fragmentation.
- Rejected.

## Consequences

- Positive: residency follows the grantor first.
- Positive: topic naming is audit-friendly without raw tenant ids.
- Positive: revocation control stays with source data owner.
- Positive: cross-region transfer requires explicit agreement terms.
- Positive: Pulsar namespaces map to pack certification boundaries.
- Negative: grantee consumers may see additional latency.
- Negative: grantor-region outages fail closed rather than falling back to grantee region.
- Negative: topic lifecycle is now part of consent lifecycle.
- Negative: Pulsar capacity planning is grantor-region skew sensitive.
- Neutral: future broker replacement can preserve the ownership rule.
- Neutral: source microservices still own source-of-truth rows.
- Follow-up work: implement topic metadata validator.
- Follow-up work: add grantor-region failover runbook.
- Follow-up work: add residency-evidence export for projection topics.
- Follow-up work: add broker capacity model by grantor pack.

## Implementation Notes

- Data shape `ProjectionTopicV1` contains `topic_id`.
- Data shape `ProjectionTopicV1` contains `agreement_id`.
- Data shape `ProjectionTopicV1` contains `projection_id`.
- Data shape `ProjectionTopicV1` contains `grantor_tenant_id_hash`.
- Data shape `ProjectionTopicV1` contains `grantee_tenant_id_hash`.
- Data shape `ProjectionTopicV1` contains `tenant_pair_hash`.
- Data shape `ProjectionTopicV1` contains `grantor_pack_id`.
- Data shape `ProjectionTopicV1` contains `grantor_region`.
- Data shape `ProjectionTopicV1` contains `grantee_pack_id`.
- Data shape `ProjectionTopicV1` contains `grantee_region`.
- Data shape `ProjectionTopicV1` contains `retention_seconds`.
- Data shape `ProjectionTopicV1` contains `replication_allowed`.
- Data shape `ProjectionTopicV1` contains `approved_destination_pack_ids`.
- Data shape `ProjectionEventV1` contains `source_event_id`.
- Data shape `ProjectionEventV1` contains `consent_epoch`.
- Data shape `ProjectionEventV1` contains `projection_schema_version`.
- Data shape `ProjectionEventV1` contains `field_allowlist_hash`.
- Data shape `ProjectionEventV1` contains `payload_ciphertext`.
- Data shape `ProjectionEventV1` contains `payload_dek_ref`.
- API endpoint `POST /v1/agreements/{agreement_id}/projection-topics` creates a topic.
- API endpoint `GET /v1/agreements/{agreement_id}/projection-topics/{topic_id}` returns metadata.
- API endpoint `POST /v1/agreements/{agreement_id}/projection-topics/{topic_id}/replication` enables approved replication.
- API endpoint `POST /v1/agreements/{agreement_id}/projection-topics/{topic_id}/retire` retires topic.
- API endpoint `GET /v1/internal/projection-topics/residency-report` exports evidence.
- Apache Pulsar version is 3.3.x.
- Pulsar namespace isolation policy pins grantor pack brokers.
- Pulsar geo-replication is disabled unless `replication_allowed=true`.
- Pulsar schema registry stores `ProjectionEventV1`.
- OpenBao path for projection DEK wrapping is `transit/consent-graph/projection/{grantor_pack_id}`.
- OCI Object Storage is used only for oversized projection payloads above 1 MiB.
- Oversized payload pointer is signed and expires after 15 minutes.
- Cedar principal for topic creation is `Oyatie::Principal::Service("consent-graph.projection-controller")`.
- Cedar principal for consumption is `Oyatie::Principal::Service("consent-graph.grantee-consumer")`.
- Cedar principal for replication is `Oyatie::Principal::Service("consent-graph.residency-controller")`.
- Cedar principal for retirement is `Oyatie::Principal::Service("consent-graph.projection-controller")`.
- Cedar resource is `ConsentGraph::ProjectionTopic`.
- Example permit: principal `consent-graph.projection-controller`, action `consent-graph.projection.topic.create`, resource `ConsentGraph::ProjectionTopic::"pt_01HY"`, context `{grantor_pack_id:"gdpr-eu", grantor_region:"eu-frankfurt-1", agreement_state:"accepted"}`.
- Example permit: principal `consent-graph.grantee-consumer`, action `consent-graph.projection.topic.consume`, resource `ConsentGraph::ProjectionTopic::"pt_01HY"`, context `{consent_epoch:9, revoked:false, grantee_pack_id:"gdpr-eu"}`.
- Example forbid: principal `consent-graph.residency-controller`, action `consent-graph.projection.topic.replicate`, resource `ConsentGraph::ProjectionTopic::"pt_01HY"`, context `{grantor_pack_id:"kr-strict", destination_pack_id:"us-commercial", transfer_terms:false}`.
- Example forbid: same create action with context `{topic_owner:"grantee"}`.
- SLO `consent-projection-topic-create.openslo.yaml` sets topic create p99 <= 2 seconds.
- SLO `consent-projection-delivery.openslo.yaml` sets delivery p99 <= 750 ms same-region.
- SLO `consent-projection-revocation-tombstone.openslo.yaml` sets tombstone p99 <= 500 ms.
- Failure mode `grantor_region_unavailable` fails closed unless same-pack secondary exists.
- Failure mode `replication_terms_missing` rejects cross-region replication.
- Failure mode `topic_owner_mismatch` quarantines topic and pages Sev-1.
- Failure mode `tenant_id_in_topic_name` fails CI and blocks deploy.
- Failure mode `payload_dek_wrong_pack` rejects event consumption.

## Verification

- Test `projection_topic_created_in_grantor_pack` verifies pack ownership.
- Test `projection_topic_name_omits_raw_tenant_ids` verifies hashing.
- Test `projection_topic_rejects_grantee_owner` verifies ownership invariant.
- Test `projection_replication_requires_transfer_terms` verifies residency.
- Test `projection_revocation_tombstone_under_budget` verifies 500 ms p99.
- Test `grantee_consumer_checks_consent_epoch` verifies freshness.
- Test `kr_strict_rejects_us_destination` verifies pack overlay.
- Test `grantor_region_failover_same_pack_only` verifies failover.
- Test `oversized_payload_pointer_expires` verifies signed pointer TTL.
- Test `payload_dek_ref_matches_grantor_pack` verifies KMS boundary.
- Metric `oya_consent_graph_projection_topic_create_ms` must meet p99 <= 2 seconds.
- Metric `oya_consent_graph_projection_delivery_ms` must meet p99 <= 750 ms.
- Metric `oya_consent_graph_projection_tombstone_ms` must meet p99 <= 500 ms.
- Metric `oya_consent_graph_projection_residency_violation_total` must remain zero.
- Dashboard `consent-graph-projection-topics.json` shows topics by grantor pack.
- Dashboard `consent-graph-residency.json` shows replication terms and destinations.
- Dashboard `consent-graph-broker-capacity.json` shows Pulsar backlog by grantor region.
- CI check `consent-projection-topic-schema` validates fixtures.
- CI check `consent-projection-topic-cedar` validates actions.
- CI check `consent-projection-topic-no-tenant-id` rejects raw tenant names.
- CI check `consent-projection-topic-pack-overlay` validates GDPR/KR/LGPD overlays.
- Chaos test kills grantor-region broker and expects same-pack failover or fail-closed.
- Chaos test delays tombstone and expects pager after 500 ms p99 breach.
- Audit query exports all active topics with grantor pack and retention.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0214: Cross-tenant real-time visibility.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- Apache Pulsar 3.3.x documentation.
- Apache Pulsar geo-replication documentation.
- OpenBao transit secrets engine documentation.
- RFC 2104: HMAC.
- RFC 8785: JSON Canonicalization Scheme.
- GDPR Arts. 44-50.
- KR PIPA Art. 28.
- LGPD Arts. 33-36.
- HIPAA 45 CFR §164.312(c)(1).
- SOC 2 CC6.7.
- NIST SP 800-53 Rev. 5 SC-7 and SC-28.
