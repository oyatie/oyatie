# finops-portal capability availability deltas vs counterparts - 2026-05-20

Anchor 1: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3666-3789` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854-4228` define OCI Always Free, six deployment contexts, OpenTofu-only IaC, OS support, Rust-only source, and benchmark disclosure requirements.
Anchor 2: `specs/master-plan-sequencing.json:704-868` defines the machine-readable deployment, infrastructure, OS, language, and OCI Always Free constraints.
Anchor 3: `microservices/finops-portal/PRD.md:35-60` defines the in-scope cost attribution, showback, chargeback, budgets, commitments, forecasting, rightsizing, and FOCUS export product surface.
Anchor 4: `microservices/finops-portal/ADR-0330 and ADR-0331 tenant_class model:15-36` defines the current local demo_trial tenant_class, but that tenant_class conflicts with OCI Always Free because it assumes multi-node 4 vCPU/16 GiB nodes and a 500 GiB Postgres footprint.
Anchor 5: `microservices/finops-portal/ARCHITECTURE.md:200-204`, `microservices/finops-portal/ARCHITECTURE.md:640-642`, and `microservices/finops-portal/ARCHITECTURE.md:915-929` define the OpenCost/Mimir/FOCUS/intelligence architecture and the current latency and portability claims.

External source A: Vantage cost reports documentation, `https://docs.vantage.sh/cost_reports`.
External source B: Vantage budgets documentation, `https://docs.vantage.sh/budgets/`.
External source C: Vantage Kubernetes documentation, `https://docs.vantage.sh/kubernetes/`.
External source D: IBM Cloudability budgets and forecasts documentation, `https://www.apptio.com/products/cloudability/budgets-forecasts/`.
External source E: IBM Cloudability Advanced Containers documentation, `https://www.ibm.com/docs/en/cloudability-commercial/cloudability-standard/saas?topic=allocation-cloudability-advanced-containers`.
External source F: IBM Cloudability Kubernetes rightsizing documentation, `https://www.ibm.com/docs/en/cloudability-commercial/cloudability-essentials/saas?topic=optimize-rightsizing-kubernetes-containers`.
External source G: VMware Tanzu CloudHealth financial management overview, `https://www.vmware.com/docs/solution-overview-vmware-tanzu-cloudhealth-simplify-cloud-financial-management`.
External source H: VMware Tanzu CloudHealth rightsizing overview, `https://www.vmware.com/docs/solution-overview-rightsize-cloud-resources-your-way-with-vmware-tanzu-cloudhealth`.

## §1 tenant_class definitions in Oyatie

Oyatie demo_trial definition 01: demo_trial is the smallest supported tenant-facing capability availability, not an excuse to skip doctrine.
Oyatie demo_trial definition 02: demo_trial must split into standard demo_trial and demo_trial OCI Always Free because `specs/master-plan-sequencing.json:857-868` makes OCI Always Free a named sub-profile.
Oyatie demo_trial definition 03: demo_trial must support tenant invoice retrieval, basic cost report summary, budget headroom alerts, and FOCUS export.
Oyatie demo_trial definition 04: demo_trial must not promise full enterprise report builder breadth until report-query contracts exist.
Oyatie demo_trial definition 05: demo_trial must use OpenTofu context modules in every supported context.
Oyatie demo_trial definition 06: demo_trial must use reduced export size, reduced concurrency, and slower freshness targets under OCI Always Free.
Oyatie demo_trial definition 07: demo_trial should expose a minimal tenant dashboard, invoice endpoint, FOCUS export job endpoint, budget alert status, and audit evidence link.
Oyatie demo_trial definition 08: demo_trial should exclude advanced anomaly collaboration, automated remediation, sustainability reporting, and large-scale recommendation automation.
Oyatie demo_trial definition 09: demo_trial should include explicit upgrade boundaries for report builder, multi-level business mapping, high-cardinality Kubernetes dimensions, and multi-year retention.
Oyatie demo_trial definition 10: current local `ADR-0330 and ADR-0331 tenant_class model:15-36` does not meet this definition because it uses resource assumptions larger than OCI Always Free.

Oyatie paid with per_seat billing_component definition 01: paid with per_seat billing_component is the paid baseline where counterpart parity begins for mainstream FinOps operation.
Oyatie paid with per_seat billing_component definition 02: paid with per_seat billing_component should support saved reports, budget CRUD, forecast charts, anomaly records, commitment summaries, and rightsizing summary.
Oyatie paid with per_seat billing_component definition 03: paid with per_seat billing_component should support all six deployment contexts with provider-neutral OpenTofu modules.
Oyatie paid with per_seat billing_component definition 04: paid with per_seat billing_component should support paid OCI resources rather than the Always Free constraints.
Oyatie paid with per_seat billing_component definition 05: paid with per_seat billing_component should support moderate tenant and account counts with bounded API throughput.
Oyatie paid with per_seat billing_component definition 06: paid with per_seat billing_component should add business owner metadata and basic cost center mapping.
Oyatie paid with per_seat billing_component definition 07: paid with per_seat billing_component should include dashboard lifecycle and alert destination management.
Oyatie paid with per_seat billing_component definition 08: paid with per_seat billing_component should include measured benchmark evidence before any public parity claim.
Oyatie paid with per_seat billing_component definition 09: paid with per_seat billing_component should expose Cedar-protected APIs for invoice, FOCUS export, reports, budgets, and anomalies.
Oyatie paid with per_seat billing_component definition 10: paid with per_seat billing_component should make OS support manifests and package formats visible.

Oyatie paid with per_usage billing_component definition 01: paid with per_usage billing_component is the production-scale enterprise tenant_class.
Oyatie paid with per_usage billing_component definition 02: paid with per_usage billing_component should support large account fleets, high row counts, lower p95 latency, stronger availability, and multi-team governance.
Oyatie paid with per_usage billing_component definition 03: paid with per_usage billing_component should support business mapping, tag quality scorecards, anomaly lifecycle, recommendation APIs, and commitment utilization.
Oyatie paid with per_usage billing_component definition 04: paid with per_usage billing_component should support Kubernetes deep allocation by cluster, namespace, pod, label, PVC, and GPU when upstream data exists.
Oyatie paid with per_usage billing_component definition 05: paid with per_usage billing_component should support downloadable recommendation and allocation reports.
Oyatie paid with per_usage billing_component definition 06: paid with per_usage billing_component should support exception workflows and policy violation reporting.
Oyatie paid with per_usage billing_component definition 07: paid with per_usage billing_component should support cost owner workflows and approval states.
Oyatie paid with per_usage billing_component definition 08: paid with per_usage billing_component should support service-local benchmark harness and raw result retention.
Oyatie paid with per_usage billing_component definition 09: paid with per_usage billing_component should support a complete tenant_class-1 OS and CI package matrix.
Oyatie paid with per_usage billing_component definition 10: paid with per_usage billing_component should be the first tenant_class where counterpart union parity becomes plausible after implementation.

Oyatie paid with compliance_pack gating definition 01: paid with compliance_pack gating is the hyperscaler bar and single-tenant capable tenant_class.
Oyatie paid with compliance_pack gating definition 02: paid with compliance_pack gating should support dedicated control planes, high concurrency, billion-row exports, and very low dashboard latency.
Oyatie paid with compliance_pack gating definition 03: paid with compliance_pack gating should support advanced governance automation, approval workflows, deep provider-specific rightsizing, and high-cardinality analytics.
Oyatie paid with compliance_pack gating definition 04: paid with compliance_pack gating should support vSphere/on-prem/colo cost modeling when the tenant environment exposes inventory and utilization feeds.
Oyatie paid with compliance_pack gating definition 05: paid with compliance_pack gating should support confidence-banded forecasts, anomaly assistant workflows, and recommendation precision feedback loops.
Oyatie paid with compliance_pack gating definition 06: paid with compliance_pack gating should expose audit, regulator, and compliance evidence as first-class exports.
Oyatie paid with compliance_pack gating definition 07: paid with compliance_pack gating should include SLO-backed multi-region or equivalent resilience where the deployment context permits it.
Oyatie paid with compliance_pack gating definition 08: paid with compliance_pack gating should include measured evidence across at least one owned and one guest deployment context before a hyperscaler-maturity claim.
Oyatie paid with compliance_pack gating definition 09: paid with compliance_pack gating should not be claimed from documentation alone because current service lacks Rust source, tests, and context IaC.
Oyatie paid with compliance_pack gating definition 10: paid with compliance_pack gating should carry the strictest provenance requirements for benchmark, security, and availability evidence.

## §2 Counterpart tenant_class mapping

Vantage mapping 01: public audited pages emphasize cost reports, budgets, Kubernetes cost, API access, anomaly alerts, exports, and FinOps agent-style investigation.
Vantage mapping 02: this audit maps Vantage self-serve/basic use to Oyatie demo_trial and paid with per_seat billing_component depending on export and budget depth.
Vantage mapping 03: this audit maps Vantage business/enterprise report and budget operation to Oyatie paid with per_seat billing_component and paid with per_usage billing_component.
Vantage mapping 04: this audit maps Vantage advanced API, Kubernetes, anomaly, and investigation surfaces to Oyatie paid with per_usage billing_component and paid with compliance_pack gating.
Vantage mapping 05: Vantage does not provide a direct analogue for Oyatie demo_trial OCI Always Free in the audited public pages.
Vantage mapping 06: Vantage does not provide a direct analogue for service-local OS packaging and OpenTofu context modules.
Vantage mapping 07: Vantage is strongest in interactive report UX, budget operation, Kubernetes visibility, and API breadth.
Vantage mapping 08: Oyatie is potentially stronger in FOCUS eventing, audit-chain posture, and local compliance evidence once implemented.
Vantage mapping 09: current Oyatie artifacts trail Vantage on report builder, dashboard lifecycle, anomaly workflow, alert routing, and public API breadth.
Vantage mapping 10: current Oyatie artifacts should not claim Vantage tenant_class parity beyond selected FOCUS and invoice surfaces.

Cloudability mapping 01: public audited pages emphasize budgets, forecasts, business mapping, commercial billing, containers, allocation, sustainability, governance, scorecards, unit economics, commitments, and rightsizing.
Cloudability mapping 02: this audit maps Cloudability entry budget/forecast operation to Oyatie paid with per_seat billing_component.
Cloudability mapping 03: this audit maps Cloudability business mapping, scorecards, sustainability, and unit economics to Oyatie paid with per_usage billing_component.
Cloudability mapping 04: this audit maps Cloudability enterprise optimization and governance operation to Oyatie paid with per_usage billing_component and paid with compliance_pack gating.
Cloudability mapping 05: Cloudability's Kubernetes rightsizing documentation uses explicit lookback windows; Oyatie does not yet define equivalent windows.
Cloudability mapping 06: Cloudability Advanced Containers maps to Oyatie paid with per_usage billing_component because it requires deeper Kubernetes allocation than demo_trial/paid with per_seat billing_component.
Cloudability mapping 07: Cloudability does not provide a direct analogue for demo_trial OCI Always Free in the audited public pages.
Cloudability mapping 08: Cloudability is strongest in business mapping, budget/forecast operation, containers, and FinOps governance.
Cloudability mapping 09: Oyatie is potentially stronger in Cedar/audit-chain/regulator evidence once executable surfaces exist.
Cloudability mapping 10: current Oyatie artifacts trail Cloudability on scorecards, sustainability, unit economics, workload planning, and recommendation workflow.

CloudHealth mapping 01: public audited pages emphasize multi-cloud, hybrid, organizational modeling, Perspectives, chargeback/showback, budgets, forecasts, anomaly, governance, automation, API access, and broad rightsizing.
CloudHealth mapping 02: this audit maps CloudHealth chargeback/showback/budget/forecast core to Oyatie paid with per_seat billing_component and paid with per_usage billing_component.
CloudHealth mapping 03: this audit maps CloudHealth governance automation and broad rightsizing to Oyatie paid with compliance_pack gating.
CloudHealth mapping 04: CloudHealth vSphere/data-center rightsizing maps to Oyatie paid with compliance_pack gating because it requires on-prem/colo inventory and utilization feeds.
CloudHealth mapping 05: CloudHealth Perspectives and FlexOrgs map to Oyatie paid with per_usage billing_component business mapping and paid with compliance_pack gating organizational modeling.
CloudHealth mapping 06: CloudHealth automated remediation maps to Oyatie paid with compliance_pack gating only if action ownership is clarified with cloud-iac.
CloudHealth mapping 07: CloudHealth does not provide a direct analogue for demo_trial OCI Always Free in the audited public pages.
CloudHealth mapping 08: CloudHealth is strongest in hybrid coverage, governance automation, and provider/resource-specific rightsizing breadth.
CloudHealth mapping 09: Oyatie is potentially stronger in OpenTofu-only doctrine and FOCUS-first export, but local implementation is absent.
CloudHealth mapping 10: current Oyatie artifacts trail CloudHealth on FlexOrgs/Perspectives, automated actions, and provider-specific recommendation breadth.

## §3 Per-Oyatie-tenant_class delta tables

### demo_trial tenant_class table

| # | Feature | Oyatie-demo_trial | Vantage-equivalent-tenant_class | Cloudability-equivalent-tenant_class | CloudHealth-equivalent-tenant_class | Gap classification |
| --- | --- | --- | --- | --- | --- | --- |
| B01 | Tenant invoice retrieval | included by contract | basic report/invoice adjacent | commercial billing adjacent | chargeback adjacent | parity |
| B02 | FOCUS export | included by contract | export feature | partial/open standard adjacent | export adjacent | ahead if implemented |
| B03 | Basic cost dashboard | dashboard files only | basic reports | dashboards | dashboards | partial |
| B04 | Saved report builder | not evidenced | standard report feature | explorer feature | custom reporting | catch-up |
| B05 | Report filters | not evidenced | standard | standard | standard | catch-up |
| B06 | Report grouping | not evidenced | standard | standard | standard | catch-up |
| B07 | Budget status | runbook-level | budgets | budgets | budgets | partial |
| B08 | Budget CRUD | not evidenced | budgets | budgets | budgets | catch-up |
| B09 | Budget hierarchy | not evidenced | advanced budgets | budgets/views | budget org model | catch-up |
| B10 | Budget CSV import | not evidenced | budget import | partial | partial | catch-up |
| B11 | Budget alert routing | not evidenced | alerts | alerts | alerts | catch-up |
| B12 | Anomaly list | dashboard/runbook only | anomaly alerts | anomaly feature | anomaly feature | partial |
| B13 | Anomaly lifecycle | not evidenced | archive/ignore | governance workflow | investigation workflow | catch-up |
| B14 | Forecast trend | docs only | report forecast | forecasts | forecasts | partial |
| B15 | Forecast confidence | not evidenced | partial | forecast analytics | partial | catch-up |
| B16 | Commitment summary | docs only | partial | commitments | commitments | partial |
| B17 | Reservation recommendation | runbook-level | partial | commitments | commitments | partial |
| B18 | Rightsizing summary | docs only | Kubernetes/rightsize | rightsizing | rightsizing | partial |
| B19 | Kubernetes namespace cost | implied only | Kubernetes reports | containers | partial | catch-up |
| B20 | Kubernetes pod cost | implied only | Kubernetes reports | containers | partial | catch-up |
| B21 | Kubernetes GPU cost | not evidenced | Kubernetes GPU | partial | partial | catch-up |
| B22 | Business mapping | not evidenced | cost categories | business mapping | Perspectives | catch-up |
| B23 | Cost center mapping | not evidenced | cost categories | business mapping | Perspectives | catch-up |
| B24 | Showback | PRD-included | partial | showback | showback | partial |
| B25 | Chargeback | PRD-included | partial | chargeback | chargeback | partial |
| B26 | Credit ledger | local additive | partial | partial | partial | ahead/additive |
| B27 | Regulator evidence | local additive | limited public evidence | limited public evidence | limited public evidence | ahead/additive |
| B28 | Cedar RBAC | policy files | RBAC equivalent | RBAC equivalent | RBAC equivalent | partial |
| B29 | Audit-chain evidence | dependency | audit adjacent | audit adjacent | audit adjacent | ahead if implemented |
| B30 | Public report API | missing | API | partial | open API | catch-up |
| B31 | Public budget API | missing | API/budgets | partial | open API | catch-up |
| B32 | Public anomaly API | missing | partial | partial | open API | catch-up |
| B33 | Dashboard lifecycle API | missing | API dashboards | partial | dashboards | catch-up |
| B34 | Alert destinations | missing | email/Slack/Teams/Jira | alerts | alerts | catch-up |
| B35 | Tag quality score | excluded | partial | scorecards | governance | demo_trial defer |
| B36 | Sustainability | excluded | no headline | sustainability | partial | demo_trial defer |
| B37 | Unit economics | excluded | no headline | unit economics | partial | demo_trial defer |
| B38 | Governance automation | excluded | partial | governance | automation | demo_trial defer |
| B39 | Automated remediation | excluded | no headline | partial | automation | demo_trial defer |
| B40 | vSphere rightsizing | excluded | no headline | no headline | rightsizing | demo_trial defer |
| B41 | OCI Always Free profile | required locally | no analogue | no analogue | no analogue | local missing |
| B42 | Six-context IaC | required locally | no analogue | no analogue | hybrid adjacent | local missing |
| B43 | OpenTofu-only modules | required locally | no analogue | no analogue | no analogue | local missing |
| B44 | tenant_class-1 OS package matrix | required locally | no analogue | no analogue | no analogue | local missing |
| B45 | Rust build proof | required locally | no analogue | no analogue | no analogue | local missing |
| B46 | Benchmark harness | missing | not public | not public | not public | local missing |
| B47 | p95 dashboard target | target only | estimated 2.5s | estimated 3s | estimated 3s | unmeasured |
| B48 | FOCUS row cap | 250k on OCI AF | export feature | export adjacent | export adjacent | constrained |
| B49 | Concurrent users | 8-35 target by context | self-serve | small team | small team | unmeasured |
| B50 | Retention | constrained | standard | standard | standard | constrained |
| B51 | Tenant count | constrained | self-serve | small team | small team | constrained |
| B52 | Cost allocation completeness | target only | allocation | allocation | allocation | unmeasured |
| B53 | Export retry behavior | not evidenced | standard | standard | standard | catch-up |
| B54 | Evidence export | docs only | partial | partial | partial | partial |
| B55 | Incident runbooks | strong | limited public | limited public | limited public | ahead |
| B56 | Onboarding guide | present with broken refs | public docs | public docs | public docs | partial |
| B57 | DPIA | present | not headline | not headline | not headline | ahead |
| B58 | Compliance mapping | present | enterprise adjacent | enterprise adjacent | enterprise adjacent | partial/ahead |
| B59 | PQC cert manifest | present | no public analogue | no public analogue | no public analogue | additive |
| B60 | demo_trial conclusion | overclaimed today | better UX | better budget features | better hybrid breadth | catch-up until code/IaC land |

### paid with per_seat billing_component tenant_class table

| # | Feature | Oyatie-paid with per_seat billing_component | Vantage-equivalent-tenant_class | Cloudability-equivalent-tenant_class | CloudHealth-equivalent-tenant_class | Gap classification |
| --- | --- | --- | --- | --- | --- | --- |
| S01 | Saved report builder | should be included | standard | explorer | custom reports | missing today |
| S02 | Report filters | should be included | standard | standard | standard | missing today |
| S03 | Report grouping | should be included | standard | standard | standard | missing today |
| S04 | Report comparison windows | should be included | standard | standard | standard | missing today |
| S05 | Report CSV export | should be included | standard | standard | standard | missing today |
| S06 | Report PDF export | optional paid with per_seat billing_component | standard | partial | standard | missing today |
| S07 | FOCUS export | included | standard | partial | export | parity if implemented |
| S08 | Dashboard lifecycle | should be included | dashboards/API | dashboards | dashboards | missing today |
| S09 | Budget CRUD | should be included | budgets | budgets | budgets | missing today |
| S10 | Budget hierarchy | should be included | hierarchical budgets | business mapping budget | org budgets | missing today |
| S11 | Budget import | should be included | CSV import | partial | partial | missing today |
| S12 | Alert destinations | should be included | collaboration alerts | alerts | alerts | missing today |
| S13 | Anomaly records | should be included | anomaly alerts | anomaly | anomaly | partial today |
| S14 | Anomaly lifecycle | should be included | archive/ignore | workflow | workflow | missing today |
| S15 | Forecast charts | should be included | forecast | forecasts | forecasts | docs only today |
| S16 | Forecast quality metric | should be included | partial | forecast analytics | partial | missing today |
| S17 | Commitment inventory | should be included | partial | commitments | commitments | missing today |
| S18 | Commitment recommendation | should be included | partial | commitments | commitments | docs only today |
| S19 | Rightsizing summary | should be included | rightsizing | rightsizing | rightsizing | docs only today |
| S20 | Kubernetes cluster cost | should be included | Kubernetes | containers | partial | missing today |
| S21 | Kubernetes namespace cost | should be included | Kubernetes | containers | partial | missing today |
| S22 | Kubernetes pod cost | should be included | Kubernetes | containers | partial | missing today |
| S23 | Business owner metadata | should be included | cost categories | business mapping | Perspectives | missing today |
| S24 | Cost center mapping | should be included | cost categories | business mapping | Perspectives | missing today |
| S25 | Cost sharing | should be included | allocation | cost sharing | allocation | partial today |
| S26 | Showback | included | partial | showback | showback | parity if implemented |
| S27 | Chargeback | included | partial | chargeback | chargeback | parity if implemented |
| S28 | Credit reconciliation | included | partial | partial | partial | additive |
| S29 | Regulator evidence | included | limited public | limited public | limited public | additive |
| S30 | Cedar RBAC | included | RBAC | RBAC | RBAC | partial today |
| S31 | Report API | should be included | API | partial | open API | missing today |
| S32 | Budget API | should be included | API | partial | open API | missing today |
| S33 | Anomaly API | should be included | partial | partial | open API | missing today |
| S34 | Recommendation API | should be included | partial | partial | open API | missing today |
| S35 | Invoice API | included | adjacent | billing | chargeback | present |
| S36 | FOCUS event API | included | partial | partial | partial | ahead |
| S37 | Tag coverage dashboard | should be included | partial | scorecards | governance | missing today |
| S38 | Tag policy exception | should be included | partial | governance | governance | missing today |
| S39 | Sustainability | optional paid with per_seat billing_component | no headline | sustainability | partial | missing today |
| S40 | Unit economics | optional paid with per_seat billing_component | no headline | unit economics | partial | missing today |
| S41 | OpenTofu six contexts | required | no analogue | no analogue | hybrid adjacent | missing today |
| S42 | OCI paid baseline | required for guest-on-oci paid with per_seat billing_component | no analogue | no analogue | no analogue | missing today |
| S43 | State backend per context | required | no analogue | no analogue | no analogue | missing today |
| S44 | Signed modules | required | no analogue | no analogue | no analogue | prose only today |
| S45 | tenant_class-1 OS manifest | required | no analogue | no analogue | no analogue | missing today |
| S46 | CI lane per OS | required | no analogue | no analogue | no analogue | missing today |
| S47 | Rust cargo build | required | no analogue | no analogue | no analogue | missing today |
| S48 | Contract tests | required | no public evidence | no public evidence | no public evidence | missing today |
| S49 | Load tests | required | no public evidence | no public evidence | no public evidence | missing today |
| S50 | Benchmark results | required | not public | not public | not public | missing today |
| S51 | Availability target | 99.8-99.95 target | enterprise | enterprise | enterprise | unmeasured |
| S52 | Dashboard p95 | 550-950 ms target | estimated 2.5s | estimated 3s | estimated 3s | ahead target |
| S53 | Export rows | 8-15M target | export feature | export feature | export feature | unmeasured |
| S54 | Tenant count | 100-350 target | business | enterprise | enterprise | unmeasured |
| S55 | Alert freshness | 8-20 min target | alerts | alerts | alerts | competitive target |
| S56 | Allocation completeness | 98 percent target | allocation | allocation | allocation | unmeasured |
| S57 | Evidence retention | included | enterprise adjacent | enterprise adjacent | enterprise adjacent | partial |
| S58 | Onboarding | present | docs | docs | docs | partial with broken refs |
| S59 | paid with per_seat billing_component conclusion | should reach mainstream parity | strong UX | strong governance | strong hybrid | cannot claim today |
| S60 | Required next proof | code, IaC, OS, tests | public feature docs | public feature docs | public feature docs | local execution gap |

### paid with per_usage billing_component tenant_class table

| # | Feature | Oyatie-paid with per_usage billing_component | Vantage-equivalent-tenant_class | Cloudability-equivalent-tenant_class | CloudHealth-equivalent-tenant_class | Gap classification |
| --- | --- | --- | --- | --- | --- | --- |
| G01 | Enterprise report builder | required | enterprise reports | explorer | custom reports | missing today |
| G02 | High-cardinality grouping | required | enterprise | enterprise | enterprise | missing today |
| G03 | Dashboard lifecycle API | required | API/dashboard | dashboards | dashboards/API | missing today |
| G04 | Report-level ACL | required | enterprise RBAC | governance | governance | missing today |
| G05 | Business mapping | required | cost categories | business mapping | Perspectives | missing today |
| G06 | Multi-level org hierarchy | required | folders/categories | Views | FlexOrgs | missing today |
| G07 | Cost owner workflow | required | partial | governance | governance | missing today |
| G08 | Shared cost rules | required | allocation | cost sharing | allocation | partial today |
| G09 | Allocation rollback | required | not headline | partial | partial | local runbook ahead |
| G10 | Budget workflow | required | budgets | budgets | budgets | missing today |
| G11 | Budget approvals | required | partial | governance | governance | missing today |
| G12 | Alert route registry | required | destinations | alerts | alerts | missing today |
| G13 | Anomaly lifecycle | required | anomaly workflow | anomaly | anomaly | missing today |
| G14 | Anomaly collaboration | required | collaboration | workflow | workflow | missing today |
| G15 | Anomaly assistant | required | FinOps Agent-style | AI assist | partial | missing today |
| G16 | Forecast confidence | required | partial | forecasting | forecasting | missing today |
| G17 | Forecast model governance | required | partial | AI governance adjacent | partial | missing today |
| G18 | Commitment utilization | required | partial | commitments | commitments | missing today |
| G19 | Commitment expiration | required | partial | commitments | commitments | missing today |
| G20 | Purchase recommendation explainability | required | partial | commitments | optimization | missing today |
| G21 | Rightsizing API | required | rightsizing | rightsizing | rightsizing | missing today |
| G22 | EC2 rightsizing | required | partial | rightsizing | rightsizing | missing today |
| G23 | EBS rightsizing | required | partial | rightsizing | rightsizing | missing today |
| G24 | Azure VM rightsizing | required | partial | rightsizing | rightsizing | missing today |
| G25 | GCE rightsizing | required | partial | partial | rightsizing | missing today |
| G26 | Kubernetes cluster allocation | required | Kubernetes | containers | partial | missing today |
| G27 | Kubernetes namespace allocation | required | Kubernetes | containers | partial | missing today |
| G28 | Kubernetes pod allocation | required | Kubernetes | containers | partial | missing today |
| G29 | Kubernetes label allocation | required | partial | containers | partial | missing today |
| G30 | Kubernetes PVC allocation | required | partial | containers | partial | missing today |
| G31 | Kubernetes GPU allocation | required | Kubernetes GPU | partial | partial | missing today |
| G32 | Kubernetes rightsizing | required | Kubernetes | rightsizing | partial | missing today |
| G33 | Downloadable recommendations | required | partial | downloadable | partial | missing today |
| G34 | Tag quality scorecard | required | partial | scorecards | governance | missing today |
| G35 | FinOps scorecards | required | partial | scorecards | governance | missing today |
| G36 | Sustainability | required or explicitly excluded | no headline | sustainability | partial | missing today |
| G37 | Unit economics | required | no headline | unit economics | partial | missing today |
| G38 | Workload planning | required | no headline | workload planning | partial | capacity only today |
| G39 | Governance policy reporting | required | partial | governance | governance | partial today |
| G40 | Policy exceptions | required | partial | governance | governance | missing today |
| G41 | Automated action proposal | required | partial | governance | automation | missing today |
| G42 | Automated action execution | optional paid with per_usage billing_component | no headline | partial | automation | missing today |
| G43 | Public API breadth | required | API | partial API | open API | missing today |
| G44 | API pagination and cursors | required | API | partial | API | missing today |
| G45 | API idempotency | required | API | partial | API | missing today |
| G46 | Export consistency SLA | required | export | export | export | missing today |
| G47 | Audit evidence export | required | partial | enterprise audit | enterprise audit | partial today |
| G48 | Regulator evidence | included | not headline | not headline | partial | ahead |
| G49 | Six-context OpenTofu | required | no analogue | no analogue | hybrid adjacent | missing today |
| G50 | Full tenant_class-1 OS matrix | required | no analogue | no analogue | no analogue | missing today |
| G51 | tenant_class-2 test-only arch | required | no analogue | no analogue | no analogue | missing today |
| G52 | Rust implementation | required | no analogue | no analogue | no analogue | missing today |
| G53 | Performance harness | required | not public | not public | not public | missing today |
| G54 | Raw benchmark retention | required | not public | not public | not public | missing today |
| G55 | paid with per_usage billing_component dashboard p95 | 350-750 ms target | estimated 2.5s | estimated 3s | estimated 3s | ahead target |
| G56 | paid with per_usage billing_component export rows | 50-150M target | export | export | export | unmeasured |
| G57 | paid with per_usage billing_component availability | 99.7-99.97 target | enterprise | enterprise | enterprise | unmeasured |
| G58 | paid with per_usage billing_component tenant scale | 1k-3k target | enterprise | enterprise | enterprise | unmeasured |
| G59 | paid with per_usage billing_component conclusion | parity target | Vantage strong in UX | Cloudability strong in governance | CloudHealth strong in hybrid | not currently achieved |
| G60 | paid with per_usage billing_component remediation | implement breadth | report APIs | business mapping | hybrid rightsizing | large implementation gap |

### paid with compliance_pack gating tenant_class table

| # | Feature | Oyatie-paid with compliance_pack gating | Vantage-equivalent-tenant_class | Cloudability-equivalent-tenant_class | CloudHealth-equivalent-tenant_class | Gap classification |
| --- | --- | --- | --- | --- | --- | --- |
| P01 | Dedicated control plane | required where selected | enterprise/dedicated adjacent | enterprise | enterprise | missing today |
| P02 | Billion-row export | target | export feature | export feature | export feature | unmeasured |
| P03 | Hyperscaler dashboard latency | target 250-600 ms p95 by context | estimated higher | estimated higher | estimated higher | ahead target |
| P04 | Multi-region resilience | required where context permits | enterprise | enterprise | enterprise | missing today |
| P05 | Tenant-isolated data plane | required | enterprise | enterprise | enterprise | docs only |
| P06 | Advanced report DSL | required | advanced reports | explorer | custom reports | missing today |
| P07 | Report versioning | required | partial | partial | partial | missing today |
| P08 | Report collaboration | required | collaboration | workflow | workflow | missing today |
| P09 | Executive package exports | required | PDF/export | reports | reports | missing today |
| P10 | Board-level cost packs | required | partial | dashboards | dashboards | missing today |
| P11 | Business hierarchy history | required | categories | business mapping | FlexOrgs | missing today |
| P12 | Perspective simulation | required | partial | workload planning | Perspectives | missing today |
| P13 | Chargeback approval flow | required | partial | governance | governance | missing today |
| P14 | Invoice dispute automation | required | partial | partial | chargeback | missing today |
| P15 | Budget scenario planning | required | budgets | forecasts | forecasts | missing today |
| P16 | Forecast confidence and backtest | required | partial | AI forecast | partial | missing today |
| P17 | Forecast model registry | required | no headline | AI governance adjacent | partial | missing today |
| P18 | Anomaly assistant workflow | required | FinOps Agent-style | AI assist | partial | missing today |
| P19 | Anomaly suppression policy | required | archive/ignore | governance | governance | missing today |
| P20 | Cross-tenant anomaly guardrails | required | enterprise | enterprise | enterprise | missing today |
| P21 | Commitment purchase workflow | required | partial | commitments | commitments | missing today |
| P22 | Commitment portfolio simulation | required | partial | commitments | optimization | missing today |
| P23 | Provider-specific rightsizing | required | partial | rightsizing | rightsizing | missing today |
| P24 | vSphere rightsizing | required for on-prem/colo | no headline | no headline | rightsizing | missing today |
| P25 | Azure SQL rightsizing | required | no headline | partial | rightsizing | missing today |
| P26 | OCI shape rightsizing | required locally | partial | partial | partial | missing today |
| P27 | Kubernetes GPU optimization | required where data exists | Kubernetes GPU | containers | partial | missing today |
| P28 | Kubernetes PVC optimization | required where data exists | partial | containers | partial | missing today |
| P29 | Container recommendation feedback | required | partial | downloadable recommendations | partial | missing today |
| P30 | Automated remediation proposal | required | partial | governance | automation | missing today |
| P31 | Automated remediation execution | gated | no headline | partial | automation | ownership unresolved |
| P32 | Policy exception lifecycle | required | partial | governance | governance | missing today |
| P33 | Tag enforcement workflow | required | partial | governance | governance | missing today |
| P34 | Sustainability analytics | required or formal exception | no headline | sustainability | partial | missing today |
| P35 | Unit economics API | required | no headline | unit economics | partial | missing today |
| P36 | Workload planning API | required | no headline | workload planning | partial | missing today |
| P37 | FinOps scorecard API | required | partial | scorecards | governance | missing today |
| P38 | Governance scorecard API | required | partial | scorecards | governance | missing today |
| P39 | Open API coverage | required | API | partial | open API | missing today |
| P40 | Bulk export API | required | API/export | export | export | partial only |
| P41 | Webhook/event delivery | required | partial | partial | automation | missing today |
| P42 | Audit-chain complete path | required | enterprise audit | enterprise audit | enterprise audit | partial today |
| P43 | Regulator package export | required | not headline | enterprise audit | enterprise audit | local additive |
| P44 | DPIA automation linkage | required | not headline | not headline | not headline | docs only |
| P45 | PQC cert lifecycle | required where relevant | no analogue | no analogue | no analogue | docs only |
| P46 | OpenTofu signed modules | required | no analogue | no analogue | no analogue | missing today |
| P47 | State backend isolation | required | no analogue | no analogue | no analogue | missing today |
| P48 | Six context production proof | required | no analogue | no analogue | hybrid | missing today |
| P49 | OCI Always Free regression lane | required | no analogue | no analogue | no analogue | missing today |
| P50 | tenant_class-1 OS CI matrix | required | no analogue | no analogue | no analogue | missing today |
| P51 | ppc64le/s390x test lane | required | no analogue | no analogue | no analogue | missing today |
| P52 | Rust service build | required | no analogue | no analogue | no analogue | missing today |
| P53 | Contract compatibility tests | required | no public evidence | no public evidence | no public evidence | missing today |
| P54 | Security tests | required | no public evidence | no public evidence | no public evidence | missing today |
| P55 | Benchmark harness | required | not public | not public | not public | missing today |
| P56 | paid with compliance_pack gating availability | 99.9-99.995 target | enterprise | enterprise | enterprise | unmeasured |
| P57 | paid with compliance_pack gating concurrency | 1.5k-6k users target | enterprise | enterprise | enterprise | unmeasured |
| P58 | paid with compliance_pack gating tenants | 3k-20k target | enterprise | enterprise | enterprise | unmeasured |
| P59 | paid with compliance_pack gating conclusion | aspirational | strong UX/API | strong governance | strong hybrid optimization | far from claimable |
| P60 | paid with compliance_pack gating remediation | build executable product and evidence | match API breadth | match governance depth | match rightsizing breadth | largest gap |

## §4 OCI Always Free demo_trial = Always Free reconciliation

OCI reconciliation 01: ADR-0328 requires `iac/oci-guest/always-free/` as the service-local profile.
OCI reconciliation 02: current finops-portal inventory contains no `iac/oci-guest/always-free/` directory.
OCI reconciliation 03: current local demo_trial in `ADR-0330 and ADR-0331 tenant_class model:15-36` assumes resource sizes incompatible with an Always Free profile.
OCI reconciliation 04: OCI Always Free demo_trial must therefore be defined as a constrained capability profile, not the current generic demo_trial row.
OCI reconciliation 05: OCI Always Free demo_trial should cap tenants at a small count until measured capacity exists.
OCI reconciliation 06: OCI Always Free demo_trial should cap FOCUS export rows around 250k per job until object storage and compute limits are measured.
OCI reconciliation 07: OCI Always Free demo_trial should cap sustained reads around 35 RPS target until measured.
OCI reconciliation 08: OCI Always Free demo_trial should allow slower anomaly freshness around 60 minutes target.
OCI reconciliation 09: OCI Always Free demo_trial should allow slower forecast recomputation around 45 minutes target.
OCI reconciliation 10: OCI Always Free demo_trial should prioritize invoice retrieval, basic report summary, budget status, and FOCUS export.
OCI reconciliation 11: OCI Always Free demo_trial should defer high-cardinality Kubernetes GPU/PVC analytics.
OCI reconciliation 12: OCI Always Free demo_trial should defer multi-year retention unless external storage is provisioned.
OCI reconciliation 13: OCI Always Free demo_trial should defer large PDF/report packs.
OCI reconciliation 14: OCI Always Free demo_trial should defer provider-specific rightsizing breadth beyond basic recommendation summaries.
OCI reconciliation 15: OCI Always Free demo_trial should defer automated remediation entirely.
OCI reconciliation 16: OCI Always Free demo_trial should use OCI-native state backend only for the OCI profile per ADR-0328 D-19.
OCI reconciliation 17: OCI Always Free demo_trial must not spill into AWS, Terraform Cloud, or local laptop state.
OCI reconciliation 18: OCI Always Free demo_trial must not use Terraform, Pulumi, CloudFormation, SSH provisioners, or local-exec.
OCI reconciliation 19: OCI Always Free demo_trial must include a README explaining capacity tradeoffs and upgrade boundaries.
OCI reconciliation 20: OCI Always Free demo_trial must include `versions.tofu` or equivalent OpenTofu version pinning.
OCI reconciliation 21: OCI Always Free demo_trial must include module outputs for endpoint, tenancy boundary, object storage, state backend, and observability wiring.
OCI reconciliation 22: OCI Always Free demo_trial must include explicit dependency declarations for OpenCost, Mimir, tenancy, and audit-chain.
OCI reconciliation 23: OCI Always Free demo_trial must include benchmark runs on the Always Free profile before any capacity number is marked measured.
OCI reconciliation 24: OCI Always Free demo_trial must include a downgrade path that preserves invoice and FOCUS export correctness while reducing optional intelligence.
OCI reconciliation 25: OCI Always Free demo_trial must include cost-budget guardrails so the Always Free profile does not silently cross into paid resources.
OCI reconciliation 26: OCI Always Free demo_trial must include alerting when capacity pressure would require paid with per_seat billing_component.
OCI reconciliation 27: OCI Always Free demo_trial must include a documented upgrade path to OCI paid with per_seat billing_component paid baseline.
OCI reconciliation 28: OCI Always Free demo_trial must include the same Rust-only and OS manifest discipline as every other tenant_class.
OCI reconciliation 29: OCI Always Free demo_trial cannot inherit the current benchmark targets for generic demo_trial without splitting context-specific rows.
OCI reconciliation 30: OCI Always Free demo_trial is therefore a P1 remediation item for finops-portal.

Features requiring paid with per_seat billing_component on OCI: saved report builder with arbitrary grouping.
Features requiring paid with per_seat billing_component on OCI: dashboard lifecycle APIs for many teams.
Features requiring paid with per_seat billing_component on OCI: budget hierarchy import at large file sizes.
Features requiring paid with per_seat billing_component on OCI: high-cardinality Kubernetes allocation.
Features requiring paid with per_seat billing_component on OCI: GPU and PVC optimization.
Features requiring paid with per_seat billing_component on OCI: large PDF export packs.
Features requiring paid with per_seat billing_component on OCI: export jobs above the Always Free row cap.
Features requiring paid with per_seat billing_component on OCI: anomaly assistant workflows.
Features requiring paid with per_seat billing_component on OCI: forecast backtesting over large history.
Features requiring paid with per_seat billing_component on OCI: commitment portfolio simulation.
Features requiring paid with per_seat billing_component on OCI: provider-specific rightsizing across large fleets.
Features requiring paid with per_seat billing_component on OCI: policy automation actions.
Features requiring paid with per_seat billing_component on OCI: multi-region resilience.
Features requiring paid with per_seat billing_component on OCI: billion-row exports.
Features requiring paid with per_seat billing_component on OCI: dedicated control planes.

## §5 Findings

Finding 01: demo_trial is over-stated in the current tenant_class adoption matrix because it assumes infrastructure larger than OCI Always Free.
Finding 02: demo_trial can still be useful if it is narrowed to invoice, basic report, budget status, FOCUS export, and audit evidence.
Finding 03: paid with per_seat billing_component is the first tenant_class that should target mainstream Vantage-style report and budget parity.
Finding 04: paid with per_seat billing_component cannot currently claim parity because report, budget, anomaly, recommendation, and dashboard APIs are missing.
Finding 05: paid with per_usage billing_component is the first tenant_class that should target Cloudability-style business mapping, scorecards, containers, unit economics, and governance depth.
Finding 06: paid with per_usage billing_component cannot currently claim parity because those capability families are absent or prose-only.
Finding 07: paid with compliance_pack gating is the first tenant_class that should target CloudHealth-style hybrid breadth, automated governance, and broad provider-specific rightsizing.
Finding 08: paid with compliance_pack gating cannot currently claim parity because local service lacks code, context IaC, OS support, benchmark harness, and provider-resource recommendation contracts.
Finding 09: Oyatie is potentially ahead on FOCUS eventing, Cedar policy, audit-chain evidence, regulator runbooks, DPIA, and compliance documents.
Finding 10: Those ahead areas are mostly documentation-backed today and still need executable APIs, tests, and measured evidence.
Finding 11: Vantage remains ahead on report builder, API breadth, budget ergonomics, Kubernetes reporting, and anomaly collaboration.
Finding 12: Cloudability remains ahead on business mapping, scorecards, sustainability, unit economics, workload planning, and container rightsizing.
Finding 13: CloudHealth remains ahead on hybrid organizational modeling, governance automation, and EC2/EBS/Azure/GCE/vSphere rightsizing breadth.
Finding 14: The local service should avoid commercial tenant_class-name parity claims until implementation exists.
Finding 15: The local service should use this tenant_class adoption matrix as a remediation map, not a marketing artifact.
Finding 16: The immediate tenant_class remediation is to split demo_trial into standard demo_trial and demo_trial OCI Always Free.
Finding 17: The immediate implementation remediation is to replace `iac/terraform-module.tf` with context-specific OpenTofu modules.
Finding 18: The immediate product remediation is to add report-query, budget, anomaly, recommendation, and dashboard APIs.
Finding 19: The immediate evidence remediation is to add Rust source, tests, OS manifest, CI lanes, and benchmark harness.
Finding 20: The final classification is demo_trial partial, paid with per_seat billing_component target-only, paid with per_usage billing_component target-only, and paid with compliance_pack gating aspirational until executable evidence lands.
