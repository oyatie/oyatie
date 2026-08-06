---
id: ADR-OBS-001
title: Tracing Substrate Choice: OpenTelemetry plus ClickHouse Cloud
status: Proposed
date: 2026-05-20
microservice: observability
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-observability
---

# ADR-OBS-001: Tracing Substrate Choice: OpenTelemetry plus ClickHouse Cloud

## Context

- Observability is the substrate that every other Oyatie microservice uses for traces, metrics, logs, profiles, SLO burn-rate evidence, and promotion eligibility.
- ADR-0263 already makes OpenTelemetry emission mandatory across workloads; this ADR binds the observability-local trace storage and query substrate.
- ADR-0186 established a layered observability backplane; this ADR decides which layer owns high-cardinality trace retention and cross-signal analytics.
- ADR-0210 pins tail sampling to preserve errors, slow traces, new endpoints, SLO-burn traces, and audit-linked traces; the chosen store must support that data shape.
- The service PRD names Grafana Alloy, Prometheus, Mimir, Loki, Tempo, Pyroscope, Grafana, Alertmanager, and Grafana OnCall as Layer-A runtime pieces.
- The local manifest includes `clickhouse-ingest-throughput`, `trace-ingest-availability`, `query-latency-logs`, and `tail-sample-fidelity` SLO files.
- The local architecture declares data classes `AUDIT`, `INTERNAL_ONLY`, and `PII_QUASI`; trace attributes may reveal tenant behavior even when payload bodies are scrubbed.
- Trace data carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `route_template`, `service.name`, `span.kind`, and audit correlation fields.
- Root cause analysis for incidents requires joining traces with audit-chain event ids, SLO windows, deployment refs, brownout signals, and FinOps attribution rows.
- Tempo gives a strong Grafana-native trace UI and TraceQL, but its cost posture depends on trace indexing and retention choices.
- Jaeger gives a familiar open-source tracing UI and collector model, but it is not the best fit for Oyatie's cross-signal analytical store.
- ClickHouse Cloud gives columnar storage, high-cardinality filtering, compression, materialized views, and SQL joins across trace, log, audit, and cost tables.
- OpenTelemetry is the portable instrumentation contract; storage choice must not leak vendor-specific SDKs into product microservices.
- Developers need a local experience that does not require ClickHouse Cloud credentials for every smoke test.
- Operators need production queries that answer "which tenant, cell, route, policy fragment, deployment, and cost center caused this burn" in one place.
- Compliance reviewers need evidence that PII scrubbing occurs before storage, not as a query-time convention.
- Security reviewers need Cedar-gated access to trace search because traces can expose quasi-identifying behavior.
- FinOps reviewers need cost controls because full-fidelity trace retention can become one of the largest shared-substrate cost centers.
- The platform must retain hot traces for active debugging and longer analytical trace facts for trend, compliance, and cost correlation.
- The platform must keep local fallback viable when ClickHouse Cloud or a region-specific ClickHouse endpoint is unavailable.
- The platform must keep sampling decisions explainable per ADR-0210; operators must know why a trace was retained.
- The platform must preserve W3C trace context and OpenTelemetry semantic conventions end-to-end.
- The platform must support tenant home-cell residency and sovereign packs per ADR-0240.
- The platform must avoid raw tenant identifiers as unbounded metric labels while retaining tenant identity in signed audit evidence.
- The platform must not make observability a hero product; ADR-0245 classifies it as shared substrate.
- The platform must support query surfaces for tenant operators without exposing other tenants' trace attributes.
- The platform must support machine-readable SLO evidence for Oya VCS promotion decisions.
- The platform must support low-cardinality operational dashboards and high-cardinality forensic queries without forcing one backend to do both poorly.
- The platform must keep the emission contract stable even if trace storage is swapped or rebalanced later.
- The platform must document why the choice differs from a pure LGTM or pure Jaeger deployment.

## Decision

- Adopt OpenTelemetry as the only supported instrumentation, context propagation, and semantic-convention contract for distributed traces.
- Use Grafana Alloy or OpenTelemetry Collector agents for local collection and gateway fan-in.
- Use ClickHouse Cloud as the canonical production trace analytics store for retained trace spans, span events, trace-derived metrics, and audit-linked trace facts.
- Use Tempo as the operator-facing short-retention trace UI and exemplar jump target where Grafana workflow benefits from TraceQL and existing dashboard links.
- Keep Jaeger only for local development, migration compatibility, and protocol interop tests; it is not the production source of truth.
- Store canonical production span rows in ClickHouse tables partitioned by `event_date`, `home_cell`, and `service_name`.
- Order hot trace tables by `(tenant_hash, service_name, route_template_hash, trace_id, span_start_ns)` to support tenant-scoped drill-down without raw-tenant metric labels.
- Retain raw span rows for 30 days by default, 90 days for regulated packs that require incident reconstruction, and 400 days only for trace-derived audit facts.
- Retain full raw payload attributes only after emission-boundary scrubbing; fields classified above `PII_QUASI` are rejected or hashed before export.
- Export trace data through OTLP over gRPC inside the mesh and OTLP/HTTP only for browser or constrained edge clients.
- Use ADR-0210 tail-sampling reason codes as first-class span attributes: `tail_sample_reason`, `sample_policy_id`, and `sample_decision_ts`.
- Preserve 100 percent of error, SLO-burn, audit-event, and new-endpoint warm-up traces before random baseline sampling is considered.
- Export ClickHouse materialized views for `trace_error_burn`, `route_latency_distribution`, `tenant_policy_denial_trace`, and `deployment_regression_trace`.
- Keep Tempo retention at 7 days for sampled full traces unless a pack-specific runbook temporarily extends it.
- Write ClickHouse trace rows with `trace_id` and `span_id` compatible with W3C Trace Context and OpenTelemetry IDs.
- Add an `audit_event_id` nullable column so audit-chain and observability can cross-link state-changing spans.
- Add `cost_center_id` and `workload_class` columns derived from ADR-0174 FinOps attribution, not from caller-supplied labels.
- Use Cedar policy before any tenant-facing trace search endpoint and before any internal operator query that crosses tenant boundaries.
- Default trace search access to deny; grant read-only scope by tenant, time range, data class, service, and purpose.
- Use separate ClickHouse databases for production, staging, CI synthetic tenants, and local developer fixtures.
- Use ClickHouse Cloud service accounts stored through OpenBao SecretReference and rotated every 30 days.
- Use compression codecs appropriate to span attributes: ZSTD for strings, Delta plus ZSTD for timestamps, and LowCardinality where cardinality is bounded.
- Reject arbitrary high-cardinality metric labels, but allow high-cardinality trace attributes in ClickHouse because traces are event data, not metric time series.
- Keep schema evolution additive-only for trace tables during the first two minor releases.
- Use a backfill worker to replay OTLP batches from the local durable buffer when ClickHouse ingestion is temporarily unavailable.
- Bound gateway collector memory at 256 MiB per replica for tail sampling and scale horizontally before dropping retained classes.
- Drop baseline random traces first during brownout, never audit-event, error, or SLO-burn traces.
- Emit `EVT-OBS-TRACE-DROPPED-BROWNOUT` when traces are dropped because of cost or ingestion protection.
- Use ClickHouse Cloud for production because managed operations reduce substrate bootstrap risk while the platform builds its in-house control plane.
- Keep the OpenTelemetry emission contract storage-agnostic so a future in-house ClickHouse or cell-local ClickHouse deployment can replace Cloud without SDK changes.

## Alternatives Considered

### Tempo as the only production trace store

- Pros: Grafana-native experience and direct TraceQL support.
- Pros: simpler operator workflow because traces stay in the Grafana stack.
- Pros: lower conceptual load for SREs already using Mimir, Loki, and Grafana.
- Cons: cross-signal SQL joins with audit-chain, FinOps, and deployment evidence are weaker than ClickHouse.
- Cons: long-retention high-cardinality trace analytics can become expensive or require careful indexing trade-offs.
- Cons: tenant-scoped analytical exports are less natural than columnar SQL projections.
- Rejected as the canonical store; retained as the short-retention UI and exemplar layer.

### Jaeger as the production trace backend

- Pros: familiar open-source tracing project with broad protocol ecosystem history.
- Pros: excellent local debugging ergonomics for simple service graphs.
- Pros: viable compatibility target for teams migrating existing tracing clients.
- Cons: weaker fit for Oyatie's analytic need to join trace data with cost, audit, and policy evidence.
- Cons: operating long-retention storage at hyperscaler trace volume would require extra storage design anyway.
- Cons: Jaeger-first UX would duplicate the Grafana operational console already chosen by the PRD.
- Rejected for production; retained for local dev compatibility and protocol regression tests.

### Vendor APM SaaS as the canonical trace store

- Pros: fast operational bootstrap and polished trace search features.
- Pros: mature alerting and incident integrations.
- Pros: fewer internal components to operate early.
- Cons: conflicts with ADR-0211 in-house stack posture and tenant portability expectations.
- Cons: regulated tenants may reject third-party trace custody because trace attributes include quasi-identifying behavior.
- Cons: cost, retention, and query semantics become vendor-specific.
- Rejected because observability is a shared substrate and must not depend on a closed APM control plane.

### Self-hosted ClickHouse from day one

- Pros: strongest control over residency, cost, and operational topology.
- Pros: easier future integration with cell-local deployments.
- Pros: no managed vendor dependency in the trace lake.
- Cons: raises bootstrap operational risk before the observability substrate itself is mature.
- Cons: storage, backup, and rebalancing expertise would compete with the first foundation milestone.
- Cons: production trace ingest must not block on in-house ClickHouse operational hardening.
- Deferred; the schema and OpenTelemetry contract preserve a migration path from ClickHouse Cloud to cell-local ClickHouse.

## Consequences

- Positive: product services instrument once with OpenTelemetry and stay insulated from trace backend churn.
- Positive: ClickHouse supports high-cardinality forensic trace queries without turning those dimensions into metric labels.
- Positive: SREs keep Grafana and Tempo for normal incident navigation while analysts use ClickHouse SQL for deep joins.
- Positive: audit-chain joins become explicit through `audit_event_id` rather than support-only tribal knowledge.
- Positive: FinOps can attribute observability storage and query cost by workload class and retention tier.
- Positive: tail-sampling evidence remains queryable by policy id and reason code.
- Positive: tenant support can produce scoped trace evidence bundles without exporting unrelated tenant rows.
- Negative: operating two trace-facing backends increases cognitive load.
- Negative: ClickHouse schema design becomes part of the observability contract and needs governance.
- Negative: Tempo and ClickHouse retention can diverge if runbooks are sloppy.
- Negative: production debugging requires knowing when to jump from Tempo to ClickHouse.
- Negative: ClickHouse Cloud introduces managed-provider dependency until the future cell-local migration lands.
- Neutral: Jaeger remains useful for local development but is not a production SLO dependency.
- Neutral: TraceQL and SQL coexist; each must have documented query ownership.
- Neutral: sampling controls remain in OpenTelemetry Collector, not in ClickHouse.
- Neutral: regional packs can force cell-local ClickHouse later without changing application instrumentation.
- Follow-up: author ClickHouse DDL under the observability contracts tree with additive schema versioning.
- Follow-up: add a dashboard linking Tempo trace ids to ClickHouse trace analytics queries.
- Follow-up: add a runbook for ClickHouse Cloud ingestion brownout and backfill replay.
- Follow-up: add a Cedar fragment for `observability::trace_search::read`.
- Follow-up: add a cost budget alert for trace storage by retention class.

## Implementation Notes

- Data shape `OtelSpanRow`: `{trace_id, span_id, parent_span_id, service_name, route_template, span_kind, span_start_ns, span_end_ns, status_code}`.
- Data shape `OtelSpanRow` includes tenant fields `{tenant_hash, tenant_id_enc, home_cell, audience_type, jurisdiction_code, pack_set_hash}`.
- Data shape `OtelSpanRow` includes policy fields `{cedar_policy_id, cedar_decision, sample_policy_id, tail_sample_reason}`.
- Data shape `OtelSpanRow` includes evidence fields `{audit_event_id, deployment_ref, source_sha, release_channel, cost_center_id}`.
- Data shape `TraceRetentionClass`: `{retention_class, raw_days, derived_days, allowed_data_classes, pack_overrides}`.
- Data shape `TraceDropEvent`: `{trace_id, service_name, drop_reason, retained_class, collector_id, audit_event_id}`.
- REST endpoint `POST /v1/observability/otlp/ingest` is mesh-internal only and accepts OTLP batches from collectors.
- REST endpoint `GET /v1/observability/traces/{trace_id}` returns a Cedar-scoped trace summary.
- REST endpoint `POST /v1/observability/traces/query` accepts bounded SQL-template parameters, never raw SQL.
- REST endpoint `GET /v1/observability/traces/{trace_id}/audit-link` returns audit-chain verification pointers.
- REST endpoint `POST /v1/observability/sampling/recipes` changes ADR-0210 recipe state after policy and audit checks.
- Async event `observability.trace.ingested.v1` records accepted batch counts by collector and retention class.
- Async event `observability.trace.dropped.v1` records brownout or policy drop reason.
- Async event `observability.trace.audit_linked.v1` records successful audit event correlation.
- Cedar permit `observability::trace_search::read` requires matching tenant, purpose, time range, and data class.
- Cedar forbid `observability::trace_search::read` blocks raw attribute access when `resource.data_class > principal.max_data_class`.
- Cedar permit `observability::sampling_recipe::update` requires `principal.role in ["sre-admin", "axis-observability-owner"]`.
- Cedar forbid `observability::trace_export::download` blocks cross-tenant exports unless an auditor-scoped engagement exists.
- SLO target `trace_ingest_availability`: 99.9 percent monthly for retained-class OTLP ingestion.
- SLO target `clickhouse_ingest_throughput`: p99 sustained ingest above the current 2x peak trace batch rate.
- SLO target `tail_sample_fidelity`: 100 percent of error, audit-event, and SLO-burn traces retained during non-brownout windows.
- SLO target `trace_query_latency`: p95 below 2 seconds for tenant-scoped 24-hour trace search.
- Dashboard `operator-burn-rate.json` links SLO burn panels to trace ids and ClickHouse query templates.
- Dashboard `tenant-slo-overview.json` shows scoped tenant trace exemplars without raw tenant ids in Prometheus labels.
- Dashboard `gate-eligibility.json` shows promotion verdict traces tied to Oya VCS evidence.
- Local development uses Jaeger all-in-one or Tempo local compose; production tests assert no Jaeger endpoint in prod manifests.
- Collector processors run in order: memory limiter, resource enrichment, PII scrubber, tail sampler, batch, ClickHouse exporter, Tempo exporter.
- Collector scrubber removes raw email, phone, IP, payload body, token, cookie, and free-form user text attributes before export.
- Collector enrichment derives `tenant_hash` and `home_cell` from tenancy, not from caller-provided baggage alone.
- ClickHouse write path uses idempotent batch id `{collector_id}:{batch_seq}:{batch_hash}` to prevent duplicate replay after partition.
- Backfill replay reads from local durable queue capped at 30 minutes or 20 GiB per collector, whichever is smaller.
- Failure behavior: if ClickHouse is unavailable, retain in local buffer, continue Tempo short-retention export, and page at 10 minutes backlog.
- Failure behavior: if the scrubber fails, stop export and emit audit evidence; never store unsanitized spans.
- Failure behavior: if Cedar is unavailable for query, deny tenant-facing trace search and serve only aggregate public dashboards.

## Verification

- Unit test `otel_span_requires_trace_context` rejects spans missing W3C trace ids.
- Unit test `tenant_hash_derived_not_caller_supplied` proves enrichment ignores untrusted baggage tenant ids.
- Unit test `pii_scrubber_removes_payload_body` rejects raw body and token attributes before export.
- Unit test `tail_sample_reason_persisted` asserts ADR-0210 reason codes reach ClickHouse.
- Unit test `trace_search_requires_cedar_scope` denies cross-tenant trace reads.
- Property test `otlp_replay_idempotent_by_batch_hash` generates duplicate collector batches.
- Property test `trace_schema_additive_only` rejects dropping or renaming existing columns.
- Integration test `tempo_to_clickhouse_trace_link_roundtrip` verifies trace id lookup across stores.
- Integration test `audit_event_id_join_finds_state_change_trace` proves audit-chain correlation works.
- Integration test `brownout_drops_baseline_before_error_trace` verifies retention priority.
- Integration test `clickhouse_unavailable_buffers_then_replays` covers 10-minute outage recovery.
- Contract test `query_api_uses_templates_not_raw_sql` prevents unbounded SQL injection surfaces.
- Contract test `tenant_export_excludes_other_tenant_rows` verifies Cedar and SQL filters compose.
- Load test `trace_ingest_2x_peak_for_30_minutes` validates batch and ClickHouse throughput.
- Load test `trace_query_24h_p95_under_2s` validates tenant-scoped search.
- Chaos test `scrubber_panic_stops_export` proves fail-closed behavior.
- Chaos test `collector_memory_pressure_preserves_audit_traces` validates retention class priority.
- Dashboard check `operator-burn-rate` contains ClickHouse trace drill-down links.
- Dashboard check `gate-eligibility` shows sample policy ids for held promotions.
- Metric check `observability_trace_dropped_total` is zero for `reason="scrubber_error"` outside incident tests.
- Metric check `observability_clickhouse_backlog_seconds` pages above 600 seconds.
- Static check production manifests contain no Jaeger collector endpoint.
- Static check every trace query endpoint declares a Cedar action id.
- Oya VCS evidence must include line count, root ADR cite count, and reference count for this ADR.

## References

- OpenTelemetry Specification, current official documentation: https://opentelemetry.io/docs/specs/otel/
- OpenTelemetry Semantic Conventions documentation: https://opentelemetry.io/docs/concepts/semantic-conventions/
- W3C Trace Context Recommendation: https://www.w3.org/TR/trace-context/
- ClickHouse Observability use case documentation: https://clickhouse.com/use-cases/observability
- ClickHouse OpenTelemetry operations documentation: https://clickhouse.com/docs/en/operations/opentelemetry
- Grafana Tempo documentation: https://grafana.com/docs/tempo/latest/
- Jaeger documentation: https://www.jaegertracing.io/docs/
- Google Dapper paper, "Dapper, a Large-Scale Distributed Systems Tracing Infrastructure."
- OpenTelemetry Collector tail sampling processor documentation.
- Cedar Policy Language schema and authorization documentation: https://docs.cedarpolicy.com/
- ADR-0042, ADR-0186, ADR-0210, ADR-0243, ADR-0244, ADR-0245, and ADR-0263.
