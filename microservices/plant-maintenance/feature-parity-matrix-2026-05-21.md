# Plant Maintenance Feature-Parity Matrix - 2026-05-20

Audit owner: sole Codex audit owner for `plant-maintenance`.
Target microservice path: `microservices/plant-maintenance/`.
Counterpart 1: SAP Plant Maintenance.
Counterpart 2: IBM Maximo.
Counterpart 3: UpKeep.
Purpose: compare the current Oyatie artifact surface against the union of the three assigned counterparts.
Constraint: this report does not introduce retired activation model or retired-delta scaffolding.
Tenant-class note: tenant_class values are `demo_trial` and `paid`; paid billing components are `per_seat` and `per_usage`.
Local source anchor: `PRD.md:28-32` defines SAP PM/EAM parity and six owned areas.
Local source anchor: `manifest.json:31-38` names the six bounded contexts.
Local source anchor: `src/domain/mod.rs:20-26` exposes only five capabilities.
Local source anchor: `src/adapter/http.rs:30-62` exposes only five HTTP routes.
Local source anchor: `contracts/openapi-v1.yaml:9-14` uses a stale benchmark roster.
External anchor: SAP Help Portal preventive maintenance page describes equipment, functional locations, maintenance orders, measuring points, task lists, time/counter schedules, and cost preview.
External URL: https://help.sap.com/docs/SAP_ERP/11825b10747e4ee4b91ecc1dba612536/d77cb6535fe6b74ce10000000a174cb4.html
External anchor: IBM Maximo Manage overview documents asset lifecycle, asset conditions/locations, work processes, inventory, scheduling, reporting, mobile capture, and integrations.
External URL: https://www.ibm.com/products/maximo/asset-management
External anchor: UpKeep official CMMS page documents work orders, preventive maintenance, asset management, parts inventory, analytics, mobile CMMS, offline sync, and AI-assisted PM.
External URL: https://upkeep.com/product/cmms-software/

## 1. Counterpart 1 - SAP Plant Maintenance capability surface

1. SAP surface: technical objects, including equipment and functional locations.
2. Oyatie evidence: equipment master is a named bounded context at `manifest.json:31-32`.
3. Oyatie evidence: source exposes `Capability::EquipmentMaster` at `src/domain/mod.rs:20-22`.
4. Parity status: partial because the Rust type does not model functional-location hierarchy or characteristic classes.
5. SAP surface: maintenance notifications or requests before formal orders.
6. Oyatie evidence: OpenAPI does not expose a separate notification/request endpoint at `contracts/openapi-v1.yaml:15-129`.
7. Parity status: gap because request intake is not explicit.
8. SAP surface: maintenance orders for planning, execution, and control.
9. Oyatie evidence: work-order bounded context exists at `manifest.json:34`.
10. Oyatie evidence: HTTP route `/v1/plant-maintenance/work-orders:release` exists at `src/adapter/http.rs:44-49`.
11. Parity status: partial because the handler returns a stub at `src/adapter/http.rs:65-67`.
12. SAP surface: preventive maintenance task lists.
13. Oyatie evidence: maintenance-plan context exists at `manifest.json:33`.
14. Oyatie evidence: IP-024 covers maintenance strategy cycle generation and due-date calculation in the inventory.
15. Parity status: partial because OpenAPI payloads are generic objects at `contracts/openapi-v1.yaml:132-176`.
16. SAP surface: time-based recurring maintenance.
17. Oyatie evidence: PRD includes maintenance plans at `PRD.md:28-32`.
18. Parity status: partial because no typed time-based schedule schema is visible in OpenAPI.
19. SAP surface: counter-based recurring maintenance.
20. Oyatie evidence: PRD names preventive maintenance but not explicit meter/counter fields in the cited schema region.
21. Parity status: gap because counter fields are not contractually typed.
22. SAP surface: measuring points and counters.
23. Oyatie evidence: IP-020 condition-based maintenance and IoT signal ingestion exists in inventory.
24. Parity status: planned gap because source domain does not expose measurement-point aggregate.
25. SAP surface: cost-based assignment and future cost preview.
26. Oyatie evidence: cost budget exists at `cost-budget.md`; PRD sends cost attribution to finops at `PRD.md:1132`.
27. Parity status: partial until cost attribution uses tenant_class plus paid billing components.
28. SAP surface: operations in a maintenance order.
29. Oyatie evidence: OpenAPI work order command uses generic payload at `contracts/openapi-v1.yaml:168-176`.
30. Parity status: gap because operations are not typed.
31. SAP surface: component requirements and spare parts.
32. Oyatie evidence: spare-part reservation context exists at `manifest.json:35`.
33. Oyatie evidence: HTTP route `/v1/plant-maintenance/spare-parts:reserve` exists at `src/adapter/http.rs:50-55`.
34. Parity status: partial because inventory reorder, lot, bin, vendor, and issue/transfer semantics are not typed.
35. SAP surface: capacity requirements and work centers.
36. Oyatie evidence: technician dispatch is named in manifest at `manifest.json:36`.
37. Oyatie evidence: technician dispatch is absent from source capabilities at `src/domain/mod.rs:20-26`.
38. Parity status: gap.
39. SAP surface: downtime integration with production planning.
40. Oyatie evidence: downtime-window context exists at `manifest.json:37`.
41. Oyatie evidence: HTTP route `/v1/plant-maintenance/downtime-windows:record` exists at `src/adapter/http.rs:56-61`.
42. Parity status: partial because source handler is stubbed.
43. SAP surface: confirmation of work performed.
44. Oyatie evidence: PRD audit events include before/after states for contexts at `PRD.md:1081-1123`.
45. Parity status: partial because confirmation fields and labor actuals are not typed.
46. SAP surface: settlement and accounting links.
47. Oyatie evidence: settlement flow exists at `PRD.md:897-904`.
48. Parity status: partial because tenant-class billing model is absent.
49. SAP surface: compliance and audit evidence.
50. Oyatie evidence: Cedar policy posture exists at `PRD.md:938-942` and policy files are in inventory.
51. Parity status: strong as a design goal, not yet runtime-proven.
52. SAP surface: migration/import from existing ERP.
53. Oyatie evidence: migration flow exists at `PRD.md:888-895`.
54. Parity status: partial because migration-playbook directory is absent.
55. SAP surface: reporting and dashboards.
56. Oyatie evidence: dashboards exist in inventory and telemetry rows exist at `PRD.md:1068-1125`.
57. Parity status: partial because benchmark roster needs deployment_context plus tenant_class labels.
58. SAP summary: Oyatie has the right top-level PM nouns.
59. SAP summary: Oyatie lacks typed depth for technical objects, order operations, counters, confirmations, and costing.
60. SAP summary: the immediate parity gate is turning the six named contexts into typed contracts and Rust behavior.

## 2. Counterpart 2 - IBM Maximo capability surface

1. Maximo surface: enterprise asset lifecycle management.
2. Oyatie evidence: equipment master is present in manifest and source at `manifest.json:31-32` and `src/domain/mod.rs:20-22`.
3. Parity status: partial because purchase, warranty, retirement, condition, and lifecycle history fields are not typed.
4. Maximo surface: asset conditions and locations.
5. Oyatie evidence: architecture says the service owns equipment master and facility reliability at `ARCHITECTURE.md:21-22`.
6. Parity status: partial because location hierarchy is not a contract schema.
7. Maximo surface: planned and unplanned work.
8. Oyatie evidence: PRD names corrective orders and preventive maintenance at `PRD.md:28-32`.
9. Parity status: partial because request/evaluation/intake is not separate.
10. Maximo surface: work order tracking.
11. Oyatie evidence: work order route exists at `src/adapter/http.rs:44-49`.
12. Parity status: partial because handler is stubbed at `src/adapter/http.rs:65-67`.
13. Maximo surface: job plans and work plans.
14. Oyatie evidence: maintenance plans exist, but OpenAPI payloads remain generic at `contracts/openapi-v1.yaml:150-167`.
15. Parity status: gap because job-plan operation lists are not typed.
16. Maximo surface: preventive maintenance records.
17. Oyatie evidence: maintenance plan command exists in contracts and source route.
18. Parity status: partial because time/meter/flexible schedules are not explicit.
19. Maximo surface: PM hierarchies and routes.
20. Oyatie evidence: no typed PM hierarchy appears in the OpenAPI schema.
21. Parity status: gap.
22. Maximo surface: meter-based maintenance.
23. Oyatie evidence: IP-020 proposes condition-based maintenance and IoT signal ingestion.
24. Parity status: planned gap because current source does not expose meter readings.
25. Maximo surface: inventory across storerooms.
26. Oyatie evidence: spare-part reservation exists at `manifest.json:35`.
27. Parity status: partial because storerooms, bins, lots, reorder points, and transfers are not typed in service contracts.
28. Maximo surface: reservations attached to approved work orders.
29. Oyatie evidence: spare reservation route exists at `src/adapter/http.rs:50-55`.
30. Parity status: partial because approval-state coupling to work orders is not executable.
31. Maximo surface: scheduling in a graphical or optimized view.
32. Oyatie evidence: technician dispatch is named in PRD and manifest.
33. Oyatie evidence: source omits technician dispatch at `src/domain/mod.rs:20-26`.
34. Parity status: gap.
35. Maximo surface: mobile technician capture, including photos and voice-to-text.
36. Oyatie evidence: no mobile frontend or mobile field contract appears in inventory.
37. Parity status: gap.
38. Maximo surface: role and application access.
39. Oyatie evidence: Cedar policies and tenant-scope docs exist.
40. Parity status: partial because tenant class and runtime policy fixtures are absent.
41. Maximo surface: regulatory compliance and audit support.
42. Oyatie evidence: compliance and DPIA docs exist.
43. Parity status: partial because repeated docs need executable evidence.
44. Maximo surface: reporting and KPIs.
45. Oyatie evidence: dashboard files and KPI scorecard IP exist.
46. Parity status: partial because dashboards are not tied to verified telemetry.
47. Maximo surface: external financial integrations.
48. Oyatie evidence: finops and marketplace settlement appear in manifest dependencies and PRD settlement flow at `manifest.json:39-46` and `PRD.md:897-904`.
49. Parity status: partial because revenue-share and paid billing semantics are absent.
50. Maximo surface: workflow process management.
51. Oyatie evidence: workflow-engine dependency is named at `manifest.json:42`.
52. Parity status: partial because cross-microservice handoff doc is absent.
53. Maximo surface: configurable business processes.
54. Oyatie evidence: IP files cover several implementation slices.
55. Parity status: partial because source remains scaffolded.
56. Maximo summary: Oyatie has a policy-rich EAM direction.
57. Maximo summary: Oyatie lacks Maximo-grade inventory, scheduling, mobile capture, and lifecycle field depth.
58. Maximo summary: technician dispatch is the most visible Maximo-family gap because it is documented but not implemented.
59. Maximo summary: six-context handoff contracts are needed before Maximo parity can be claimed.
60. Maximo summary: tenant-class and deployment-context overlays must be added before enterprise operability can be evaluated.

## 3. Counterpart 3 - UpKeep capability surface

1. UpKeep surface: mobile-first work order creation, assignment, and completion.
2. Oyatie evidence: work-order route exists at `src/adapter/http.rs:44-49`.
3. Parity status: partial because no mobile workflow, offline sync, push notification, or completion capture is typed.
4. UpKeep surface: instant assignment to technicians.
5. Oyatie evidence: technician dispatch is named in docs but absent from source at `src/domain/mod.rs:20-26`.
6. Parity status: gap.
7. UpKeep surface: work request intake with broad requester access.
8. Oyatie evidence: OpenAPI exposes command endpoints but not request/intake endpoints at `contracts/openapi-v1.yaml:15-129`.
9. Parity status: gap.
10. UpKeep surface: preventive maintenance by time, meter, or AI-recommended intervals.
11. Oyatie evidence: maintenance-plan route exists at `src/adapter/http.rs:38-43`; IP-020 and IP-024 exist in inventory.
12. Parity status: partial because time/meter/AI interval fields are not typed.
13. UpKeep surface: asset management with lifecycle, warranty, history, and condition.
14. Oyatie evidence: equipment master exists in manifest and source.
15. Parity status: partial because lifecycle and warranty fields are absent from contracts.
16. UpKeep surface: parts inventory with reorder points and purchase orders.
17. Oyatie evidence: spare-part reservation route exists at `src/adapter/http.rs:50-55`.
18. Parity status: gap because purchase orders and reorder points are not modeled.
19. UpKeep surface: analytics and reporting.
20. Oyatie evidence: dashboard files exist and PRD telemetry is broad at `PRD.md:1068-1125`.
21. Parity status: partial because SLO thresholds and rosters need alignment.
22. UpKeep surface: offline mobile capability.
23. Oyatie evidence: no mobile frontend or offline sync contract is present in inventory.
24. Parity status: gap.
25. UpKeep surface: photo capture and guided closeout.
26. Oyatie evidence: generic payload schemas do not type photo or checklist evidence at `contracts/openapi-v1.yaml:132-176`.
27. Parity status: gap.
28. UpKeep surface: checklists and pass/fail criteria.
29. Oyatie evidence: OpenAPI payloads are generic object fields.
30. Parity status: gap.
31. UpKeep surface: QR or barcode asset lookup.
32. Oyatie evidence: equipment master exists but asset lookup and QR identifiers are not typed.
33. Parity status: gap.
34. UpKeep surface: safety and compliance connection.
35. Oyatie evidence: compliance and Cedar policy artifacts exist.
36. Parity status: partial, with strong potential if policy fixtures are executable.
37. UpKeep surface: IoT condition monitoring and automated PM triggers.
38. Oyatie evidence: IP-020 condition-based maintenance exists in inventory.
39. Parity status: planned gap.
40. UpKeep surface: embedded AI assistant and insights.
41. Oyatie evidence: manifest has `intelligence_dispatch` at `manifest.json:164`.
42. Parity status: partial because no runtime AI-assisted maintenance path is typed.
43. UpKeep surface: vendor and provider workflows.
44. Oyatie evidence: marketplace settlement and partner references exist.
45. Parity status: partial because no provider assignment or vendor PO flow is typed.
46. UpKeep surface: multi-site operations.
47. Oyatie evidence: multi-region doc exists; deployment-context proof is absent.
48. Parity status: gap until six deployment contexts are modeled.
49. UpKeep surface: simple onboarding and free requester flows.
50. Oyatie evidence: onboarding and FAQ directories are absent.
51. Parity status: gap.
52. UpKeep surface: free trial / paid commercial motion.
53. Oyatie evidence: tenant-class adoption is absent by scan.
54. Parity status: gap because demo-trial and paid semantics are required by current doctrine.
55. UpKeep summary: Oyatie has enterprise governance strength that UpKeep-style teams may lack.
56. UpKeep summary: Oyatie lacks UpKeep's field-technician ergonomics, mobile-first offline path, request intake, and simple PM workflows.
57. UpKeep summary: adopting UpKeep parity means adding frontend/mobile contracts, not only backend records.
58. UpKeep summary: technician dispatch and mobile closeout should be treated as first-class product surfaces.
59. UpKeep summary: demo_trial tenant_class needs an onboarding and usage-cap path, not an old feature ladder.
60. UpKeep summary: UpKeep is the strongest prompt to reduce scaffolded ERP language into usable operator workflows.

## 4. Union-Coverage Matrix

| Capability family | SAP | Maximo | UpKeep | Oyatie current evidence | Status |
| --- | --- | --- | --- | --- | --- |
| Equipment master | Strong | Strong | Strong | `manifest.json:31-32`, `src/domain/mod.rs:20-22` | Partial |
| Functional locations | Strong | Strong | Medium | PRD mentions facility reliability at `ARCHITECTURE.md:21-22` | Gap |
| Asset hierarchy | Strong | Strong | Strong | IP-025 exists in inventory | Planned gap |
| Asset lifecycle | Medium | Strong | Strong | Generic OpenAPI payload at `contracts/openapi-v1.yaml:132-176` | Gap |
| Asset warranty | Medium | Medium | Strong | No typed field in contracts | Gap |
| Asset condition | Medium | Strong | Strong | IP-020 exists in inventory | Planned gap |
| Measurement points | Strong | Strong | Medium | No typed meter/counter schema | Gap |
| Meter readings | Strong | Strong | Strong | IP-020 exists; OpenAPI generic | Planned gap |
| Work requests | Medium | Strong | Strong | No request endpoint in OpenAPI | Gap |
| Notifications | Strong | Medium | Medium | No separate notification aggregate | Gap |
| Work orders | Strong | Strong | Strong | `src/adapter/http.rs:44-49` | Partial |
| Work order release | Strong | Strong | Medium | Route exists; handler stub at `src/adapter/http.rs:65-67` | Partial |
| Work order closeout | Strong | Strong | Strong | No typed completion schema | Gap |
| Work order cost actuals | Strong | Strong | Medium | Cost docs exist; no typed actuals | Gap |
| Work order labor | Strong | Strong | Medium | No typed labor requirement schema | Gap |
| Work order materials | Strong | Strong | Strong | Spare route exists; no component list | Partial |
| Work order tools | Strong | Strong | Medium | No typed tools schema | Gap |
| Work order safety plan | Strong | Strong | Medium | IP-016 and IP-017 exist | Planned gap |
| Permit to work | Medium | Medium | Low | IP-017 exists | Planned gap |
| LOTO state machine | Medium | Medium | Low | IP-016 exists | Planned gap |
| Preventive maintenance | Strong | Strong | Strong | `src/adapter/http.rs:38-43` | Partial |
| Time-based PM | Strong | Strong | Strong | No typed schedule fields | Gap |
| Meter-based PM | Strong | Strong | Strong | IP-020 exists | Planned gap |
| Condition-based PM | Medium | Strong | Strong | IP-020 exists | Planned gap |
| AI recommended PM | Low | Medium | Strong | `manifest.json:164` mentions intelligence dispatch | Planned gap |
| PM hierarchy | Strong | Strong | Low | No typed hierarchy | Gap |
| PM route | Strong | Strong | Low | No typed route | Gap |
| Task lists | Strong | Medium | Medium | No typed task list schema | Gap |
| Checklists | Medium | Medium | Strong | Generic payload only | Gap |
| Pass/fail criteria | Medium | Medium | Strong | Generic payload only | Gap |
| Spare parts | Strong | Strong | Strong | `src/adapter/http.rs:50-55` | Partial |
| Parts inventory | Medium | Strong | Strong | No inventory aggregate | Gap |
| Storerooms | Medium | Strong | Medium | No typed storeroom | Gap |
| Bins and lots | Medium | Strong | Medium | No typed bin/lot | Gap |
| Reorder point | Low | Strong | Strong | No typed reorder point | Gap |
| Purchase orders | Medium | Strong | Strong | No typed PO flow | Gap |
| Vendor records | Medium | Strong | Medium | No typed vendor flow | Gap |
| Parts reservation | Strong | Strong | Strong | Route exists | Partial |
| Auto-reserve parts | Medium | Strong | Strong | No executable coupling | Gap |
| Technician dispatch | Medium | Strong | Strong | In docs, absent from source capabilities | Gap |
| Skill matrix | Low | Strong | Medium | IP-018 exists | Planned gap |
| Assignment by location | Medium | Strong | Strong | No typed dispatch source route | Gap |
| Assignment by workload | Low | Strong | Strong | No typed scheduler | Gap |
| Mobile technician UI | Medium | Strong | Strong | No frontend/mobile artifact | Gap |
| Offline sync | Medium | Strong | Strong | No offline contract | Gap |
| Push notification | Low | Strong | Strong | No push contract | Gap |
| Photo evidence | Medium | Strong | Strong | No typed attachment evidence | Gap |
| Voice capture | Low | Medium | Strong | No mobile capture schema | Gap |
| Downtime window | Strong | Medium | Medium | `src/adapter/http.rs:56-61` | Partial |
| Production-planning link | Strong | Medium | Low | No cross-service handoff doc | Gap |
| OEE metrics | Medium | Medium | Medium | IP-023 exists | Planned gap |
| MTTR metrics | Medium | Strong | Strong | IP-023 exists | Planned gap |
| First-time fix | Low | Strong | Strong | IP-023 exists | Planned gap |
| Reliability analytics | Medium | Strong | Medium | IP-021 and IP-022 exist | Planned gap |
| Weibull fitting | Low | Medium | Low | IP-022 exists | Planned gap |
| Failure codes | Medium | Strong | Medium | No typed code schema | Gap |
| Audit trail | Strong | Strong | Medium | `PRD.md:1081-1123`, policy inventory | Partial |
| Compliance packs | Medium | Strong | Medium | `manifest.json:271-280` | Partial |
| Data residency | Medium | Strong | Medium | policy data-residency doc exists | Partial |
| Default-deny authorization | Medium | Strong | Medium | `PRD.md:938-942` | Partial |
| Tenant isolation | Low | Strong | Medium | `manifest.json:54-58` | Partial |
| Tenant class | Not equivalent | Not equivalent | Commercially relevant | No scan hits | Gap |
| Demo trial usage caps | Not equivalent | Low | Strong | No scan hits | Gap |
| Paid contractual scaling | Medium | Strong | Strong | No tenant-class model | Gap |
| Revenue-share settlement | Not equivalent | Low | Low | Marketplace flow exists, class absent | Gap |
| Marketplace settlement | Low | Low | Medium | `PRD.md:897-904` | Partial |
| Finops chargeback | Medium | Strong | Medium | `PRD.md:1132` tenant_class plus paid billing component field | Partial |
| Six deployment contexts | Not product-native | Enterprise deployable | SaaS deployable | No context paths | Gap |
| OpenTofu IaC | Not product-native | Deployable infra | SaaS deployable | `iac/terraform-module/main.tf` | Gap |
| Supported OS matrix | Not product-native | Enterprise deployable | Mobile/backend deployable | No `supported-oses.json` | Gap |
| OCI Always Free profile | Not product-native | Not product-native | Trial relevant | No `iac/oci-guest/always-free/` | Gap |
| Availability SLO | Medium | Strong | Strong | SLO file exists | Partial |
| Latency SLO | Medium | Strong | Strong | p99 SLO exists; target conflict | Partial |
| Throughput SLO | Medium | Strong | Medium | SLO file exists | Partial |
| Capacity model | Medium | Strong | Medium | capacity doc exists | Partial |
| Incident response | Medium | Strong | Medium | runbooks exist | Partial |
| Migration replay | Strong | Strong | Medium | backfill-replay doc exists; no migration dir | Partial |
| Import/export | Strong | Strong | Medium | PRD names import/export; no typed import schema | Partial |
| API integrations | Medium | Strong | Strong | contracts exist; generic payloads | Partial |
| SDK plan | Low | Medium | Strong | `sdk-plan.md` exists | Partial |
| Reporting | Strong | Strong | Strong | dashboard files exist | Partial |
| Ad hoc reporting | Medium | Strong | Medium | no report builder | Gap |
| Custom workflows | Medium | Strong | Strong | workflow-engine dependency exists | Partial |
| No-code app extensions | Low | Medium | Strong | no extension contract | Gap |
| Learning/training link | Low | Low | Medium | no training workflow | Gap |
| EHS incident linkage | Medium | Medium | Strong | IP-016/017 plus compliance docs | Planned gap |
| Fleet maintenance | Low | Medium | Strong | not in service purpose | Correctly out of scope |
| Provider marketplace | Low | Low | Medium | marketplace dependency exists | Partial |
| Multi-site reporting | Medium | Strong | Strong | multi-region doc exists | Partial |
| Cross-site RBAC | Medium | Strong | Strong | Cedar docs exist; fixtures ignored | Partial |
| Requester seats/requesters | Low | Medium | Strong | no requester model | Gap |
| QR/barcode lookup | Low | Medium | Strong | no lookup schema | Gap |
| Attachments | Medium | Strong | Strong | no attachment schema | Gap |
| Signature capture | Medium | Medium | Strong | no signature schema | Gap |
| External financial systems | Strong | Strong | Medium | finops dependency exists | Partial |
| Inventory financial valuation | Strong | Strong | Medium | no typed valuation | Gap |
| Source-system provenance | Strong | Strong | Medium | PRD requires source refs at `PRD.md:35` | Partial |
| Idempotency | Medium | Strong | Strong | OpenAPI requires idempotency at `contracts/openapi-v1.yaml:132-176` | Partial |
| Audit-chain sealing | Oyatie additive | Oyatie additive | Oyatie additive | PRD and manifest include audit chain | Partial |
| Ontology projection | Oyatie additive | Low | Low | PRD and manifest name ontology dependency | Partial |
| PQC/ECH transport | Oyatie additive | Low | Low | `manifest.json:161-164` | Partial |

## 5. Family Summary

1. Equipment family status: partial.
2. Equipment family reason: equipment master exists, but functional-location hierarchy, lifecycle, warranty, QR lookup, and condition fields are not typed.
3. Maintenance-plan family status: partial.
4. Maintenance-plan family reason: plan route exists, but time/meter/flexible schedule and PM hierarchy are not typed.
5. Work-order family status: partial.
6. Work-order family reason: work-order route exists, but handler is a stub and operations/labor/materials/closeout are not typed.
7. Spare-part family status: partial.
8. Spare-part family reason: reservation exists, but inventory, stores, bins, lots, reorder, vendors, and POs are not modeled.
9. Technician-dispatch family status: gap.
10. Technician-dispatch family reason: docs and policy name it, but Rust domain and HTTP routes omit it.
11. Downtime-window family status: partial.
12. Downtime-window family reason: downtime route exists, but production-planning handoff and runtime behavior are absent.
13. Mobile-frontline family status: gap.
14. Mobile-frontline family reason: UpKeep and Maximo mobile surfaces are not represented in contracts or frontend artifacts.
15. Compliance-policy family status: partial.
16. Compliance-policy family reason: Cedar and compliance artifacts exist, but tenant-class semantics and executable fixtures are absent.
17. Operations family status: partial.
18. Operations family reason: SLOs and runbooks exist, but deployment overlays and benchmark rosters are stale.
19. Deployment family status: gap.
20. Deployment family reason: six contexts, OpenTofu modules, supported OS manifest, and OCI Always Free profile are missing.
21. Commercial model family status: gap.
22. Commercial model reason: tenant-class semantics need adoption evidence.
23. Counterpart metadata family status: gap.
24. Counterpart metadata reason: assigned UpKeep counterpart is missing from current rosters.
25. Implementation family status: early scaffold.
26. Implementation family reason: source has useful module structure but not parity behavior.

## 6. Headline Gap Analysis

1. Gap A: UpKeep is missing from the service's official comparator roster.
2. Evidence: `manifest.json:25-29`, `PRD.md:55-60`, and `contracts/openapi-v1.yaml:9-14`.
3. Impact: feature priorities miss mobile-first, offline, requester, and simple CMMS workflows.
4. Closure: normalize rosters and add UpKeep-driven requirements.
5. Gap B: technician dispatch is a documented context but not an implementation context.
6. Evidence: `manifest.json:36` versus `src/domain/mod.rs:20-26` and `src/adapter/http.rs:30-62`.
7. Impact: SAP work-center planning, Maximo scheduling, and UpKeep technician assignment cannot be claimed.
8. Closure: implement technician-dispatch across domain, contracts, routes, tests, and policy fixtures.
9. Gap C: contracts do not type PM-specific business data.
10. Evidence: generic `payload` object at `contracts/openapi-v1.yaml:132-176`.
11. Impact: implementers cannot infer equipment classifications, meter schedules, parts, labor, safety gates, or closeout evidence.
12. Closure: split typed command schemas by bounded context.
13. Gap D: mobile and frontline workflow is absent.
14. Evidence: no mobile/frontend/offline artifacts in inventory.
15. Impact: UpKeep and Maximo field-technician parity is not present.
16. Closure: define Leptos web SSR/selective hydration and native frontend contracts where applicable, plus offline sync rules.
17. Gap E: canonical deployability is absent.
18. Evidence: no six-context IaC directories, no OpenTofu, no supported-oses manifest, no OCI Always Free profile.
19. Impact: the service cannot be honestly called deployable across all six contexts.
20. Closure: add OpenTofu modules and OS support manifest.
21. Gap F: retired vocabulary cleanup is a remediation lane.
22. Evidence: retired vocabulary was cataloged in `coherence-audit-2026-05-20.md` Section 3.4.T.
23. Impact: commercial, policy, telemetry, and cost semantics will drift from tenant-class doctrine.
24. Closure: replace with tenant class, deployment context, criticality class, and cell role.
25. Gap G: source-level tests do not enforce parity.
26. Evidence: ignored tests at `tests/integration.rs:59-77`.
27. Impact: five-versus-six context drift went uncaught.
28. Closure: unignore and expand tests before implementation claims.
29. Gap H: cross-service ownership is scattered.
30. Evidence: dependencies listed at `manifest.json:39-46` and `manifest.json:282-288`, but no handoff file.
31. Impact: warehouse, workflow-engine, ontology, finops, and quality-management boundaries are ambiguous under failure.
32. Closure: add a handoff artifact and test cross-service contract assumptions.
33. Gap I: performance targets need a single industry-leader model.
34. Evidence: p99 prose and OpenSLO threshold conflict, plus deployment_context and tenant_class capacity model at `capacity-model.md:16-23`.
35. Impact: implementers cannot size per deployment context or tenant class.
36. Closure: use the performance benchmark report as the new target set.

## 7. Additive Oyatie Surface

1. Additive surface: Cedar default-deny authorization can exceed smaller CMMS products when executable.
2. Evidence: `PRD.md:938-942` and policy inventory.
3. Additive surface: audit-chain sealing can provide stronger compliance evidence than simple activity logs.
4. Evidence: PRD telemetry and manifest seal-event references.
5. Additive surface: ontology projection can make PM data interoperable across ERP and operational services.
6. Evidence: ontology dependency at `manifest.json:43`.
7. Additive surface: marketplace and finops settlement can connect maintenance actions to commercial events.
8. Evidence: settlement flow at `PRD.md:897-904`.
9. Additive surface: deployment-context portability can exceed SaaS-only products once OpenTofu and OS support land.
10. Evidence: canonical six-context requirement at `specs/master-plan-sequencing.json:704-745`.
11. Additive surface: OCI Always Free profile can support `demo_trial` usage without lowering quality.
12. Evidence: canonical OCI profile requirement at `specs/master-plan-sequencing.json:857-868`.
13. Additive surface: Rust-only backend can keep runtime and tooling surface narrow.
14. Evidence: no forbidden language source files found; Rust crate exists at `Cargo.toml:1-62`.
15. Additive surface: PQC/ECH transport posture is already named.
16. Evidence: `manifest.json:161-164` and `contracts/openapi-v1.yaml:20-24`.
17. Additive surface: explicit downtime windows are stronger than many CMMS-lite workflows.
18. Evidence: downtime-window context at `manifest.json:37` and route at `src/adapter/http.rs:56-61`.
19. Additive surface: reliability analytics IPs can move Oyatie beyond reactive CMMS parity.
20. Evidence: IP-021, IP-022, and IP-023 in inventory.
21. Additive surface: permit-to-work and LOTO IPs can support regulated plant operations.
22. Evidence: IP-016 and IP-017 in inventory.
23. Additive surface: policy-checked technician skill matrix can improve dispatch safety.
24. Evidence: IP-018 in inventory.
25. Additive surface: tenant-class model can clarify commercial treatment without fragmenting quality.
26. Evidence: current doctrine in prompt; current gap by scan.
27. Additive surface: revenue-share class can support marketplace sellers and embedded SaaS resellers.
28. Evidence: current prompt; service currently lacks adoption.
29. Additive surface: single quality target across tenant classes avoids old feature ladder degradation.
30. Evidence: current prompt; service is adopting tenant_class plus paid billing components.

## 8. Recommended Remediation Order

1. First, normalize counterpart roster to SAP Plant Maintenance, IBM Maximo, and UpKeep in manifest, PRD, contracts, SLO descriptions, and parity docs.
2. Second, remove or rewrite retired activation language into tenant_class, deployment_context, cell_role, and criticality_class.
3. Third, add tenant-class contract/config/policy/billing fields.
4. Fourth, add six-context OpenTofu module layout and `supported-oses.json`.
5. Fifth, implement technician-dispatch in Rust domain, route, usecase, tests, and descriptors.
6. Sixth, fix capability descriptors so each maps to its real bounded context.
7. Seventh, replace generic OpenAPI payloads with typed command schemas.
8. Eighth, unignore contract, proto, AsyncAPI, Cedar, and repository tests.
9. Ninth, add cross-microservice handoffs for warehouse, real-estate, workflow-engine, ontology, finops, and quality-management.
10. Tenth, add mobile/offline/requester workflow contracts to cover UpKeep and Maximo field-technician parity.
11. Eleventh, add inventory, reorder, bin/lot, vendor, and PO semantics or explicitly delegate them to warehouse with tested handoff.
12. Twelfth, reconcile p99 target conflict and bind performance to context and tenant-class overlays.
13. Thirteenth, convert implementation-plan IP files from front-matter retired activation metadata to accepted replacement metadata.
14. Fourteenth, replace repeated docs with implementer-runbook material and validation commands.
15. Fifteenth, reopen the stale audit closure record until these gates are verified.
