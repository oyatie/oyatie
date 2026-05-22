---
doc_class: FeatureParityMatrix
microservice: observability
audit_date: 2026-05-20
status: landed
---

# Observability Feature-Parity Matrix - 2026-05-20

## Five-Citation Anchor Block

1. Canonical sequence and batch discipline: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-4235`.
2. Master-plan machine source: `specs/master-plan-sequencing.json:704-868`.
3. Service PRD source: `microservices/observability/PRD.md:20-309`.
4. Service architecture source: `microservices/observability/ARCHITECTURE.md:1-754`.
5. Documentation-rigor source: `docs/standards/documentation-rigor.md:58-156` and `docs/standards/documentation-rigor.md:222-261`.

## Source Notes

Source note 1: Datadog public docs opened during this audit at `https://docs.datadoghq.com/`, `https://docs.datadoghq.com/getting_started/application/`, `https://docs.datadoghq.com/data_security/data_retention_periods/`, and `https://docs.datadoghq.com/api/latest/logs/`.
Source note 2: New Relic public docs opened during this audit at `https://docs.newrelic.com/docs/new-relic-solutions/get-started/intro-new-relic/`, `https://docs.newrelic.com/docs/opentelemetry/get-started/apm-monitoring/opentelemetry-apm-ui/`, `https://docs.newrelic.com/docs/service-level-management/alerts-slm/`, and `https://docs.newrelic.com/docs/data-apis/manage-data/view-system-limits/`.
Source note 3: Grafana Cloud public docs opened during this audit at `https://grafana.com/docs/grafana-cloud/introduction/`, `https://grafana.com/docs/grafana-cloud/send-data/traces/use-traces-with-grafana/`, `https://grafana.com/pricing/`, and `https://grafana.com/docs/mimir/latest/manage/run-production-environment/planning-capacity/`.
Source note 4: Oyatie service evidence is from `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, `capability-tiers/tier-matrix.md`, `ADR-OBS-001`, contracts, SLOs, runbooks, dashboards, and IaC inventory read in the coherence audit.
Source note 5: This matrix compares union capability coverage, not brand-by-brand product packaging.

## §1 Counterpart 1 - Datadog Capability Surface

1. Infrastructure host monitoring: official docs root lists Infrastructure for host and infrastructure health.
2. Metrics explorer and custom metrics: official docs root lists Metrics and Datadog metrics pages describe ingestion/indexing controls.
3. Container monitoring: official docs root lists Container Monitoring.
4. Kubernetes monitoring: container monitoring and infrastructure docs cover cluster/container telemetry.
5. Serverless monitoring: official docs root lists Serverless.
6. Network monitoring: official docs root lists Network Monitoring.
7. Cloud cost management: official docs root lists Cloud Cost Management, and CCM docs describe cost monitors and allocation.
8. Cloudcraft infrastructure diagrams: official docs root lists Cloudcraft.
9. Storage management: official docs root lists Storage Management.
10. APM distributed tracing: official getting-started docs describe APM side by side with logs and infrastructure.
11. Universal Service Monitoring: official docs root lists USM for code-free service discovery/monitoring.
12. Continuous profiler: official docs root lists Continuous Profiler.
13. Database monitoring: official docs root lists Database Monitoring.
14. Data streams monitoring: official docs root lists Data Streams Monitoring.
15. Data observability: official docs root lists Data Observability.
16. Log management: official getting-started docs describe log send/process/search/retention controls.
17. Log Live Tail: Datadog log docs describe real-time log observation.
18. Log indexing and retention controls: Datadog log index docs describe daily quotas and retention by index.
19. Sensitive Data Scanner: official docs root lists PII/API-key/card redaction across telemetry.
20. Observability pipelines: official docs root lists telemetry pipeline management.
21. Error tracking: official docs root lists Error Tracking.
22. Real User Monitoring: official getting-started docs describe RUM across web/mobile, Core Web Vitals, mobile vitals, and feature flags.
23. Product analytics: official docs root lists Product Analytics.
24. Session replay: official getting-started docs describe capture and replay of user sessions.
25. Synthetic monitoring: official getting-started docs describe API, browser, mobile, and Network Path tests.
26. Mobile app testing: official docs root lists Mobile App Testing.
27. Dashboards: official getting-started docs describe real-time dashboards combining metrics, logs, APM, and RUM.
28. Monitors and alerting: official getting-started docs describe metric and integration monitors with notification routing.
29. Downtime scheduling: official getting-started docs describe monitor notification suppression.
30. Integrations catalog: official docs describe 1,000-plus integrations across cloud, incident, data, security, AI, and more.
31. Software Catalog/internal developer portal: Datadog docs describe entity definitions and telemetry-fed service catalog.
32. Incident management: getting-started docs list incident management and workflow tooling.
33. Workflow automation: getting-started docs list end-to-end automation in response to alerts and security signals.
34. Teams/ownership model: getting-started docs list Teams syncing identity/GitHub ownership.
35. Organization topology and multi-org isolation: getting-started docs list org topology and access controls.
36. API and automation: Datadog API docs expose ingestion and control APIs with rate-limit semantics.
37. Event management: retention docs list event retention; monitors/events appear in UI.
38. LLM Observability: official docs root lists LLM Observability under AI.
39. Watchdog anomaly detection: official docs root lists Watchdog.
40. Bits AI agents: official docs root lists Bits AI Agents.
41. Code Security: getting-started docs list code-security capabilities.
42. Cloud SIEM: getting-started docs list Cloud SIEM.
43. Cloud Security/Posture: getting-started docs list Cloud Security.
44. Workload Protection: official docs root lists workload protection.
45. App and API Protection: getting-started docs list AAP.
46. CI Visibility: getting-started docs list CI Visibility.
47. Continuous Testing: getting-started docs list continuous testing and synthetic tests in CI/IDE.
48. Feature Flags: getting-started docs list feature delivery with built-in observability.
49. Test Optimization and Test Impact Analysis: getting-started docs list test suite optimization.
50. Data retention matrix: Datadog retention docs define retention windows for metrics, profiles, logs, errors, events, incidents, and LLM traces.

## §2 Counterpart 2 - New Relic Capability Surface

1. Alerts: New Relic getting-started docs describe alerts and notification integrations.
2. APM and services: getting-started docs list APM and microservice monitoring.
3. Browser monitoring: getting-started docs describe real user data, page load, network requests, JavaScript errors, and interactions.
4. Dashboards: getting-started docs list dashboards.
5. Errors inbox: getting-started docs list error triage and New Relic errors docs describe grouping/impact.
6. Infrastructure monitoring: official docs describe hosts, containers, cloud infra, CPU, memory, network, disk.
7. On-host integrations: infrastructure docs describe over 400 integrations for third-party apps.
8. Kubernetes monitoring: getting-started docs list Kubernetes and Pixie.
9. Pixie auto-telemetry: official Pixie docs describe short-term full telemetry and selected long-term persistence.
10. Logs: getting-started docs list logs.
11. Logs in context: infrastructure docs describe logs alongside app/host data.
12. Mobile monitoring: getting-started docs list mobile monitoring.
13. OpenTelemetry ingestion: getting-started docs list OpenTelemetry.
14. OpenTelemetry APM UI: official docs say OTel services get curated APM experience.
15. Original OTel data preservation: New Relic OTel docs say original data remains available for dashboards and alerts.
16. Serverless and Lambda: getting-started docs list serverless and Lambda.
17. Service levels: getting-started docs list service levels.
18. SLI/SLO support: getting-started docs say service levels support SLIs and SLOs.
19. Error-budget burn alerting: service-level docs explain fast-burn/slow-burn alerting and budget consumption.
20. Synthetic monitoring: getting-started docs list proactive monitoring.
21. Vulnerability management: getting-started docs list vulnerability management.
22. Interactive Application Security Testing: getting-started docs list IAST.
23. Query builder: platform docs describe Query Builder and NRQL query/charting options.
24. Metrics and events explorer: platform docs describe metrics/events exploration.
25. NRQL chart introspection: platform docs describe viewing chart queries.
26. Platform search: platform docs describe searching accounts, capabilities, and entities.
27. Entity explorer: platform docs describe entity search and alert severity.
28. Distributed tracing: official tracing docs describe end-to-end request path tracking.
29. Infinite Tracing planning: distributed tracing docs point to advanced tracing topics.
30. Error impact metrics: errors docs describe users, sessions, and devices impacted.
31. Browser JS error grouping: browser errors docs describe grouped errors and session impact.
32. Data limits: data-limits docs list 55 million NRDB records per account per minute.
33. NRDB account limits: data-limits docs list payload, attribute, agent instance, browser page-view, metric timeslice, entity, and crash limits.
34. Data ingest APIs/SDKs: NRDB docs describe core ingest and query substrate.
35. Account and user settings: platform docs describe user/account settings.
36. Account switching: platform docs describe multiple accounts and access.
37. Keyboard shortcuts and UI navigation: platform docs describe operational UX.
38. APM database query performance: APM docs list DB query analysis.
39. External service monitoring: APM docs list external call monitoring.
40. Entity relationships: APM docs list service connection/dependency view.
41. Intelligent alerting: APM docs describe learned behavior and alerting.
42. Logs/API query integration: platform query docs support querying data with NRQL.
43. OpenTelemetry normalization: OTel docs describe mapping OTel data to New Relic APM conventions.
44. Error ignore/manipulation policy: OTel docs describe pre-ingest manipulation for ignored errors.
45. Infrastructure events feed: infrastructure docs describe config changes, restarts, SSH sessions, and key event changes.
46. Cloud integrations: infrastructure docs describe AWS/Azure/GCP integrations.
47. Secure data collection posture: infrastructure docs describe secure collection and current data.
48. Pricing editions: getting-started docs reference Standard, Pro, Enterprise pricing.
49. Data options: New Relic Data/Data Plus docs and limits disclose scaled query/ingest posture.
50. Incident/noise reduction: alert docs emphasize critical issue context and alert fatigue management.

## §3 Counterpart 3 - Grafana Cloud Capability Surface

1. Fully managed Grafana Cloud observability platform: introduction docs define managed collection, connection, analysis, and action.
2. Grafana Alloy collector: Grafana Cloud docs describe Alloy as OTel distribution for logs, metrics, traces, profiles.
3. Metrics store: Grafana Cloud docs describe Cloud Metrics and Prometheus/Graphite endpoints.
4. Logs store: Grafana Cloud docs describe Cloud Logs powered by Loki.
5. Traces store: Grafana Cloud docs describe Cloud Traces powered by Tempo.
6. Profiles store: Grafana Cloud docs describe Cloud Profiles/Pyroscope.
7. Four telemetry signals: Grafana docs describe metrics, logs, traces, and profiles.
8. Telemetry correlation: Grafana docs describe moving from metric to trace to profile.
9. Application Observability: Grafana docs describe service overview with RED metrics, operations, traces, and logs.
10. Service graph from spans: trace docs describe service graph and span metrics.
11. RED metrics from traces: trace docs describe rate, errors, duration derived from spans.
12. TraceQL search: trace docs describe TraceQL and visual query builder.
13. Trace-to-logs links: trace docs describe links between traces and Loki logs.
14. Trace-to-profiles links: trace docs describe links from trace spans to Pyroscope profiles.
15. Exemplars: trace docs describe metrics-to-trace exemplars.
16. Traces Drilldown: trace docs describe visual investigation without writing TraceQL.
17. Grafana Assistant: trace docs describe private-preview AI query/troubleshooting assistant.
18. Grafana dashboards: Grafana Cloud includes Grafana dashboards.
19. Grafana Alerting: getting-started metrics/logs docs describe alerting and rule evaluation.
20. Kubernetes Monitoring: pricing and docs list end-to-end Kubernetes observability.
21. Application Observability pricing: pricing docs list application observability host-hour model.
22. Frontend Observability: pricing/docs describe RUM for end-user experience.
23. Frontend sessions: pricing docs define sessions and free 50k sessions.
24. k6 performance testing: introduction/pricing docs mention k6 performance testing.
25. Synthetic monitoring: Grafana Cloud includes synthetics through k6/Grafana synthetics docs.
26. Private connectivity: Grafana Cloud docs describe private connectivity from AWS/Azure/GCP networks.
27. External data sources: introduction docs say connect externally hosted data sources.
28. Integrations: Grafana docs describe Alloy configuration snippets, dashboards, and alert defaults.
29. Fleet Management: send-data docs mention managing Alloy collectors.
30. Adaptive Metrics: pricing docs describe eliminating unused time series.
31. Adaptive Logs: pricing docs describe identifying log patterns to drop unused telemetry.
32. Cardinality and usage controls: pricing docs describe active series and DPM billing concepts.
33. Free metrics tier: pricing docs list 10k active series and 14-day retention.
34. Free logs tier: pricing docs list 50 GB ingested/month and 14-day retention.
35. Free traces tier: pricing docs list 50 GB ingested/month and 14-day retention.
36. Free profiles tier: pricing docs list 50 GB ingested/month and 14-day retention.
37. Kubernetes free tier: pricing docs list host/container-hour limits.
38. BYOC/Federal deployment flexibility: pricing docs list Public Cloud, Federal Cloud, or BYOC for Enterprise.
39. Mimir capacity planning: official Mimir docs provide CPU/memory estimates by samples/sec, series, and query load.
40. Loki log aggregation: Grafana Cloud pricing describes Loki-backed log aggregation.
41. Mimir metrics backend: Grafana pricing describes highly scalable metrics service.
42. Tempo distributed tracing backend: Grafana pricing describes scalable tracing backend.
43. Pyroscope profiling backend: pricing/docs describe continuous profiling.
44. OpenTelemetry native ingestion: pricing docs call out OTel-native ingestion without vendor lock-in.
45. Cost optimization tools: Adaptive Metrics, Adaptive Logs, billing alerts, and usage dashboards.
46. Grafana OnCall/incident response: Grafana Cloud ecosystem includes on-call/IRM surfaces.
47. SLO product/slo management: Grafana Cloud ecosystem includes SLO management, though current source pages opened emphasize alerting and signal stores.
48. Observability Architect support: Enterprise pricing lists observability architect support.
49. Custom retention: Enterprise pricing lists custom retention.
50. Composable open-source lineage: pricing and docs emphasize Grafana, Mimir, Loki, Tempo, Pyroscope, Alloy.

## §4 UNION-Coverage Matrix

| Capability | Datadog | New Relic | Grafana Cloud | UNION required | Oyatie observability has | Gap classification |
|---|---|---|---|---|---|---|
| Metrics ingest/store/query | yes | yes | yes | yes | yes, Mimir/OpenSLO SLOs at `PRD.md:41-45` | parity |
| Log ingest/store/query | yes | yes | yes | yes | yes, Loki docs/SLOs | parity |
| Distributed tracing | yes | yes | yes | yes | yes, OTel/Tempo/ClickHouse at `ADR-OBS-001:55-85` | parity |
| Continuous profiling | yes | partial | yes | yes | yes, Pyroscope named in PRD | partial, docs only |
| Dashboards | yes | yes | yes | yes | yes, dashboard JSON files | parity |
| Alerting/monitors | yes | yes | yes | yes | yes, Alertmanager/Grafana OnCall at `PRD.md:31-47` | parity |
| SLO/service levels | yes | yes | yes | yes | yes, OpenSLO and burn-rate engine | ahead on promotion gate |
| Error-budget burn alerts | yes | yes | yes | yes | yes, `PRD.md:41` and SLO engine | parity |
| Release promotion gate | no | no | no | no commercial union, Oyatie required | yes, `PRD.md:42-48` | additive |
| Automated rollback on SLO burn | partial | partial | partial | yes for platform quality | yes, `PRD.md:43` | ahead |
| OpenSLO source-native authoring | partial | no | partial | yes for Oyatie doctrine | yes, `PRD.md:39` | ahead |
| Audit-chain evidence anchoring | no | no | no | yes for Oyatie | yes, `PRD.md:72-74` | additive |
| Tenant SLO dashboards | yes | yes | yes | yes | yes, tenant dashboard JSON | parity |
| RUM/browser monitoring | yes | yes | yes | yes | no direct artifact | missing |
| Session replay | yes | no/limited | no/limited | yes due Datadog | no artifact | missing |
| Mobile monitoring | yes | yes | partial | yes | no artifact | missing |
| Synthetic API tests | yes | yes | yes | yes | weak, no broad synthetic plan | missing |
| Synthetic browser tests | yes | yes | yes | yes | weak, axe runner only | missing |
| Network path monitoring | yes | partial | no/limited | yes | no artifact | missing |
| Network flow monitoring | yes | partial | no/limited | yes | no artifact | missing |
| Infrastructure host monitoring | yes | yes | partial | yes | partial via Kubernetes/Helm | partial |
| Kubernetes monitoring | yes | yes | yes | yes | partial via Helm/K8s charts | partial |
| Serverless monitoring | yes | yes | no/limited | yes | no artifact | missing |
| Database monitoring | yes | partial | dashboards possible | yes | substrate ClickHouse only | partial |
| Data streams monitoring | yes | no/limited | no/limited | yes | no artifact | missing |
| Data observability/data quality | yes | partial | no/limited | yes | no artifact | missing |
| Product analytics | yes | no/limited | no/limited | yes | no artifact | missing |
| Service catalog | yes | entity explorer | integrations/dashboards | yes | partial Backstage chart/catalog YAML | partial |
| Entity dependency map | yes | yes | service graph | yes | partial via traces and ontology | partial |
| Universal/code-free service discovery | yes | Pixie/eBPF | Beyla/Alloy possible | yes | no eBPF artifact | missing |
| eBPF observability | no/partial | yes | Beyla possible | yes | no artifact | missing |
| Observability pipelines | yes | ingest APIs | Alloy/Fleet | yes | partial Alloy config only | partial |
| Collector fleet management | partial | agents | yes | yes | no fleet controller | missing |
| Sensitive data scanner | yes | partial | partial | yes | policy/scrubber docs only | partial |
| Cloud cost management | yes | usage/cost | billing usage | yes | partial OpenCost/cost-budget | partial |
| Usage/billing telemetry | yes | yes | yes | yes | partial `cost-budget.md` | partial |
| Incident management | yes | alerts/integrations | IRM/OnCall | yes | yes, OnCall/runbooks | partial |
| On-call rotations | yes | integrations | yes | yes | yes, `runbooks/oncall-rotation.md` | parity |
| Workflow automation | yes | partial | alerting/actions | yes | partial Workflow events | partial |
| Integrations marketplace/catalog | yes | yes | yes | yes | weak, OTel-first only | missing |
| OpenTelemetry ingest | yes | yes | yes | yes | yes, PRD/ADR | parity |
| OTel data preservation | partial | yes | yes | yes | yes in ClickHouse design | parity |
| Trace-to-logs correlation | yes | yes | yes | yes | planned via Grafana stack | partial |
| Trace-to-profile correlation | partial | partial | yes | yes | planned via Pyroscope/Tempo | partial |
| Service graph | yes | yes | yes | yes | partial from traces | partial |
| Error grouping/inbox | yes | yes | partial | yes | no issue inbox artifact | missing |
| Impacted users/sessions/devices | yes | yes | frontend obs | yes | no RUM artifact | missing |
| Feature flag observability | yes | no/limited | no/limited | yes | no artifact | missing |
| CI visibility | yes | no/limited | k6/CI possible | yes | promotion gate only | partial |
| Test optimization | yes | no/limited | k6 possible | yes | no artifact | missing |
| Security monitoring/SIEM | yes | vulnerability/IAST | plugins | yes | compliance/policy only | partial |
| Cloud security posture | yes | vulnerability | integrations | yes | no artifact | missing |
| LLM observability | yes | no/limited | possible via traces | yes | no artifact | missing |
| Anomaly detection/AI assist | yes Watchdog/Bits | alert intelligence | Grafana Assistant preview | yes | no artifact | missing |
| Query language | Datadog query/log/APM | NRQL | PromQL/LogQL/TraceQL | yes | PromQL/LogQL/TraceQL + SQL | parity |
| Long retention controls | yes | yes | yes | yes | yes in tier matrix | parity |
| Custom retention by tier | yes | yes | yes | yes | yes, demo_trial tenant_class-paid tenant_class baseline-paid tenant_class scale-compliance_pack-gated paid tenant_class | partial due OCI conflict |
| Sovereign/air-gapped deployment | limited | limited | enterprise/BYOC | yes for Oyatie | yes in compliance_pack-gated paid tenant_class docs | ahead |
| Per-tenant database isolation | no/limited | no/limited | stack tenanting | yes for Oyatie | yes compliance_pack-gated paid tenant_class `tier-matrix.md:111-114` | ahead |
| demo_trial tenant_class/free tier | trial/free | free data | free tier | yes | no OCI Always Free mapping | missing |
| BYOC/private connectivity | partial | partial | yes | yes | no six-context matrix | missing |
| OS/package matrix | agents support many | agents support many | collector/container | yes by ADR-0328 | absent | missing |
| IaC modules | Terraform/provider modules | Terraform/provider modules | Helm/Terraform docs | yes OpenTofu | missing canonical modules | missing |
| Sigstore/module signing | no/limited | no/limited | no/limited | yes Oyatie | not evidenced | missing |
| Regulated-pack evidence | limited | limited | limited | yes | yes packs/compliance | ahead |
| Data residency policy | yes | yes | yes | yes | yes but OCI-biased | partial |
| Tenant self-service SLO authoring | yes | yes | yes | yes | yes OpenSLO | parity |
| Tenant custom telemetry pipelines | yes | partial | Alloy/Fleet | yes | not implemented | missing |
| Migration from Datadog | n/a | n/a | n/a | yes for Oyatie | yes playbook | parity |
| Migration from New Relic | n/a | n/a | n/a | yes for union coverage | absent | missing |
| Migration from Grafana Cloud | n/a | n/a | n/a | yes for union coverage | absent | missing |

## §5 Capability Families Summary

| Family | UNION required count | Oyatie present count | Status |
|---|---:|---:|---|
| Telemetry pillars: metrics/logs/traces/profiles | 4 | 4 | strong but profile implementation evidence is docs-first |
| SLO and release governance | 6 | 6 | ahead due signed promotion gate |
| Dashboards and incident response | 7 | 6 | near parity |
| User experience monitoring | 6 | 0 | major gap |
| Synthetic and test monitoring | 5 | 1 | major gap |
| Infrastructure and Kubernetes | 6 | 3 | partial |
| Network and serverless | 5 | 0 | major gap |
| Database/data streams/data quality | 4 | 1 | major gap |
| Error tracking and impact analysis | 5 | 1 | major gap |
| Service catalog and ownership | 4 | 2 | partial |
| Telemetry pipeline/fleet management | 5 | 2 | partial |
| Cost and retention governance | 6 | 4 | partial due OCI Always Free demo_trial tenant_class gap |
| Security observability | 6 | 2 | partial |
| AI/anomaly/LLM observability | 4 | 0 | gap |
| Deployment portability/IaC/OS | 6 | 0 | canonical blocker |
| Sovereign compliance evidence | 5 | 5 | ahead |

## §6 Headline Gap Analysis - Top 15 Missing Capabilities

1. RUM/browser monitoring gap.
   Evidence: Datadog, New Relic, and Grafana Cloud all expose frontend/user-experience monitoring; Oyatie has no frontend observability artifact under service inventory.
   Hook: add `frontend-observability` bounded context or explicitly route it to a separate experience microservice while observability owns ingestion contracts.
2. Session replay gap.
   Evidence: Datadog Session Replay is in official docs; Oyatie has no replay storage, privacy redaction, or tenant playback policy.
   Hook: if owned, add privacy-gated replay contract with DPIA update; if excluded, document why another service owns it.
3. Mobile monitoring gap.
   Evidence: Datadog and New Relic both list mobile monitoring/testing; Oyatie has no mobile signal SDK or mobile SLO recipe.
   Hook: add mobile OTel semantic-convention profile and Swift/Kotlin-only frontend exceptions.
4. Synthetic monitoring gap.
   Evidence: all three counterparts have synthetic or k6-based testing; Oyatie only has an axe/pa11y runner chart and promotion SLO drills.
   Hook: add `synthetic-probe` contract and tie results into OpenSLO burn-rate windows.
5. Network monitoring gap.
   Evidence: Datadog has Network Monitoring and Network Path tests; Oyatie has network policy but no network telemetry product.
   Hook: add eBPF/flow ingest plan or route ownership to network microservice with observability consuming signals.
6. Database monitoring gap.
   Evidence: Datadog and New Relic expose DB monitoring; Oyatie monitors its own ClickHouse but not tenant database fleets.
   Hook: add DB metric conventions and SLO recipes, or explicitly assign DBM to storage/data-plane services.
7. Error inbox gap.
   Evidence: Datadog and New Relic have error grouping/impact workflows; Oyatie stores traces/errors but lacks a triage inbox.
   Hook: add error-grouping materialized views and tenant-visible issue lifecycle contract.
8. Impacted users/sessions/devices gap.
   Evidence: New Relic tracks impacted users/sessions/devices; Oyatie lacks RUM identity-safe impact rollups.
   Hook: add privacy-preserving impacted-principal metrics with tenant-safe aggregation thresholds.
9. Service catalog gap.
   Evidence: Datadog Software Catalog and New Relic entity search connect ownership and telemetry; Oyatie has catalog YAML but no operator UX.
   Hook: integrate with Backstage chart or Oyatie catalog service and expose owner/runbook/SLO links.
10. Collector fleet management gap.
   Evidence: Grafana Cloud Fleet Management manages Alloy collectors; Oyatie has charts but no fleet lifecycle controller.
   Hook: add OpenTofu/agent lifecycle outputs and collector health reconciliation.
11. Integrations breadth gap.
   Evidence: Datadog has 1,000-plus integrations; Oyatie uses OTel-first strategy with limited migration docs.
   Hook: define integration ingestion tiers and choose explicit "OTel standard only" scope if broad marketplace is out of scope.
12. Observability pipeline builder gap.
   Evidence: Datadog Observability Pipelines and Grafana Alloy/Fleet expose pipeline management; Oyatie has static Alloy config.
   Hook: add policy-gated pipeline configuration API and signed recipe registry.
13. AI/anomaly/LLM observability gap.
   Evidence: Datadog Watchdog/Bits/LLM Observability and Grafana Assistant represent this family; Oyatie has no model-specific observability.
   Hook: define LLM spans, prompt/tool safety events, and anomaly-detection ownership.
14. BYOC/private connectivity gap.
   Evidence: Grafana Cloud Enterprise advertises deployment flexibility; ADR-0328 requires six contexts; Oyatie has no context matrix.
   Hook: add six OpenTofu context modules and private endpoint outputs.
15. OCI Always Free demo_trial tenant_class/free-tier gap.
   Evidence: Grafana has a free tier, OCI Always Free doctrine mandates demo_trial tenant_class sub-profile; Oyatie demo_trial tenant_class is oversized.
   Hook: split demo_trial tenant_class Always Free constraints from paid telemetry envelopes.

## §7 Additive Surface

1. Signed Oya VCS promotion eligibility is additive; counterparts monitor releases but do not own Oyatie release-pointer advancement.
2. OpenSLO source-native release prerequisite is additive; counterparts allow SLOs but do not require repository-local OpenSLO before promotion.
3. Audit-chain anchoring of eligibility verdicts is additive; counterpart audit trails are not equivalent to Oyatie Merkle/Ed25519 evidence.
4. Automated rollback from SLO burn tied to release branches is additive in this repo-specific form.
5. Per-microservice SLO manifests as canonical source of truth are additive.
6. Sovereign-pack telemetry custody with pack-resident HSM posture is additive.
7. Per-tenant database isolation in compliance_pack-gated paid tenant_class is additive relative to SaaS org/account tenancy.
8. Cedar-scoped trace search is additive as an explicit authorization surface.
9. WORM evidence anchoring for promotion-gate evidence is additive.
10. Context-aware data residency tied to Oyatie packs is additive, though currently OCI-biased.
11. Tail-sampling reason codes as first-class audit evidence are additive.
12. ClickHouse join model across traces, audit, deployment refs, and FinOps is additive if implemented.
13. No-SaaS-portal tenant-visible dashboards at every tier are additive.
14. Air-gapped compliance_pack-gated paid tenant_class behavior is additive relative to normal SaaS observability products.
15. The additive surface is valuable but depends on fixing the deployment/IaC/OS substrate blockers identified in the coherence audit.

## §8 Capability Ownership Hooks for Wave 14

This section converts the parity gaps into observability-owned hooks so the aggregation wave can route work without guessing.

1. APM service maps: observability owns the derived graph API because PRD lines 20-26 name the shared telemetry substrate.
2. APM service maps: architecture must add the graph read model because current architecture lines 445-456 only cover Helm deployment shape.
3. Distributed trace search: observability owns tenant-scoped search because ADR-OBS-001 lines 55-85 make traces a first-class store.
4. Distributed trace search: policy integration must use Cedar boundaries because policy/tenant-isolation.md lines 192-198 already frames drift controls.
5. Log analytics: observability owns query semantics because PRD lines 37-48 names telemetry and SLO features as functional requirements.
6. Log analytics: storage planning must reconcile with capacity-model.md because it already defines ingest envelopes.
7. Metrics explorer: observability owns PromQL-facing query behavior because capability-tiers/tier-matrix.md lines 15-126 defines tier capacities.
8. Metrics explorer: canonical IaC must expose dashboard outputs because ADR-0328 lines 2537-2538 requires observability module outputs.
9. SLO product surface: observability owns OpenSLO ingestion because PRD lines 274-287 references e2e validation around SLO behavior.
10. SLO product surface: Wave 14 should not route SLO API ownership to another microservice without reverse-reference evidence.
11. Incident notification: observability owns alert materialization because PRD lines 20-26 includes Alertmanager and OnCall in the platform purpose.
12. Incident notification: IAM-scoped escalation ownership remains unresolved because the service path lacks cross-microservice-handoffs.md.
13. On-call roster integration: observability owns the event bridge but not HR or identity source records.
14. On-call roster integration: the missing handoff file is a P2 aggregation blocker because reverse ownership cannot be verified.
15. Synthetic monitoring: observability owns collection and alerting hooks, while application teams own target definitions.
16. Synthetic monitoring: no current service artifact proves browser-step playback support, so parity is missing.
17. RUM telemetry: observability owns ingestion schemas, while frontend platform teams own SDK embedding.
18. RUM telemetry: current SDK plan lines 30-38 drifts into forbidden-language roadmap unless generated bindings are isolated.
19. Session replay: observability should own storage, redaction, and replay lookup only if privacy policy is added.
20. Session replay: absence from PRD lines 37-48 means this is a parity gap, not a committed feature.
21. Infrastructure monitoring: observability owns node/pod metric ingestion because Helm/Kustomize files exist under iac.
22. Infrastructure monitoring: canonical context OpenTofu must replace Helm-only install instructions for ADR-0328 alignment.
23. Kubernetes monitoring: observability owns cluster telemetry but must avoid direct vendor APIs in business logic.
24. Kubernetes monitoring: current iac/kustomize overlays are useful inputs but not ADR-0328-complete IaC.
25. Network monitoring: observability owns metrics/logs/traces, while SDN enforcement belongs to network/IAM owners.
26. Network monitoring: no packet, flow, DNS, or path telemetry product contract exists in current artifacts.
27. Database monitoring: observability owns exporter ingestion and dashboards, not database operator lifecycle.
28. Database monitoring: no counterpart-level query insight capability is documented in the observability PRD.
29. Cloud cost visibility: observability can expose telemetry cost, but FinOps accounting belongs to the cost ledger owner.
30. Cloud cost visibility: ClickHouse cost fields should be added only after ownership is documented.
31. Fleet management: observability owns collector health and rollout status because Grafana Alloy is named in PRD lines 20-26.
32. Fleet management: current static Helm values are not enough for counterpart parity with Grafana Cloud Fleet Management.
33. Data retention policies: observability owns tier retention because capability-tiers/tier-matrix.md lines 15-126 defines retention bands.
34. Data retention policies: OCI Always Free demo_trial tenant_class retention must be recalculated from ADR-0328 lines 3514-3571 resource limits.
35. Privacy redaction: observability owns pipeline-stage redaction for logs, traces, and replay artifacts.
36. Privacy redaction: dpia.md and compliance.md should become enforcement specs, not only narrative controls.
37. Compliance dashboards: observability owns telemetry evidence display, while compliance service owns legal interpretation.
38. Compliance dashboards: current compliance.md line references Terraform RBAC and therefore needs OpenTofu repair.
39. Audit trails: observability owns queryable telemetry evidence, but immutable signing belongs to the shared provenance substrate.
40. Audit trails: PRD lines 90-94 names ledger writer adjacency, so the handoff must be explicit.
41. Anomaly detection: observability owns signal extraction and alert candidates, while policy owns automated response authority.
42. Anomaly detection: no Watchdog-equivalent model or rule system exists in current artifacts.
43. AI/LLM observability: observability owns spans, logs, safety events, and model-cost metrics.
44. AI/LLM observability: no model-specific telemetry schema is currently present, so this is missing.
45. Profiling: observability owns Pyroscope ingestion because PRD lines 20-26 names Pyroscope.
46. Profiling: parity requires profile retention, query limits, and symbol handling that current artifacts do not specify.
47. Error tracking: observability owns event grouping if grouped errors become part of promotion eligibility.
48. Error tracking: no current artifact describes grouping, fingerprinting, or release regression detection.
49. Software catalog: observability owns telemetry links, while catalog authority belongs to the service registry.
50. Software catalog: manifest.json lines 6-28 is too thin to act as the catalog source of truth.
51. Integrations marketplace: observability should prefer OTel-first integration contracts before marketplace breadth.
52. Integrations marketplace: broad vendor-style integration count is not necessary for Wave 14 if explicit scope is recorded.
53. Mobile monitoring: observability owns backend ingestion and dashboards, while platform teams own Swift/Kotlin SDKs.
54. Mobile monitoring: this must stay inside ADR-0328 frontend allowlist and not introduce JavaScript app code.
55. Windows desktop monitoring: observability owns backend ingestion, while WinUI3 frontends own local instrumentation.
56. Windows desktop monitoring: Windows Server remains out-of-scope per ADR-0328 lines 2838-2854.
57. Data residency: observability owns storage placement tags and query restrictions per deployment context.
58. Data residency: missing six-context OpenTofu makes residency claims unprovable.
59. BYOC packaging: observability owns telemetry pack install surfaces, but pack lifecycle belongs to the cloud substrate.
60. BYOC packaging: missing iac/oyatie-iaas and iac/colo directories block parity with private/deployment-flexible offerings.
61. Air-gap mode: observability owns offline metrics/logs/traces/profiles if compliance_pack-gated paid tenant_class remains sovereign-pack capable.
62. Air-gap mode: current docs imply compliance_pack-gated paid tenant_class ambitions but lack package format and CI evidence.
63. Chargeback telemetry: observability owns usage counters; billing owns invoices and rate cards.
64. Chargeback telemetry: no service artifact currently maps tenant telemetry usage to cost events.
65. Promotion gates: observability owns SLO and burn-rate facts needed by Oya VCS promotion.
66. Promotion gates: the chat-history ADR-0263 prompt requires mandatory tenant_id and OTel fields across examples.
67. Build verification: observability owns `cargo build --workspace --release --all-features --locked` evidence only if Rust crates exist.
68. Build verification: current path has no src tree, so Wave 14 should classify implementation readiness separately from documentation parity.
69. OS packaging: observability owns package declarations for agents and collectors.
70. OS packaging: absent supported-oses.json makes all 13 Tier-1 package lanes unverified.
