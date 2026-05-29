---
id: ADR-SVC-CG-003
title: "Consent sharing modes are projection, aggregate, and attested query"
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
decision_owner: axis-consent-graph + axis-data-products
---

# ADR-SVC-CG-003: Consent sharing modes are projection, aggregate, and attested query

## Context

- The named architectural pressure is `least-disclosing-sharing-contracts`.
- Consent-graph cannot treat every data-sharing agreement as a raw row export.
- ADR-0214 requires cross-tenant real-time visibility without collapsing tenant boundaries.
- ADR-0243 requires every access path to be Cedar-gated.
- ADR-0244 requires tenant and audience scoping on every data movement.
- ADR-0251 requires compliance packs to constrain sharing across jurisdictions.
- Prior incident class `raw-export-by-default` exposed more fields than the grantor intended.
- Prior incident class `aggregate-small-cell-leakage` allowed inference from a small group.
- Prior incident class `query-result-replay` reused an attested query answer after consent revocation.
- Prior incident class `mode-confusion` let a grantee treat an aggregate grant as projection.
- Regulatory pressure comes from GDPR Art. 5(1)(c), GDPR Art. 6, GDPR Art. 9, HIPAA §164.514(b), KR PIPA Art. 15, and LGPD Art. 11.
- Data minimization requires mode-specific semantics.
- Projection mode is needed for operational collaboration.
- Aggregate mode is needed for analytics without identifiable records.
- Attested query mode is needed for one-off proofs and verifier workflows.
- Each mode has different SLOs and failure modes.
- Each mode has different Cedar action names.
- Each mode has different audit-chain evidence.
- Each mode must be explicit in `CrossPointerV1`.
- The mode cannot be changed in place after grant acceptance.
- A mode change creates a new agreement version.
- The API must reject unknown modes.
- The data product catalog must show the mode at review time.
- The consent receipt must show the mode in user-readable text.
- The implementation must be detailed enough for an intern to build fixtures and tests.

## Decision

- We choose exactly three sharing modes for M03.
- The named pattern is `mode-specific data minimization contract`.
- Mode one is `projection`.
- Mode two is `aggregate`.
- Mode three is `attested_query`.
- `projection` shares named fields from named source entities.
- `projection` preserves row identity.
- `projection` requires explicit field allowlist.
- `projection` requires purpose binding.
- `projection` requires row-level Cedar authorization.
- `aggregate` shares grouped metrics only.
- `aggregate` removes row identity.
- `aggregate` requires minimum cell size.
- `aggregate` requires privacy budget metadata.
- `aggregate` defaults to k-anonymity k >= 10 for sensitive data.
- `aggregate` defaults to k-anonymity k >= 5 for non-sensitive data.
- `aggregate` defaults to differential privacy epsilon <= 1.0 per 30-day agreement budget when pack requires DP.
- `attested_query` shares an answer to a named query template.
- `attested_query` requires query hash and result hash.
- `attested_query` requires a short replay window.
- `attested_query` response TTL is 15 minutes.
- `attested_query` p99 completion target is <= 5 seconds.
- Projection delivery p99 target is <= 500 ms for cached projection manifests.
- Aggregate delivery p99 target is <= 2 seconds for precomputed aggregates.
- Unknown mode fails closed.
- Mode mismatch fails closed.
- Mode downgrade is allowed only through a new agreement version.
- Mode upgrade is allowed only through a new agreement version.
- Cross-pointer `sharing_mode` is mandatory and immutable.
- Cedar action `consent-graph.project.read` gates projection delivery.
- Cedar action `consent-graph.aggregate.read` gates aggregate delivery.
- Cedar action `consent-graph.attested_query.run` gates query execution.
- Cedar action `consent-graph.attested_query.verify` gates verifier replay.
- Cedar action `consent-graph.mode.change` gates versioned mode replacement.
- Audit event names are mode-specific.
- Projection event is `ConsentGraphProjectionDelivered`.
- Aggregate event is `ConsentGraphAggregateDelivered`.
- Attested query event is `ConsentGraphAttestedQueryAnswered`.
- Every mode emits `purpose_id`, `agreement_id`, and `consent_epoch`.
- Every mode emits `pack_id` and `audience_type`.

## Alternatives Considered

### Single generic sharing mode

- Pro: easiest API.
- Pro: simplest UI.
- Pro: fewer Cedar actions.
- Con: hides minimization differences.
- Con: raw projection can be disguised as aggregate.
- Con: audit-chain events lose semantic clarity.
- Con: compliance reviews cannot reason about exposure.
- Tradeoff: simpler product but weaker privacy model.
- Rejected.

### Projection-only sharing

- Pro: operationally useful.
- Pro: maps directly to field allowlists.
- Pro: easiest to debug.
- Con: over-discloses for analytics use cases.
- Con: creates more identifiable data movement.
- Con: cannot satisfy de-identified reporting workflows.
- Tradeoff: simple collaboration but weak minimization.
- Rejected.

### Aggregate-only sharing

- Pro: privacy-preserving by default.
- Pro: avoids row-level transfer.
- Pro: strong analytics posture.
- Con: cannot support operational workflows that need records.
- Con: grantee cannot reconcile individual entitlements.
- Con: attestation workflows still need proofs.
- Tradeoff: strong privacy but insufficient utility.
- Rejected.

### Free-form SQL grants

- Pro: very flexible.
- Pro: data teams understand SQL.
- Pro: fewer product-specific abstractions.
- Con: impossible to review safely at scale.
- Con: SQL text leaks schema and creates injection risk.
- Con: Cedar cannot reason about arbitrary SQL semantics.
- Tradeoff: flexibility but poor governance.
- Rejected.

### Differential privacy for every aggregate

- Pro: strong privacy guarantee.
- Pro: good research pedigree.
- Pro: consistent aggregate semantics.
- Con: too complex for low-risk non-sensitive aggregates.
- Con: epsilon budgeting is hard for small tenants.
- Con: noise can break billing and compliance metrics.
- Tradeoff: stronger privacy but unnecessary operational cost.
- Partial accept: DP is mandatory only for pack-marked sensitive aggregate grants.

## Consequences

- Positive: every grant has explicit data-minimization semantics.
- Positive: Cedar policies remain understandable.
- Positive: audit evidence can be reviewed by mode.
- Positive: aggregate leakage controls are named and testable.
- Positive: attested queries have replay protection.
- Negative: product UI must explain three modes clearly.
- Negative: mode migration creates agreement-version churn.
- Negative: aggregate DP budgets add accounting complexity.
- Negative: projection still moves identifiable rows and requires stricter controls.
- Neutral: future modes require new ADR rather than enum extension by code owners alone.
- Neutral: source microservices can implement only the modes they support.
- Follow-up work: implement data-product catalog mode badges.
- Follow-up work: add aggregate privacy-budget ledger.
- Follow-up work: add attested-query template registry.
- Follow-up work: add user-facing consent receipt language for all modes.

## Implementation Notes

- Data shape `SharingModeV1` enum values are `projection`, `aggregate`, and `attested_query`.
- Data shape `ProjectionGrantV1` contains `source_service`.
- Data shape `ProjectionGrantV1` contains `entity_name`.
- Data shape `ProjectionGrantV1` contains `field_allowlist`.
- Data shape `ProjectionGrantV1` contains `row_filter_ref`.
- Data shape `ProjectionGrantV1` contains `purpose_id`.
- Data shape `ProjectionGrantV1` contains `delivery_topic`.
- Data shape `AggregateGrantV1` contains `metric_name`.
- Data shape `AggregateGrantV1` contains `group_by_dimensions`.
- Data shape `AggregateGrantV1` contains `cell_size_floor`.
- Data shape `AggregateGrantV1` contains `dp_epsilon_budget`.
- Data shape `AggregateGrantV1` contains `budget_window_days`.
- Data shape `AggregateGrantV1` contains `suppression_reason`.
- Data shape `AttestedQueryGrantV1` contains `query_template_id`.
- Data shape `AttestedQueryGrantV1` contains `query_template_hash`.
- Data shape `AttestedQueryGrantV1` contains `max_runtime_ms`.
- Data shape `AttestedQueryGrantV1` contains `result_ttl_seconds`.
- Data shape `AttestedQueryGrantV1` contains `verifier_audience`.
- API endpoint `POST /v1/agreements/{agreement_id}/projection/read` serves projection reads.
- API endpoint `POST /v1/agreements/{agreement_id}/aggregate/read` serves aggregate reads.
- API endpoint `POST /v1/agreements/{agreement_id}/attested-query/run` executes attested query.
- API endpoint `GET /v1/agreements/{agreement_id}/attested-query/{answer_id}/verify` verifies answer.
- API endpoint `POST /v1/agreements/{agreement_id}/mode-change` creates a replacement agreement version.
- API endpoint `GET /v1/sharing-modes` returns mode metadata for consent UI.
- Projection delivery uses Apache Pulsar 3.3.x topics.
- Projection topic name is `persistent://consent-graph/{pack_id}/projection/{agreement_id}`.
- Aggregate results are stored in PostgreSQL 16.6 with Citus 12.1 for tenant-sharded aggregates.
- Attested query templates are stored in `consent_query_template` table.
- Query template hashes use SHA-256 over RFC 8785 canonical JSON.
- Attested answers are Ed25519-signed.
- Attested answer IDs are ULID prefixed by `cg_ans_`.
- Cedar principal for projection is `Oyatie::Principal::Service("consent-graph.projection-api")`.
- Cedar principal for aggregate is `Oyatie::Principal::Service("consent-graph.aggregate-api")`.
- Cedar principal for attested query is `Oyatie::Principal::Service("consent-graph.attested-query-api")`.
- Cedar resource for projection is `ConsentGraph::ProjectionGrant`.
- Cedar resource for aggregate is `ConsentGraph::AggregateGrant`.
- Cedar resource for attested query is `ConsentGraph::AttestedQueryGrant`.
- Example permit: principal `consent-graph.projection-api`, action `consent-graph.project.read`, resource `ConsentGraph::ProjectionGrant::"dsa_01HY"`, context `{sharing_mode:"projection", consent_epoch:7, purpose_id:"payroll-benefits-sync", field_count:12}`.
- Example permit: principal `consent-graph.aggregate-api`, action `consent-graph.aggregate.read`, resource `ConsentGraph::AggregateGrant::"dsa_01HZ"`, context `{sharing_mode:"aggregate", cell_size:24, cell_size_floor:10, dp_epsilon_remaining:0.6}`.
- Example forbid: same aggregate action with context `{sharing_mode:"aggregate", cell_size:4, cell_size_floor:10}`.
- Example permit: principal `consent-graph.attested-query-api`, action `consent-graph.attested_query.run`, resource `ConsentGraph::AttestedQueryGrant::"dsa_01JA"`, context `{sharing_mode:"attested_query", query_template_id:"qt_income_proof_v1", max_runtime_ms:5000}`.
- Example forbid: projection principal using action `consent-graph.aggregate.read`.
- SLO `consent-projection-read.openslo.yaml` sets p99 <= 500 ms.
- SLO `consent-aggregate-read.openslo.yaml` sets p99 <= 2 seconds.
- SLO `consent-attested-query.openslo.yaml` sets p99 <= 5 seconds.
- Failure mode `mode_mismatch` fails closed and emits `ConsentGraphModeMismatch`.
- Failure mode `aggregate_small_cell` suppresses result and emits `ConsentGraphAggregateSuppressed`.
- Failure mode `dp_budget_exhausted` rejects aggregate and emits `ConsentGraphDpBudgetExhausted`.
- Failure mode `attested_query_expired` rejects verifier replay after 15 minutes.
- Failure mode `projection_field_not_allowlisted` rejects read and opens security event.

## Verification

- Test `sharing_mode_enum_rejects_unknown` verifies enum closure.
- Test `projection_requires_field_allowlist` verifies projection minimization.
- Test `projection_rejects_field_not_allowlisted` verifies field safety.
- Test `aggregate_suppresses_small_cell_sensitive` verifies k >= 10.
- Test `aggregate_suppresses_small_cell_non_sensitive` verifies k >= 5.
- Test `aggregate_dp_budget_decrements` verifies epsilon accounting.
- Test `attested_query_hash_matches_template` verifies template binding.
- Test `attested_query_answer_expires` verifies 15-minute TTL.
- Test `mode_change_creates_new_agreement_version` verifies immutability.
- Test `mode_mismatch_fails_closed` verifies Cedar context.
- Test `projection_principal_cannot_read_aggregate` verifies action separation.
- Test `aggregate_principal_cannot_run_attested_query` verifies action separation.
- Metric `oya_consent_graph_projection_read_ms` must meet p99 <= 500 ms.
- Metric `oya_consent_graph_aggregate_read_ms` must meet p99 <= 2 seconds.
- Metric `oya_consent_graph_attested_query_ms` must meet p99 <= 5 seconds.
- Metric `oya_consent_graph_aggregate_suppressed_total` tracks suppression reasons.
- Metric `oya_consent_graph_mode_mismatch_total` must remain zero outside tests.
- Dashboard `consent-graph-sharing-modes.json` shows volume by mode.
- Dashboard `consent-graph-aggregate-privacy.json` shows small-cell and DP budget.
- Dashboard `consent-graph-attested-query.json` shows runtime, expiry, and verifier failures.
- CI check `consent-sharing-mode-schema` validates fixtures.
- CI check `consent-sharing-mode-cedar` validates mode-specific permits.
- CI check `consent-sharing-mode-openapi` validates endpoints.
- CI check `consent-sharing-mode-receipt-text` validates user-visible receipt fields.
- Chaos test revokes consent during attested query and expects fail-closed answer.
- Load test runs 100,000 projection reads and 10,000 aggregate reads.
- Privacy test verifies aggregate small-cell suppression.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0214: Cross-tenant real-time visibility.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- GDPR Art. 5(1)(c), Art. 6, and Art. 9.
- HIPAA 45 CFR §164.514(b).
- KR PIPA Art. 15 and Art. 29.
- LGPD Art. 11.
- NIST Privacy Framework 1.0.
- Dwork and Roth, The Algorithmic Foundations of Differential Privacy.
- RFC 8032: Ed25519 signatures.
- RFC 8785: JSON Canonicalization Scheme.
- PostgreSQL 16.6 documentation.
- Citus 12.1 documentation.
- Apache Pulsar 3.3.x documentation.
