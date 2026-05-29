# finops-portal feature parity matrix - 2026-05-20

Anchor 1: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854-4228` defines the six deployment contexts, OpenTofu-only substrate, OS matrix, Rust-only implementation policy, OCI Always Free floor, audit dimensions, severity model, and deliverable requirements.
Anchor 2: `specs/master-plan-sequencing.json:704-868` defines `deployment_contexts`, `iac_substrate`, `supported_oses`, `language_policy`, and `oci_always_free` as machine-readable canonical direction.
Anchor 3: `microservices/finops-portal/PRD.md:12-75` defines finops-portal as the cost intelligence, chargeback, budget, forecasting, commitment, and FOCUS export surface.
Anchor 4: `microservices/finops-portal/ARCHITECTURE.md:200-204`, `microservices/finops-portal/ARCHITECTURE.md:585-642`, and `microservices/finops-portal/ARCHITECTURE.md:915-929` define the local OpenCost/Mimir/FOCUS architecture, deployment assumptions, intelligence surface, and core NFR claims.
Anchor 5: `docs/standards/documentation-rigor.md:133-156` and `docs/standards/brief-template.md:1727-1806` define intern-buildability, hyperscaler-grade evidence expectations, and the anti-patterns for scaffold text and scripted-looking body prose.

External source A: Vantage cost reports documentation, `https://docs.vantage.sh/cost_reports`.
External source B: Vantage budgets documentation, `https://docs.vantage.sh/budgets/`.
External source C: Vantage Kubernetes documentation, `https://docs.vantage.sh/kubernetes/`.
External source D: Vantage API documentation, `https://docs.vantage.sh/api`.
External source E: IBM Cloudability budgets and forecasts product documentation, `https://www.apptio.com/products/cloudability/budgets-forecasts/`.
External source F: IBM Cloudability Advanced Containers documentation, `https://www.ibm.com/docs/en/cloudability-commercial/cloudability-standard/saas?topic=allocation-cloudability-advanced-containers`.
External source G: IBM Cloudability Kubernetes rightsizing documentation, `https://www.ibm.com/docs/en/cloudability-commercial/cloudability-essentials/saas?topic=optimize-rightsizing-kubernetes-containers`.
External source H: VMware Tanzu CloudHealth financial management overview, `https://www.vmware.com/docs/solution-overview-vmware-tanzu-cloudhealth-simplify-cloud-financial-management`.
External source I: VMware Tanzu CloudHealth rightsizing overview, `https://www.vmware.com/docs/solution-overview-rightsize-cloud-resources-your-way-with-vmware-tanzu-cloudhealth`.
External source J: FinOps FOCUS specification home, `https://focus.finops.org/focus-specification/`.

## §1 Counterpart 1 - Vantage capability surface

Vantage capability 01: Multi-cloud cost report construction across connected cloud accounts; local finops-portal has cost reporting intent in `PRD.md:35-39` and report-query bounded context in `manifest.json:15-24`, but lacks a documented report builder DSL.
Vantage capability 02: Cost report filters by provider, account, service, tag, region, and time range; local contracts show invoice and FOCUS endpoints in `contracts/tenant-invoice-public.openapi.yaml:26-90`, but do not expose a full arbitrary filter surface.
Vantage capability 03: Cost report grouping by provider resource hierarchy; local architecture mentions OpenCost/Mimir integration at `ARCHITECTURE.md:200-204`, but does not specify grouping grammar.
Vantage capability 04: Saved report URLs and report sharing; local dashboard files exist, but no contract defines saved report lifecycle.
Vantage capability 05: Forecasts rendered inside cost reports; local PRD includes forecasting in `PRD.md:47-48`, and architecture includes intelligence at `ARCHITECTURE.md:640-642`, but benchmark proof is absent.
Vantage capability 06: CSV export of report data; local FOCUS export exists in `contracts/tenant-invoice-public.openapi.yaml:169-205`, but CSV is not evidenced.
Vantage capability 07: PDF export for stakeholder reporting; local artifacts do not evidence PDF export.
Vantage capability 08: FOCUS export; local artifacts strongly evidence this through `contracts/tenant-invoice-public.openapi.yaml:169-205` and `contracts/focus-export-internal.asyncapi.yaml:20-49`.
Vantage capability 09: Percent-based cost allocation; local cost allocation and chargeback intent appears in `PRD.md:39-42`, but allocation rule grammar is not in the public contract.
Vantage capability 10: Custom dashboards over cost reports; local dashboard JSON files exist under `dashboards/`, but no dashboard API exists.
Vantage capability 11: Budget creation from cost reports; local budget alerts appear in `PRD.md:43-44` and runbooks, but budget creation contract is not surfaced.
Vantage capability 12: Hierarchical budgets; local tenant_class and runbook material mentions tenant budget operation, but no hierarchy/import semantics are evidenced.
Vantage capability 13: CSV budget import; local artifacts do not evidence budget CSV import.
Vantage capability 14: Budget alert routing to team destinations; local runbooks evidence alert operations, but contracts do not show destination management.
Vantage capability 15: Cost anomaly detection at report scope; local anomaly support is evidenced by `runbooks/tenant-cost-anomaly-spike.md` and `dashboards/anomaly-investigation.json`, but automation detail is thin.
Vantage capability 16: Anomaly alert delivery to collaboration tools; local artifacts do not evidence Slack, Teams, Jira, or email routing contract.
Vantage capability 17: Anomaly ignore/archive workflow; local artifacts do not define anomaly lifecycle states.
Vantage capability 18: Anomaly investigation assistant; local architecture mentions intelligence, but no FinOps agent UX contract exists.
Vantage capability 19: Kubernetes cost visibility by cluster; local OpenCost dependency implies cluster allocation at `ARCHITECTURE.md:200-204`, but cluster-specific API is absent.
Vantage capability 20: Kubernetes cost visibility by namespace; local architecture implies it, but local contracts do not expose it.
Vantage capability 21: Kubernetes cost visibility by pod; local architecture implies it, but local contracts do not expose it.
Vantage capability 22: Kubernetes GPU cost reporting; local artifacts do not evidence GPU-specific allocation.
Vantage capability 23: Kubernetes efficiency metrics; local artifacts mention rightsizing, but no efficiency metric schema is present.
Vantage capability 24: Kubernetes rightsizing recommendations; local runbook and dashboard material suggest rightsizing, but no recommendation API is visible.
Vantage capability 25: API access to cost reports; local OpenAPI exposes invoice and FOCUS export, not cost report CRUD.
Vantage capability 26: API access to folders and dashboards; local artifacts do not evidence folder/dashboard APIs.
Vantage capability 27: Programmatic resource inventory for cost objects; local artifacts do not evidence a resource inventory endpoint.
Vantage capability 28: Cost report comparison period controls; local benchmark docs discuss dashboard latency, but contract does not define comparison windows.
Vantage capability 29: Marketplace and SaaS account integrations; local architecture is provider-agnostic and OpenTofu-based, but service-local integration catalog is absent.
Vantage capability 30: Custom cost category views; local cost category policy may exist through Cedar and allocation artifacts, but public contract does not define user-managed categories.
Vantage capability 31: Managed business unit rollups; local tenant and fleet rollup dashboards exist, but enterprise business hierarchy is not explicit.
Vantage capability 32: Report comments or workflow collaboration; local artifacts do not evidence collaboration.
Vantage capability 33: Report-level access controls; Cedar policies exist under `policy/cedar/`, but report-level policy contract is not complete.
Vantage capability 34: Forecast confidence visualization; local docs mention forecasting but no confidence bands appear in contracts.
Vantage capability 35: Cost report API pagination and cursoring; local OpenAPI pagination for FOCUS export is not fully specified in the audit evidence.
Vantage capability 36: Self-serve user experience for FinOps analysts; local PRD names target users at `PRD.md:24-31`, but implementation code is absent.
Vantage capability 37: Provider invoice normalization; local PRD delegates provider bill ingestion to cloud-iac at `PRD.md:59`, so finops-portal owns normalized presentation rather than ingestion.
Vantage capability 38: Spend forecast alerting; local budget runbooks imply alerting, but forecast alert contracts are missing.
Vantage capability 39: Credit visualization; local credit ledger artifacts exist and are additive relative to many Vantage headline pages.
Vantage capability 40: Transparent public API documentation; local OpenAPI exists for selected surfaces, but lacks the breadth of Vantage API docs.

## §2 Counterpart 2 - Cloudability (IBM) capability surface

Cloudability capability 01: Budgets linked to business mappings; local cost allocation exists but business mapping model is not explicit.
Cloudability capability 02: Forecasts based on historical spend and modeled trends; local forecasting intent exists in `PRD.md:47-48`, but measured forecast quality is absent.
Cloudability capability 03: Budget variance detection; local runbooks imply budget exhaustion and headroom events.
Cloudability capability 04: Daily budget tracking; local SLOs and runbooks mention alerting, but daily evaluation cadence is not a contract.
Cloudability capability 05: Budget alerts to stakeholders; local alert runbooks exist, but destination management is missing.
Cloudability capability 06: Business mapping of cloud spend; local tenant attribution is present, but multi-level business mapping is not evidenced.
Cloudability capability 07: Commercial billing normalization; local invoice contract exists, but commercial billing ingestion is delegated or under-specified.
Cloudability capability 08: Container cost allocation; local OpenCost base supports this conceptually, but contract surface is absent.
Cloudability capability 09: Cost allocation rule management; local policy files exist, but allocation rule CRUD is not shown.
Cloudability capability 10: Cost sharing across teams; local chargeback/showback exists at product intent level, but shared-cost allocation semantics are thin.
Cloudability capability 11: Dashboards for FinOps teams; local dashboards exist under `dashboards/`, with no dashboard lifecycle API.
Cloudability capability 12: Sustainability reporting; local artifacts do not evidence carbon or energy reporting.
Cloudability capability 13: Tagging quality and governance; local documents mention allocation, but tag hygiene scorecards are not surfaced.
Cloudability capability 14: True Cost Explorer style exploratory analytics; local service has report-query intent but no explorer grammar.
Cloudability capability 15: Anomaly detection; local anomaly runbooks and dashboards exist.
Cloudability capability 16: Governance workflows; local Cedar policies exist, but workflow enforcement is not complete.
Cloudability capability 17: Scorecards for FinOps performance; local artifacts do not evidence scorecards.
Cloudability capability 18: Unit economics; local docs do not show unit metric definitions such as cost per request or cost per tenant action.
Cloudability capability 19: Workload planning; local capacity model exists, but proactive workload planning UX is not specified.
Cloudability capability 20: Commitment discount planning; local PRD includes commitment recommendations at `PRD.md:45-46`.
Cloudability capability 21: Reserved instance and savings plan analysis; local docs mention commitments, but provider-specific recommendation engines are not contractually described.
Cloudability capability 22: Rightsizing recommendations; local PRD includes rightsizing at `PRD.md:49-50`.
Cloudability capability 23: Kubernetes advanced allocation; local OpenCost base exists but advanced allocation contract is missing.
Cloudability capability 24: Kubernetes container recommendation window; local artifacts do not define lookback windows by container.
Cloudability capability 25: Downloadable recommendation reports; local artifacts do not evidence downloadable recommendation exports.
Cloudability capability 26: Views for account group organization; local tenant model exists, but Cloudability-style Views are absent.
Cloudability capability 27: Forecasting with AI assist; local architecture says intelligence, but implementation and model governance are not evidenced.
Cloudability capability 28: Automated anomaly routing; local artifacts do not define collaboration routing.
Cloudability capability 29: Cost owner accountability workflows; local PRD target users include Finance and SRE, but accountability workflow states are not specified.
Cloudability capability 30: Budget approval workflow; local artifacts do not evidence approvals.
Cloudability capability 31: Cost policy exception management; local Cedar policy exists, but exception workflow is not specified.
Cloudability capability 32: Public cloud provider billing integration; local ingestion is intentionally outside finops-portal per `PRD.md:59`.
Cloudability capability 33: Kubernetes cost by namespace and label; local OpenCost/Mimir base implies this but contract is absent.
Cloudability capability 34: Cloud rightsizing across EC2, EBS, VMs, and databases; local rightsizing is not provider-resource complete.
Cloudability capability 35: Chargeback and showback; local PRD directly includes this at `PRD.md:41-42`.
Cloudability capability 36: FinOps operational playbooks; local runbook suite is stronger than pure product docs for several incident cases.
Cloudability capability 37: Budget headroom notifications; local `runbooks/tenant-budget-headroom-low.md` evidences this.
Cloudability capability 38: Regulatory evidence emission; local quarterly regulator artifacts are additive relative to common Cloudability product pages.
Cloudability capability 39: Credit application reconciliation; local `runbooks/credit-application-reconciliation.md` evidences this.
Cloudability capability 40: Open standard FOCUS export; local contract explicitly exposes this.

## §3 Counterpart 3 - CloudHealth (VMware) capability surface

CloudHealth capability 01: Public, private, hybrid, and multi-cloud cost aggregation; local six-context doctrine requires this, but service IaC does not support six contexts.
CloudHealth capability 02: FlexOrgs-style organizational modeling; local tenant model exists, but FlexOrg-equivalent hierarchy is absent.
CloudHealth capability 03: Perspectives-style business views; local report-query bounded context exists, but perspective modeling is absent.
CloudHealth capability 04: Cost allocation across organizational views; local cost allocation intent exists, but view-scoped rules are missing.
CloudHealth capability 05: Showback reporting; local PRD includes showback.
CloudHealth capability 06: Chargeback reporting; local PRD includes chargeback.
CloudHealth capability 07: Budgets; local PRD and runbooks include budgets.
CloudHealth capability 08: Forecasting; local PRD includes forecasting.
CloudHealth capability 09: Anomaly detection; local dashboard and runbooks include anomalies.
CloudHealth capability 10: Commitment and reservation management; local PRD includes commitment recommendations.
CloudHealth capability 11: Rightsizing recommendations; local PRD includes rightsizing recommendations.
CloudHealth capability 12: Governance policy automation; local Cedar policies exist, but automation actions are not defined.
CloudHealth capability 13: Automated remediation actions; local docs do not clarify if actioning belongs here or in cloud-iac.
CloudHealth capability 14: Open APIs; local OpenAPI and AsyncAPI exist for selected surfaces only.
CloudHealth capability 15: Multi-cloud accounts and data source integration; local ingestion responsibility is not owned here.
CloudHealth capability 16: vSphere/data-center rightsizing; local artifacts do not evidence vSphere rightsizing.
CloudHealth capability 17: EC2 rightsizing; local rightsizing is generic and not EC2-specific.
CloudHealth capability 18: EBS rightsizing; local artifacts do not evidence EBS-specific recommendations.
CloudHealth capability 19: Azure VM rightsizing; local artifacts do not evidence Azure VM-specific recommendations.
CloudHealth capability 20: Azure SQL rightsizing; local artifacts do not evidence Azure SQL-specific recommendations.
CloudHealth capability 21: Google Compute Engine rightsizing; local artifacts do not evidence GCE-specific recommendations.
CloudHealth capability 22: Data center inventory correlation; local on-prem and colo contexts are canonical, but finops-portal lacks deployment IaC and inventory connector docs.
CloudHealth capability 23: Chargeback invoice generation; local invoice OpenAPI exists.
CloudHealth capability 24: Budget alerting; local budget runbooks exist.
CloudHealth capability 25: Forecast trend reporting; local benchmark targets discuss forecast, but no measured implementation exists.
CloudHealth capability 26: Custom report dashboards; local dashboard files exist.
CloudHealth capability 27: Cross-account cost normalization; local FOCUS export suggests normalized output.
CloudHealth capability 28: Account hierarchy and ownership; local tenant model exists but account ownership contract is thin.
CloudHealth capability 29: Policy violation reporting; local Cedar policies exist, but violation event schema is not complete.
CloudHealth capability 30: Cloud provider optimization recommendations; local commitment and rightsizing docs are present but incomplete.
CloudHealth capability 31: Reserved commitment utilization tracking; local commitment documents imply this but no contract surface is present.
CloudHealth capability 32: Cost history retention; local capacity model provides retention assumptions, but enforcement is not executable.
CloudHealth capability 33: Cost data export; local FOCUS export exists.
CloudHealth capability 34: Role-based access control; local Cedar schema and policy files exist.
CloudHealth capability 35: Tenant isolation; local manifest and architecture emphasize tenancy and audit-chain dependencies.
CloudHealth capability 36: Enterprise reporting for executives; local dashboards include fleet cost rollup and tenant drilldown.
CloudHealth capability 37: Collaboration workflows; local artifacts do not evidence collaboration.
CloudHealth capability 38: Provider-neutral packaging; local canonical direction requires provider-neutral packaging, but `iac/terraform-module.tf:1-40` drifts.
CloudHealth capability 39: Operational runbooks; local runbook suite is comparatively substantive.
CloudHealth capability 40: Regulatory/audit evidence; local compliance and audit docs are additive compared with common CloudHealth overview material.

## §4 UNION-coverage matrix

| # | Capability | Vantage has | Cloudability has | CloudHealth has | UNION required | Oyatie finops-portal has | Gap classification |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 001 | Multi-cloud cost reports | yes | yes | yes | yes | partial: `PRD.md:35-39` and dashboard files | catch-up |
| 002 | Provider-neutral six-context operation | partial | partial | yes | yes | doctrine yes; local IaC no | P1 platform gap |
| 003 | Cost report filter grammar | yes | yes | yes | yes | not evidenced in OpenAPI | catch-up |
| 004 | Cost report grouping grammar | yes | yes | yes | yes | not evidenced | catch-up |
| 005 | Saved report lifecycle | yes | yes | yes | yes | not evidenced | catch-up |
| 006 | Custom report query language | yes | yes | yes | yes | report-query BC only | catch-up |
| 007 | Dashboard lifecycle API | yes | yes | yes | yes | dashboard files only | catch-up |
| 008 | CSV export | yes | yes | yes | yes | not evidenced | catch-up |
| 009 | PDF export | yes | yes | yes | yes | not evidenced | catch-up |
| 010 | FOCUS export | yes | partial | partial | yes | yes: `tenant-invoice-public.openapi.yaml:169-205` | parity/additive |
| 011 | FOCUS event stream | partial | partial | partial | yes | yes: `focus-export-internal.asyncapi.yaml:20-49` | ahead |
| 012 | Percent cost allocation | yes | yes | yes | yes | partial | catch-up |
| 013 | Shared cost allocation | partial | yes | yes | yes | partial | catch-up |
| 014 | Tenant chargeback | partial | yes | yes | yes | yes: `PRD.md:41-42` | parity |
| 015 | Tenant showback | partial | yes | yes | yes | yes: `PRD.md:41-42` | parity |
| 016 | Invoice generation | partial | partial | yes | yes | yes: `contracts/tenant-invoice-public.openapi.yaml:26-90` | parity |
| 017 | Invoice dispute workflow | partial | partial | partial | yes | runbook-level only | catch-up |
| 018 | Budget definition API | yes | yes | yes | yes | not evidenced | catch-up |
| 019 | Budget hierarchy | yes | yes | partial | yes | not evidenced | catch-up |
| 020 | Budget CSV import | yes | partial | partial | yes | not evidenced | catch-up |
| 021 | Budget variance detection | yes | yes | yes | yes | runbook-level | partial |
| 022 | Budget headroom alert | yes | yes | yes | yes | yes: `runbooks/tenant-budget-headroom-low.md` | parity |
| 023 | Budget exhaustion response | partial | yes | yes | yes | yes: `runbooks/tenant-budget-exhausted.md` | parity |
| 024 | Alert destination management | yes | yes | yes | yes | not evidenced | catch-up |
| 025 | Email alert delivery | yes | yes | yes | yes | not evidenced | catch-up |
| 026 | Slack alert delivery | yes | yes | yes | yes | not evidenced | catch-up |
| 027 | Teams alert delivery | yes | partial | partial | yes | not evidenced | catch-up |
| 028 | Jira or ticket alert delivery | yes | partial | partial | yes | not evidenced | catch-up |
| 029 | Forecast trend charting | yes | yes | yes | yes | docs only | catch-up |
| 030 | Forecast confidence bands | partial | yes | partial | yes | not evidenced | catch-up |
| 031 | Forecast alerting | yes | yes | yes | yes | not evidenced | catch-up |
| 032 | Forecast quality metric | partial | partial | partial | yes | target only | catch-up |
| 033 | Anomaly detection | yes | yes | yes | yes | runbooks/dashboard | partial |
| 034 | Anomaly state lifecycle | yes | yes | partial | yes | not evidenced | catch-up |
| 035 | Anomaly archive/ignore | yes | partial | partial | yes | not evidenced | catch-up |
| 036 | Anomaly investigation assistant | yes | partial | partial | yes | not evidenced | catch-up |
| 037 | Anomaly routing workflow | yes | yes | yes | yes | not evidenced | catch-up |
| 038 | Commitment discount inventory | partial | yes | yes | yes | partial | catch-up |
| 039 | Savings-plan recommendation | partial | yes | yes | yes | partial | catch-up |
| 040 | Reservation recommendation | partial | yes | yes | yes | runbook-level | partial |
| 041 | Commitment utilization tracking | partial | yes | yes | yes | not evidenced | catch-up |
| 042 | Commitment expiration alert | partial | yes | yes | yes | not evidenced | catch-up |
| 043 | Rightsizing recommendation summary | yes | yes | yes | yes | docs only | catch-up |
| 044 | EC2 rightsizing | partial | yes | yes | yes | not evidenced | catch-up |
| 045 | EBS rightsizing | partial | yes | yes | yes | not evidenced | catch-up |
| 046 | Azure VM rightsizing | partial | yes | yes | yes | not evidenced | catch-up |
| 047 | Azure SQL rightsizing | no | partial | yes | yes | not evidenced | catch-up |
| 048 | Google Compute Engine rightsizing | partial | partial | yes | yes | not evidenced | catch-up |
| 049 | vSphere rightsizing | no | no | yes | yes | not evidenced | catch-up |
| 050 | Kubernetes cluster cost | yes | yes | partial | yes | implied only | catch-up |
| 051 | Kubernetes namespace cost | yes | yes | partial | yes | implied only | catch-up |
| 052 | Kubernetes pod cost | yes | yes | partial | yes | implied only | catch-up |
| 053 | Kubernetes label cost | partial | yes | partial | yes | not evidenced | catch-up |
| 054 | Kubernetes GPU cost | yes | partial | partial | yes | not evidenced | catch-up |
| 055 | Kubernetes PVC/storage cost | partial | yes | partial | yes | not evidenced | catch-up |
| 056 | Kubernetes efficiency score | yes | partial | partial | yes | not evidenced | catch-up |
| 057 | Kubernetes container rightsizing | yes | yes | partial | yes | not evidenced | catch-up |
| 058 | Downloadable recommendations | partial | yes | yes | yes | not evidenced | catch-up |
| 059 | Business mapping | partial | yes | yes | yes | not evidenced | catch-up |
| 060 | Organizational perspectives/views | partial | yes | yes | yes | not evidenced | catch-up |
| 061 | Account hierarchy | yes | yes | yes | yes | partial tenant model | catch-up |
| 062 | Cost owner workflow | partial | yes | yes | yes | not evidenced | catch-up |
| 063 | Policy exception workflow | no | yes | yes | yes | not evidenced | catch-up |
| 064 | Governance policy reporting | partial | yes | yes | yes | Cedar files only | partial |
| 065 | Automated policy action | no | partial | yes | yes | not evidenced | catch-up |
| 066 | Tag quality scoring | partial | yes | yes | yes | not evidenced | catch-up |
| 067 | Tag coverage dashboard | partial | yes | partial | yes | not evidenced | catch-up |
| 068 | FinOps scorecards | no | yes | partial | yes | not evidenced | catch-up |
| 069 | Unit economics | no | yes | partial | yes | not evidenced | catch-up |
| 070 | Sustainability/carbon reporting | no | yes | partial | yes | not evidenced | catch-up |
| 071 | Workload planning | no | yes | partial | yes | capacity model only | catch-up |
| 072 | Cost trend explorer | yes | yes | yes | yes | not evidenced | catch-up |
| 073 | True cost explorer style analysis | partial | yes | partial | yes | not evidenced | catch-up |
| 074 | Public API for reports | yes | partial | yes | yes | not evidenced | catch-up |
| 075 | Public API for budgets | yes | partial | yes | yes | not evidenced | catch-up |
| 076 | Public API for anomalies | partial | partial | yes | yes | not evidenced | catch-up |
| 077 | Public API for recommendations | partial | partial | yes | yes | not evidenced | catch-up |
| 078 | Public API for dashboards | yes | partial | partial | yes | not evidenced | catch-up |
| 079 | Public API for folders/projects | yes | partial | partial | yes | not evidenced | catch-up |
| 080 | Resource inventory endpoint | yes | partial | yes | yes | not evidenced | catch-up |
| 081 | RBAC | yes | yes | yes | yes | Cedar policies exist | partial |
| 082 | Report-level ACL | yes | yes | yes | yes | not evidenced | catch-up |
| 083 | Tenant isolation | partial | yes | yes | yes | architecture yes | partial |
| 084 | Audit chain | partial | partial | partial | yes | yes via dependency and compliance docs | ahead |
| 085 | Evidence export | partial | partial | partial | yes | yes, but path proof absent | partial |
| 086 | Regulator quarterly emission | no | no | partial | yes | local runbook | additive |
| 087 | Credit reconciliation | partial | partial | partial | yes | local runbook | additive |
| 088 | Credit ledger reporting | partial | partial | partial | yes | local artifacts | additive |
| 089 | Provider bill ingestion | yes | yes | yes | yes | explicitly delegated by `PRD.md:59` | external dependency |
| 090 | Provider invoice normalization | yes | yes | yes | yes | output yes, ingestion no | partial |
| 091 | SaaS marketplace integration | yes | partial | partial | yes | not evidenced | catch-up |
| 092 | Cloud account onboarding | yes | yes | yes | yes | delegated/not evidenced | catch-up |
| 093 | Data freshness SLO | yes | yes | yes | yes | SLO file exists | partial |
| 094 | Query latency SLO | yes | yes | yes | yes | SLO file exists | partial |
| 095 | Export latency SLO | yes | partial | yes | yes | SLO file exists | partial |
| 096 | Measured benchmark harness | partial | partial | partial | yes | absent despite benchmark prose | catch-up |
| 097 | Benchmark result artifacts | partial | partial | partial | yes | absent | catch-up |
| 098 | demo_trial OCI Always Free | no | no | no | yes locally | missing | P1 local doctrine gap |
| 099 | OpenTofu context modules | no | no | no | yes locally | missing | P1 local doctrine gap |
| 100 | Terraform-free substrate | not applicable | not applicable | not applicable | yes locally | no: `iac/terraform-module.tf:1-40` | P1 local doctrine gap |
| 101 | Rust backend build proof | not applicable | not applicable | not applicable | yes locally | no source | P1 local doctrine gap |
| 102 | tenant_class-1 OS package matrix | not applicable | not applicable | not applicable | yes locally | missing | P1 local doctrine gap |
| 103 | Native Swift frontend | no | no | no | allowed locally | absent | platform gap |
| 104 | Native Kotlin frontend | no | no | no | allowed locally | absent | platform gap |
| 105 | WinUI3 frontend | no | no | no | allowed locally | absent | platform gap |
| 106 | Leptos Rust web frontend | partial | partial | partial | allowed locally | absent | platform gap |
| 107 | Data retention policy | yes | yes | yes | yes | capacity docs | partial |
| 108 | Cold-start onboarding guide | yes | yes | yes | yes | onboarding exists | partial due broken references |
| 109 | Incident runbooks | partial | partial | partial | yes | strong local runbook suite | ahead |
| 110 | Failure-mode catalog | partial | partial | partial | yes | yes: `failure-modes.md` | ahead |
| 111 | DPIA/privacy analysis | partial | partial | partial | yes | yes: `dpia.md` | ahead |
| 112 | Compliance mapping | partial | partial | partial | yes | yes: `compliance.md` | ahead |
| 113 | Cedar authorization policy | no | no | no | yes locally | yes | additive |
| 114 | Sigstore module signing | not applicable | not applicable | not applicable | yes locally | prose only | catch-up |
| 115 | State backend per context | not applicable | not applicable | not applicable | yes locally | missing | P1 local doctrine gap |
| 116 | Hand-edited state prohibition | not applicable | not applicable | not applicable | yes locally | not evidenced | catch-up |
| 117 | No SSH provisioners | not applicable | not applicable | not applicable | yes locally | no SSH provisioners found | aligned |
| 118 | No null_resource | not applicable | not applicable | not applicable | yes locally | no null_resource found | aligned except Terraform file |
| 119 | No local-exec | not applicable | not applicable | not applicable | yes locally | no local-exec found | aligned except Terraform file |
| 120 | Provider-agnostic business logic | yes | yes | yes | yes | docs yes; code absent | partial |
| 121 | On-prem cost source support | no | partial | yes | yes locally | not evidenced | catch-up |
| 122 | Colo cost source support | no | partial | partial | yes locally | not evidenced | catch-up |
| 123 | Oyatie-as-cloud-provider reporting | no | no | no | yes locally | not evidenced | local doctrine gap |
| 124 | AWS guest reporting | yes | yes | yes | yes locally | no context IaC | local doctrine gap |
| 125 | OCI guest reporting | partial | partial | partial | yes locally | no context IaC | local doctrine gap |
| 126 | Public-cloud Oyatie reporting | no | no | no | yes locally | no context IaC | local doctrine gap |
| 127 | Cost anomaly drilldown dashboard | yes | yes | yes | yes | local dashboard exists | partial |
| 128 | Fleet rollup dashboard | partial | yes | yes | yes | local dashboard exists | partial |
| 129 | Tenant drilldown dashboard | yes | yes | yes | yes | local dashboard exists | partial |
| 130 | Rightsizing dashboard | yes | yes | yes | yes | local dashboard exists | partial |
| 131 | Budget alerts dashboard | yes | yes | yes | yes | local dashboard exists | partial |
| 132 | Cost allocation rollback | partial | partial | partial | yes | local runbook exists | additive |
| 133 | Tenant mismatch investigation | partial | partial | partial | yes | local runbook exists | additive |
| 134 | Reservation engine stall runbook | partial | partial | partial | yes | local runbook exists | additive |
| 135 | Commitment planner automation | partial | yes | yes | yes | partial only | catch-up |
| 136 | Financial approval workflow | no | partial | partial | yes | not evidenced | catch-up |
| 137 | Executive monthly report generation | yes | yes | yes | yes | not evidenced as API/export | catch-up |
| 138 | Cross-service handoff ledger | not applicable | not applicable | not applicable | yes locally | missing | P2 local coherence gap |
| 139 | Chat-derived counterpart alignment | not applicable | not applicable | not applicable | yes for this audit | PRD stale | P3 documentation gap |
| 140 | Intern-build complete implementation path | partial | partial | partial | yes locally | incomplete due no source/tests/IaC | P1 buildability gap |

## §5 Capability families summary table

| Family | UNION-required count | Oyatie present | Oyatie partial | Oyatie missing | Main citation |
| --- | ---: | ---: | ---: | ---: | --- |
| Cost reports and analytics | 18 | 3 | 5 | 10 | `PRD.md:35-39`; `manifest.json:15-24` |
| Export and interoperability | 8 | 3 | 2 | 3 | `contracts/tenant-invoice-public.openapi.yaml:169-205` |
| Allocation, showback, and chargeback | 12 | 4 | 5 | 3 | `PRD.md:39-42`; `ADR-0330 and ADR-0331 tenant_class model:15-30` |
| Budgeting and alerting | 14 | 4 | 4 | 6 | `runbooks/tenant-budget-headroom-low.md`; `runbooks/tenant-budget-exhausted.md` |
| Forecasting and anomaly management | 16 | 2 | 5 | 9 | `ARCHITECTURE.md:640-642`; `dashboards/anomaly-investigation.json` |
| Commitment and rightsizing | 18 | 1 | 6 | 11 | `PRD.md:45-50`; `runbooks/reservation-recommendation-engine-stall.md` |
| Kubernetes and container cost | 10 | 0 | 4 | 6 | `ARCHITECTURE.md:200-204` |
| Governance, RBAC, and policy | 14 | 4 | 5 | 5 | `policy/cedar/tenant-cost-access.cedar`; `compliance.md` |
| APIs and automation | 18 | 2 | 3 | 13 | `contracts/tenant-invoice-public.openapi.yaml`; `contracts/focus-export-internal.asyncapi.yaml` |
| Deployment and platform doctrine | 16 | 2 | 3 | 11 | `specs/master-plan-sequencing.json:704-868`; `iac/terraform-module.tf:1-40` |
| Operational evidence | 12 | 7 | 3 | 2 | `failure-modes.md`; `incident-playbook.md`; `runbooks/*` |
| Regulatory and audit evidence | 8 | 6 | 2 | 0 | `dpia.md`; `compliance.md`; `runbooks/quarterly-regulator-emit-miss.md` |

The headline count is 164 evaluated family-level capability obligations.
The local service clearly evidences 38 of those obligations.
The local service partially evidences 47 obligations.
The local service lacks direct evidence for 79 obligations.
The largest product gap is not cost-domain imagination; it is executable public surfaces for reports, budgets, anomalies, recommendations, and dashboard lifecycle.
The largest doctrine gap is the absence of six OpenTofu context modules and OS support matrix evidence.
The most differentiated local strengths are FOCUS export, audit-chain posture, Cedar policy, DPIA/compliance documents, and incident runbooks.
The most dangerous overclaim is the README full-pack-ready statement at `README.md:6-7` because the local evidence cannot support it.
The counterpart union demands a user-manageable FinOps operating surface, not just reports and runbooks.
The current microservice has the outlines of that surface but not enough implementable contracts.

## §6 Headline gap analysis - top 15 missing capabilities

Gap 01: Custom report query language.
Evidence: `manifest.json:15-24` names report-query as a bounded context, while the local OpenAPI only exposes invoices and FOCUS export.
Counterpart pressure: Vantage cost reports and Cloudability explorer-style surfaces make arbitrary filtering and grouping a baseline.
Implementation hook: add a report query contract that supports dimensions, measures, filters, groupings, comparison windows, and pagination.

Gap 02: Budget hierarchy and budget import.
Evidence: `PRD.md:43-44` names budget alerts, but no budget CRUD/import contract exists in `contracts/`.
Counterpart pressure: Vantage and Cloudability both emphasize budget creation, hierarchy, imports, and recurring alerts.
Implementation hook: add budget aggregate resources, parent-child budget constraints, CSV import validation, and alert destination binding.

Gap 03: Anomaly lifecycle.
Evidence: `dashboards/anomaly-investigation.json` and `runbooks/tenant-cost-anomaly-spike.md` show operation, but no API defines anomaly states.
Counterpart pressure: Vantage and CloudHealth expose anomaly investigation and lifecycle controls.
Implementation hook: define anomaly states such as open, acknowledged, suppressed, archived, escalated, and resolved.

Gap 04: Collaboration and alert routing.
Evidence: runbooks mention operator response, but no destination registry exists.
Counterpart pressure: Vantage routes anomalies to collaboration systems; enterprise tools expect team routing.
Implementation hook: model destinations as tenant-scoped resources with audit-chain writes and Cedar authorization.

Gap 05: Business mapping and perspectives.
Evidence: tenant and allocation models exist, but there is no business view hierarchy.
Counterpart pressure: Cloudability Views and CloudHealth Perspectives make business mapping a core enterprise feature.
Implementation hook: add business unit, owner, cost center, environment, and product-line dimensions with effective-date history.

Gap 06: Tag governance and scorecards.
Evidence: cost allocation docs do not expose tag quality metrics.
Counterpart pressure: Cloudability scorecards and governance surfaces frame cost hygiene as a managed program.
Implementation hook: emit scorecards for untagged spend, conflicting tags, stale owner mappings, and policy exceptions.

Gap 07: Sustainability and carbon reporting.
Evidence: no artifact in the inventory covers carbon, energy, or sustainability cost allocation.
Counterpart pressure: Cloudability advertises sustainability alongside cost governance.
Implementation hook: add optional carbon intensity dimensions and mark unsupported providers explicitly.

Gap 08: Unit economics.
Evidence: capacity documents provide scale assumptions, not business unit-cost metrics.
Counterpart pressure: Cloudability positions unit economics as a FinOps outcome.
Implementation hook: let tenants attach denominators such as requests, seats, builds, inference tokens, or jobs.

Gap 09: Kubernetes deep allocation.
Evidence: `ARCHITECTURE.md:200-204` names OpenCost and Mimir, but local contracts do not expose cluster, namespace, pod, label, GPU, or PVC cost views.
Counterpart pressure: Vantage and Cloudability both emphasize Kubernetes allocation and container optimization.
Implementation hook: add Kubernetes dimensions to report-query and separate rightsizing recommendation resources.

Gap 10: Provider-specific rightsizing.
Evidence: `PRD.md:49-50` names rightsizing recommendations, but the artifacts do not enumerate EC2, EBS, Azure VM, Azure SQL, GCE, vSphere, or OCI shapes.
Counterpart pressure: CloudHealth rightsizing breadth is resource-type specific.
Implementation hook: keep provider-specific collectors outside business logic, but expose normalized recommendation types through FOCUS-compatible dimensions.

Gap 11: Commitment utilization lifecycle.
Evidence: commitment recommendations are mentioned in `PRD.md:45-46`, but utilization, expiration, and coverage contracts are absent.
Counterpart pressure: Cloudability and CloudHealth include reservation and commitment management.
Implementation hook: define commitment inventory, utilization time series, expiration alerts, and purchase recommendation justifications.

Gap 12: Public API breadth.
Evidence: current OpenAPI exposes invoices and export, not reports, budgets, dashboards, anomalies, recommendations, scorecards, or policy exceptions.
Counterpart pressure: Vantage and CloudHealth expose API-driven operation.
Implementation hook: expand contracts in slices so every dashboard/runbook-owned domain has a stable API.

Gap 13: Six-context IaC.
Evidence: only `iac/helm/` and `iac/terraform-module.tf` exist; required `iac/<context>/` modules are missing.
Counterpart pressure: CloudHealth emphasizes hybrid and multi-cloud; ADR-0328 requires all six contexts.
Implementation hook: replace generic Terraform module with context-specific OpenTofu modules using signed module inputs and state backend declarations.

Gap 14: OS support and package matrix.
Evidence: there is no `supported-oses.json` and no tenant_class-1 package matrix.
Counterpart pressure: local Oyatie doctrine requires this even if SaaS competitors do not.
Implementation hook: add service-local OS support manifest, CI lane map, package artifacts, and out-of-scope declarations.

Gap 15: Executable implementation proof.
Evidence: no `src/`, no `tests/`, no Rust source, no `Cargo.toml`, and benchmark docs reference absent harnesses.
Counterpart pressure: union parity cannot be credited from prose alone.
Implementation hook: land Rust service skeleton, contract tests, OpenCost/Mimir adapters, Cedar enforcement tests, and benchmark harnesses.

## §7 Additive surface - Oyatie capabilities not clearly present in the counterpart union

Additive capability 01: Explicit FOCUS export as a first-class public API, evidenced by `contracts/tenant-invoice-public.openapi.yaml:169-205`.
Additive capability 02: Internal FOCUS export event stream, evidenced by `contracts/focus-export-internal.asyncapi.yaml:20-49`.
Additive capability 03: Audit-chain dependency as a product invariant, evidenced by `manifest.json:69-74`.
Additive capability 04: Cedar authorization policy corpus under `policy/cedar/`.
Additive capability 05: DPIA document for the microservice, evidenced by `dpia.md`.
Additive capability 06: Compliance mapping document, evidenced by `compliance.md`.
Additive capability 07: Quarterly regulator emission runbook, evidenced by `runbooks/quarterly-regulator-emit-miss.md`.
Additive capability 08: Credit application reconciliation runbook, evidenced by `runbooks/credit-application-reconciliation.md`.
Additive capability 09: Cost allocation policy rollback runbook, evidenced by `runbooks/cost-allocation-policy-rollback.md`.
Additive capability 10: Tenant bill mismatch response runbook, evidenced by `runbooks/tenant-bill-mismatch-resolution.md`.
Additive capability 11: PQC certificate manifest, evidenced by `iac/pqc-cert.yaml`.
Additive capability 12: Explicit six-context local deployment doctrine, evidenced by `specs/master-plan-sequencing.json:704-746`, although local implementation is missing.
Additive capability 13: demo_trial OCI Always Free doctrine, evidenced by `specs/master-plan-sequencing.json:857-868`, although local implementation is missing.
Additive capability 14: Rust-only backend doctrine, evidenced by `specs/master-plan-sequencing.json:817-856`, although local implementation is missing.
Additive capability 15: OS package and support matrix doctrine, evidenced by `specs/master-plan-sequencing.json:777-816`, although local implementation is missing.
Additive capability 16: Explicit anti-script documentation quality doctrine, evidenced by `docs/standards/brief-template.md:1782-1806`.
Additive capability 17: Microservice-level ownership coherence audit requirement, evidenced by `feedback_microservice_ownership_coherence_2026_05_20.md:18-83`.
Additive capability 18: Structured Wave-level escalation of unresolved cross-service ownership questions.
Additive capability 19: Provider-agnostic enforcement through OpenTofu-only substrate rather than provider SDK business logic.
Additive capability 20: Capability tenant_class linkage to local deployment economics rather than only SaaS commercial packaging.
Additive capability 21: Regulator-oriented evidence capture as a first-class operational story.
Additive capability 22: Tenant-specific budget headroom runbook with operational response.
Additive capability 23: Reservation engine stall handling as an explicit FinOps incident.
Additive capability 24: Financial incident playbook integrating reliability and billing correctness.
Additive capability 25: Cost budget document tying service operation to its own cost envelope.

The additive surface is valuable but not sufficient for parity.
The union bar requires adding executable report, budget, anomaly, rightsizing, and governance APIs.
The local doctrine bar requires replacing the Terraform file, adding context IaC, adding OS manifests, and creating Rust build/test evidence.
This feature matrix therefore classifies finops-portal as product-rich but implementation-incomplete.
