# Observability Capability Tier Deltas vs Counterparts - 2026-05-20

## Header - Five-Citation Anchor Block

1. Canonical sequence anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2240`, `2243-2581`, `2646-2927`, `3047-3435`, `3493-3796`, and `3831-4224`.
2. Machine-readable plan anchor: `specs/master-plan-sequencing.json:704-868` for the six contexts, OpenTofu substrate, OS support, language allowlist, and OCI Always Free profile.
3. Microservice PRD anchor: `microservices/observability/PRD.md:20-26`, `37-48`, `90-94`, `121-123`, and `231-240`.
4. Microservice architecture anchor: `microservices/observability/ARCHITECTURE.md:445-456`, `631-642`, and `693-704`.
5. Documentation rigor anchor: `docs/standards/documentation-rigor.md:58-129`, `133-156`, and `222-261`.

External counterpart sources used for tier shape: Datadog pricing at `https://www.datadoghq.com/pricing/list/`, New Relic pricing at `https://newrelic.com/pricing`, New Relic usage-plan docs at `https://docs.newrelic.com/pt/docs/licenses/license-information/usage-plans/new-relic-usage-plan/`, Grafana Cloud pricing/support plans at `https://grafana.com/pricing/` and `https://grafana.com/support/plans/`.
The comparison below treats counterpart tiers as equivalent buying postures, not as one-to-one SKU clones.
Datadog packages observability by product and usage unit; New Relic packages by edition plus data/compute; Grafana Cloud packages Free/Pro/Advanced/Enterprise-style cloud plans and usage allotments.

## §1 Tier Definitions in Oyatie Observability

1. demo_trial tenant_class is the smallest deployable observability tier and must include the OCI Always Free sub-profile for guest-on-oci.
2. demo_trial tenant_class must remain useful enough to emit metrics, logs, traces, profiles, SLO events, and audit evidence for a small tenant.
3. demo_trial tenant_class must not silently require paid OCI resources when selected under guest-on-oci.
4. demo_trial tenant_class currently drifts because `capability-tiers/tier-matrix.md:15-47` defines a multi-node paid Grafana stack.
5. demo_trial tenant_class current evidence: the tier matrix names Grafana Alloy, Prometheus, Loki, Tempo, Alertmanager, Grafana, and Pyroscope.
6. demo_trial tenant_class current gap: no `iac/oci-guest/always-free/` directory exists to encode the resource-constrained demo_trial tenant_class profile.
7. demo_trial tenant_class current gap: no supported-oses manifest exists, so demo_trial tenant_class package support cannot be verified.
8. demo_trial tenant_class current gap: no OpenTofu context modules exist for the six deployment contexts.
9. demo_trial tenant_class target metric envelope: 25,000 to 125,000 metric points/sec depending on context.
10. demo_trial tenant_class target trace envelope: 5,000 to 30,000 spans/sec depending on context.
11. demo_trial tenant_class target log envelope: 1 GiB/day on OCI Always Free and 15 to 30 GiB/day on paid small contexts.
12. demo_trial tenant_class target retention: 14 to 30 days hot telemetry, with compressed audit evidence where storage allows.
13. demo_trial tenant_class target user posture: internal, small production, development, or a small regulated tenant pilot.
14. demo_trial tenant_class mandatory controls: tenant_id on all telemetry, basic SLO burn, alert routing, and redaction hooks.
15. demo_trial tenant_class forbidden controls: paid OCI resources under Always Free, README-only IaC, and manual console provisioning.
16. paid tenant_class baseline is the paid baseline production tier for normal customer workloads.
17. paid tenant_class baseline must support 90-day hot telemetry and 365-day cold evidence as a normal target.
18. paid tenant_class baseline current evidence: `capability-tiers/tier-matrix.md:48-75` defines higher ingest and retention than demo_trial tenant_class.
19. paid tenant_class baseline target metric envelope: 300,000 to 650,000 metric points/sec depending on context.
20. paid tenant_class baseline target trace envelope: 120,000 to 260,000 spans/sec depending on context.
21. paid tenant_class baseline target log envelope: 120 to 260 GiB/day depending on context.
22. paid tenant_class baseline target posture: production tenant with standard support, SLOs, dashboards, and on-call routing.
23. paid tenant_class baseline mandatory controls: OpenSLO import, alert routing, retention classes, privacy redaction, and context-level state backends.
24. paid tenant_class baseline current gap: no context-specific state backend exists because the canonical OpenTofu directories are absent.
25. paid tenant_class scale is the high-scale production tier for multi-region or heavy multi-tenant use.
26. paid tenant_class scale current evidence: `capability-tiers/tier-matrix.md:77-105` defines larger ingest and retention ambitions.
27. paid tenant_class scale target metric envelope: 3,000,000 to 6,500,000 metric points/sec depending on context.
28. paid tenant_class scale target trace envelope: 600,000 to 1,300,000 spans/sec depending on context.
29. paid tenant_class scale target log envelope: 1.2 to 2.6 TiB/day depending on context.
30. paid tenant_class scale target posture: customer production with strong SLO enforcement, high cardinality, and longer evidence retention.
31. paid tenant_class scale mandatory controls: horizontal sharding, cell-aware routing, dedicated query paths, and release-gate SLO evidence.
32. paid tenant_class scale current gap: no benchmark harness proves any p99 latency or ingest number.
33. compliance_pack-gated paid tenant_class is the sovereign, single-tenant, regulated, or Oyatie-pack tier.
34. compliance_pack-gated paid tenant_class current evidence: `capability-tiers/tier-matrix.md:107-126` names the top observability posture.
35. compliance_pack-gated paid tenant_class target metric envelope: 1,200,000 to 2,500,000 metric points/sec per sovereign pack.
36. compliance_pack-gated paid tenant_class target trace envelope: 300,000 to 625,000 spans/sec per sovereign pack.
37. compliance_pack-gated paid tenant_class target log envelope: 600 GiB/day to 1.25 TiB/day per sovereign pack.
38. compliance_pack-gated paid tenant_class target posture: custody-first, HSM/WORM capable, pack-local query and alerting.
39. compliance_pack-gated paid tenant_class mandatory controls: key custody, air-gap behavior, WORM evidence, data residency, and one-tenant default isolation.
40. compliance_pack-gated paid tenant_class current gap: the architecture describes credential isolation at `ARCHITECTURE.md:693-704` but not full pack IaC.
41. All tiers must use OpenTofu for provisioning because ADR-0328 §D-16 forbids Terraform/Pulumi/CloudFormation as implementation engines.
42. All tiers must publish OS support because ADR-0328 §D-17 treats 13 Tier-1 OSes as blocking support.
43. All tiers must keep backend/server/runtime code Rust-only because ADR-0328 §D-18 forbids Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, F#, and similar languages for service implementation.
44. All tiers must state frontend exceptions only under platform-specific frontend paths for Swift, Kotlin, or WinUI3.
45. All tiers must preserve the six-context deployment matrix unless an explicit N/A manifest proves non-applicability.
46. All tiers must provide observable outputs from OpenTofu modules because ADR-0328 lines 2537-2538 names dashboard, alert, log sink, and trace collector outputs for observability.
47. All tiers must distinguish design targets from measured benchmarks until benchmark artifacts exist.
48. All tiers must route counterpart gaps through service-owned artifacts rather than a generic platform backlog.
49. All tiers must keep additive Oyatie release-gate observability distinct from normal SaaS observability features.
50. All tiers currently share one central blocker: documentation ambition is ahead of buildable substrate.

## §2 Counterpart Tier Mapping

1. Datadog equivalent demo_trial tenant_class posture: Free Trial or small usage of Infrastructure Pro, APM, Logs, Synthetics, and other product SKUs.
2. Datadog equivalent paid tenant_class baseline posture: Infrastructure Pro plus selected APM/logs/RUM/synthetics/on-call SKUs.
3. Datadog equivalent paid tenant_class scale posture: Infrastructure Enterprise plus broader product set, higher usage, and enterprise support.
4. Datadog equivalent compliance_pack-gated paid tenant_class posture: enterprise-negotiated, dedicated/compliance-oriented deployment posture where available, not a single public universal SKU.
5. Datadog pricing source: `https://www.datadoghq.com/pricing/list/` lists Infrastructure Pro and Infrastructure Enterprise among product-specific price rows.
6. Datadog tier axis: product breadth plus usage units, not a single all-inclusive platform tier.
7. Datadog emphasizes Infrastructure, APM, Logs, RUM, Synthetics, Network, Database Monitoring, Incident Management, Workflow Automation, Error Tracking, LLM Observability, and security-adjacent SKUs.
8. Datadog delta implication: Oyatie cannot claim union parity merely by shipping metrics/logs/traces.
9. Datadog delta implication: Oyatie must decide whether network monitoring, database monitoring, RUM, synthetics, and LLM observability are observability-owned.
10. Datadog delta implication: Oyatie's additive release-gate SLO model can be ahead if built, but not a substitute for missing product breadth.
11. New Relic equivalent demo_trial tenant_class posture: Standard edition or free-start usage with limited data and users.
12. New Relic equivalent paid tenant_class baseline posture: Pro edition with production teams, broader telemetry, and more platform usage.
13. New Relic equivalent paid tenant_class scale posture: Enterprise edition with organizational controls and higher operational expectations.
14. New Relic equivalent compliance_pack-gated paid tenant_class posture: Enterprise plus Data Plus/advanced support/compliance posture where contracted.
15. New Relic pricing source: `https://newrelic.com/pricing` states customers can have Standard, Pro, or Enterprise edition.
16. New Relic usage-plan source: New Relic docs list Standard, Pro, and Enterprise Full Platform User pricing rows.
17. New Relic tier axis: users, data, compute, and edition controls.
18. New Relic emphasizes APM, infrastructure, logs, browser, mobile, synthetics, errors inbox, service levels, and OTel ingestion.
19. New Relic delta implication: Oyatie must cover service-level management, entity experience, and errors workflow to approach parity.
20. New Relic delta implication: Oyatie needs explicit event attribute and ingest limit policy before high-scale comparisons are meaningful.
21. Grafana Cloud equivalent demo_trial tenant_class posture: Free tier with 10k metric series, 50 GB each for logs/traces/profiles, and 14-day retention.
22. Grafana Cloud equivalent paid tenant_class baseline posture: Pro or paid usage of Grafana Cloud with the same product families and overage pricing.
23. Grafana Cloud equivalent paid tenant_class scale posture: Advanced or larger managed-cloud usage with support and scale expectations.
24. Grafana Cloud equivalent compliance_pack-gated paid tenant_class posture: Enterprise or private/contracted posture, with enterprise plugins and deployment flexibility where offered.
25. Grafana Cloud pricing source: `https://grafana.com/pricing/` and `https://grafana.com/support/plans/` publish included usage and product unit rates.
26. Grafana Cloud tier axis: included telemetry volume, active users, product-specific usage, and enterprise features.
27. Grafana emphasizes metrics, logs, traces, profiles, dashboards, alerting, IRM, synthetics, frontend observability, application observability, Kubernetes monitoring, database observability, and fleet management.
28. Grafana delta implication: Oyatie's stack choice is closest to Grafana Cloud but still lacks the managed product surfaces.
29. Grafana delta implication: Oyatie must reconcile demo_trial tenant_class with Grafana Free and OCI Always Free simultaneously.
30. Grafana delta implication: Mimir/Tempo/Loki/Pyroscope component presence is not the same as Grafana Cloud product parity.
31. Cross-counterpart demo_trial tenant_class axis: free/small usage, short retention, constrained active users, and limited support.
32. Cross-counterpart paid tenant_class baseline axis: production usage, paid retention, team workflows, and standard support.
33. Cross-counterpart paid tenant_class scale axis: higher scale, enterprise controls, advanced workflows, and larger retention.
34. Cross-counterpart compliance_pack-gated paid tenant_class axis: compliance, custody, dedicated capacity, negotiated support, and private deployment posture.
35. Oyatie demo_trial tenant_class maps most closely to Grafana Cloud Free for telemetry volume but must be stricter under OCI Always Free.
36. Oyatie paid tenant_class baseline maps to normal paid Grafana Cloud/New Relic/Datadog production usage.
37. Oyatie paid tenant_class scale maps to high-scale managed usage and must be benchmarked against Grafana Mimir capacity examples.
38. Oyatie compliance_pack-gated paid tenant_class maps less to normal SaaS plans and more to private, sovereign, or enterprise-contract posture.
39. Mapping confidence is high for Grafana Cloud because the Oyatie stack reuses Grafana ecosystem components.
40. Mapping confidence is medium for Datadog and New Relic because their pricing and product packaging are less component-aligned.

## §3 Per-Oyatie-Tier Delta Tables

### demo_trial tenant_class tier table

| Feature | Oyatie demo_trial tenant_class | Datadog equivalent | New Relic equivalent | Grafana equivalent | Gap classification |
|---|---|---|---|---|---|
| Metrics ingest | 25k-125k points/sec target | Paid/custom metrics by usage | Data ingest by usage | 10k series free baseline | Partial, target exists but unmeasured |
| Logs ingest | 1-30 GiB/day target | Logs product with 5 MB payload API limit | Logs by data usage | 50 GB included free/pro envelope | Partial, event-size policy missing |
| Traces ingest | 5k-30k spans/sec target | APM traces product | APM via native/OTel | 50 GB traces free envelope | Partial, search UX missing |
| Profiles | 500-3k samples/sec target | Continuous Profiler SKU | Profiling product surface | 50 GB profiles free envelope | Partial, symbol policy missing |
| Dashboards | Grafana dashboards implied | Datadog dashboards | New Relic dashboards | Grafana dashboards native | Partial, tenant UX unspecified |
| Alerting | Alertmanager/OnCall named | Monitors/on-call SKUs | Alerts/workflows | Grafana Alerting/IRM | Partial, escalation ownership missing |
| SLOs | OpenSLO/release-gate design | SLO feature | Service levels | SLO dashboards possible | Ahead in design, unbuilt in evidence |
| Incident management | OnCall named | Incident Management SKU | Incident Intelligence/workflows | IRM | Catch-up, workflow depth missing |
| Synthetics | Not documented | API/browser tests | Synthetics monitors | Synthetics included usage | Missing |
| RUM | Not documented | RUM/session replay SKUs | Browser/mobile monitoring | Frontend Observability | Missing |
| Mobile monitoring | Not documented | Mobile RUM/testing | Mobile monitoring | Frontend/mobile-adjacent | Missing |
| Network monitoring | Not documented | Network path/device/cloud monitoring | Network telemetry | Limited through integrations | Missing |
| Database monitoring | Not documented | Database Monitoring SKU | Database monitoring | Database Observability | Missing |
| Kubernetes monitoring | Helm/Kustomize charts | Kubernetes monitoring | Kubernetes infrastructure | Kubernetes Monitoring included usage | Partial, OpenTofu missing |
| Fleet management | Alloy static config | Agent fleet concepts | Agent management | Grafana Fleet Management | Missing |
| Integrations | OTel-first implied | 1,000-plus integrations | Broad integrations | Integrations and Alloy | Catch-up, explicit scope needed |
| Error tracking | Not documented | Error Tracking SKU | Errors Inbox | App observability errors | Missing |
| LLM observability | Not documented | LLM Observability SKU | AI monitoring surfaces | Grafana Assistant-adjacent | Missing |
| Data retention | 14-30 day target | Product retention matrix | Data retention by plan | 14-day free baseline | Partial, OCI math missing |
| Private deployment | Six contexts required | Enterprise/private negotiated | Enterprise posture | Enterprise/private posture | Missing until OpenTofu contexts exist |
| OS support | No manifest | SaaS/agent support docs | SaaS/agent support docs | Agent support docs | Missing |
| IaC | No canonical contexts | Vendor SaaS/API provisioning | Vendor SaaS/API provisioning | Cloud provisioning | Missing OpenTofu |
| Rust-only backend | No forbidden source found | Vendor implementation opaque | Vendor implementation opaque | Vendor implementation opaque | Aligned by absence, not build proof |
| OCI Always Free | Required but absent | Not applicable | Not applicable | Free tier analogous | Missing and P1-aligned |
| Tenant isolation | Policy docs exist | Org/account controls | Account/entity controls | Stack/org controls | Partial |
| Audit evidence | Additive design | Audit logs | Audit logs | Audit logs | Ahead in design, unbuilt |
| Cost guardrails | Not tiered | Usage pricing | Usage pricing | Included usage/overage | Missing |
| CI/build lane | Absent | Vendor-managed | Vendor-managed | Vendor-managed | Missing |
| API contracts | Contracts present but incomplete | Public APIs | Public APIs | Public APIs | Partial |
| Runbooks | Runbooks present, one missing reference | Docs/runbooks | Docs/runbooks | Docs/runbooks | Partial |

### paid tenant_class baseline tier table

| Feature | Oyatie paid tenant_class baseline | Datadog equivalent | New Relic equivalent | Grafana equivalent | Gap classification |
|---|---|---|---|---|---|
| Metrics ingest | 300k-650k points/sec target | Paid custom metrics | Pro/paid data | Pro paid metrics | Partial, benchmark missing |
| Active series | 900k-2M target | Custom metrics billing unit | Data volume/cardinality managed | Paid series overage | Partial |
| Logs | 120-260 GiB/day target | Logs product | Logs/data usage | Paid logs | Partial |
| Traces | 120k-260k spans/sec target | APM | APM/OTel | Tempo traces | Partial |
| Profiles | 12k-26k samples/sec target | Profiler | Profiling | Pyroscope profiles | Partial |
| Dashboard p99 | 1.05-1.7s target | Product not directly benchmarked | Product not directly benchmarked | Grafana dashboards | Measurement-required |
| SLO detection | 40-75s target | SLO/monitors | Service levels alerts | Alerting/SLO workflows | Partial |
| Alert routing | Standard on-call expected | On-Call/Incident SKUs | Alert workflows | IRM | Partial |
| Tenant count | 12-35 tenants/cell target | Account/org segmentation | Account/org segmentation | Stack/org segmentation | Partial |
| Retention | 60-90d hot, 365d cold | Retention matrix | Data retention plan | Paid retention | Partial |
| Synthetics | Not present | API/browser tests | Synthetics | Synthetics | Missing |
| Frontend/RUM | Not present | RUM | Browser monitoring | Frontend Observability | Missing |
| Error tracking | Not present | Error Tracking | Errors Inbox | App errors | Missing |
| Database monitoring | Not present | DBM | Database monitoring | Database Observability | Missing |
| Network monitoring | Not present | Network Monitoring | Network telemetry | Limited | Missing |
| Collector management | Static config | Agent fleet | Agent controls | Fleet Management | Missing |
| Integrations | OTel-first | Integration marketplace | Integrations | Integrations | Catch-up |
| Compliance | compliance.md present | Compliance features | Compliance features | Enterprise controls | Partial |
| RBAC/IAM | Terraform RBAC reference exists | Enterprise access controls | Edition controls | RBAC/plugins | Drifted because Terraform reference |
| OpenTofu | Absent | Not equivalent | Not equivalent | Not equivalent | Missing canonical requirement |
| OS matrix | Absent | Agent support docs | Agent support docs | Agent support docs | Missing canonical requirement |
| Build proof | No Rust crate | Vendor managed | Vendor managed | Vendor managed | Missing |
| State backend | Absent | Vendor managed | Vendor managed | Vendor managed | Missing |
| Sigstore module signing | Absent | Vendor managed | Vendor managed | Vendor managed | Missing |
| Data residency | Implied | Region/account controls | Region/account controls | Region/stack controls | Partial |
| Cost controls | Not tiered | Usage pricing | Usage pricing | Usage/overage | Missing |
| Service catalog | Manifest thin | Software Catalog | Entity explorer | Catalog/integrations | Catch-up |
| Promotion gates | Additive design | CI visibility/monitors | Deploy markers/service levels | Alerting/deploy annotations | Ahead in design |
| Incident evidence | Partial | Incident workflows | Incident workflows | IRM | Partial |
| Customer deployment | Six contexts required | SaaS/private varies | SaaS/private varies | Cloud/Enterprise varies | Missing until context IaC exists |

### paid tenant_class scale tier table

| Feature | Oyatie paid tenant_class scale | Datadog equivalent | New Relic equivalent | Grafana equivalent | Gap classification |
|---|---|---|---|---|---|
| Metrics ingest | 3M-6.5M points/sec target | Enterprise usage | Enterprise usage | Mimir high-scale examples | Partial, needs benchmark |
| Active series | 9M-20M target | High custom-metric usage | High data/cardinality | High series usage | Partial |
| Logs | 1.2-2.6 TiB/day target | High-volume logs | High-volume data | High-volume Loki | Partial |
| Traces | 600k-1.3M spans/sec target | High APM volume | High APM/OTel volume | Tempo scale | Partial |
| Profiles | 60k-130k samples/sec target | Profiler at scale | Profiling at scale | Pyroscope scale | Partial |
| Dashboard p99 | 750-1,300 ms target | Not public universal | Not public universal | Grafana depends on store | Measurement-required |
| Query fanout | Dedicated read path expected | Product managed | Product managed | Mimir/Loki/Tempo managed | Missing architecture detail |
| Sharding | Required by target | Vendor managed | Vendor managed | Mimir/Loki sharding | Missing implementation |
| SLO detection | 12-35s target | SLO/monitors | Service levels | Alerting | Partial |
| Retention | 120-180d hot, 7y cold | Product retention matrix | Retention by plan | Retention by plan | Partial |
| Multi-region | Expected | SaaS regions | SaaS regions | Cloud regions | Missing context IaC |
| Synthetics | Not present | Advanced synthetic usage | Synthetics | Synthetics | Missing |
| RUM/session replay | Not present | RUM/session replay | Browser/mobile | Frontend Observability | Missing |
| Network monitoring | Not present | Network path/device/cloud | Network monitoring | Limited | Missing |
| Database monitoring | Not present | DBM | DB monitoring | DB Observability | Missing |
| Error tracking | Not present | Error tracking scale | Errors Inbox | App errors | Missing |
| AIOps/anomaly | Not present | Watchdog/Bits | Applied intelligence | Assistant/anomaly options | Missing |
| LLM observability | Not present | LLM Observability | AI monitoring surfaces | Emerging ecosystem | Missing |
| Integrations | Limited OTel-first | Broad marketplace | Broad integrations | Broad integrations | Catch-up |
| Fleet management | Not present | Agent controls | Agent controls | Fleet Management | Missing |
| OpenTofu modules | Required absent | Not comparable | Not comparable | Not comparable | P1 canonical gap |
| OS support | Required absent | Agent support docs | Agent support docs | Agent support docs | P1 canonical gap |
| Rust build proof | Absent | Not comparable | Not comparable | Not comparable | P2 readiness gap |
| Compliance evidence | Narrative | Enterprise controls | Enterprise controls | Enterprise controls | Partial |
| Tenant isolation | Policy docs | Org/account controls | Account controls | Stack/org controls | Partial |
| Release gates | Additive SLO gate | Deploy events/monitors | Deploy markers/service levels | Alerting/deploy annotations | Design-ahead |
| FinOps telemetry | Not present | Cloud Cost Management | Usage/cost views | Usage billing | Missing |
| Data residency | Not concrete | Enterprise/region controls | Region controls | Region/private options | Partial |
| Capacity model | Present but target-only | Vendor managed | Vendor managed | Mimir capacity docs | Partial |
| Runbooks | Present but missing referenced file | Vendor docs | Vendor docs | Vendor docs | Partial |

### compliance_pack-gated paid tenant_class tier table

| Feature | Oyatie compliance_pack-gated paid tenant_class | Datadog equivalent | New Relic equivalent | Grafana equivalent | Gap classification |
|---|---|---|---|---|---|
| Single-tenant pack | Default target | Enterprise/private posture | Enterprise/private posture | Enterprise/private posture | Partial, no IaC |
| Metric ingest | 1.2M-2.5M per pack | Contracted scale | Contracted scale | Enterprise stack scale | Partial |
| Logs | 600GiB-1.25TiB/day per pack | High-volume logs | High-volume data | Loki at scale | Partial |
| Traces | 300k-625k spans/sec per pack | APM scale | APM/OTel scale | Tempo scale | Partial |
| Profiles | 45k-90k samples/sec per pack | Profiler scale | Profiling scale | Pyroscope scale | Partial |
| HSM/key custody | Required target | Enterprise controls vary | Enterprise controls vary | Enterprise/private controls | Additive if built |
| WORM evidence | Required target | Audit/compliance features | Audit/compliance features | Enterprise controls | Additive if built |
| Air-gap behavior | Required target | Not standard public SaaS | Not standard public SaaS | Enterprise/private varies | Ahead in design |
| Data residency | Pack-local target | Region/private controls | Region/private controls | Region/private controls | Partial |
| Dedicated query plane | Required target | Vendor managed | Vendor managed | Enterprise stack | Missing implementation |
| Dedicated ingest plane | Required target | Vendor managed | Vendor managed | Enterprise stack | Missing implementation |
| On-prem/colo support | Required context | Hybrid/private varies | Hybrid/private varies | Enterprise/private varies | Missing canonical IaC |
| Oyatie-as-cloud-provider | Required context | Not equivalent | Not equivalent | Not equivalent | Additive platform posture |
| OCI Always Free | Not compliance_pack-gated paid tenant_class | Not applicable | Not applicable | Free tier analogous | Not applicable to compliance_pack-gated paid tenant_class |
| Synthetics | Not present | Available SKU | Available | Available | Missing |
| RUM/session replay | Not present | Available SKU | Available | Available | Missing |
| Database monitoring | Not present | Available SKU | Available | Available | Missing |
| Network monitoring | Not present | Available SKU | Available | Partial | Missing |
| Error tracking | Not present | Available SKU | Available | Available | Missing |
| LLM observability | Not present | Available SKU | Emerging | Emerging | Missing |
| Fleet management | Not present | Agent controls | Agent controls | Fleet Management | Missing |
| Enterprise RBAC | Required | Available | Edition controls | Enterprise controls | Drifted by Terraform reference |
| Compliance reports | Narrative docs | Enterprise controls | Enterprise controls | Enterprise controls | Partial |
| Provenance signing | Required by doctrine | Vendor audit logs | Vendor audit logs | Vendor audit logs | Additive if built |
| Release rollback gates | Required design | Monitors/workflows | Alerts/workflows | Alerting/IRM | Additive if built |
| OpenTofu state backend | Required | Not comparable | Not comparable | Not comparable | Missing |
| Sigstore module signing | Required | Not comparable | Not comparable | Not comparable | Missing |
| OS package matrix | Required | Agent support docs | Agent support docs | Agent support docs | Missing |
| Bench harness | Required before claim | Vendor internal | Vendor internal | Public capacity examples | Missing |
| Support evidence | Required | Enterprise support | Enterprise support | Enterprise support | Missing until docs/build align |

## §4 OCI Always Free demo_trial tenant_class = Always Free Reconciliation

1. OCI Always Free demo_trial tenant_class must map to ADR-0328 §D-19 resources, not to the current paid demo_trial tenant_class tier shape.
2. Compute ceiling: 4 OCPU and 24 GiB RAM across Arm Ampere Always Free instances.
3. Block storage ceiling: 200 GiB total block volume.
4. Object storage ceiling: 10 GiB standard object storage.
5. Database ceiling: two Autonomous Database instances at 20 GiB each when used.
6. Egress ceiling: 10 TB outbound data transfer per month.
7. Load balancer ceiling: 10 Mbps flexible load balancer.
8. Monitoring substrate: OCI Logging, Monitoring, and Notifications are allowed Always Free surfaces.
9. Observability obligation: demo_trial tenant_class still emits metrics, logs, traces, audit events, and SLO burns.
10. demo_trial tenant_class adjustment: logs must be capped around 1 GiB/day to preserve 14-day hot retention on 200 GiB block storage.
11. demo_trial tenant_class adjustment: traces must rely on sampling and short hot retention.
12. demo_trial tenant_class adjustment: profiles should default off or low-rate until paid tenant_class baseline.
13. demo_trial tenant_class adjustment: dashboards must remain small and bounded to avoid overloading constrained compute.
14. demo_trial tenant_class adjustment: one production-light tenant or three development tenants is the credible envelope.
15. demo_trial tenant_class adjustment: cold retention should be compressed and selective, not full-fidelity telemetry.
16. demo_trial tenant_class requires `iac/oci-guest/always-free/main.tf`.
17. demo_trial tenant_class requires `iac/oci-guest/always-free/variables.tf`.
18. demo_trial tenant_class requires `iac/oci-guest/always-free/outputs.tf`.
19. demo_trial tenant_class requires `iac/oci-guest/always-free/versions.tf`.
20. demo_trial tenant_class requires `iac/oci-guest/always-free/README.md` as operational explanation, not as a replacement for IaC.
21. demo_trial tenant_class requires OpenTofu resource limits that fail plan/apply if paid resources are selected.
22. demo_trial tenant_class requires output variables for dashboard URL, alert route, log sink, trace collector, and SLO endpoint.
23. demo_trial tenant_class requires state backend choice that matches the context and does not rely on hand-edited tfstate.
24. demo_trial tenant_class requires signed module provenance before reuse across tenants.
25. demo_trial tenant_class requires supported-oses.json to indicate which Tier-1 OSes can run the collector/agent.
26. demo_trial tenant_class requires CI lanes or explicit package proof for Talos.
27. demo_trial tenant_class requires CI lanes or explicit package proof for RHEL 9+.
28. demo_trial tenant_class requires CI lanes or explicit package proof for Oracle Linux 9+.
29. demo_trial tenant_class requires CI lanes or explicit package proof for SLES 15 SP6+.
30. demo_trial tenant_class requires CI lanes or explicit package proof for Ubuntu 24.04 LTS+.
31. demo_trial tenant_class requires CI lanes or explicit package proof for Debian 13+.
32. demo_trial tenant_class requires CI lanes or explicit package proof for Rocky Linux 9+.
33. demo_trial tenant_class requires CI lanes or explicit package proof for AlmaLinux 9+.
34. demo_trial tenant_class requires CI lanes or explicit package proof for CentOS Stream 10+.
35. demo_trial tenant_class requires CI lanes or explicit package proof for Amazon Linux 2023+.
36. demo_trial tenant_class requires CI lanes or explicit package proof for Flatcar Container Linux.
37. demo_trial tenant_class requires CI lanes or explicit package proof for VMware Photon OS 5+.
38. demo_trial tenant_class requires CI lanes or explicit package proof for macOS Apple Silicon M5+ client-side instrumentation only.
39. paid tenant_class required feature: long-retention full-fidelity traces beyond the constrained demo_trial tenant_class window.
40. paid tenant_class required feature: high-cardinality metrics beyond 50,000 active series on OCI Always Free.
41. paid tenant_class required feature: heavy profile sampling and long symbol retention.
42. paid tenant_class required feature: multi-tenant production cell with more than one production-light tenant.
43. paid tenant_class required feature: high-volume log retention beyond the demo_trial tenant_class block-storage envelope.
44. paid tenant_class required feature: synthetic browser testing at meaningful production cadence.
45. paid tenant_class required feature: broad RUM/session replay data retention.
46. paid tenant_class required feature: database monitoring at high query-cardinality.
47. paid tenant_class required feature: network monitoring beyond basic exporter metrics.
48. paid tenant_class required feature: SLO detection below 60 seconds in normal cases.
49. paid tenant_class required feature: cross-region replication and disaster recovery.
50. paid tenant_class required feature: customer-facing on-call and incident workflows beyond basic alert routing.
51. compliance_pack-gated feature: HSM-backed sovereign pack custody.
52. compliance_pack-gated feature: full WORM evidence with regulated retention.
53. compliance_pack-gated feature: air-gap operation with pack-local query and alerting.
54. compliance_pack-gated feature: single-tenant dedicated ingest/query/storage plane.
55. compliance_pack-gated feature: customer-owned key lifecycle with audited break-glass.
56. Reconciliation finding: current demo_trial tenant_class is not Always Free and must be split or renamed.
57. Reconciliation finding: current tier docs can keep paid demo_trial tenant_class only if an explicit OCI Always Free demo_trial tenant_class sub-profile is added.
58. Reconciliation finding: the capability-tier matrix must name which features degrade under Always Free.
59. Reconciliation finding: OpenTofu must enforce resource ceilings, not merely document them.
60. Reconciliation finding: all demo_trial tenant_class claims remain non-contractual until implemented and benchmarked.

## §5 Findings

1. demo_trial tenant_class headline: catch-up against all three counterparts because core telemetry exists in concept but OCI Always Free and product breadth are missing.
2. demo_trial tenant_class ahead surface: release-gate SLO evidence is differentiated if implemented.
3. demo_trial tenant_class parity surface: Grafana-style metrics/logs/traces/profiles are conceptually aligned with Grafana Cloud.
4. demo_trial tenant_class catch-up surface: Datadog/New Relic/Grafana all provide clearer free/small-tier product activation than the current Oyatie docs.
5. demo_trial tenant_class blocker: absent `iac/oci-guest/always-free/` makes the tier incoherent with ADR-0328.
6. demo_trial tenant_class blocker: absent supported-oses manifest makes Tier-1 OS support unclaimable.
7. paid tenant_class baseline headline: partial parity for telemetry substrate, catch-up for managed product workflows.
8. paid tenant_class baseline ahead surface: Oyatie can tie SLO burn to release promotion more tightly than normal counterpart surfaces.
9. paid tenant_class baseline parity surface: metrics/logs/traces/profiles/dashboard/alerting stack choice aligns most closely with Grafana Cloud.
10. paid tenant_class baseline catch-up surface: synthetics, RUM, session replay, database monitoring, and network monitoring are missing.
11. paid tenant_class baseline blocker: OpenTofu context directories are absent across all six required contexts.
12. paid tenant_class baseline blocker: Terraform RBAC reference is incompatible with OpenTofu-only doctrine.
13. paid tenant_class scale headline: target numbers are ambitious but undocumented as measured performance.
14. paid tenant_class scale ahead surface: additive provenance and release-gate evidence could exceed counterpart workflow coupling.
15. paid tenant_class scale parity surface: Mimir-scale metric targets can approach Grafana Mimir published capacity examples.
16. paid tenant_class scale catch-up surface: Datadog/New Relic product breadth remains much larger than current Oyatie artifacts.
17. paid tenant_class scale blocker: no benchmark harness, source tree, or test evidence proves ingest/query numbers.
18. paid tenant_class scale blocker: no shard/cell/control-plane implementation exists in the microservice path.
19. compliance_pack-gated paid tenant_class headline: differentiated sovereignty story, weak implementation proof.
20. compliance_pack-gated paid tenant_class ahead surface: single-tenant pack, HSM custody, WORM evidence, and air-gap behavior are strong additive ambitions.
21. compliance_pack-gated paid tenant_class parity surface: private/enterprise posture maps to counterpart enterprise tiers only at a conceptual level.
22. compliance_pack-gated paid tenant_class catch-up surface: counterpart enterprise-market offerings have mature product workflows that Oyatie has not documented.
23. compliance_pack-gated paid tenant_class blocker: no pack OpenTofu module exists for `iac/oyatie-iaas/`, `iac/colo/`, or other context paths.
24. compliance_pack-gated paid tenant_class blocker: architecture lines 693-704 discuss credential isolation but not complete sovereign-pack runtime.
25. Cross-tier P1: six-context deployment support is absent and affects all tiers.
26. Cross-tier P1: OpenTofu-only compliance is absent and one Terraform artifact remains.
27. Cross-tier P1: OCI Always Free demo_trial tenant_class Always Free is absent.
28. Cross-tier P2: OS matrix support is absent across all tiers.
29. Cross-tier P2: Rust-strict scan found no forbidden source files, but SDK roadmap prose drifts into forbidden languages.
30. Cross-tier P2: no canonical build invocation can succeed because there is no Rust source tree under this microservice path.
31. Cross-tier P2: counterpart parity matrix in existing docs uses a different competitor set than this audit's Datadog/New Relic/Grafana Cloud bar.
32. Cross-tier P2: manifest tiers use `T0/T1/T2` while capability tiers use demo_trial tenant_class/paid tenant_class baseline/paid tenant_class scale/compliance_pack-gated paid tenant_class.
33. Cross-tier P2: PRD references missing e2e tests and a missing runbook path.
34. Cross-tier P2: compliance references Terraform RBAC instead of OpenTofu RBAC.
35. Cross-tier P3: additive release-gate observability is worth preserving through implementation.
36. Cross-tier P3: ClickHouse trace analytics may remain acceptable only if provider/deployment-context posture is explicit.
37. Cross-tier P3: OTel-first integration posture should be documented as a deliberate scope choice if marketplace breadth is deferred.
38. Wave 14 recommendation: split OCI Always Free demo_trial tenant_class Always Free from paid demo_trial tenant_class in the tier matrix.
39. Wave 14 recommendation: add context-specific OpenTofu modules before any performance or parity claim is accepted.
40. Wave 14 recommendation: add supported-oses.json and CI/package evidence before claiming deployable support.
41. Wave 14 recommendation: repair Terraform references and delete or migrate `iac/terraform/grafana-rbac.tf`.
42. Wave 14 recommendation: update existing competitor docs to Datadog/New Relic/Grafana Cloud or justify multiple counterpart bars.
43. Wave 14 recommendation: add an observability product-surface ADR covering synthetics, RUM, error tracking, database, network, LLM observability, and fleet management.
44. Wave 14 recommendation: keep release-gate SLO evidence as an additive Oyatie differentiator, not a replacement for commodity parity.
45. Wave 14 recommendation: require measured benchmark artifacts before upgrading any target number to a production claim.
