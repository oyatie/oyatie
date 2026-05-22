---
id: ADR-GOV-001
title: Audit-Event Aggregation and Per-Pack Retention Policy
status: Accepted
date: 2026-05-20
microservice: governance
related_oyatie_adrs:
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0005-eventing-backbone-outbox-pattern.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0010-regional-pack-architecture.md
  - docs/decisions/ADR-0128-hyperscaler-architecture-invariants.md
decision_owner: axis-governance
---

# ADR-GOV-001: Audit-Event Aggregation and Per-Pack Retention Policy

## Context

- Governance owns lane runtime, policy-engine bounded context, evidence emitter, aggregation indexer, check crates, conformance lanes, and governance dashboards.
- Existing IPs include `IP-008-evidence-emitter-kernel-domain.md`, `IP-010-aggregation-indexer-full-stack.md`, `IP-011-industry-best-practice-conformance-lane.md`, and `IP-013-aggregation-index-generation-lane.md`.
- Existing SLOs include conformance evidence freshness, gate validate latency, per-lane runtime budget, and check-crate availability.
- Existing runbooks include `aggregation-rebuild.md`, `evidence-replay.md`, `lane-failure-triage.md`, `industry-baseline-refresh.md`, and `lane-bypass-emergency.md`.
- Named precedent: AWS CloudTrail Lake separates immutable audit event ingestion from queryable aggregations.
- Named precedent: Google Cloud Audit Logs plus Log Router sinks separate audit emission from export and retention.
- Named precedent: Palantir Foundry's governance and lineage planes aggregate evidence while preserving source-of-truth provenance.
- Constraint GOV-C1: audit-chain remains source of truth per ADR-0003; governance aggregation is a projection, not a replacement.
- Constraint GOV-C2: aggregation consumes canonical CloudEvents / Kafka outbox streams per ADR-0005.
- Constraint GOV-C3: Cedar authorizes aggregation query, retention override, pack view, and lane bypass per ADR-0007.
- Constraint GOV-C4: regional and compliance pack retention follows ADR-0010 pack semantics.
- Constraint GOV-C5: hyperscaler invariants in ADR-0128 require cell isolation, idempotency, progressive delivery, and auditability.
- Constraint GOV-C6: pack retention can be stricter than product retention but cannot silently shorten audit-chain retention.
- Constraint GOV-C7: governance must aggregate across microservices without creating cross-tenant data leakage.
- Constraint GOV-C8: aggregation must support evidence freshness dashboards and daily conformance reports.
- Constraint GOV-C9: raw audit payloads can contain sensitive fields; indexed projections must be minimized.
- Constraint GOV-C10: retention decisions must explain which pack rule won.
- Constraint GOV-C11: governance lanes must be able to rebuild aggregation from audit-chain and event backbone.
- Constraint GOV-C12: bypasses must be rare, expiring, and always auditable.
- Constraint GOV-C13: query performance must support auditors without giving them raw data-plane access.
- Constraint GOV-C14: per-pack retention must support SOC 2, ISO 27001, GDPR, HIPAA, KR PIPA, FedRAMP, PCI, and SOX.
- Constraint GOV-C15: aggregation failure must not block product operations unless evidence emission itself is unavailable.
- The architecture must distinguish evidence collection, evidence aggregation, conformance evaluation, and retention policy.
- The architecture must expose enough query shape for compliance and ops without making governance the audit source of truth.
- The architecture must make stale aggregation visible and repairable.
- The architecture must handle tenant, pack, cell, microservice, lane, and decision owner dimensions.

## Decision

- Build governance aggregation as a read-optimized projection over audit-chain events and evidence event streams.
- Keep audit-chain as immutable source; governance stores materialized indexes and rollups.
- Ingest audit events via Kafka topics and verify each event hash against audit-chain anchor when available.
- Define a `GovernanceEvidenceIndex` for per-tenant, per-pack, per-lane, per-service search.
- Define `PackRetentionPolicy` as the decision table for retention class, minimum duration, legal hold behavior, and deletion eligibility.
- Apply higher-retention-wins when multiple packs apply.
- Apply litigation hold, incident hold, regulator hold, and lifecycle lock before normal expiration.
- Store raw payload pointers, not raw sensitive payloads, in aggregation indexes.
- Store normalized dimensions: `tenant_id_hash`, `pack_id`, `microservice`, `event_class`, `cell_id`, `policy_id`, `decision_owner`, `retention_class`.
- Store tenant id encrypted or hash-tokenized in high-cardinality metrics and query indexes.
- Keep auditor query APIs scope-bound by Cedar and time-bounded grants.
- Emit aggregation health events for stale partitions, schema drift, retention conflicts, and replay completion.
- Make aggregation rebuild idempotent by source event id and projection version.
- Version every aggregation projection.
- Make retention policy changes soak for at least 60 seconds before applying to deletes.
- Deny destructive retention actions while projection freshness is red.
- Use pack-specific retention classes: `operational_30d`, `security_1y`, `audit_7y`, `regulated_10y`, and `legal_hold_indefinite`.
- Let product services declare recommended retention class, but governance computes the effective retention class.
- Publish daily retention conflict reports for tenants with multiple packs.
- Publish conformance rollups to dashboards and compliance service through API, not direct DB sharing.
- Keep bypass grants separate from policy changes and require expiry.
- Require dual approval for retention shortening.
- Require incident commander or compliance owner approval for emergency retention extension.
- Keep governance query surface read-only for auditors.

## Alternatives Considered

### Product Services Own Their Own Retention Aggregates

- Pros: each service can optimize its own query shape.
- Pros: lower central governance complexity.
- Pros: product teams can move independently.
- Cons: retention semantics drift by product.
- Cons: auditors must query many services.
- Cons: cross-pack conflict resolution becomes inconsistent.
- Rejected because governance needs one retention policy projection over the portfolio.

### Governance Stores Raw Audit Payloads

- Pros: simpler query and report generation.
- Pros: fewer round trips to audit-chain or evidence stores.
- Pros: easier auditor portal implementation.
- Cons: duplicates sensitive data.
- Cons: increases breach blast radius.
- Cons: risks governance becoming a shadow source of truth.
- Rejected because aggregation should minimize data and preserve provenance.

### Single Global Retention Duration

- Pros: easy to explain.
- Pros: simple deletion jobs.
- Pros: low policy complexity.
- Cons: over-retains low-risk data.
- Cons: under-retains regulated audit evidence.
- Cons: cannot satisfy pack-specific obligations.
- Rejected because Oyatie serves multiple packs and jurisdictions.

### Retention Policy in Compliance Service Only

- Pros: keeps regulatory logic in compliance.
- Pros: avoids duplicate pack modeling.
- Pros: aligns with auditor workflows.
- Cons: governance lanes need retention for evidence and conformance artifacts.
- Cons: compliance would become a bottleneck for platform evidence.
- Cons: the policy affects lane runtime, dashboards, and bypass evidence.
- Rejected; compliance owns pack legal semantics, governance owns platform evidence projection and retention execution.

## Consequences

- Positive: auditors get fast queryable evidence without weakening audit-chain immutability.
- Positive: pack retention conflicts produce explicit reports.
- Positive: aggregation can be rebuilt after index corruption.
- Positive: retention policy becomes testable and versioned.
- Positive: bypasses and retention overrides are auditable commands.
- Positive: high-cardinality tenant labels are minimized in metrics.
- Positive: governance dashboards can show evidence freshness by pack and service.
- Positive: product services do not implement their own retention conflict rules.
- Negative: governance must keep projection schema in lockstep with audit event taxonomy.
- Negative: replay and rebuild jobs can be expensive at portfolio scale.
- Negative: strict retention and hold logic can delay deletion.
- Negative: auditor queries may need raw-payload follow-up through compliance or audit-chain.
- Negative: dual approval for shortening adds operational latency.
- Neutral: product services still declare event and retention intent.
- Neutral: compliance service remains legal-pack authority for pack definitions.
- Neutral: governance projection failure is degraded reporting, not source event loss.
- Neutral: retention extension is easier than retention shortening.
- Neutral: exact tenant id can stay out of metrics while audit-chain preserves signed evidence.

## Implementation Notes

- Data shape `GovernanceEvidenceIndex`: `{projection_version, source_event_id, source_hash, tenant_token, pack_id, microservice, event_class, cell_id, retention_class, occurred_at}`.
- Data shape `PackRetentionPolicy`: `{pack_id, event_class, data_class, minimum_duration, max_duration, legal_hold_behavior, delete_approval_class, effective_from}`.
- Data shape `RetentionDecision`: `{source_event_id, candidate_rules[], winning_rule, effective_retention_until, hold_refs[], decision_reason}`.
- Data shape `AggregationPartition`: `{partition_id, topic, tenant_token_range, projection_version, high_watermark, freshness_state, last_replay_at}`.
- Data shape `GovernanceBypassGrant`: `{grant_id, tenant_id, lane_id, action, reason, approved_by[], expires_at, audit_event_id}`.
- REST endpoint `GET /v1/governance/evidence` queries indexed evidence by scoped dimensions.
- REST endpoint `GET /v1/governance/evidence/{source_event_id}` returns projection metadata and source pointer.
- REST endpoint `POST /v1/governance/retention/policies` creates a versioned policy.
- REST endpoint `POST /v1/governance/retention/evaluate` dry-runs retention decision.
- REST endpoint `POST /v1/governance/aggregation/replay` starts a partition replay.
- REST endpoint `POST /v1/governance/bypass-grants` creates expiring bypass.
- REST endpoint `DELETE /v1/governance/bypass-grants/{grant_id}` revokes bypass.
- AsyncAPI channel `governance.aggregation.partition.stale.v1` reports freshness issues.
- AsyncAPI channel `governance.retention.conflict.detected.v1` reports pack conflict.
- AsyncAPI channel `governance.retention.policy.changed.v1` reports policy changes.
- AsyncAPI channel `governance.bypass.granted.v1` reports bypass creation.
- AsyncAPI channel `governance.aggregation.replay.completed.v1` reports rebuild completion.
- Cedar permit `governance::evidence::query` requires auditor or service owner scope.
- Cedar permit `governance::retention::shorten` requires dual approval and no active hold.
- Cedar permit `governance::aggregation::replay` requires governance operator role.
- Cedar forbid `governance::retention::delete` when projection freshness is red.
- Cedar forbid `governance::bypass::grant` when requested expiry exceeds pack maximum.
- Audit event `EVT-GOV-RETENTION-POLICY-CHANGED` includes policy diff.
- Audit event `EVT-GOV-RETENTION-DECISION-EVALUATED` includes winning rule.
- Audit event `EVT-GOV-AGGREGATION-REPLAY-COMPLETED` includes high watermark and count.
- Audit event `EVT-GOV-BYPASS-GRANTED` includes approvals and expiry.
- Metric `governance_evidence_freshness_lag_seconds` tracks projection lag.
- Metric `governance_retention_conflict_total` tracks pack conflicts.
- Metric `governance_aggregation_replay_events_per_second` tracks rebuild throughput.
- Metric `governance_bypass_active_total` tracks live bypasses by lane.
- Capacity math: 100k audit events/s with 300 byte index rows yields about 30 MB/s raw index ingest before compression; partition by topic and tenant token.
- Capacity math: 7-year audit retention at 10 billion events/year cannot sit only in hot index; keep hot 90 days and cold query pointers beyond.
- Rollback path: retention policy rollback activates previous policy version and recomputes decisions before delete jobs resume.
- Rollback path: bad projection version is abandoned and replayed from audit-chain into prior version.
- Multi-region path: aggregation runs per cell; portfolio views are federated summaries unless pack permits central projection.
- Sovereign path: regulated packs keep raw payload pointers in jurisdiction; central governance sees minimized rollups.
- Versioning: projection version `governance-evidence-index-v1`.
- Deprecation: event dimensions remain queryable for at least 365 days after deprecation.

## Verification

- Unit test `higher_retention_wins_multi_pack_conflict` verifies retention selection.
- Unit test `legal_hold_blocks_delete_even_after_expiry` verifies hold precedence.
- Unit test `bypass_requires_expiry_and_approval` verifies bypass shape.
- Unit test `projection_never_stores_raw_payload` verifies minimization.
- Unit test `retention_shorten_requires_dual_approval` verifies governance control.
- Property test `retention_decision_deterministic_for_rule_order` generates pack rule permutations.
- Property test `aggregation_replay_idempotent_by_source_event_id` covers rebuild semantics.
- Fuzz test `evidence_query_rejects_unbounded_filters` protects auditor API.
- Integration test `audit_event_ingest_verifies_source_hash` proves provenance.
- Integration test `stale_partition_blocks_retention_delete` proves freshness guard.
- Integration test `retention_conflict_publishes_event` verifies reporting.
- Integration test `bypass_grant_expires_and_denies_after_expiry` verifies Cedar time behavior.
- Load test `ingest_100k_audit_events_per_second` keeps projection lag below 60 seconds.
- Load test `auditor_query_90_day_hot_index` keeps p95 below 2 seconds.
- Chaos test `projection_corruption_rebuild_from_audit_chain` proves recovery.
- Chaos test `kafka_partition_lag_pages_freshness_slo` proves observability.
- Metric SLO: `governance_evidence_freshness_lag_seconds` p95 below 60 seconds.
- Metric SLO: `governance_aggregation_replay_events_per_second` above planned partition target.
- Metric SLO: active bypass count older than expiry equals zero.
- Audit check: every retention policy change has `EVT-GOV-RETENTION-POLICY-CHANGED`.
- Audit check: every bypass has approvals and expiry.
- Static check: aggregation index schema has no raw payload column.
- Static check: delete jobs call retention evaluate before delete.
- Contract check: OpenAPI marks raw source payload as pointer-only.

