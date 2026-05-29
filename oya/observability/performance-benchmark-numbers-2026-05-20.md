# Observability Performance Benchmark Numbers - 2026-05-20

## Header - Five-Citation Anchor Block

1. Canonical sequence anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2240` for §D-15 multi-context, `2243-2581` for §D-16 OpenTofu, `2646-2927` for §D-17 OS matrix, `3047-3435` for §D-18 Rust-strict, `3493-3796` for §D-19 OCI Always Free, and `3831-4224` for §D-20 audit rules.
2. Machine-readable plan anchor: `specs/master-plan-sequencing.json:704-868` for deployment contexts, OpenTofu substrate, supported OSes, language policy, and OCI Always Free.
3. Microservice product anchor: `microservices/observability/PRD.md:20-26`, `37-48`, `54-61`, `90-94`, and `231-240`.
4. Microservice architecture anchor: `microservices/observability/ARCHITECTURE.md:445-456`, `631-642`, and `693-704`.
5. Documentation rigor anchor: `docs/standards/documentation-rigor.md:58-129` and `133-156`.

Methodology disclosure: the Oyatie numbers below are target envelopes, not measured benchmarks.
Measured results must be added during the build phase using ADR-0212-style benchmark artifacts and ADR-0328 §D-20.152 disclosures.
The counterpart numbers are public-documentation limits, pricing-envelope capacities, or capacity-planning examples from official product documentation.
Where a vendor does not publish a direct comparable SLO, the row is marked as a documented limit rather than a measured latency claim.

## §1 Methodology

1. Benchmark claim type: target envelope for design acceptance, not production proof.
2. Measurement status: no local benchmark binary exists under `microservices/observability/src`, so no measured p50/p95/p99 values are available.
3. Primary latency dimension: telemetry write acknowledgement p50/p95/p99 for metrics, logs, traces, and profile samples.
4. Primary query dimension: dashboard load p95/p99, PromQL-like query p95/p99, LogQL-like query p95/p99, trace lookup p95/p99.
5. Primary throughput dimension: metric points per second, spans per second, log GiB per day, profile samples per second.
6. Primary concurrency dimension: concurrent dashboards, concurrent API queries, active tenants per cell, active services per tenant.
7. Primary scale ceiling dimension: active series, hot retention days, cold retention days, and cell-level storage envelope.
8. Workload A metrics: 10 labels per series median, 25 labels per series p95, one tenant_id on every event, one service_id on every event.
9. Workload B logs: 1 KiB median event, 8 KiB p95 event, 1 percent structured exception events, 100 percent tenant-scoped events.
10. Workload C traces: 8 spans median per trace, 75 spans p95 per trace, 1 KiB span payload median, 8 KiB p95 span payload.
11. Workload D profiles: 10-second continuous profile windows, service and build identifiers attached, symbol files retained per tier.
12. Workload E dashboards: 12 panels per dashboard, four metric panels, four log panels, two trace panels, two SLO panels.
13. Workload F SLO burn: 30-day rolling window, 1-hour fast burn, 6-hour medium burn, 24-hour slow burn.
14. Workload G tenant isolation: all rows include tenant_id, deployment_context, service_id, region, and retention_class.
15. OS disclosure baseline: Linux arm64 on OCI Ampere for guest-on-oci demo_trial tenant_class because ADR-0328 §D-19 makes Always Free a hard sub-profile.
16. OS disclosure paid baseline: Linux amd64 and arm64 on Tier-1 Kubernetes nodes for paid tenant_class baseline, paid tenant_class scale, and compliance_pack-gated paid tenant_class.
17. OS disclosure desktop: macOS Apple Silicon M5+ and WinUI3 are client-side instrumentation targets only, not server runtime targets.
18. Architecture disclosure: current service docs name Helm/Kustomize and one Terraform file, so the target matrix assumes future OpenTofu modules are built.
19. Tenant class demo_trial tenant_class: single small tenant or internal development tenant with strict retention and sampling limits.
20. Tenant class paid tenant_class baseline: paid baseline tenant with production SLOs and standard retention.
21. Tenant class paid tenant_class scale: multi-region production tenant with high-cardinality telemetry and longer retention.
22. Tenant class compliance_pack-gated paid tenant_class: sovereign or single-tenant pack with strict custody, HSM-backed keys, and context-local storage.
23. Context baseline: oyatie-public-cloud targets are the service-owned public SaaS baseline.
24. Context baseline: guest-on-aws targets assume customer VPC deployment and cloud-provider limits outside Oyatie control.
25. Context baseline: guest-on-oci targets include the Always Free demo_trial tenant_class sub-profile and paid OCI tiers above demo_trial tenant_class.
26. Context baseline: on-prem targets assume customer-owned hardware with Oyatie support matrix boundaries.
27. Context baseline: colo targets assume dedicated racks with lower storage/network jitter than generic on-prem.
28. Context baseline: oyatie-as-cloud-provider targets assume Oyatie-controlled IaaS and highest horizontal scale.
29. Availability target definition: percentage applies to telemetry ingest and query API availability, not customer app uptime.
30. Retention target definition: hot retention means queryable without restore; cold retention means retained for audit or rehydration.
31. Query latency target definition: p99 under normal cardinality guardrails, not adversarial unbounded wildcard scans.
32. Ingest latency target definition: p99 acknowledgement after durable write into the tier-appropriate buffer or store.
33. SLO detection target definition: p99 time from window violation to alertable event.
34. Scale target definition: sustained one-hour load, with burst values intentionally excluded until measured tests exist.
35. Storage target definition: post-compression usable telemetry storage, excluding object-store replication overhead.
36. Comparison posture: Datadog and New Relic publish many product limits but not a universal ingest/query benchmark.
37. Comparison posture: Grafana Mimir publishes capacity-planning examples that are useful for metric-scale targets.
38. Comparison posture: Grafana Cloud pricing publishes free-tier telemetry quotas useful for demo_trial tenant_class comparison.
39. Data quality rule: target values that cannot be traced to public counterpart limits or internal tier docs are marked "Oyatie design target".
40. Anti-placeholder rule: every target row below includes a concrete value, context, and rationale.

## §2 Counterpart Numbers

### Datadog public documentation numbers

1. Datadog Logs HTTP intake payload maximum: 5 MB uncompressed per payload, source `https://docs.datadoghq.com/api/latest/logs/`.
2. Datadog log event maximum: 1 MB per individual log event, source `https://docs.datadoghq.com/api/latest/logs/`.
3. Datadog Logs API array maximum: 1,000 log events per array payload, source `https://docs.datadoghq.com/api/latest/logs/`.
4. Datadog metric retention: long-window metric retention is documented in the Datadog data-retention matrix, source `https://docs.datadoghq.com/data_security/data_retention_periods/`.
5. Datadog APM scope: traces, service maps, deployment tracking, error tracking, and service health are documented product surfaces, source `https://docs.datadoghq.com/getting_started/application/`.
6. Datadog infrastructure scope: host, container, Kubernetes, network, and cloud-resource observability are documented product surfaces, source `https://docs.datadoghq.com/`.
7. Datadog synthetics scope: API and browser synthetic checks are documented product surfaces, source `https://docs.datadoghq.com/`.
8. Datadog RUM/session scope: browser and mobile real-user monitoring are documented product surfaces, source `https://docs.datadoghq.com/`.
9. Datadog profiling scope: continuous profiler is a documented product surface, source `https://docs.datadoghq.com/`.
10. Datadog comparison status: public docs disclose API payload limits but not a single universal p99 query latency benchmark.
11. Datadog target implication for Oyatie: keep log event size below 1 MB to preserve ingest compatibility with common industry envelopes.
12. Datadog target implication for Oyatie: design batch ingest around sub-5 MB payloads for connector compatibility.

### New Relic public documentation numbers

1. New Relic NRDB record ingest ceiling: 55 million records per account per minute is listed as a system limit, source `https://docs.newrelic.com/docs/data-apis/manage-data/view-system-limits/`.
2. New Relic event attribute ceiling: 254 attributes per event is listed as a system limit, source `https://docs.newrelic.com/docs/data-apis/manage-data/view-system-limits/`.
3. New Relic user-facing surface: 30-plus capabilities are described across APM, infrastructure, logs, browser, mobile, synthetics, errors, and service levels, source `https://docs.newrelic.com/docs/new-relic-solutions/get-started/intro-new-relic/`.
4. New Relic OpenTelemetry APM UI: OpenTelemetry data can populate New Relic APM experiences, source `https://docs.newrelic.com/docs/opentelemetry/get-started/apm-monitoring/opentelemetry-apm-ui/`.
5. New Relic service levels: service-level alerting and burn-rate style alerting are documented, source `https://docs.newrelic.com/docs/service-level-management/alerts-slm/`.
6. New Relic logs scope: logs and logs-in-context are documented product surfaces, source `https://docs.newrelic.com/docs/new-relic-solutions/get-started/intro-new-relic/`.
7. New Relic infrastructure scope: hosts, containers, Kubernetes, cloud integrations, and network telemetry are documented surfaces.
8. New Relic synthetics scope: scripted browser/API monitors are documented surfaces.
9. New Relic errors inbox scope: error grouping and incident context are documented surfaces.
10. New Relic comparison status: public docs disclose ingest/system limits but not a single universal p99 dashboard benchmark.
11. New Relic target implication for Oyatie: per-tenant event attributes should stay below 254 unless transformed before ingest.
12. New Relic target implication for Oyatie: compliance_pack-gated paid tenant_class account-scale targets should be evaluated against tens of millions of records per minute.

### Grafana Cloud public documentation numbers

1. Grafana Cloud Free metrics quota: 10,000 metric series, source `https://grafana.com/pricing/`.
2. Grafana Cloud Free logs quota: 50 GB logs, source `https://grafana.com/pricing/`.
3. Grafana Cloud Free traces quota: 50 GB traces, source `https://grafana.com/pricing/`.
4. Grafana Cloud Free profiles quota: 50 GB profiles, source `https://grafana.com/pricing/`.
5. Grafana Cloud Free retention: 14 days for the free telemetry envelope, source `https://grafana.com/pricing/`.
6. Grafana Mimir 1M samples/sec example: about 140 CPU cores, 800 GiB memory, and 2 TiB storage per day, source `https://grafana.com/docs/mimir/latest/manage/run-production-environment/planning-capacity/`.
7. Grafana Mimir 10M samples/sec example: about 900 CPU cores, 7,300 GiB memory, and 21 TiB storage per day, source `https://grafana.com/docs/mimir/latest/manage/run-production-environment/planning-capacity/`.
8. Grafana Cloud traces: Tempo-backed traces, span metrics, service graph, and trace-to-logs/metrics workflows are documented, source `https://grafana.com/docs/grafana-cloud/send-data/traces/use-traces-with-grafana/`.
9. Grafana Cloud profile scope: Pyroscope profiles are part of the cloud product surface, source `https://grafana.com/docs/grafana-cloud/introduction/`.
10. Grafana Cloud alerting scope: Grafana Alerting, IRM, on-call, and incident workflows are documented surfaces.
11. Grafana comparison status: metric-scale capacity examples are stronger public benchmark anchors than Datadog/New Relic query latency docs.
12. Grafana target implication for Oyatie: demo_trial tenant_class must be explicitly below or equal to a free/small envelope, while paid tenant_class scale/compliance_pack-gated paid tenant_class must be benchmarked against Mimir 1M-plus sample/sec examples.

## §3 Oyatie Target Numbers by Tier and Deployment Context

### §3.1 oyatie-public-cloud demo_trial tenant_class

1. Metric write target: 100,000 points/sec sustained, because public-cloud demo_trial tenant_class is paid small-production rather than OCI Always Free.
2. Active series target: 250,000 active series, with hard cardinality guardrails at tenant and service boundaries.
3. Log ingest target: 25 GiB/day hot logs, compressed storage, tenant-scoped indexes.
4. Trace ingest target: 25,000 spans/sec sustained, with tail sampling enabled for noisy tenants.
5. Profile ingest target: 2,500 profile samples/sec sustained, with symbol retention limited to hot window.
6. Dashboard latency target: p99 1,800 ms for 12-panel dashboards under normal cardinality.
7. Metric query latency target: p99 900 ms for 30-day window queries below active-series guardrails.
8. Log query latency target: p99 2,500 ms for 24-hour scoped searches.
9. Trace lookup latency target: p99 1,500 ms for trace-id lookup and p99 3,000 ms for attribute search.
10. SLO burn detection target: p99 90 seconds from violation to alertable event.
11. Retention target: 30 days hot metrics/logs/traces and 180 days cold audit evidence.
12. Tenant scale target: 10 tenants per cell and 50 services per tenant.

### §3.2 oyatie-public-cloud paid tenant_class baseline

1. Metric write target: 500,000 points/sec sustained.
2. Active series target: 1,500,000 active series per cell.
3. Log ingest target: 200 GiB/day hot logs.
4. Trace ingest target: 200,000 spans/sec sustained.
5. Profile ingest target: 20,000 profile samples/sec sustained.
6. Dashboard latency target: p99 1,200 ms.
7. Metric query latency target: p99 500 ms for bounded PromQL.
8. Log query latency target: p99 1,500 ms for indexed 24-hour searches.
9. Trace lookup latency target: p99 800 ms by trace id and p99 2,000 ms by attributes.
10. SLO burn detection target: p99 45 seconds.
11. Retention target: 90 days hot telemetry and 365 days cold evidence.
12. Tenant scale target: 25 tenants per cell and 150 services per tenant.

### §3.3 oyatie-public-cloud paid tenant_class scale

1. Metric write target: 5,000,000 points/sec sustained.
2. Active series target: 15,000,000 active series per region cell.
3. Log ingest target: 2 TiB/day hot logs.
4. Trace ingest target: 1,000,000 spans/sec sustained.
5. Profile ingest target: 100,000 profile samples/sec sustained.
6. Dashboard latency target: p99 900 ms.
7. Metric query latency target: p99 300 ms for bounded queries and p99 1,200 ms for large fanout queries.
8. Log query latency target: p99 900 ms for indexed searches.
9. Trace lookup latency target: p99 350 ms by trace id and p99 1,200 ms by attributes.
10. SLO burn detection target: p99 15 seconds.
11. Retention target: 180 days hot telemetry and 7 years audit evidence.
12. Tenant scale target: 75 tenants per cell and 500 services per tenant.

### §3.4 oyatie-public-cloud compliance_pack-gated paid tenant_class

1. Metric write target: 2,000,000 points/sec per sovereign pack, horizontally multiplied by pack count.
2. Active series target: 8,000,000 active series per sovereign pack.
3. Log ingest target: 1 TiB/day per sovereign pack with customer-owned keys.
4. Trace ingest target: 500,000 spans/sec per sovereign pack.
5. Profile ingest target: 75,000 profile samples/sec per sovereign pack.
6. Dashboard latency target: p99 1,000 ms inside pack-local control plane.
7. Metric query latency target: p99 400 ms for pack-local bounded queries.
8. Log query latency target: p99 1,200 ms because custody and encryption checks add overhead.
9. Trace lookup latency target: p99 500 ms by trace id and p99 1,500 ms by attributes.
10. SLO burn detection target: p99 20 seconds because HSM-backed signing is in the alert path.
11. Retention target: 365 days hot telemetry and 7 years WORM evidence.
12. Tenant scale target: 1 tenant per pack by default, 5 regulated sub-tenants when explicitly configured.

### §3.5 guest-on-aws demo_trial tenant_class

1. Metric write target: 80,000 points/sec sustained inside customer VPC.
2. Active series target: 200,000 active series.
3. Log ingest target: 20 GiB/day.
4. Trace ingest target: 20,000 spans/sec.
5. Profile ingest target: 2,000 profile samples/sec.
6. Dashboard latency target: p99 2,000 ms because network path and storage class are customer-controlled.
7. Metric query latency target: p99 1,000 ms.
8. Log query latency target: p99 2,800 ms.
9. Trace lookup latency target: p99 1,800 ms by trace id.
10. SLO burn detection target: p99 120 seconds.
11. Retention target: 30 days hot and 180 days cold when customer storage meets baseline.
12. Tenant scale target: 5 tenants per cell and 40 services per tenant.

### §3.6 guest-on-aws paid tenant_class baseline

1. Metric write target: 400,000 points/sec sustained.
2. Active series target: 1,200,000 active series.
3. Log ingest target: 160 GiB/day.
4. Trace ingest target: 160,000 spans/sec.
5. Profile ingest target: 16,000 profile samples/sec.
6. Dashboard latency target: p99 1,400 ms.
7. Metric query latency target: p99 650 ms.
8. Log query latency target: p99 1,800 ms.
9. Trace lookup latency target: p99 950 ms by trace id.
10. SLO burn detection target: p99 60 seconds.
11. Retention target: 90 days hot and 365 days cold.
12. Tenant scale target: 20 tenants per cell and 125 services per tenant.

### §3.7 guest-on-aws paid tenant_class scale

1. Metric write target: 4,000,000 points/sec sustained.
2. Active series target: 12,000,000 active series.
3. Log ingest target: 1.6 TiB/day.
4. Trace ingest target: 800,000 spans/sec.
5. Profile ingest target: 80,000 profile samples/sec.
6. Dashboard latency target: p99 1,100 ms.
7. Metric query latency target: p99 400 ms.
8. Log query latency target: p99 1,200 ms.
9. Trace lookup latency target: p99 500 ms by trace id.
10. SLO burn detection target: p99 25 seconds.
11. Retention target: 180 days hot and 7 years cold evidence.
12. Tenant scale target: 60 tenants per cell and 400 services per tenant.

### §3.8 guest-on-aws compliance_pack-gated paid tenant_class

1. Metric write target: 1,600,000 points/sec per customer-controlled pack.
2. Active series target: 6,000,000 active series per pack.
3. Log ingest target: 800 GiB/day per pack.
4. Trace ingest target: 400,000 spans/sec per pack.
5. Profile ingest target: 60,000 profile samples/sec per pack.
6. Dashboard latency target: p99 1,250 ms.
7. Metric query latency target: p99 500 ms.
8. Log query latency target: p99 1,500 ms.
9. Trace lookup latency target: p99 650 ms by trace id.
10. SLO burn detection target: p99 30 seconds.
11. Retention target: 365 days hot and 7 years WORM or customer-equivalent evidence.
12. Tenant scale target: single-tenant pack by default.

### §3.9 guest-on-oci demo_trial tenant_class - Always Free

1. Metric write target: 25,000 points/sec sustained within 4 OCPU and 24 GiB memory.
2. Active series target: 50,000 active series, explicitly below Grafana Cloud Free metric quota times five for Oyatie internal cardinality.
3. Log ingest target: 1 GiB/day so 200 GiB block storage remains viable with retention and replicas.
4. Trace ingest target: 5,000 spans/sec with aggressive sampling after the first 24 hours.
5. Profile ingest target: 500 profile samples/sec and disabled by default for idle tenants.
6. Dashboard latency target: p99 3,500 ms because OCI Always Free compute and storage are constrained.
7. Metric query latency target: p99 2,000 ms for 7-day bounded queries.
8. Log query latency target: p99 5,000 ms for 24-hour indexed searches.
9. Trace lookup latency target: p99 3,500 ms by trace id.
10. SLO burn detection target: p99 180 seconds.
11. Retention target: 14 days hot telemetry and 30 days compressed audit evidence.
12. Tenant scale target: 1 production-light tenant or 3 development tenants.

### §3.10 guest-on-oci paid tenant_class baseline

1. Metric write target: 450,000 points/sec sustained on paid OCI compute.
2. Active series target: 1,300,000 active series.
3. Log ingest target: 180 GiB/day.
4. Trace ingest target: 180,000 spans/sec.
5. Profile ingest target: 18,000 profile samples/sec.
6. Dashboard latency target: p99 1,300 ms.
7. Metric query latency target: p99 600 ms.
8. Log query latency target: p99 1,700 ms.
9. Trace lookup latency target: p99 900 ms by trace id.
10. SLO burn detection target: p99 55 seconds.
11. Retention target: 90 days hot and 365 days cold.
12. Tenant scale target: 20 tenants per cell and 125 services per tenant.

### §3.11 guest-on-oci paid tenant_class scale

1. Metric write target: 4,500,000 points/sec sustained.
2. Active series target: 13,000,000 active series.
3. Log ingest target: 1.8 TiB/day.
4. Trace ingest target: 900,000 spans/sec.
5. Profile ingest target: 90,000 profile samples/sec.
6. Dashboard latency target: p99 1,000 ms.
7. Metric query latency target: p99 350 ms.
8. Log query latency target: p99 1,050 ms.
9. Trace lookup latency target: p99 450 ms by trace id.
10. SLO burn detection target: p99 20 seconds.
11. Retention target: 180 days hot and 7 years cold evidence.
12. Tenant scale target: 70 tenants per cell and 450 services per tenant.

### §3.12 guest-on-oci compliance_pack-gated paid tenant_class

1. Metric write target: 1,800,000 points/sec per OCI sovereign pack.
2. Active series target: 7,000,000 active series per pack.
3. Log ingest target: 900 GiB/day per pack.
4. Trace ingest target: 450,000 spans/sec per pack.
5. Profile ingest target: 65,000 profile samples/sec per pack.
6. Dashboard latency target: p99 1,150 ms.
7. Metric query latency target: p99 450 ms.
8. Log query latency target: p99 1,350 ms.
9. Trace lookup latency target: p99 600 ms by trace id.
10. SLO burn detection target: p99 28 seconds.
11. Retention target: 365 days hot and 7 years WORM evidence.
12. Tenant scale target: single-tenant pack by default.

### §3.13 on-prem demo_trial tenant_class

1. Metric write target: 60,000 points/sec on reference two-node x86_64 or arm64 customer hardware.
2. Active series target: 150,000 active series.
3. Log ingest target: 15 GiB/day.
4. Trace ingest target: 15,000 spans/sec.
5. Profile ingest target: 1,500 profile samples/sec.
6. Dashboard latency target: p99 2,500 ms.
7. Metric query latency target: p99 1,400 ms.
8. Log query latency target: p99 3,500 ms.
9. Trace lookup latency target: p99 2,200 ms by trace id.
10. SLO burn detection target: p99 150 seconds.
11. Retention target: 21 days hot and 90 days cold.
12. Tenant scale target: 3 tenants per cell and 25 services per tenant.

### §3.14 on-prem paid tenant_class baseline

1. Metric write target: 300,000 points/sec.
2. Active series target: 900,000 active series.
3. Log ingest target: 120 GiB/day.
4. Trace ingest target: 120,000 spans/sec.
5. Profile ingest target: 12,000 profile samples/sec.
6. Dashboard latency target: p99 1,700 ms.
7. Metric query latency target: p99 800 ms.
8. Log query latency target: p99 2,100 ms.
9. Trace lookup latency target: p99 1,100 ms by trace id.
10. SLO burn detection target: p99 75 seconds.
11. Retention target: 60 days hot and 365 days cold when customer storage meets baseline.
12. Tenant scale target: 12 tenants per cell and 80 services per tenant.

### §3.15 on-prem paid tenant_class scale

1. Metric write target: 3,000,000 points/sec.
2. Active series target: 9,000,000 active series.
3. Log ingest target: 1.2 TiB/day.
4. Trace ingest target: 600,000 spans/sec.
5. Profile ingest target: 60,000 profile samples/sec.
6. Dashboard latency target: p99 1,300 ms.
7. Metric query latency target: p99 500 ms.
8. Log query latency target: p99 1,500 ms.
9. Trace lookup latency target: p99 700 ms by trace id.
10. SLO burn detection target: p99 35 seconds.
11. Retention target: 120 days hot and 7 years cold evidence.
12. Tenant scale target: 40 tenants per cell and 250 services per tenant.

### §3.16 on-prem compliance_pack-gated paid tenant_class

1. Metric write target: 1,200,000 points/sec per regulated customer pack.
2. Active series target: 5,000,000 active series per pack.
3. Log ingest target: 600 GiB/day per pack.
4. Trace ingest target: 300,000 spans/sec per pack.
5. Profile ingest target: 45,000 profile samples/sec per pack.
6. Dashboard latency target: p99 1,500 ms.
7. Metric query latency target: p99 650 ms.
8. Log query latency target: p99 1,800 ms.
9. Trace lookup latency target: p99 800 ms by trace id.
10. SLO burn detection target: p99 40 seconds.
11. Retention target: 365 days hot and 7 years customer-custody evidence.
12. Tenant scale target: single-tenant pack by default.

### §3.17 colo demo_trial tenant_class

1. Metric write target: 90,000 points/sec on reference small rack footprint.
2. Active series target: 225,000 active series.
3. Log ingest target: 22 GiB/day.
4. Trace ingest target: 22,000 spans/sec.
5. Profile ingest target: 2,200 profile samples/sec.
6. Dashboard latency target: p99 1,900 ms.
7. Metric query latency target: p99 950 ms.
8. Log query latency target: p99 2,600 ms.
9. Trace lookup latency target: p99 1,600 ms by trace id.
10. SLO burn detection target: p99 100 seconds.
11. Retention target: 30 days hot and 180 days cold.
12. Tenant scale target: 8 tenants per cell and 45 services per tenant.

### §3.18 colo paid tenant_class baseline

1. Metric write target: 475,000 points/sec.
2. Active series target: 1,400,000 active series.
3. Log ingest target: 190 GiB/day.
4. Trace ingest target: 190,000 spans/sec.
5. Profile ingest target: 19,000 profile samples/sec.
6. Dashboard latency target: p99 1,250 ms.
7. Metric query latency target: p99 550 ms.
8. Log query latency target: p99 1,600 ms.
9. Trace lookup latency target: p99 850 ms by trace id.
10. SLO burn detection target: p99 50 seconds.
11. Retention target: 90 days hot and 365 days cold.
12. Tenant scale target: 24 tenants per cell and 140 services per tenant.

### §3.19 colo paid tenant_class scale

1. Metric write target: 4,750,000 points/sec.
2. Active series target: 14,000,000 active series.
3. Log ingest target: 1.9 TiB/day.
4. Trace ingest target: 950,000 spans/sec.
5. Profile ingest target: 95,000 profile samples/sec.
6. Dashboard latency target: p99 950 ms.
7. Metric query latency target: p99 325 ms.
8. Log query latency target: p99 975 ms.
9. Trace lookup latency target: p99 425 ms by trace id.
10. SLO burn detection target: p99 18 seconds.
11. Retention target: 180 days hot and 7 years cold evidence.
12. Tenant scale target: 70 tenants per cell and 475 services per tenant.

### §3.20 colo compliance_pack-gated paid tenant_class

1. Metric write target: 1,900,000 points/sec per colo sovereign pack.
2. Active series target: 7,500,000 active series per pack.
3. Log ingest target: 950 GiB/day per pack.
4. Trace ingest target: 475,000 spans/sec per pack.
5. Profile ingest target: 70,000 profile samples/sec per pack.
6. Dashboard latency target: p99 1,050 ms.
7. Metric query latency target: p99 425 ms.
8. Log query latency target: p99 1,300 ms.
9. Trace lookup latency target: p99 550 ms by trace id.
10. SLO burn detection target: p99 25 seconds.
11. Retention target: 365 days hot and 7 years WORM evidence.
12. Tenant scale target: single-tenant pack by default.

### §3.21 oyatie-as-cloud-provider demo_trial tenant_class

1. Metric write target: 125,000 points/sec.
2. Active series target: 300,000 active series.
3. Log ingest target: 30 GiB/day.
4. Trace ingest target: 30,000 spans/sec.
5. Profile ingest target: 3,000 profile samples/sec.
6. Dashboard latency target: p99 1,600 ms.
7. Metric query latency target: p99 800 ms.
8. Log query latency target: p99 2,200 ms.
9. Trace lookup latency target: p99 1,300 ms by trace id.
10. SLO burn detection target: p99 75 seconds.
11. Retention target: 30 days hot and 180 days cold.
12. Tenant scale target: 12 tenants per cell and 60 services per tenant.

### §3.22 oyatie-as-cloud-provider paid tenant_class baseline

1. Metric write target: 650,000 points/sec.
2. Active series target: 2,000,000 active series.
3. Log ingest target: 260 GiB/day.
4. Trace ingest target: 260,000 spans/sec.
5. Profile ingest target: 26,000 profile samples/sec.
6. Dashboard latency target: p99 1,050 ms.
7. Metric query latency target: p99 450 ms.
8. Log query latency target: p99 1,300 ms.
9. Trace lookup latency target: p99 700 ms by trace id.
10. SLO burn detection target: p99 40 seconds.
11. Retention target: 90 days hot and 365 days cold.
12. Tenant scale target: 35 tenants per cell and 180 services per tenant.

### §3.23 oyatie-as-cloud-provider paid tenant_class scale

1. Metric write target: 6,500,000 points/sec.
2. Active series target: 20,000,000 active series.
3. Log ingest target: 2.6 TiB/day.
4. Trace ingest target: 1,300,000 spans/sec.
5. Profile ingest target: 130,000 profile samples/sec.
6. Dashboard latency target: p99 750 ms.
7. Metric query latency target: p99 250 ms.
8. Log query latency target: p99 800 ms.
9. Trace lookup latency target: p99 300 ms by trace id.
10. SLO burn detection target: p99 12 seconds.
11. Retention target: 180 days hot and 7 years cold evidence.
12. Tenant scale target: 100 tenants per cell and 650 services per tenant.

### §3.24 oyatie-as-cloud-provider compliance_pack-gated paid tenant_class

1. Metric write target: 2,500,000 points/sec per Oyatie-controlled sovereign pack.
2. Active series target: 10,000,000 active series per pack.
3. Log ingest target: 1.25 TiB/day per pack.
4. Trace ingest target: 625,000 spans/sec per pack.
5. Profile ingest target: 90,000 profile samples/sec per pack.
6. Dashboard latency target: p99 850 ms.
7. Metric query latency target: p99 350 ms.
8. Log query latency target: p99 1,000 ms.
9. Trace lookup latency target: p99 400 ms by trace id.
10. SLO burn detection target: p99 18 seconds.
11. Retention target: 365 days hot and 7 years WORM evidence.
12. Tenant scale target: single-tenant pack by default, 10 regulated sub-tenants maximum.

## §4 Per-Context Overlay

1. oyatie-public-cloud overlay: treat the values above as the reference envelope for shared managed SaaS.
2. oyatie-public-cloud overlay: region expansion must preserve p99 dashboard and SLO detection targets before claims advance from paid tenant_class baseline to paid tenant_class scale.
3. oyatie-public-cloud overlay: demo_trial tenant_class is not the OCI Always Free profile and can use paid managed object/block storage.
4. guest-on-aws overlay: subtract 15 to 20 percent from baseline ingest until OpenTofu modules prove storage, IAM, and network paths.
5. guest-on-aws overlay: support must document customer-account responsibility for EBS/FSx/S3/KMS equivalents before targets are contractual.
6. guest-on-aws overlay: direct AWS SDK calls from business logic would be a canonical-direction violation; IaC modules must own cloud integration.
7. guest-on-oci overlay: demo_trial tenant_class is explicitly constrained by 4 OCPU, 24 GiB RAM, 200 GiB block, 10 GiB object, and 10 Mbps load balancer.
8. guest-on-oci overlay: paid tenant_class baseline and above may leave Always Free but must still keep demo_trial tenant_class as a deployable sub-profile.
9. guest-on-oci overlay: OCI Monitoring/Logging/Notifications may be integrated only through the context module contract.
10. on-prem overlay: lower demo_trial tenant_class and paid tenant_class baseline values reflect unknown customer disk, network, and operations maturity.
11. on-prem overlay: paid tenant_class scale and compliance_pack-gated paid tenant_class claims require the OS support matrix because supportability spans 13 Tier-1 OSes.
12. on-prem overlay: support must define hardware certification before performance numbers become contractual.
13. colo overlay: values are higher than generic on-prem because dedicated rack/network assumptions reduce noisy-neighbor risk.
14. colo overlay: values remain below Oyatie-as-cloud-provider because colocation operations still depend on third-party facility controls.
15. colo overlay: compliance_pack-gated paid tenant_class must preserve HSM and WORM evidence locally; otherwise the pack is paid tenant_class scale, not compliance_pack-gated paid tenant_class.
16. oyatie-as-cloud-provider overlay: highest scale targets assume Oyatie controls the IaaS substrate and can co-design storage/network topology.
17. oyatie-as-cloud-provider overlay: paid tenant_class scale metric target intentionally approaches Grafana Mimir public 10M-sample planning examples while remaining below them.
18. oyatie-as-cloud-provider overlay: compliance_pack-gated paid tenant_class is lower per pack than paid tenant_class scale aggregate because custody isolation is prioritized over shared-scale efficiency.
19. all-context overlay: every target is blocked from production claim until canonical OpenTofu directories exist.
20. all-context overlay: every target is blocked from Tier-1 support claim until supported-oses.json and CI lanes exist.

## §5 Comparison Narrative

1. Datadog log payload limits: Oyatie targets should stay compatible with 1 MB event and 5 MB payload envelopes; current docs do not state event-size limits.
2. Datadog APM breadth: Oyatie target numbers cover trace ingest and lookup but not full APM service-map UX; status is catch-up.
3. Datadog infrastructure breadth: Oyatie targets cover metrics but not network device or cloud-resource integration breadth; status is catch-up.
4. Datadog SLO posture: Oyatie SLO burn targets are explicit and may be ahead if implemented with release gates; current docs remain unmeasured.
5. New Relic record-scale ceiling: Oyatie compliance_pack-gated paid tenant_class per-pack targets are below New Relic's public NRDB account/minute limit; status is catch-up for aggregate event scale.
6. New Relic attribute ceiling: Oyatie should set a tenant-safe attribute limit below 254 attributes per event; current docs do not define this.
7. New Relic service levels: Oyatie release-gate SLO coupling is additive but not measured; status is design-ahead and implementation-catch-up.
8. Grafana Free tier: OCI Always Free demo_trial tenant_class targets are near the same spirit as Grafana Cloud Free but constrained by OCI resource limits; status is partial parity.
9. Grafana Mimir 1M samples/sec: Oyatie paid tenant_class scale public-cloud and OCI/colo targets exceed 1M samples/sec and therefore require serious Mimir-style capacity proof.
10. Grafana Mimir 10M samples/sec: Oyatie-as-cloud-provider paid tenant_class scale at 6.5M points/sec is below the 10M example but in the same planning family.
11. Grafana traces: Oyatie has ADR-OBS-001 trace-store strategy, but parity requires measured trace-search latency and service-graph features.
12. Grafana profiles: Oyatie names Pyroscope but lacks profile retention and symbol handling targets in current docs; this report supplies targets.
13. Public benchmark gap: none of the counterpart docs provide one-to-one p99 dashboard numbers across every product surface.
14. Oyatie benchmark gap: no local source tree or test harness currently proves any number in this document.
15. Production claim boundary: all numbers are acceptance targets until implemented, load-tested, and attached to signed benchmark artifacts.
