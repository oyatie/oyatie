# finops-portal performance benchmark numbers - 2026-05-20

Anchor 1: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854-4228` defines deployment contexts, OpenTofu-only infrastructure, OS support disclosure, Rust build invocation, demo_trial OCI Always Free, severity mapping, and the audit evidence rules.
Anchor 2: `specs/master-plan-sequencing.json:704-868` defines the six deployment contexts, OpenTofu substrate, tenant_class-1 and tenant_class-2 OS matrix, Rust-only language policy, canonical cargo invocation, and OCI Always Free profile.
Anchor 3: `microservices/finops-portal/PRD.md:12-75` defines the product scope: tenant cost attribution, showback, chargeback, budgets, forecasting, commitments, rightsizing, and FOCUS export.
Anchor 4: `microservices/finops-portal/ARCHITECTURE.md:200-204`, `microservices/finops-portal/ARCHITECTURE.md:585-642`, and `microservices/finops-portal/ARCHITECTURE.md:915-929` define the OpenCost/Mimir/FOCUS architecture, deployment shape, intelligence path, and latency/resilience claims.
Anchor 5: `microservices/finops-portal/benchmarks/aws-cost-explorer-gcp-billing-apptio-vs-oyatie.md:11-119` is the local benchmark-intent document; it provides target-style comparison language but lacks the referenced harness/results artifacts.

Methodology disclosure: every Oyatie number in this report is a target number for build-phase validation, not a measured benchmark result.
Methodology disclosure: measured benchmarks must be added later through a reproducible harness, retained raw results, and ADR-0212-style benchmark evidence.
Methodology disclosure: the existing local benchmark document references harness and results paths that were not found in the service inventory.
Methodology disclosure: counterpart public pages rarely publish full p50/p95/p99/RPS numbers, so counterpart rows below are labeled as public source claims, documented qualitative capability, or estimated comparator target.
Methodology disclosure: no row below should be used as production readiness evidence until implementation, load harness, and raw results exist.

External source A: Vantage cost reports documentation, `https://docs.vantage.sh/cost_reports`.
External source B: Vantage budgets documentation, `https://docs.vantage.sh/budgets/`.
External source C: Vantage Kubernetes documentation, `https://docs.vantage.sh/kubernetes/`.
External source D: Vantage API documentation, `https://docs.vantage.sh/api`.
External source E: IBM Cloudability budgets and forecasts documentation, `https://www.apptio.com/products/cloudability/budgets-forecasts/`.
External source F: IBM Cloudability Advanced Containers documentation, `https://www.ibm.com/docs/en/cloudability-commercial/cloudability-standard/saas?topic=allocation-cloudability-advanced-containers`.
External source G: IBM Cloudability Kubernetes rightsizing documentation, `https://www.ibm.com/docs/en/cloudability-commercial/cloudability-essentials/saas?topic=optimize-rightsizing-kubernetes-containers`.
External source H: VMware Tanzu CloudHealth financial management overview, `https://www.vmware.com/docs/solution-overview-vmware-tanzu-cloudhealth-simplify-cloud-financial-management`.
External source I: VMware Tanzu CloudHealth rightsizing overview, `https://www.vmware.com/docs/solution-overview-rightsize-cloud-resources-your-way-with-vmware-tanzu-cloudhealth`.

## §1 Methodology

Benchmark dimension 01: interactive dashboard latency p50, p95, and p99.
Benchmark dimension 02: report-query API latency p50, p95, and p99.
Benchmark dimension 03: invoice retrieval API latency p50, p95, and p99.
Benchmark dimension 04: FOCUS export job enqueue latency.
Benchmark dimension 05: FOCUS export completion latency for 1 million rows.
Benchmark dimension 06: anomaly detection batch latency after cost data arrival.
Benchmark dimension 07: budget evaluation latency after cost data arrival.
Benchmark dimension 08: forecast recomputation latency for one tenant.
Benchmark dimension 09: recommendation recomputation latency for rightsizing.
Benchmark dimension 10: commitment recommendation recomputation latency.
Benchmark dimension 11: sustained read throughput in requests per second.
Benchmark dimension 12: burst read throughput in requests per second.
Benchmark dimension 13: concurrent interactive users.
Benchmark dimension 14: concurrent tenants.
Benchmark dimension 15: maximum monthly cost line items ingested into query store.
Benchmark dimension 16: maximum FOCUS export rows per job.
Benchmark dimension 17: maximum cost-report group-by cardinality.
Benchmark dimension 18: maximum active budget rules.
Benchmark dimension 19: maximum anomaly rules.
Benchmark dimension 20: maximum recommendation objects retained.
Benchmark dimension 21: dashboard freshness after upstream ingestion.
Benchmark dimension 22: budget alert freshness after upstream ingestion.
Benchmark dimension 23: anomaly alert freshness after upstream ingestion.
Benchmark dimension 24: export consistency window.
Benchmark dimension 25: availability target by tenant_class and deployment context.
Benchmark dimension 26: recovery point objective by tenant_class and context.
Benchmark dimension 27: recovery time objective by tenant_class and context.
Benchmark dimension 28: forecast error target measured as weighted MAPE.
Benchmark dimension 29: rightsizing precision target measured by recommendation acceptance feedback.
Benchmark dimension 30: cost allocation completeness target measured by allocated spend percentage.

Workload A: dashboard home loads tenant spend summary, budget status, anomaly count, forecast trend, and five top cost drivers.
Workload B: report query filters by tenant, provider, service, account, tag, region, environment, and 90-day date range.
Workload C: invoice retrieval fetches one monthly tenant invoice plus line-item summary.
Workload D: FOCUS export produces normalized rows with allocation and invoice identifiers.
Workload E: anomaly detection scans rolling 7-day, 30-day, and 90-day windows for one tenant.
Workload F: budget evaluation recomputes thresholds for all active tenant budgets.
Workload G: commitment recommendation evaluates utilization, expiration, and amortized savings opportunity.
Workload H: rightsizing recommendation evaluates CPU, memory, storage, network, and idle resource signals.
Workload I: Kubernetes allocation groups cost by cluster, namespace, pod, label, PVC, and GPU where supported.
Workload J: audit evidence export writes immutable decision and actor context to audit-chain.

OS/arch disclosure 01: tenant_class-1 Linux x86_64 must include Debian 12, Ubuntu 24.04 LTS, RHEL 9, Rocky 9, Alma 9, SLES 15 SP6, Amazon Linux 2023, Oracle Linux 9, Fedora latest-1, Talos latest stable, Flatcar stable, Bottlerocket stable, and Alpine 3.20.
OS/arch disclosure 02: tenant_class-1 macOS is Apple Silicon M5 or newer only.
OS/arch disclosure 03: tenant_class-2 ppc64le and s390x are test-only targets.
OS/arch disclosure 04: Intel macOS, pre-M5 Apple Silicon, FreeBSD, OpenBSD, Windows Server, and Solaris are explicitly out of scope per `specs/master-plan-sequencing.json:777-816`.
OS/arch disclosure 05: finops-portal currently lacks a service-local `supported-oses.json`, so all OS benchmark numbers are target envelopes.
Architecture disclosure 01: the backend must be Rust per `specs/master-plan-sequencing.json:817-856`.
Architecture disclosure 02: current inventory found no Rust source or `Cargo.toml`, so target numbers assume future Rust implementation rather than current code.
Architecture disclosure 03: OpenCost and Mimir are architectural sources per `ARCHITECTURE.md:200-204`.
Architecture disclosure 04: FOCUS export compatibility is architectural direction per `ARCHITECTURE.md:915-918`.
Architecture disclosure 05: the existing generic Terraform file conflicts with the OpenTofu-only benchmark environment and must be replaced before measured infrastructure benchmarks.

Tenant class demo_trial: small tenant, 1 to 25 accounts/projects, 1 to 50 Kubernetes namespaces, up to 5 million monthly cost rows.
Tenant class paid with per_seat billing_component: medium tenant, 25 to 250 accounts/projects, 50 to 500 namespaces, up to 50 million monthly cost rows.
Tenant class paid with per_usage billing_component: large tenant, 250 to 2,500 accounts/projects, 500 to 5,000 namespaces, up to 500 million monthly cost rows.
Tenant class paid with compliance_pack gating: hyperscaler or dedicated tenant, 2,500+ accounts/projects, 5,000+ namespaces, up to 5 billion monthly cost rows.
Context disclosure: `oyatie-public-cloud` targets owned Oyatie cloud infrastructure.
Context disclosure: `guest-on-aws` targets customer AWS infrastructure controlled through OpenTofu modules.
Context disclosure: `guest-on-oci` targets customer OCI infrastructure with an Always Free demo_trial sub-profile.
Context disclosure: `on-prem` targets customer-owned data center infrastructure.
Context disclosure: `colo` targets colocation infrastructure with constrained control-plane assumptions.
Context disclosure: `oyatie-as-cloud-provider` targets Oyatie operating as provider infrastructure for tenants.

## §2 Counterpart numbers

Vantage comparator 01: cost report interactive view target p95 2.5 seconds; source basis: Vantage cost reports public UX capability, estimated comparator target.
Vantage comparator 02: saved report filter change target p95 1.5 seconds; source basis: public report interaction pattern, estimated comparator target.
Vantage comparator 03: report export enqueue target p95 5 seconds; source basis: CSV/PDF/FOCUS export feature, estimated comparator target.
Vantage comparator 04: 1 million row export target completion 5 minutes; source basis: export feature and enterprise SaaS expectation, estimated comparator target.
Vantage comparator 05: budget evaluation freshness target 15 minutes; source basis: budget alerting feature, estimated comparator target.
Vantage comparator 06: anomaly alert freshness target 30 minutes; source basis: cost anomaly alerts documentation, estimated comparator target.
Vantage comparator 07: Kubernetes namespace report p95 3 seconds; source basis: Kubernetes cost report documentation, estimated comparator target.
Vantage comparator 08: rightsizing recommendation refresh target 24 hours; source basis: Kubernetes efficiency and rightsizing surface, estimated comparator target.
Vantage comparator 09: API read throughput target 500 RPS per enterprise tenant; source basis: Vantage API breadth, estimated comparator target.
Vantage comparator 10: concurrent interactive users target 200 per enterprise tenant; source basis: enterprise FinOps SaaS usage, estimated comparator target.
Vantage comparator 11: forecast recompute target 15 minutes for a large tenant; source basis: report forecasting feature, estimated comparator target.
Vantage comparator 12: cost allocation recalculation target 30 minutes; source basis: percent-based allocation feature, estimated comparator target.

Cloudability comparator 01: budget dashboard p95 3 seconds; source basis: Cloudability budget and forecast product documentation, estimated comparator target.
Cloudability comparator 02: budget forecast recompute target 30 minutes; source basis: forecasting product surface, estimated comparator target.
Cloudability comparator 03: daily budget tracking cadence 24 hours or better; source basis: public budget tracking documentation.
Cloudability comparator 04: business mapping refresh target 60 minutes; source basis: business mapping feature, estimated comparator target.
Cloudability comparator 05: advanced container allocation refresh target 30 minutes; source basis: Advanced Containers documentation, estimated comparator target.
Cloudability comparator 06: Kubernetes rightsizing recommendation window 10-day and 30-day lookbacks; source basis: IBM Kubernetes rightsizing documentation.
Cloudability comparator 07: rightsizing recommendation refresh target 24 hours; source basis: rightsizing documentation, estimated comparator target.
Cloudability comparator 08: anomaly detection freshness target 30 minutes; source basis: anomaly feature listing, estimated comparator target.
Cloudability comparator 09: scorecard refresh target 24 hours; source basis: scorecard/governance feature listing, estimated comparator target.
Cloudability comparator 10: unit economics refresh target 24 hours; source basis: unit economics feature listing, estimated comparator target.
Cloudability comparator 11: commitment recommendation refresh target 24 hours; source basis: commitment discount feature listing, estimated comparator target.
Cloudability comparator 12: sustained read throughput target 400 RPS per enterprise tenant; source basis: enterprise SaaS planning estimate.

CloudHealth comparator 01: multi-cloud dashboard p95 3 seconds; source basis: CloudHealth overview, estimated comparator target.
CloudHealth comparator 02: FlexOrgs/Perspectives view refresh target 60 minutes; source basis: overview feature, estimated comparator target.
CloudHealth comparator 03: chargeback report generation target 30 minutes for a large enterprise; source basis: chargeback/showback overview, estimated comparator target.
CloudHealth comparator 04: forecast refresh target 60 minutes; source basis: forecasting overview, estimated comparator target.
CloudHealth comparator 05: anomaly detection freshness target 30 minutes; source basis: anomaly overview, estimated comparator target.
CloudHealth comparator 06: governance policy evaluation target 15 minutes; source basis: governance automation overview, estimated comparator target.
CloudHealth comparator 07: EC2 rightsizing refresh target 24 hours; source basis: rightsizing overview.
CloudHealth comparator 08: EBS rightsizing refresh target 24 hours; source basis: rightsizing overview.
CloudHealth comparator 09: Azure VM rightsizing refresh target 24 hours; source basis: rightsizing overview.
CloudHealth comparator 10: GCE rightsizing refresh target 24 hours; source basis: rightsizing overview.
CloudHealth comparator 11: vSphere/data-center rightsizing refresh target 24 hours; source basis: rightsizing overview.
CloudHealth comparator 12: API read throughput target 400 RPS per enterprise tenant; source basis: open API overview, estimated comparator target.

## §3 Oyatie target numbers

demo_trial target 01: `oyatie-public-cloud` dashboard p95 900 ms, p99 1.8 s, sustained read 150 RPS, concurrent users 25, tenants 25, export rows 1 million, anomaly freshness 15 min, availability 99.5 percent.
demo_trial target 02: `guest-on-aws` dashboard p95 1.1 s, p99 2.2 s, sustained read 125 RPS, concurrent users 20, tenants 20, export rows 1 million, anomaly freshness 20 min, availability 99.3 percent.
demo_trial target 03: `guest-on-oci` OCI Always Free dashboard p95 1.8 s, p99 3.8 s, sustained read 35 RPS, concurrent users 8, tenants 5, export rows 250k, anomaly freshness 60 min, availability 99.0 percent.
demo_trial target 04: `on-prem` dashboard p95 1.4 s, p99 2.8 s, sustained read 100 RPS, concurrent users 15, tenants 10, export rows 750k, anomaly freshness 30 min, availability 99.0 percent.
demo_trial target 05: `colo` dashboard p95 1.3 s, p99 2.6 s, sustained read 110 RPS, concurrent users 15, tenants 10, export rows 750k, anomaly freshness 30 min, availability 99.1 percent.
demo_trial target 06: `oyatie-as-cloud-provider` dashboard p95 800 ms, p99 1.6 s, sustained read 200 RPS, concurrent users 35, tenants 50, export rows 2 million, anomaly freshness 10 min, availability 99.7 percent.

demo_trial target 07: report-query p95 is 1.2 s for owned cloud, 1.4 s for guest AWS, 2.4 s for OCI Always Free, 1.8 s for on-prem, 1.7 s for colo, and 1.0 s for Oyatie provider.
demo_trial target 08: invoice API p95 is 300 ms for owned cloud, 400 ms for guest AWS, 700 ms for OCI Always Free, 500 ms for on-prem, 500 ms for colo, and 250 ms for Oyatie provider.
demo_trial target 09: FOCUS export enqueue p95 is 2 s for owned cloud, 3 s for guest AWS, 5 s for OCI Always Free, 4 s for on-prem, 4 s for colo, and 2 s for Oyatie provider.
demo_trial target 10: budget evaluation completes within 5 minutes for owned cloud, 8 minutes for guest AWS, 20 minutes for OCI Always Free, 10 minutes for on-prem, 10 minutes for colo, and 3 minutes for Oyatie provider.
demo_trial target 11: forecast recomputation completes within 10 minutes for owned cloud, 12 minutes for guest AWS, 45 minutes for OCI Always Free, 20 minutes for on-prem, 20 minutes for colo, and 8 minutes for Oyatie provider.
demo_trial target 12: weighted MAPE target is 15 percent for mature tenants and 25 percent for new tenants across all contexts.

paid with per_seat billing_component target 01: `oyatie-public-cloud` dashboard p95 650 ms, p99 1.3 s, sustained read 800 RPS, concurrent users 150, tenants 250, export rows 10 million, anomaly freshness 10 min, availability 99.9 percent.
paid with per_seat billing_component target 02: `guest-on-aws` dashboard p95 750 ms, p99 1.5 s, sustained read 700 RPS, concurrent users 125, tenants 200, export rows 10 million, anomaly freshness 12 min, availability 99.8 percent.
paid with per_seat billing_component target 03: `guest-on-oci` paid baseline dashboard p95 850 ms, p99 1.8 s, sustained read 600 RPS, concurrent users 100, tenants 150, export rows 8 million, anomaly freshness 15 min, availability 99.8 percent.
paid with per_seat billing_component target 04: `on-prem` dashboard p95 950 ms, p99 2.0 s, sustained read 500 RPS, concurrent users 80, tenants 100, export rows 8 million, anomaly freshness 20 min, availability 99.5 percent.
paid with per_seat billing_component target 05: `colo` dashboard p95 900 ms, p99 1.9 s, sustained read 550 RPS, concurrent users 90, tenants 125, export rows 8 million, anomaly freshness 18 min, availability 99.6 percent.
paid with per_seat billing_component target 06: `oyatie-as-cloud-provider` dashboard p95 550 ms, p99 1.1 s, sustained read 1,000 RPS, concurrent users 200, tenants 350, export rows 15 million, anomaly freshness 8 min, availability 99.95 percent.

paid with per_seat billing_component target 07: report-query p95 is 850 ms for owned cloud, 950 ms for guest AWS, 1.1 s for guest OCI, 1.3 s for on-prem, 1.2 s for colo, and 700 ms for Oyatie provider.
paid with per_seat billing_component target 08: invoice API p95 is 200 ms for owned cloud, 250 ms for guest AWS, 300 ms for guest OCI, 350 ms for on-prem, 330 ms for colo, and 180 ms for Oyatie provider.
paid with per_seat billing_component target 09: FOCUS export enqueue p95 is 1.5 s for owned cloud, 2 s for guest AWS, 2.2 s for guest OCI, 2.5 s for on-prem, 2.5 s for colo, and 1.2 s for Oyatie provider.
paid with per_seat billing_component target 10: budget evaluation completes within 2 minutes for owned cloud, 3 minutes for guest AWS, 4 minutes for guest OCI, 5 minutes for on-prem, 5 minutes for colo, and 90 seconds for Oyatie provider.
paid with per_seat billing_component target 11: forecast recomputation completes within 5 minutes for owned cloud, 6 minutes for guest AWS, 8 minutes for guest OCI, 10 minutes for on-prem, 10 minutes for colo, and 4 minutes for Oyatie provider.
paid with per_seat billing_component target 12: weighted MAPE target is 12 percent for mature tenants and 20 percent for new tenants across all contexts.

paid with per_usage billing_component target 01: `oyatie-public-cloud` dashboard p95 450 ms, p99 900 ms, sustained read 3,000 RPS, concurrent users 800, tenants 2,500, export rows 100 million, anomaly freshness 5 min, availability 99.95 percent.
paid with per_usage billing_component target 02: `guest-on-aws` dashboard p95 550 ms, p99 1.1 s, sustained read 2,500 RPS, concurrent users 650, tenants 2,000, export rows 100 million, anomaly freshness 7 min, availability 99.9 percent.
paid with per_usage billing_component target 03: `guest-on-oci` paid scale dashboard p95 650 ms, p99 1.3 s, sustained read 2,000 RPS, concurrent users 500, tenants 1,500, export rows 75 million, anomaly freshness 8 min, availability 99.9 percent.
paid with per_usage billing_component target 04: `on-prem` dashboard p95 750 ms, p99 1.6 s, sustained read 1,500 RPS, concurrent users 350, tenants 1,000, export rows 50 million, anomaly freshness 12 min, availability 99.7 percent.
paid with per_usage billing_component target 05: `colo` dashboard p95 700 ms, p99 1.5 s, sustained read 1,750 RPS, concurrent users 400, tenants 1,250, export rows 60 million, anomaly freshness 10 min, availability 99.8 percent.
paid with per_usage billing_component target 06: `oyatie-as-cloud-provider` dashboard p95 350 ms, p99 750 ms, sustained read 4,000 RPS, concurrent users 1,000, tenants 3,000, export rows 150 million, anomaly freshness 3 min, availability 99.97 percent.

paid with per_usage billing_component target 07: report-query p95 is 600 ms for owned cloud, 700 ms for guest AWS, 800 ms for guest OCI, 1.0 s for on-prem, 900 ms for colo, and 500 ms for Oyatie provider.
paid with per_usage billing_component target 08: invoice API p95 is 150 ms for owned cloud, 180 ms for guest AWS, 220 ms for guest OCI, 300 ms for on-prem, 260 ms for colo, and 120 ms for Oyatie provider.
paid with per_usage billing_component target 09: FOCUS export enqueue p95 is 1.0 s for owned cloud, 1.2 s for guest AWS, 1.5 s for guest OCI, 2.0 s for on-prem, 1.8 s for colo, and 800 ms for Oyatie provider.
paid with per_usage billing_component target 10: budget evaluation completes within 45 seconds for owned cloud, 60 seconds for guest AWS, 75 seconds for guest OCI, 120 seconds for on-prem, 100 seconds for colo, and 30 seconds for Oyatie provider.
paid with per_usage billing_component target 11: forecast recomputation completes within 2 minutes for owned cloud, 3 minutes for guest AWS, 4 minutes for guest OCI, 6 minutes for on-prem, 5 minutes for colo, and 90 seconds for Oyatie provider.
paid with per_usage billing_component target 12: weighted MAPE target is 9 percent for mature tenants and 15 percent for new tenants across all contexts.

paid with compliance_pack gating target 01: `oyatie-public-cloud` dashboard p95 300 ms, p99 650 ms, sustained read 12,000 RPS, concurrent users 4,000, tenants 10,000, export rows 1 billion, anomaly freshness 2 min, availability 99.99 percent.
paid with compliance_pack gating target 02: `guest-on-aws` dashboard p95 380 ms, p99 800 ms, sustained read 10,000 RPS, concurrent users 3,000, tenants 8,000, export rows 750 million, anomaly freshness 3 min, availability 99.95 percent.
paid with compliance_pack gating target 03: `guest-on-oci` paid dedicated dashboard p95 450 ms, p99 900 ms, sustained read 8,000 RPS, concurrent users 2,500, tenants 6,000, export rows 600 million, anomaly freshness 4 min, availability 99.95 percent.
paid with compliance_pack gating target 04: `on-prem` dedicated dashboard p95 600 ms, p99 1.2 s, sustained read 5,000 RPS, concurrent users 1,500, tenants 3,000, export rows 300 million, anomaly freshness 8 min, availability 99.9 percent.
paid with compliance_pack gating target 05: `colo` dedicated dashboard p95 550 ms, p99 1.1 s, sustained read 6,000 RPS, concurrent users 1,800, tenants 4,000, export rows 400 million, anomaly freshness 6 min, availability 99.93 percent.
paid with compliance_pack gating target 06: `oyatie-as-cloud-provider` hyperscaler bar dashboard p95 250 ms, p99 500 ms, sustained read 20,000 RPS, concurrent users 6,000, tenants 20,000, export rows 2 billion, anomaly freshness 90 seconds, availability 99.995 percent.

paid with compliance_pack gating target 07: report-query p95 is 450 ms for owned cloud, 550 ms for guest AWS, 650 ms for guest OCI, 850 ms for on-prem, 750 ms for colo, and 350 ms for Oyatie provider.
paid with compliance_pack gating target 08: invoice API p95 is 100 ms for owned cloud, 130 ms for guest AWS, 160 ms for guest OCI, 240 ms for on-prem, 200 ms for colo, and 80 ms for Oyatie provider.
paid with compliance_pack gating target 09: FOCUS export enqueue p95 is 600 ms for owned cloud, 800 ms for guest AWS, 1.0 s for guest OCI, 1.5 s for on-prem, 1.2 s for colo, and 400 ms for Oyatie provider.
paid with compliance_pack gating target 10: budget evaluation completes within 15 seconds for owned cloud, 20 seconds for guest AWS, 30 seconds for guest OCI, 60 seconds for on-prem, 45 seconds for colo, and 10 seconds for Oyatie provider.
paid with compliance_pack gating target 11: forecast recomputation completes within 60 seconds for owned cloud, 90 seconds for guest AWS, 120 seconds for guest OCI, 180 seconds for on-prem, 150 seconds for colo, and 45 seconds for Oyatie provider.
paid with compliance_pack gating target 12: weighted MAPE target is 7 percent for mature tenants and 12 percent for new tenants across all contexts.

## §4 Per-context overlay

Overlay 01: `oyatie-public-cloud` may assume Oyatie-managed observability, Mimir, object storage, audit-chain, and tenancy primitives.
Overlay 02: `oyatie-public-cloud` should target the lowest latency envelope because provider integration and platform SLOs are under Oyatie control.
Overlay 03: `oyatie-public-cloud` demo_trial should not inherit OCI Always Free resource ceilings.
Overlay 04: `oyatie-public-cloud` benchmark harness must pin OS/arch and cluster shape in the result artifact.
Overlay 05: `guest-on-aws` must account for cross-account IAM, customer network policy, and AWS billing-data delivery cadence.
Overlay 06: `guest-on-aws` should target slightly higher p95 latency than owned cloud because customer VPC and IAM paths add variance.
Overlay 07: `guest-on-aws` must not use CloudFormation, CDK, or hand-edited Terraform state because ADR-0328 requires OpenTofu.
Overlay 08: `guest-on-aws` measured runs must disclose region, account count, CUR/FOCUS source cadence, and tenant isolation settings.
Overlay 09: `guest-on-oci` has two distinct profiles: Always Free demo_trial and paid baseline.
Overlay 10: `guest-on-oci` Always Free demo_trial is capacity constrained and must downshift export rows, tenants, concurrency, and freshness targets.
Overlay 11: `guest-on-oci` paid baseline can converge toward paid with per_seat billing_component and paid with per_usage billing_component targets when external object storage and database capacity exist.
Overlay 12: `guest-on-oci` benchmark artifacts must prove no AWS spillover or non-OCI state backend in the Always Free profile.
Overlay 13: `on-prem` must account for slower procurement, customer-managed storage, and heterogeneous network.
Overlay 14: `on-prem` targets should trade lower throughput for explicit offline and regulated-environment operability.
Overlay 15: `on-prem` benchmark artifacts must disclose hardware, storage, Kubernetes distribution, OS image, and network topology.
Overlay 16: `colo` should target better network predictability than generic on-prem when hardware is reserved and known.
Overlay 17: `colo` should still assume limited platform control relative to Oyatie-owned cloud.
Overlay 18: `colo` benchmark artifacts must separate data-plane latency from upstream billing data arrival latency.
Overlay 19: `oyatie-as-cloud-provider` is the highest target envelope because Oyatie controls substrate, isolation, observability, and capacity.
Overlay 20: `oyatie-as-cloud-provider` paid with compliance_pack gating is the only profile that should claim hyperscaler-level concurrency before cross-context measurements exist.
Overlay 21: all contexts must record RPO, RTO, raw data freshness, and query-store freshness separately.
Overlay 22: all contexts must record whether OpenCost, Mimir, Postgres, object storage, and audit-chain are local, shared, or managed dependencies.
Overlay 23: all contexts must publish workload data shape because row count alone hides grouping cardinality.
Overlay 24: all contexts must report cost allocation completeness as a percentage of spend mapped to owner and tenant.
Overlay 25: all contexts must report security overhead because Cedar and audit-chain writes affect p99.
Overlay 26: all contexts must report cold-cache and warm-cache dashboard latency separately.
Overlay 27: all contexts must report export chunk size and object-store latency.
Overlay 28: all contexts must report failed export retry behavior.
Overlay 29: all contexts must report anomaly false-positive sample review.
Overlay 30: all contexts must report rightsizing recommendation precision using post-action feedback where available.

## §5 Comparison narrative

Comparison 01: Dashboard latency targets are intentionally ahead of the estimated comparator p95 values for paid with per_usage billing_component and paid with compliance_pack gating.
Comparison 02: demo_trial OCI Always Free targets are below counterpart enterprise expectations by design because the resource envelope is constrained.
Comparison 03: paid with per_seat billing_component targets aim for parity with Vantage, Cloudability, and CloudHealth interactive SaaS expectations.
Comparison 04: paid with per_usage billing_component targets aim to beat public qualitative counterpart expectations for common dashboard and report-query paths.
Comparison 05: paid with compliance_pack gating targets are hyperscaler-bar targets and should remain unclaimed until measured across at least owned cloud and one guest context.
Comparison 06: FOCUS export is a potential advantage because local contracts make it first-class, but export speed is not measured.
Comparison 07: Budget freshness targets are competitive at paid with per_seat billing_component and better at paid with per_usage billing_component/paid with compliance_pack gating, but local budget CRUD is missing.
Comparison 08: Anomaly freshness targets are competitive at paid with per_seat billing_component and better at paid with per_usage billing_component/paid with compliance_pack gating, but local anomaly lifecycle API is missing.
Comparison 09: Forecast quality targets are ambitious and need real tenant history, confidence intervals, and model governance.
Comparison 10: Commitment and rightsizing refresh targets align with Cloudability and CloudHealth daily recommendation windows.
Comparison 11: Kubernetes targets are currently aspirational because local contracts do not expose cluster, namespace, pod, label, GPU, or PVC dimensions.
Comparison 12: API throughput targets are ambitious because current OpenAPI breadth is narrow.
Comparison 13: OS benchmark breadth is wider than public SaaS counterparts because Oyatie doctrine requires package and OS disclosure.
Comparison 14: OpenTofu context benchmarks are blocked until `iac/<context>/` modules exist.
Comparison 15: Rust build benchmarks are blocked until Rust source and canonical cargo invocation exist in service-local artifacts.
Comparison 16: Existing benchmark prose should be retained as comparative intent, not as performance proof.
Comparison 17: The benchmark set must include negative cases such as missing upstream data, stale allocations, and export retries.
Comparison 18: The benchmark set must include tenant isolation checks because multi-tenant cost data has confidentiality impact.
Comparison 19: The benchmark set must include audit-chain write overhead because compliance evidence is part of the product surface.
Comparison 20: The benchmark set must include low-resource OCI Always Free runs because ADR-0328 treats this as a first-class profile.
Comparison 21: The benchmark set must include p99 and error-rate evidence, not only median latency.
Comparison 22: The benchmark set must store raw data, workload generator version, environment manifest, and result summaries.
Comparison 23: The benchmark set must distinguish measured values from target values in filenames and document headers.
Comparison 24: The service cannot honestly claim counterpart performance parity yet because there is no executable service or benchmark harness.
Comparison 25: The service can claim a defined target envelope for future implementation because this report gives context, tenant_class, and workload-specific targets.

## §6 Build-phase acceptance ledger for measured benchmarks

Acceptance 01: measured benchmark artifacts must include the git commit, branch, OpenTofu module commit, and Rust compiler version.
Acceptance 02: measured benchmark artifacts must include the canonical build command from `specs/master-plan-sequencing.json:817-856`.
Acceptance 03: measured benchmark artifacts must include the deployment context id from `specs/master-plan-sequencing.json:704-746`.
Acceptance 04: measured benchmark artifacts must include the tenant_class name and whether `guest-on-oci` is using Always Free or paid baseline.
Acceptance 05: measured benchmark artifacts must include the OS name, OS version, CPU architecture, package format, and kernel version.
Acceptance 06: measured benchmark artifacts must include the Kubernetes distribution or non-Kubernetes runtime declaration.
Acceptance 07: measured benchmark artifacts must include OpenCost version, Mimir version, database version, object-store backend, and audit-chain backend.
Acceptance 08: measured benchmark artifacts must include tenant count, account count, namespace count, cost-row count, active budget count, active anomaly rule count, and report count.
Acceptance 09: measured benchmark artifacts must include cold-cache and warm-cache dashboard timings separately.
Acceptance 10: measured benchmark artifacts must include p50, p95, p99, maximum latency, error count, and timeout count for every API workload.
Acceptance 11: measured benchmark artifacts must include sustained throughput, burst throughput, and saturation point.
Acceptance 12: measured benchmark artifacts must include CPU, memory, storage IO, network IO, and database query summaries.
Acceptance 13: measured benchmark artifacts must include export row count, export byte count, chunk count, retry count, and object-store write latency.
Acceptance 14: measured benchmark artifacts must include anomaly input size, detected anomaly count, false-positive sample review, and detection freshness.
Acceptance 15: measured benchmark artifacts must include budget evaluation count, alert count, late alert count, and evaluation freshness.
Acceptance 16: measured benchmark artifacts must include forecast horizon, historical window, weighted MAPE, and confidence-band calibration.
Acceptance 17: measured benchmark artifacts must include recommendation input resource count, recommendation count, precision sample, and stale recommendation count.
Acceptance 18: measured benchmark artifacts must include Cedar authorization overhead for report, invoice, export, and admin operations.
Acceptance 19: measured benchmark artifacts must include audit-chain write overhead for critical state changes.
Acceptance 20: measured benchmark artifacts must include tenant isolation checks for report, invoice, export, budget, and anomaly data.
Acceptance 21: measured benchmark artifacts must include provider-neutrality checks showing business logic did not call AWS, OCI, Azure, or GCP SDKs directly.
Acceptance 22: measured benchmark artifacts must include OpenTofu plan output path and state backend identifier for the context.
Acceptance 23: measured benchmark artifacts must include proof that no Terraform, Pulumi, CloudFormation, SSH provisioner, local-exec, or null_resource path was used.
Acceptance 24: measured benchmark artifacts must include sigstore or cosign module signing evidence when modules are loaded from a registry.
Acceptance 25: measured benchmark artifacts must include failure-mode tests for upstream cost data delay.
Acceptance 26: measured benchmark artifacts must include failure-mode tests for export object-store write failure.
Acceptance 27: measured benchmark artifacts must include failure-mode tests for audit-chain unavailability.
Acceptance 28: measured benchmark artifacts must include failure-mode tests for stale budget thresholds.
Acceptance 29: measured benchmark artifacts must include failure-mode tests for anomaly storm suppression.
Acceptance 30: measured benchmark artifacts must include failure-mode tests for recommendation engine backlog.
Acceptance 31: measured benchmark artifacts must include a per-context comparison against the targets in §3.
Acceptance 32: measured benchmark artifacts must classify every target as passed, failed, or intentionally revised.
Acceptance 33: any revised target must cite the measured evidence and the operational tradeoff.
Acceptance 34: any passed target must include raw evidence path rather than only a summarized table.
Acceptance 35: any failed target must include a remediation issue or implementation-plan update path.
Acceptance 36: demo_trial OCI Always Free measurements must run on the constrained profile, not a paid OCI shape.
Acceptance 37: demo_trial OCI Always Free measurements must show actual resource consumption and confirm no paid-resource drift.
Acceptance 38: paid with per_seat billing_component measurements must include at least one paid guest context in addition to owned cloud.
Acceptance 39: paid with per_usage billing_component measurements must include high-cardinality report and Kubernetes allocation workloads.
Acceptance 40: paid with compliance_pack gating measurements must include dedicated or isolated control-plane assumptions and must say which context cannot support the target.
Acceptance 41: measured counterpart comparison must keep public-source claims separate from Oyatie raw results.
Acceptance 42: measured counterpart comparison must not imply the public counterparts published p99 values when they did not.
Acceptance 43: measured counterpart comparison must use conservative labels such as ahead target, parity target, catch-up target, or not comparable.
Acceptance 44: measured benchmark reports must be stored under a service-local evidence path referenced from `benchmarks/`.
Acceptance 45: measured benchmark reports must be discoverable from `README.md` once evidence exists.
Acceptance 46: measured benchmark reports must not overwrite target reports because targets and measurements serve different purposes.
Acceptance 47: measured benchmark reports must include enough command detail for a cold engineer to repeat the run.
Acceptance 48: measured benchmark reports must include workload seed data or seed generator version.
Acceptance 49: measured benchmark reports must include clock synchronization status because freshness claims depend on time.
Acceptance 50: measured benchmark reports must include data-retention settings because retention affects query performance.
Acceptance 51: measured benchmark reports must include cache configuration because dashboard latency depends heavily on cache state.
Acceptance 52: measured benchmark reports must include database index and partition settings.
Acceptance 53: measured benchmark reports must include query-store compaction settings if Mimir or an equivalent time-series backend is used.
Acceptance 54: measured benchmark reports must include object-store lifecycle settings when export retention is tested.
Acceptance 55: measured benchmark reports must include alert delivery sink behavior if alert routing is implemented.
Acceptance 56: measured benchmark reports must include security mode because authentication, authorization, and audit writes change p99.
Acceptance 57: measured benchmark reports must include a clear statement when a capability is not implemented and therefore cannot be measured.
Acceptance 58: measured benchmark reports must not backfill missing implementation with estimates.
Acceptance 59: measured benchmark reports must satisfy documentation-rigor intern-buildability expectations by explaining how to reproduce every number.
Acceptance 60: measured benchmark reports must be reviewed before any Wave aggregation labels finops-portal as counterpart-performance-ready.
