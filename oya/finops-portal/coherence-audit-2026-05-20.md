# finops-portal coherence audit - 2026-05-20

## Citation anchor block

1. Canonical sequence and constraint source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3831-4228`, especially §D-20 nine-dimension audit, severity rules, and per-microservice deliverable requirements; §D-15..§D-19 are embedded in the same decision range and are expanded by `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3600-3828` for OCI Always Free.
2. Machine control surface: `specs/master-plan-sequencing.json:704-868`, covering six deployment contexts, OpenTofu-only substrate, OS matrix, Rust-strict language policy, and demo_trial OCI Always Free profile.
3. Service PRD audited: `microservices/finops-portal/PRD.md:1-116`; this includes the product boundary, out-of-scope clauses, NFRs, competitive parity notes, phase plan, and evidence references.
4. Service architecture audited: `microservices/finops-portal/ARCHITECTURE.md:1-1058`; this includes ownership, substrate binding, six-dimension matrix, deployment shape, and portability posture.
5. Documentation rigor source: `docs/standards/documentation-rigor.md:40-220`, especially retroactive applicability, service documentation set floor, intern-buildability test, hyperscaler-grade sub-test, and cross-reference invariants.

## Investigation scope statement

Auditor: single-owner audit lane for `microservices/finops-portal/`.
Deliverable scope: audit only; no remediation; no commits.
Files seen under target path: 161.
Lines audited under target path: 27,245.
Bytes inventoried under target path: 3,059,770.
Chat history searched: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
Counterparts confirmed from current user directive and chat history: Vantage, Cloudability (IBM), CloudHealth (VMware).
External public docs checked: Vantage docs, IBM Apptio Cloudability docs/pages, VMware Tanzu CloudHealth solution docs, FinOps Foundation FOCUS.
Constraint memory files read: the five doctrine files plus ownership-coherence, verify-deliverables, and docs-substance feedback files under `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/`.
Write boundary observed: this report and sibling audit reports are the only new files touched in the target microservice path.

## §1 microservice purpose summary

`finops-portal` is the tenant-facing and operator-facing FinOps presentation layer for Oyatie.
The PRD says the cost data plane already exists through OpenCost, Mimir, and FOCUS 1.3, while `finops-portal` supplies the branded UX and workflow layer above that substrate (`microservices/finops-portal/PRD.md:12-20`).
The target users are tenant admins, ops-finops, customer success, and auditors or regulators (`microservices/finops-portal/PRD.md:22-31`).
The product scope includes invoice presentation, drill-down dashboards, cost-allocation policy, anomaly explanation, FOCUS export, credit ledger, and quarterly regulator evidence (`microservices/finops-portal/PRD.md:33-50`).
The explicit non-owners are cost aggregation, anomaly detection, chargeback formula, payment processing, and per-cloud provider bill ingestion (`microservices/finops-portal/PRD.md:52-60`).
The architecture reinforces the same posture: `finops-portal` is bound to observability, cloud-iac, tenancy, and audit-chain rather than being the raw billing ingestion or anomaly detector itself (`microservices/finops-portal/ARCHITECTURE.md:200-204`).
The manifest lists five bounded contexts: tenant-billing-presentation, cost-allocation-policy, anomaly-explanation, focus-export, and credit-ledger (`microservices/finops-portal/manifest.json:15-67`).
The later Wave 3 artifact expansion adds additional business surfaces: budget alerts, forecasting, commitment management, rightsizing recommendations, and showback/chargeback (`microservices/finops-portal/catalog/oya-finops-portal-budget-alert-kernel.yaml:1-15`, `microservices/finops-portal/catalog/oya-finops-portal-rightsizing-recommender-worker.yaml:1-14`).
The local artifacts therefore describe two overlapping product shapes.
Shape A is the original PRD/manifest: invoice, dashboard, policy, anomaly explanation, export, credit ledger.
Shape B is the expanded FinOps suite: budget alerts, forecasts, commitments, rightsizing, chargeback, and risk/tax/finance journey overlays.
Those shapes are compatible as a roadmap if explicitly staged.
They are not fully coherent as a "full-pack-ready" implementation state because many added IPs are thin, code is absent, deployment context modules are absent, and some references point to missing evidence or missing runbooks.
The primary audit result is partial product coherence with hard canonical-direction drift.
The strongest coherent areas are API contract coverage, Cedar policy presence, SLO declaration breadth, runbook breadth, and FOCUS 1.3 intent.
The weakest coherent areas are OpenTofu context coverage, supported OS manifest coverage, demo_trial OCI Always Free coverage, line-level evidence for measured benchmarks, and stale internal references.
The service does not contain forbidden non-Rust source files by extension.
The service also does not contain Rust source or Cargo manifests in its own path.
That means Rust-strict is not violated by stray forbidden languages, but the implementation is not buildable from this path.
The service is a planning and documentation pack, not a currently buildable microservice implementation.

## §2 Inventory snapshot

| file | size | role | coherent_with_purpose? |
|---|---:|---|---|
| `ARCHITECTURE.md` | 1058 lines / 191213 bytes | architecture deep dive | partial |
| `AUDIT-FINDINGS-2026-05-20.json` | 20 lines / 1305 bytes | prior audit summary | partial |
| `CHANGELOG.md` | 31 lines / 1061 bytes | change log | partial |
| `IP-journey-j100-pack-rollout-first-action.md` | 400 lines / 65994 bytes | journey IP | partial |
| `IP-journey-j115-usage-chargeback.md` | 863 lines / 63274 bytes | journey IP | yes |
| `IP-journey-j117-slo-credit-ledger.md` | 430 lines / 58696 bytes | journey IP | yes |
| `IP-journey-j119-receivable-cash-forecast.md` | 430 lines / 60981 bytes | journey IP | partial |
| `IP-journey-j120-exposure-dashboard.md` | 430 lines / 59110 bytes | journey IP | partial |
| `IP-journey-j121-financial-statement-export.md` | 430 lines / 61603 bytes | journey IP | partial |
| `IP-journey-j122-ap-batch-control-panel.md` | 430 lines / 59968 bytes | journey IP | partial |
| `IP-journey-j125-purchase-price-ledger.md` | 430 lines / 59731 bytes | journey IP | partial |
| `IP-journey-j133-severance-computation-and-budget-update.md` | 425 lines / 52880 bytes | journey IP | partial |
| `IP-journey-j146-income-categorization-and-1099.md` | 425 lines / 63335 bytes | journey IP | partial |
| `IP-journey-j149-personal-earnings-aggregation.md` | 430 lines / 62181 bytes | journey IP | partial |
| `IP-journey-j150-parental-earnings-dashboard.md` | 430 lines / 61643 bytes | journey IP | partial |
| `IP-journey-j42-spend-attribution.md` | 420 lines / 68558 bytes | journey IP | yes |
| `IP-journey-j48-tax-filing-console.md` | 420 lines / 68964 bytes | journey IP | partial |
| `IP-journey-j53-revenue-recognition.md` | 430 lines / 114952 bytes | journey IP | partial |
| `IP-journey-j58-comp-budget.md` | 430 lines / 112717 bytes | journey IP | partial |
| `IP-journey-j60-budget-allocation.md` | 430 lines / 110868 bytes | journey IP | yes |
| `IP-journey-j66-quarterly-tax-report.md` | 430 lines / 115893 bytes | journey IP | partial |
| `IP-journey-j82-finance-risk-console.md` | 430 lines / 37782 bytes | journey IP | partial |
| `IP-journey-j86-finance-risk-console.md` | 430 lines / 38238 bytes | journey IP | partial |
| `IP-journey-j91-us-msb-mtl-overlay.md` | 400 lines / 68996 bytes | regulatory journey IP | partial |
| `IP-journey-j92-br-lgpd-us-parent-dsar.md` | 400 lines / 67343 bytes | regulatory journey IP | partial |
| `IP-journey-j93-in-dpdpa-rbi-overlay.md` | 400 lines / 68695 bytes | regulatory journey IP | partial |
| `IP-journey-j94-sox404-public-company-controls.md` | 400 lines / 70184 bytes | regulatory journey IP | partial |
| `IP-journey-j95-iso27001-soc2-annual-audit.md` | 400 lines / 68080 bytes | regulatory journey IP | partial |
| `IP-journey-j96-ksa-uae-mena-onboarding.md` | 400 lines / 68842 bytes | regulatory journey IP | partial |
| `IP-journey-j97-sg-pdpa-mas-tenant.md` | 400 lines / 65297 bytes | regulatory journey IP | partial |
| `IP-journey-j98-au-privacy-apra-cps234.md` | 400 lines / 66283 bytes | regulatory journey IP | partial |
| `IP-journey-j99-multi-pack-conflict-resolution.md` | 400 lines / 75587 bytes | regulatory journey IP | partial |
| `PHASE-01-tenant-billing-presentation.md` | 115 lines / 3692 bytes | phase plan | yes |
| `PRD.md` | 116 lines / 5270 bytes | product requirements | partial |
| `README.md` | 76 lines / 3269 bytes | operator overview | partial |
| `backfill-plan.md` | 151 lines / 5556 bytes | data recovery plan | yes |
| `benchmarks/aws-cost-explorer-gcp-billing-apptio-vs-oyatie.md` | 119 lines / 7663 bytes | benchmark narrative | partial |
| `capabilities/anomaly-explanation.capability.yaml` | 59 lines / 2267 bytes | capability record | yes |
| `capabilities/focus-export.capability.yaml` | 53 lines / 1780 bytes | capability record | yes |
| `capabilities/tenant-invoice-render.capability.yaml` | 61 lines / 2240 bytes | capability record | yes |
| `ADR-0330 and ADR-0331 tenant_class model` | 147 lines / 8967 bytes | tenant_class adoption matrix | partial |
| `capacity-model.md` | 121 lines / 4465 bytes | capacity model | partial |
| `catalog/bnf-v4.1.yaml` | 143 lines / 4232 bytes | catalog root | yes |
| `catalog/oya-finops-portal-budget-alert-kernel.yaml` | 15 lines / 415 bytes | catalog record | partial |
| `catalog/oya-finops-portal-commitment-management-domain.yaml` | 14 lines / 450 bytes | catalog record | partial |
| `catalog/oya-finops-portal-forecasting-usecase.yaml` | 14 lines / 419 bytes | catalog record | partial |
| `catalog/oya-finops-portal-rightsizing-recommender-worker.yaml` | 14 lines / 490 bytes | catalog record | partial |
| `catalog/oya-finops-portal-showback-chargeback-domain.yaml` | 14 lines / 449 bytes | catalog record | partial |
| `competitor-parity.md` | 99 lines / 7212 bytes | competitor matrix | partial |
| `compliance-matrix.md` | 101 lines / 6228 bytes | compliance mapping | partial |
| `compliance.md` | 876 lines / 122167 bytes | compliance control expansion | partial |
| `contracts/cost-allocation-policy-internal.proto` | 143 lines / 3968 bytes | proto contract | yes |
| `contracts/focus-export-internal.asyncapi.yaml` | 128 lines / 4577 bytes | AsyncAPI contract | yes |
| `contracts/tenant-invoice-public.openapi.yaml` | 306 lines / 10587 bytes | OpenAPI contract | yes |
| `cost-model.md` | 101 lines / 4194 bytes | self-cost model | yes |
| `dashboards/anomaly-investigation.grafana.json` | 123 lines / 4204 bytes | dashboard | yes |
| `dashboards/budget-alerts.grafana.json` | 17 lines / 855 bytes | dashboard | partial |
| `dashboards/fleet-cost-rollup.grafana.json` | 134 lines / 4148 bytes | dashboard | yes |
| `dashboards/rightsizing-recommendations.grafana.json` | 17 lines / 928 bytes | dashboard | partial |
| `dashboards/tenant-cost-drilldown.grafana.json` | 167 lines / 5194 bytes | dashboard | yes |
| `dashboards/tenant-cost-drilldown.md` | 47 lines / 1389 bytes | dashboard spec | yes |
| `decisions/ADR-FIN-001-focus-spec-adoption-and-multi-cloud-cost-normalization.md` | 245 lines / 19765 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-001-focus-spec-version.md` | 57 lines / 1832 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-002-cost-attribution-label-strategy.md` | 64 lines / 1949 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-003-tenant-billing-export-cadence.md` | 63 lines / 2047 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-004-credit-ledger-append-only.md` | 57 lines / 1801 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-005-grafana-iframe-embed.md` | 57 lines / 1771 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-006-cedar-residency-double-guard.md` | 66 lines / 1968 bytes | service ADR | yes |
| `decisions/ADR-finops-portal-007-ed25519-quarterly-key.md` | 63 lines / 1985 bytes | service ADR | yes |
| `dpia.md` | 115 lines / 4757 bytes | DPIA | partial |
| `failure-modes.md` | 169 lines / 6573 bytes | failure catalog | partial |
| `faqs/finops-engineer-faq.md` | 113 lines / 9180 bytes | FAQ | partial |
| `iac/ech-config.yaml` | 11 lines / 275 bytes | infra support | partial |
| `iac/edge-waf.yaml` | 32 lines / 952 bytes | infra support | partial |
| `iac/helm/finops-portal/Chart.yaml` | 30 lines / 824 bytes | Helm chart | partial |
| `iac/helm/finops-portal/templates/_helpers.tpl` | 37 lines / 1141 bytes | Helm helper | partial |
| `iac/helm/finops-portal/templates/deployment.yaml` | 63 lines / 2526 bytes | K8s template | partial |
| `iac/helm/finops-portal/templates/hpa.yaml` | 29 lines / 764 bytes | K8s template | partial |
| `iac/helm/finops-portal/templates/networkpolicy.yaml` | 37 lines / 919 bytes | K8s template | partial |
| `iac/helm/finops-portal/templates/prometheusrule.yaml` | 59 lines / 2498 bytes | K8s template | partial |
| `iac/helm/finops-portal/templates/service.yaml` | 19 lines / 461 bytes | K8s template | partial |
| `iac/helm/finops-portal/templates/servicemonitor.yaml` | 22 lines / 709 bytes | K8s template | partial |
| `iac/helm/finops-portal/values-eu.yaml` | 26 lines / 607 bytes | Helm values | partial |
| `iac/helm/finops-portal/values-kr.yaml` | 23 lines / 563 bytes | Helm values | partial |
| `iac/helm/finops-portal/values-us-healthcare.yaml` | 30 lines / 804 bytes | Helm values | partial |
| `iac/helm/finops-portal/values.yaml` | 92 lines / 2021 bytes | Helm values | partial |
| `iac/k8s-network-policy.yaml` | 38 lines / 1170 bytes | K8s policy | partial |
| `iac/openbao-policy.hcl` | 29 lines / 722 bytes | secret policy | partial |
| `iac/pqc-cert.yaml` | 13 lines / 328 bytes | certificate config | partial |
| `iac/secret-bindings.yaml` | 37 lines / 1323 bytes | secret config | partial |
| `iac/terraform-module.tf` | 40 lines / 1029 bytes | IaC module | no |
| `implementation-plans/IP-001-finops-portal-tenant-billing-presentation-kernel.md` | 130 lines / 6132 bytes | implementation plan | yes |
| `implementation-plans/IP-002-finops-portal-tenant-billing-presentation-domain.md` | 123 lines / 4955 bytes | implementation plan | yes |
| `implementation-plans/IP-003-finops-portal-helm-chart-bootstrap.md` | 133 lines / 5352 bytes | implementation plan | partial |
| `implementation-plans/IP-004-finops-portal-tenant-billing-presentation-usecase.md` | 127 lines / 5170 bytes | implementation plan | yes |
| `implementation-plans/IP-005-finops-portal-tenant-billing-presentation-api.md` | 143 lines / 5313 bytes | implementation plan | partial |
| `implementation-plans/IP-006-finops-portal-tenant-billing-presentation-app.md` | 133 lines / 5147 bytes | implementation plan | yes |
| `implementation-plans/IP-007-finops-portal-cedar-policy-tenant-isolation.md` | 140 lines / 5333 bytes | implementation plan | yes |
| `implementation-plans/IP-008-finops-portal-grafana-embed-dashboards.md` | 123 lines / 4758 bytes | implementation plan | yes |
| `implementation-plans/IP-009-finops-portal-cost-allocation-policy-kernel.md` | 124 lines / 4559 bytes | implementation plan | yes |
| `implementation-plans/IP-010-finops-portal-cost-allocation-policy-domain.md` | 137 lines / 4586 bytes | implementation plan | yes |
| `implementation-plans/IP-011-finops-portal-anomaly-explanation-kernel.md` | 143 lines / 4903 bytes | implementation plan | yes |
| `implementation-plans/IP-012-finops-portal-anomaly-explanation-domain.md` | 122 lines / 4087 bytes | implementation plan | yes |
| `implementation-plans/IP-013-finops-portal-credit-ledger-kernel.md` | 143 lines / 4936 bytes | implementation plan | yes |
| `implementation-plans/IP-014-finops-portal-focus-export-pipeline.md` | 152 lines / 5900 bytes | implementation plan | yes |
| `implementation-plans/IP-015-finops-portal-quarterly-regulator-evidence-emit.md` | 148 lines / 5639 bytes | implementation plan | yes |
| `implementation-plans/IP-016-budget-alert-kernel.md` | 25 lines / 654 bytes | implementation plan | partial |
| `implementation-plans/IP-017-budget-alert-domain.md` | 24 lines / 501 bytes | implementation plan | partial |
| `implementation-plans/IP-018-forecasting-usecase.md` | 26 lines / 549 bytes | implementation plan | partial |
| `implementation-plans/IP-019-commitment-management-domain.md` | 25 lines / 655 bytes | implementation plan | partial |
| `implementation-plans/IP-020-rightsizing-recommender-worker.md` | 26 lines / 678 bytes | implementation plan | partial |
| `implementation-plans/IP-021-showback-chargeback-domain.md` | 24 lines / 550 bytes | implementation plan | partial |
| `implementation-plans/IP-022-budget-alert-rest.md` | 26 lines / 438 bytes | implementation plan | partial |
| `implementation-plans/IP-023-forecasting-rest-and-cache.md` | 24 lines / 454 bytes | implementation plan | partial |
| `implementation-plans/IP-024-commitment-management-grpc.md` | 19 lines / 435 bytes | implementation plan | partial |
| `implementation-plans/IP-025-rightsizing-rest-and-dashboard.md` | 24 lines / 559 bytes | implementation plan | partial |
| `implementation-plans/IP-026-showback-chargeback-emit.md` | 22 lines / 396 bytes | implementation plan | partial |
| `incident-playbook.md` | 136 lines / 5913 bytes | incident playbook | partial |
| `manifest.json` | 158 lines / 5737 bytes | service manifest | partial |
| `migration-playbooks/from-apptio-cloudability.md` | 189 lines / 8323 bytes | migration playbook | yes |
| `multi-region-strategy.md` | 109 lines / 4505 bytes | regional strategy | partial |
| `onboarding/finops-engineer-first-week.md` | 244 lines / 9150 bytes | onboarding | partial |
| `policy/cedar/abuse-defence.cedar` | 54 lines / 2056 bytes | Cedar policy | yes |
| `policy/cedar/action-authorization.cedar` | 51 lines / 1609 bytes | Cedar policy | yes |
| `policy/cedar/auditor-scope.cedar` | 24 lines / 673 bytes | Cedar policy | partial |
| `policy/cedar/ci-scope.cedar` | 25 lines / 692 bytes | Cedar policy | partial |
| `policy/cedar/customer-success-credit-application.cedar` | 57 lines / 1911 bytes | Cedar policy | yes |
| `policy/cedar/data-residency.cedar` | 25 lines / 920 bytes | Cedar policy | partial |
| `policy/cedar/ops-finops-dashboard-access.cedar` | 45 lines / 1526 bytes | Cedar policy | yes |
| `policy/cedar/regulator-evidence-emit.cedar` | 54 lines / 1899 bytes | Cedar policy | yes |
| `policy/cedar/schema.cedarschema.json` | 122 lines / 5039 bytes | Cedar schema | yes |
| `policy/cedar/tenant-isolation.cedar` | 59 lines / 2120 bytes | Cedar policy | yes |
| `reference-implementations/cost-query-rust-sdk.md` | 238 lines / 8568 bytes | reference implementation | partial |
| `runbooks/budget-alert-runaway-firings.md` | 281 lines / 32422 bytes | runbook | yes |
| `runbooks/cost-allocation-policy-rollback.md` | 281 lines / 32927 bytes | runbook | yes |
| `runbooks/cost-attribution-mismatch-investigation.md` | 281 lines / 34265 bytes | runbook | yes |
| `runbooks/credit-application-reconciliation.md` | 281 lines / 33279 bytes | runbook | yes |
| `runbooks/focus-export-failure.md` | 281 lines / 31069 bytes | runbook | yes |
| `runbooks/quarterly-regulator-emit-miss.md` | 281 lines / 32597 bytes | runbook | yes |
| `runbooks/reservation-recommendation-engine-stall.md` | 281 lines / 34260 bytes | runbook | yes |
| `runbooks/tenant-bill-mismatch-resolution.md` | 281 lines / 32915 bytes | runbook | yes |
| `runbooks/tenant-budget-exhausted.md` | 281 lines / 31589 bytes | runbook | yes |
| `runbooks/tenant-budget-headroom-low.md` | 281 lines / 32084 bytes | runbook | yes |
| `runbooks/tenant-cost-anomaly-spike.md` | 281 lines / 31915 bytes | runbook | yes |
| `scorecards/adr-0064-canonical-base.md` | 52 lines / 2761 bytes | scorecard | partial |
| `scorecards/adr-0083-kernel-tenant_class-invariants.md` | 59 lines / 2423 bytes | scorecard | partial |
| `scorecards/adr-0186-observability-backplane.md` | 55 lines / 2911 bytes | scorecard | partial |
| `scorecards/adr-0199-finops-substrate.md` | 52 lines / 2753 bytes | scorecard | partial |
| `scorecards/overrides.json` | 38 lines / 983 bytes | scorecard control | partial |
| `sdk-reference.md` | 194 lines / 5470 bytes | SDK plan | partial |
| `slos/anomaly-explanation-latency.openslo.yaml` | 49 lines / 1426 bytes | SLO | yes |
| `slos/cost-allocation-policy-change-latency.openslo.yaml` | 53 lines / 1664 bytes | SLO | yes |
| `slos/credit-application-correctness.openslo.yaml` | 51 lines / 1517 bytes | SLO | yes |
| `slos/drilldown-query-latency-p99.openslo.yaml` | 50 lines / 1434 bytes | SLO | yes |
| `slos/focus-export-availability.openslo.yaml` | 50 lines / 1435 bytes | SLO | yes |
| `slos/quarterly-regulator-evidence-emit-correctness.openslo.yaml` | 50 lines / 1533 bytes | SLO | yes |
| `slos/regulator-emit-availability.openslo.yaml` | 50 lines / 1486 bytes | SLO | yes |
| `slos/tenant-invoice-pdf-render-availability.openslo.yaml` | 50 lines / 1496 bytes | SLO | yes |
| `slos/tenant-invoice-render-latency.openslo.yaml` | 49 lines / 1446 bytes | SLO | yes |
| `threat-model.md` | 136 lines / 5553 bytes | threat model | partial |
| `tutorials/build-chargeback-dashboard.md` | 331 lines / 10084 bytes | tutorial | yes |

Inventory verdict: the file suite is large and mostly domain-shaped, but the "full-pack-ready" claim is overstated because implementation code, context OpenTofu modules, OS support manifest, OCI Always Free module, and several referenced evidence paths are missing.

## §3 9-dimension audit

### §3.1 Dimension 1 - internal coherence within the microservice path

D1-01 PRD purpose resolves to README purpose: both describe a FinOps UX/presentation layer over OpenCost + Mimir + FOCUS (`PRD.md:12-20`, `README.md:11-23`).
D1-02 PRD target users resolve to policy surfaces: tenant admin, ops-finops, customer success, and regulator/auditor appear in Cedar policies and incident docs (`PRD.md:22-31`, `policy/cedar/*.cedar` inventory).
D1-03 PRD in-scope invoice surface resolves to OpenAPI endpoints for invoice listing, invoice fetch, PDF stream, and finalize (`PRD.md:35-36`, `contracts/tenant-invoice-public.openapi.yaml:26-90`).
D1-04 PRD FOCUS export resolves to OpenAPI export trigger and AsyncAPI export events (`PRD.md:45-46`, `contracts/tenant-invoice-public.openapi.yaml:169`, `contracts/focus-export-internal.asyncapi.yaml:20-49`).
D1-05 PRD quarterly regulator evidence resolves to SLOs and incident playbook entries (`PRD.md:49-50`, `slos/quarterly-regulator-evidence-emit-correctness.openslo.yaml:7-47`, `incident-playbook.md:52-58`).
D1-06 PRD out-of-scope cost aggregation resolves only partially because capability availability and onboarding now make `cloud-billing` a source of truth, while PRD names `cloud-iac` for provider bill ingestion (`PRD.md:54-60`, `ADR-0330 and ADR-0331 tenant_class model:25-26`, `onboarding/finops-engineer-first-week.md:22-45`).
D1-07 The manifest depends on `cloud-iac`, not `cloud-billing`, while FAQ and onboarding call `cloud-billing` the upstream billing source (`manifest.json:69-74`, `faqs/finops-engineer-faq.md:89-96`).
D1-08 This is a P2 inconsistency because `cloud-billing` exists in the repository and the relation can be normalized without changing product intent.
D1-09 README says implementation plans are IP-001 through IP-015, but the directory contains IP-001 through IP-026 (`README.md:38`, inventory rows for IP-016..IP-026).
D1-10 README says 8 runbooks, but the directory contains 11 runbooks (`README.md:40`, inventory runbook rows).
D1-11 README says 3 Grafana dashboards plus spec, but the directory contains 5 JSON dashboards plus one Markdown spec (`README.md:41`, inventory dashboard rows).
D1-12 README says 4 Cedar policies plus schema, but the directory contains 9 Cedar policies plus schema (`README.md:43`, inventory Cedar rows).
D1-13 README says OpenAPI 3.1 and AsyncAPI 3.0, while contracts are OpenAPI 3.2.0 and AsyncAPI 3.1.0 (`README.md:47`, `contracts/tenant-invoice-public.openapi.yaml:1`, `contracts/focus-export-internal.asyncapi.yaml:1`).
D1-14 Capability tenant_class adoption matrix claims contract paths `contracts/openapi/finops-portal.yaml` and `contracts/asyncapi/finops-portal-events.yaml`; those files do not exist in the inventory (`ADR-0330 and ADR-0331 tenant_class model:121-126`).
D1-15 Capability tenant_class adoption matrix says FOCUS v1.0 at demo_trial and in invariant column set, while PRD, manifest, scorecard, ADR, contracts, and capability file consistently say FOCUS 1.3 (`ADR-0330 and ADR-0331 tenant_class model:25-27`, `PRD.md:13`, `manifest.json:13`, `decisions/ADR-FIN-001-focus-spec-adoption-and-multi-cloud-cost-normalization.md:55-58`).
D1-16 That FOCUS version conflict is P1 because FOCUS is a central product contract and export schema.
D1-17 Capacity model claims load tests exist in `tests/load_focus_export.rs`, but there is no `tests/` directory or Rust code under the microservice path (`capacity-model.md:107-115`, inventory absence).
D1-18 Benchmark doc claims a harness under `benchmarks/finopsbench/` and results under `benchmarks/results/finops-portal/<date>.csv`, but neither directory exists (`benchmarks/aws-cost-explorer-gcp-billing-apptio-vs-oyatie.md:108-119`, inventory absence).
D1-19 Incident playbook routes `FinopsPortalSloBudgetBurnFast` to `runbooks/finops-portal-deploy-rollback.md`, which is absent (`incident-playbook.md:56`, inventory absence).
D1-20 Incident playbook repeats the missing rollback runbook in the drill schedule (`incident-playbook.md:91-97`).
D1-21 Multi-region strategy also references the missing rollback runbook (`multi-region-strategy.md:72`).
D1-22 Onboarding asks a new engineer to read `runbooks/README.md`; that file is absent (`onboarding/finops-engineer-first-week.md:19`, inventory absence).
D1-23 Onboarding names runbooks such as `cost-ingest-stalled.md`, `budget-threshold-spurious.md`, `forecast-mape-degraded.md`, `anomaly-storm.md`, `dashboard-render-slow.md`, `fx-rate-stale.md`, `cloud-billing-reconciliation-drift.md`, and `chargeback-allocation-orphan.md`; those are absent (`onboarding/finops-engineer-first-week.md:19`).
D1-24 Onboarding later says to read `runbooks/anomaly-storm.md`, which is absent (`onboarding/finops-engineer-first-week.md:173`).
D1-25 Failure modes list F-05 rollback target `runbooks/finops-portal-deploy-rollback.md`; the file is absent (`failure-modes.md:57-64`).
D1-26 PRD references `evidence/storage-batch-followup-scope.json#finops-portal-ip-fanout`; no `evidence/` directory exists under the service path (`PRD.md:106-107`, inventory absence).
D1-27 Manifest references `evidence/storage-batch-followup-scope.json` and `evidence/finops-portal-full-pack-expansion-report.json`; neither exists under the service path (`manifest.json:96-97`, inventory absence).
D1-28 DPIA references `evidence/dpo-consult-finops-portal.json`; no such file exists under the service path (`dpia.md:86`, inventory absence).
D1-29 Incident playbook references incident communication templates under `evidence/incident-comms/finops-portal/`; no service-local evidence directory exists (`incident-playbook.md:107-117`).
D1-30 Incident playbook references post-mortem templates under `evidence/post-mortems/finops-portal/`; no service-local evidence directory exists (`incident-playbook.md:119-129`).
D1-31 The architecture states `FOCUS 1.3 export IS the portability format`, aligning with PRD and ADR-FIN-001 (`ARCHITECTURE.md:915-918`, `decisions/ADR-FIN-001...:55-58`).
D1-32 The architecture six-dimension matrix gives drill-down P99 <= 500 ms and invoice PDF P99 <= 3 s, while PRD NFR gives invoice first-paint <= 2 s p95 and drill-down <= 1 s p95; these are not a direct contradiction because percentile and surface differ (`ARCHITECTURE.md:920-929`, `PRD.md:62-66`).
D1-33 Capability tenant_class adoption matrix demo_trial dashboard p99 <=1.5 s is compatible with PRD p95 <=2 s but lacks percentile harmonization (`ADR-0330 and ADR-0331 tenant_class model:35-39`, `PRD.md:64-66`).
D1-34 Capability tenant_class adoption matrix paid with per_seat billing_component/paid with per_usage billing_component capacities are stronger than PRD and should be treated as tenant_class-specific refinements, not conflicts (`ADR-0330 and ADR-0331 tenant_class model:43-97`).
D1-35 The top-level benchmark doc uses "Workloads measured" language, but absence of harness and result directories means the numbers must be interpreted as targets or synthetic claims, not measured evidence (`benchmarks/...:11`, inventory absence).
D1-36 The `AUDIT-FINDINGS-2026-05-20.json` says the evidence status includes "terraform + k8s-network-policy + edge-waf + ech-config", exposing earlier tooling drift rather than a current OpenTofu context matrix (`AUDIT-FINDINGS-2026-05-20.json:1-20`).
D1-37 The existing `iac/terraform-module.tf` begins with `terraform {}` and uses provider sources under `hashicorp/`, which conflicts with the current OpenTofu doctrine unless explicitly documented as HCL-compatible OpenTofu module syntax (`iac/terraform-module.tf:1-6`).
D1-38 No internal `cross-microservice-handoffs.md` file exists even though the service has many inbound and outbound cross-service relationships.
D1-39 No service-local `supported-oses.json` exists, so OS support is claimed nowhere in machine-readable service scope.
D1-40 No service-local `src/` exists, so crate names in manifest are planned names rather than buildable crates (`manifest.json:15-67`, inventory absence).
D1-41 IP-001 through IP-015 are moderately substantive, ranging from 122 to 152 lines and mapping the original PRD slices.
D1-42 IP-016 through IP-026 are much thinner, ranging from 19 to 26 lines, and do not meet the same implementation-plan substance bar.
D1-43 The thin IP tail creates a partial contradiction with README's "all planning artifacts authored to hyperscaler bar" claim (`README.md:6-7`, inventory rows IP-016..IP-026).
D1-44 The compliance doc is very long and claims IaC evidence surfaces are present (`compliance.md:688-711`), but context-specific IaC surfaces are not present.
D1-45 The strongest local coherence cluster is FOCUS export: PRD, architecture, ADR-FIN-001, capability, OpenAPI, AsyncAPI, and SLO all point to the same bounded context.
D1-46 The second strongest cluster is tenant invoice presentation: PRD, OpenAPI, PDF SLO, runbook, dashboard, and phase doc all exist.
D1-47 The weakest local cluster is deployment substrate: Helm exists, but the required six-context OpenTofu matrix does not.
D1-48 Internal cross-reference severity summary: no P0 found, P1 found for FOCUS version and canonical-direction conflicts, P2 found for stale README and missing referenced artifacts.
D1-49 Internal coherence verdict: partial.
D1-50 Remediation direction: normalize upstream service dependency, refresh README inventory counts, rename or restate Terraform-shaped IaC as OpenTofu, add missing context modules and machine manifests, and remove or land missing evidence/runbook references.

### §3.2 Dimension 2 - outbound cross-references

D2-01 PRD outbound ADR references: ADR-0199, ADR-0174, ADR-0186, ADR-0162, ADR-0197, and FOCUS 1.3 (`PRD.md:109-116`).
D2-02 PRD outbound service references: OpenCost, Prometheus, `billing-rails`, and `cloud-iac` (`PRD.md:52-60`).
D2-03 `billing-rails` does not appear anywhere else in the repo search; this is a broken or renamed service reference (`PRD.md:57`).
D2-04 `cloud-billing` exists as a microservice and is used by onboarding/FAQ/capability availability, so `billing-rails` likely predates the current billing-service naming (`onboarding/finops-engineer-first-week.md:22-45`, repo path `microservices/cloud-billing/`).
D2-05 Manifest outbound microservices: observability, cloud-iac, tenancy, audit-chain (`manifest.json:69-74`).
D2-06 Manifest duplicate dependency array repeats the same four microservices (`manifest.json:148-153`).
D2-07 FAQ outbound microservice references: `cloud-billing` as upstream accrual ledger (`faqs/finops-engineer-faq.md:89-96`).
D2-08 Capacity model outbound refs: `multi-region-strategy.md`, `cost-model.md`, ADR-0152, ADR-0186 (`capacity-model.md:34-35`, `capacity-model.md:116-121`).
D2-09 Failure modes outbound refs: observability, audit-chain, secrets, cloud-iac, tenancy, chaos-substrate, incident playbook, and runbooks (`failure-modes.md:29-169`).
D2-10 Incident playbook outbound refs: incident-management microservice and evidence paths (`incident-playbook.md:100-129`).
D2-11 Compliance outbound refs include OpenTofu and cosign/SLSA provenance, but service-local OpenTofu context wiring is absent (`compliance.md:695-699`).
D2-12 Scorecard `adr-0186-observability-backplane.md` states 3 dashboards at `dashboards/*.grafana.json`, but the inventory has five JSON dashboards (`scorecards/adr-0186-observability-backplane.md:24`).
D2-13 Service-level ADR-FIN-001 references older ADR-finops-portal-001 and expands FOCUS 1.3 normalization (`decisions/ADR-FIN-001...:24-26`, `decisions/ADR-FIN-001...:55-79`).
D2-14 Outbound file references to `evidence/*` are unresolved within this microservice path.
D2-15 Outbound file references to missing runbooks are unresolved within this microservice path.
D2-16 Outbound paths to non-existent contract locations in capability tenant_class adoption matrix are unresolved (`ADR-0330 and ADR-0331 tenant_class model:121-126`).
D2-17 Chat line 6713 describes finops-portal as a FinOps surface with BCs cost-attribution, per-tenant-billing, showback-chargeback, budget-alerts, forecasting, commitment-management, and rightsizing-recommendations.
D2-18 Chat line 10944 describes finops-portal as AWS Cost Explorer/GCP Billing/Apptio/CloudHealth displacement with FOCUS, ML forecast, and chargeback model.
D2-19 Chat line 15698 confirms the current counterpart tuple: Vantage, Cloudability (IBM), CloudHealth (VMware).
D2-20 Chat line 15702 confirms a dedicated finops-portal codex prompt was launched for Wave 3 Batch 3.1.
D2-21 Reverse reference from `specs/master-plan-sequencing.json:440` places finops-portal in Phase 1 foundations.
D2-22 Reverse reference from `docs/standards/documentation-rigor.md:92` previously called finops-portal a borderline service with 85 files.
D2-23 Reverse references from marketplace docs feed settlement and invoice analytics into finops-portal; for example marketplace ADR says settlement feeds finops-portal and invoice presentation/chargeback analytics stay in finops-portal.
D2-24 Reverse references from ops-dashboard-control-center mention a finops integration adapter and an expected OpenAPI path that likely does not match current contract file names.
D2-25 Reverse references from developer-sdk, CRM, supply-chain-planning, and real-estate PRDs cite cost attribution or cost center integration with finops-portal.
D2-26 Missing reverse-reference surface: finops-portal lacks `cross-microservice-handoffs.md`, so many consumers have no local contract ledger to point at.
D2-27 Missing reverse-reference surface: manifest dependencies omit `cloud-billing` despite local docs describing it as upstream.
D2-28 Missing reverse-reference surface: PRD omits Vantage/Cloudability/CloudHealth even though chat and current audit bar use them.
D2-29 Missing reverse-reference surface: docs do not explain if `cloud-iac` owns per-cloud bill ingestion while `cloud-billing` owns accrual and invoice source of truth.
D2-30 ADR-FIN-001 helps resolve FOCUS schema ownership but does not close deployment/OS/OpenTofu constraints.
D2-31 The service references `analytics` ClickHouse in capability availability but manifest dependency list does not include `analytics` (`ADR-0330 and ADR-0331 tenant_class model:19-21`, `manifest.json:69-74`).
D2-32 The service references `payments` for FX snapshots in capability availability and onboarding but manifest dependency list does not include `payments` (`ADR-0330 and ADR-0331 tenant_class model:51`, `onboarding/finops-engineer-first-week.md:42-45`).
D2-33 The service references `intelligence` for anomaly explanation, forecasting, and rightsizing in architecture and IP-018, but manifest dependencies omit it (`ARCHITECTURE.md:640-642`, `implementation-plans/IP-018-forecasting-usecase.md:25`).
D2-34 The service references `secrets` in failure modes but manifest dependencies omit it (`failure-modes.md:66-73`).
D2-35 The service references `chaos-substrate` in failure modes but manifest dependencies omit it (`failure-modes.md:141-145`).
D2-36 The manifest dependency list is therefore too narrow for the full expanded feature surface.
D2-37 Outbound cross-reference severity summary: P1 for missing `cross-microservice-handoffs.md` and broken service naming where it affects buildability; P2 for stale docs and missing reverse refs.
D2-38 No outbound references to Terraform Cloud, Pulumi, or CloudFormation were found in body prose, but `terraform-module.tf` itself remains.
D2-39 Outbound docs frequently cite ADRs without line anchors; this weakens provenance but is not a contradiction.
D2-40 Current counterpart references in service docs are wider than user tuple and include AWS/GCP/Azure/Oracle/OpenCost/Kubecost/Cast.ai/Spot.io/Finout (`competitor-parity.md:16-31`).
D2-41 The wider list is useful context, but Wave 2 Batch 2.1 requires the three-counterpart union bar.
D2-42 The migration playbook is specifically from Apptio Cloudability and supports one of the three required counterpart surfaces (`migration-playbooks/from-apptio-cloudability.md:1-20`).
D2-43 There is no equivalent migration playbook from Vantage or CloudHealth.
D2-44 There is no explicit local doc that maps Cloudability after IBM acquisition naming to the current counterpart label; migration playbook uses Apptio Cloudability.
D2-45 There is no explicit local doc that maps CloudHealth to VMware/Tanzu CloudHealth naming; competitor docs use CloudHealth only.
D2-46 Reverse-reference coverage across the repo exists but is noisy and not captured locally.
D2-47 The lack of local cross-service handoff ledger is the main ownership-coherence gap for Dimension 2.
D2-48 Outbound coherence verdict: partial.
D2-49 Remediation direction: create a handoff ledger that lists every upstream/downstream microservice, artifact path, API/event dependency, ownership contract, reverse references, and stale names.
D2-50 Stop condition for this audit: enough outbound and reverse-reference evidence was gathered to classify the gaps without touching other microservices.

### §3.3 Dimension 3 - substance bar and intern-buildability

D3-01 A cold intern can understand the product goal from PRD and README in under one hour (`PRD.md:12-20`, `README.md:11-23`).
D3-02 A cold intern cannot build the service from the current microservice path because there is no `src/`, `Cargo.toml`, or service-local Rust crate code.
D3-03 The manifest names crate packages but they do not exist under this service path (`manifest.json:15-67`, inventory absence).
D3-04 IP-001 through IP-015 give a reasonable original build sequence for invoice, dashboard, policy, anomaly, credit, FOCUS, and regulator emit.
D3-05 IP-016 through IP-026 are too thin to guide implementation of budget alerts, forecasting, commitments, rightsizing, and showback/chargeback.
D3-06 The OpenAPI contract is substantive and uses OpenAPI 3.2.0 (`contracts/tenant-invoice-public.openapi.yaml:1-90`).
D3-07 The AsyncAPI contract is substantive and uses AsyncAPI 3.1.0 (`contracts/focus-export-internal.asyncapi.yaml:1-65`).
D3-08 The proto contract is present and large enough to support internal policy work (`contracts/cost-allocation-policy-internal.proto` inventory row).
D3-09 Cedar policies are broad enough to orient an implementer, with 9 policy files plus schema.
D3-10 SLOs are broad enough to orient observability, with 9 OpenSLO files.
D3-11 Runbooks are broad enough for many incident classes, with 11 files at 281 lines each.
D3-12 Buildability gap: no crate layout mapping from manifest crate names to actual workspace locations.
D3-13 Buildability gap: no local cargo invocation proof or workspace membership proof.
D3-14 Buildability gap: capacity model points to absent load-test code (`capacity-model.md:107-115`).
D3-15 Buildability gap: benchmark doc points to absent harness and result directory (`benchmarks/...:108-119`).
D3-16 Buildability gap: rollout and incident docs point to missing deploy rollback runbook (`incident-playbook.md:56`, `multi-region-strategy.md:72`, `failure-modes.md:57-64`).
D3-17 Buildability gap: onboarding commands assume `oya finops-portal` CLI subcommands without local code or CLI docs proving they exist (`onboarding/finops-engineer-first-week.md:26-46`).
D3-18 Buildability gap: ClickHouse schema is described in onboarding, but no SQL schema file exists under the service path (`onboarding/finops-engineer-first-week.md:64-76`, inventory absence).
D3-19 Buildability gap: data model terms such as `cost_event`, `resource_attribution`, `tag_set`, `budget_rule`, and `chargeback_allocation` are listed in tenant_class adoption matrix without schema files (`ADR-0330 and ADR-0331 tenant_class model:121-126`).
D3-20 Buildability gap: no CI lane manifest exists for service build/test/OS matrix.
D3-21 Buildability gap: no `supported-oses.json` or equivalent manifest field exists.
D3-22 Buildability gap: no OpenTofu context modules exist for the six deployable contexts.
D3-23 Buildability gap: no OCI Always Free module exists for demo_trial guest-on-OCI.
D3-24 Buildability gap: no state backend per context is declared under `iac/<context>/`.
D3-25 Buildability gap: no sigstore/cosign OpenTofu module signing wiring exists in context modules.
D3-26 Buildability gap: no tenant onboarding `tofu init`, `tofu plan`, `tofu apply` command path exists.
D3-27 Buildability gap: no frontend code appears under `frontend/<platform>/`.
D3-28 The service docs are highly substantive for domain semantics but not for executable implementation.
D3-29 The benchmark document uses exact numbers but lacks provenance and results files, so it fails measured-benchmark rigor.
D3-30 The compliance doc claims every build emits SBOM/provenance/signatures, but no build manifest or CI evidence exists in the service path (`compliance.md:695-703`).
D3-31 The architecture content-pass sections are broad and sometimes repeated; they help coverage but are not a substitute for exact crate interfaces (`ARCHITECTURE.md:43-89`).
D3-32 The onboarding doc is practical but references missing runbooks and assumed commands, reducing intern reliability (`onboarding/finops-engineer-first-week.md:14-46`).
D3-33 The migration playbook is useful and concrete for Cloudability migration, including drivers and risks (`migration-playbooks/from-apptio-cloudability.md:1-20`, `migration-playbooks/from-apptio-cloudability.md:188`).
D3-34 The FAQ helps distinguish `cloud-billing` and `finops-portal`, but it conflicts with PRD/manifest dependency naming (`faqs/finops-engineer-faq.md:89-96`, `PRD.md:57-60`).
D3-35 A cold intern could author docs or tests from these artifacts.
D3-36 A cold intern could not run the service from these artifacts alone.
D3-37 A cold intern could not validate OCI Always Free demo_trial Always Free from these artifacts.
D3-38 A cold intern could not validate tenant_class-1 OS support from these artifacts.
D3-39 A cold intern could not promote the service under ADR-0328 §D-20 because the deployment matrix is absent.
D3-40 The service should not claim "full-pack-ready" until executable and deployment proof exists (`README.md:6-7`, `manifest.json:91-95`).
D3-41 Substance severity: P1 for canonical buildability gaps, P2 for stale internal references, P3 for formatting/provenance refinements.
D3-42 Documentation-rigor §1.1 makes the service doc set retroactively subject to the bar (`docs/standards/documentation-rigor.md:40-56`).
D3-43 Documentation-rigor §2 requires broad suites and explicitly listed finops-portal as borderline in a prior snapshot (`docs/standards/documentation-rigor.md:62-92`).
D3-44 Documentation-rigor intern-buildability test is not passed because executable paths are missing (`docs/standards/documentation-rigor.md:133-141`).
D3-45 Documentation-rigor hyperscaler-grade sub-test is partially passed in product and failure-mode prose, but not in code, measured benchmark, or deployment evidence (`docs/standards/documentation-rigor.md:143-156`).
D3-46 The anti-pattern most visible here is "scaffold without substance" for thin IP-016..IP-026 and "line count as completion" for large but repetitive journey docs (`docs/standards/brief-template.md:1727-1751`).
D3-47 The service avoids empty docs in most core areas; the issue is not absence of docs, but unverified claims and absent executable artifacts.
D3-48 Substance verdict: partial, not intern-buildable.
D3-49 Remediation direction: add crate/code skeletons, service-local build manifest, schema files, tests, context IaC, OS manifest, and evidence artifact policy; then shrink stale claims.
D3-50 Stop condition for this audit: buildability cannot be proven from docs alone; this report flags the exact missing proofs.

### §3.4 Dimension 4 - canonical-direction alignment

D4-01 Multi-context doctrine requires six deployment contexts for service audit classification (`ADR-0328:3854-3871`, `specs/master-plan-sequencing.json:704-746`).
D4-02 The finops-portal `iac/` directory contains Helm and miscellaneous YAML/HCL but no `iac/oyatie-public-cloud`.
D4-03 No `iac/guest-on-aws` directory exists.
D4-04 No `iac/oci-guest` directory exists.
D4-05 No `iac/on-prem` directory exists.
D4-06 No `iac/colo` directory exists.
D4-07 No `iac/oyatie-iaas` directory exists.
D4-08 Multi-context classification: drifted-fixable, because no context was proven correctly out-of-scope.
D4-09 OpenTofu doctrine requires OpenTofu as engine and forbids Terraform/Pulumi/CloudFormation as the engine (`specs/master-plan-sequencing.json:747-776`).
D4-10 The service has `iac/terraform-module.tf`, whose first line is `terraform {` and provider sources are `hashicorp/kubernetes` and `hashicorp/helm` (`iac/terraform-module.tf:1-6`).
D4-11 HCL syntax is not automatically fatal, but the file name and README/AUDIT framing present it as Terraform, not OpenTofu.
D4-12 OpenTofu classification: drifted-fixable to incoherent until context modules are added and Terraform phrasing is removed.
D4-13 OS doctrine requires a per-microservice manifest for 13 tenant_class-1 OSes and 2 tenant_class-2 test-only arches (`specs/master-plan-sequencing.json:777-816`).
D4-14 No `supported-oses.json` exists.
D4-15 No `supported_oses` field exists in `manifest.json`.
D4-16 OS classification: drifted-fixable.
D4-17 Rust-strict doctrine allows Rust backend and specific non-Rust file extensions; it forbids Python/JS/TS/Ruby/Go/Java/Scala/Groovy/PHP/F# as runtime/tooling (`specs/master-plan-sequencing.json:817-856`).
D4-18 Forbidden source extension grep found no `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, `*.fs`, or `*.fsx` under the microservice path.
D4-19 No Rust source files or Cargo manifests were found under the microservice path.
D4-20 Rust-strict classification: aligned for forbidden-language absence; drifted-fixable for missing Rust implementation proof.
D4-21 OCI Always Free doctrine requires `iac/oci-guest/always-free/` per microservice (`ADR-0328:3666-3678`, `specs/master-plan-sequencing.json:857-868`).
D4-22 No `iac/oci-guest/always-free/` directory exists.
D4-23 Capability tenant_class adoption matrix demo_trial does not state OCI Always Free demo_trial = Always Free; it instead describes 4 vCPU/16 GiB nodes and Postgres 4 vCPU/16 GiB/500 GiB (`ADR-0330 and ADR-0331 tenant_class model:15-22`).
D4-24 That demo_trial shape exceeds OCI Always Free assumptions and is incoherent for guest-on-OCI Always Free demo_trial.
D4-25 OCI Always Free classification: incoherent for OCI Always Free demo_trial until the tenant_class is split into OCI Always Free and non-OCI Always Free demo_trial profiles.
D4-26 ADR-0328 §D-20.104 requires canonical build invocation `cargo build --workspace --release --all-features --locked`; service docs mention this only in general Rust references, not as a verified build lane.
D4-27 ADR-0328 §D-20.152 requires benchmark disclosure by OS, arch, deployment context, and tenant class; existing benchmark doc discloses on-prem paid with per_seat billing_component hardware only (`benchmarks/...:13`).
D4-28 ADR-0328 §D-20.153 requires OCI Always Free demo_trial Always Free reconciliation; local tenant_class adoption matrix does not provide that reconciliation.
D4-29 Brief-template §3.9 µservice audit anchors require own PRD, architecture, documentation rigor, and canonical sequence; this deliverable supplies those anchors.
D4-30 Constraint memory `feedback_multi_context_provider_agnostic_2026_05_20.md:10-39` says absence of context coverage is a P1.
D4-31 Constraint memory `feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35` says missing context IaC is P1.
D4-32 Constraint memory `feedback_os_support_matrix_2026_05_20.md:56-76` says service-level OS manifest absence is P1.
D4-33 Constraint memory `feedback_rust_strict_only_no_python_2026_05_20.md:10-64` makes forbidden language absence necessary but not sufficient for build proof.
D4-34 Constraint memory `feedback_oci_always_free_maximization_2026_05_20.md:65-82` requires per-service module and demo_trial Always Free mapping.
D4-35 Ownership coherence memory `feedback_microservice_ownership_coherence_2026_05_20.md:18-63` requires reading full local service artifacts and chat history; this audit did.
D4-36 Verify-deliverables memory `feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-64` requires substance validation beyond line count; this audit flags weak/thin sections.
D4-37 Docs-substance memory `feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` warns against scaffold and padding; this audit flags specific scaffold risks.
D4-38 Canonical-direction summary: one aligned subdimension, four drifted or incoherent subdimensions.
D4-39 Severity: P1 for missing multi-context IaC, missing OpenTofu context modules, missing OS manifest, and missing OCI Always Free module.
D4-40 Severity: P2 for missing build proof where no forbidden languages were found.
D4-41 The service can remain in audit inventory but cannot pass Wave 14 aggregation gate without remediation.
D4-42 Canonical alignment verdict: drifted-fixable with one OCI Always Free demo_trial incoherence.
D4-43 Product docs are mostly directionally compatible with canonical direction, but deployment/OS/IaC proof is not.
D4-44 Remediation priority 1: create six `iac/<context>/` modules with OpenTofu README/state/signing.
D4-45 Remediation priority 2: add `iac/oci-guest/always-free/` and restate demo_trial OCI tenant_class.
D4-46 Remediation priority 3: add `supported-oses.json` with all tenant_class-1/tenant_class-2/out-of-scope statuses.
D4-47 Remediation priority 4: add Rust crate membership or explicitly relocate build proof to workspace paths.
D4-48 Remediation priority 5: rename Terraform-shaped docs and declare OpenTofu execution surface.
D4-49 Stop condition: no further canonical source reading would change these classifications.
D4-50 Dimension verdict: P1 blocker set present.

### §3.5 Dimension 5 - industry-counterpart parity

D5-01 Headline finding: partial union coverage against Vantage, Cloudability, and CloudHealth.
D5-02 Vantage public docs show cost reports across connected cloud accounts with filtering, date ranges, forecasts, and exports (`https://docs.vantage.sh/cost_reports`).
D5-03 Vantage supports grouping by account, billing account, region, service, resource, provider, category, charge type, tagged state, and tag (`https://docs.vantage.sh/cost_reports`).
D5-04 Vantage supports percent-based cost allocation for shared resources (`https://docs.vantage.sh/cost_reports`).
D5-05 Vantage supports report CSV/PDF exports and FOCUS-schema CSV exports (`https://docs.vantage.sh/cost_reports`).
D5-06 Vantage API supports programmatic cost data, cost resources, folders, dashboards, and cost reports (`https://docs.vantage.sh/api`).
D5-07 Vantage budgets support standard and hierarchical budgets, budget periods, CSV import, budget performance, and alerts (`https://docs.vantage.sh/budgets/`).
D5-08 Vantage anomaly detection analyzes costs in Cost Reports and sends email, Slack, Teams, or Jira alerts (`https://docs.vantage.sh/cost_anomaly_alerts`).
D5-09 Vantage anomaly docs include issue creation, archive/ignore actions, and FinOps Agent investigation (`https://docs.vantage.sh/cost_anomaly_alerts`).
D5-10 Vantage Kubernetes docs include cost reports, efficiency metrics, pod/namespace/cluster views, GPU costs, and rightsizing recommendations (`https://docs.vantage.sh/kubernetes/`).
D5-11 Cloudability public pages show budgets, forecasts, Views, daily trend tracking, multi-model watsonx forecasting, and proactive alerts (`https://www.apptio.com/products/cloudability/budgets-forecasts/`).
D5-12 Cloudability feature list includes business mapping, commercial billing, container cost allocation, cost sharing, dashboards, sustainability, tagging, True Cost Explorer, Views, anomaly detection, governance, scorecards, unit economics, workload planning, commitment discounts, and rightsizing (`https://www.apptio.com/products/cloudability/budgets-forecasts/`).
D5-13 Cloudability Kubernetes rightsizing uses 10 or 30 days of resource usage and exports recommendation spreadsheets (`https://www.ibm.com/docs/en/cloudability-commercial/cloudability-essentials/saas?topic=optimize-rightsizing-kubernetes-containers`).
D5-14 Cloudability Advanced Containers include Kubernetes constructs such as clusters, nodes, namespaces, deployments, services, labels, annotations, showback/chargeback, and Kubecost data infrastructure (`https://www.ibm.com/docs/en/cloudability-commercial/cloudability-standard/saas?topic=allocation-cloudability-advanced-containers`).
D5-15 CloudHealth public docs cover on-prem, public, hybrid, and multi-cloud aggregation plus open APIs and third-party tools (`https://www.vmware.com/docs/solution-overview-vmware-tanzu-cloudhealth-simplify-cloud-financial-management`).
D5-16 CloudHealth docs describe FlexOrgs, Perspectives, cost allocation, chargeback/showback, budgets, forecasting, anomaly detection, commitments, rightsizing, governance, and automated actions (`https://www.vmware.com/docs/solution-overview-vmware-tanzu-cloudhealth-simplify-cloud-financial-management`).
D5-17 CloudHealth rightsizing docs cover EC2/EBS, Azure VMs/Azure SQL, GCE, and data center/vSphere machines (`https://www.vmware.com/docs/solution-overview-rightsize-cloud-resources-your-way-with-vmware-tanzu-cloudhealth`).
D5-18 Oyatie present: invoice presentation, drilldown dashboards, FOCUS export, credit ledger, regulator evidence, Cedar isolation, audit-chain, and cost allocation policy.
D5-19 Oyatie present from expanded docs: budget alerts, forecasting, commitments, rightsizing, showback/chargeback, cost model, and migration from Cloudability.
D5-20 Oyatie missing or weak: standard/hierarchical budget import semantics equivalent to Vantage.
D5-21 Oyatie missing or weak: Vantage-like VQL/custom report filter language.
D5-22 Oyatie missing or weak: report folder/dashboard API parity.
D5-23 Oyatie missing or weak: FinOps Agent equivalent for automated anomaly investigation in Slack/Teams.
D5-24 Oyatie missing or weak: Cloudability Views/business mapping formal model.
D5-25 Oyatie missing or weak: Cloudability scorecards.
D5-26 Oyatie missing or weak: sustainability cost/carbon reporting.
D5-27 Oyatie missing or weak: workload planning beyond high-level IP.
D5-28 Oyatie missing or weak: commitment-based discount lifecycle automation.
D5-29 Oyatie missing or weak: CloudHealth FlexOrgs/Perspectives equivalent.
D5-30 Oyatie missing or weak: dynamic governance policy engine that takes automated cloud actions.
D5-31 Oyatie missing or weak: automated resizing/start/stop/terminate actions.
D5-32 Oyatie missing or weak: on-prem/vSphere rightsizing details.
D5-33 Oyatie missing or weak: detailed Kubernetes PVC/node/GPU cost model equivalent to Vantage.
D5-34 Oyatie missing or weak: public API coverage for all advanced FinOps resources.
D5-35 Oyatie additive: cryptographic audit-chain integration.
D5-36 Oyatie additive: quarterly regulator evidence emit.
D5-37 Oyatie additive: pack-bound residency and sovereign tiers.
D5-38 Oyatie additive: Cedar row-level policy and regulatory pack overlays.
D5-39 Oyatie additive: tenant-visible audit events for dashboard render and export.
D5-40 Oyatie additive: in-house FOCUS 1.3 normalization across tenant drilldown and exports (`ADR-FIN-001:55-58`).
D5-41 Existing competitor-parity doc is broader than the required three-counterpart union and contains some unproven claims (`competitor-parity.md:16-53`).
D5-42 Existing competitor-parity doc admits optimization recommendations lag Cast.ai and Spot.io, but current top-3 union also requires rightsizing parity (`competitor-parity.md:77-88`).
D5-43 The current target tuple does not include AWS/GCP/Azure/Oracle, so PRD competitive parity section is stale for this audit (`PRD.md:77-95`).
D5-44 Feature parity matrix deliverable must reset the bar to Vantage, Cloudability, CloudHealth.
D5-45 Industry parity severity: P1 for missing OpenTofu/deployment proof that makes product parity non-deployable; P2 for advanced capability gaps.
D5-46 Headline union verdict: partial.
D5-47 The product domain is correct for the top-3 counterpart set.
D5-48 The local artifacts do not yet meet the full union of the top-3.
D5-49 Remediation direction: implement custom report/filter language, full budget hierarchy, business mapping, automated governance/action loops, Kubernetes/PVC/GPU detail, and commitment lifecycle.
D5-50 Stop condition: current public sources are sufficient to establish union-coverage gaps.

### §3.6 Dimension 6 - multi-context deployment support

D6-01 Required contexts from canonical source: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` (`specs/master-plan-sequencing.json:704-746`).
D6-02 Required directory naming in master plan differs slightly for sixth context: `iac/oyatie-iaas` (`specs/master-plan-sequencing.json:704-746`).
D6-03 Context `oyatie-public-cloud`: no `iac/oyatie-public-cloud/` directory found.
D6-04 Context `guest-on-aws`: no `iac/guest-on-aws/` directory found.
D6-05 Context `guest-on-oci`: no `iac/oci-guest/` directory found.
D6-06 Context `on-prem`: no `iac/on-prem/` directory found.
D6-07 Context `colo`: no `iac/colo/` directory found.
D6-08 Context `oyatie-as-cloud-provider`: no `iac/oyatie-iaas/` or semantically equivalent directory found.
D6-09 Existing deployable substrate: Helm chart under `iac/helm/finops-portal/`.
D6-10 Existing deployable substrate: Kubernetes network policy under `iac/k8s-network-policy.yaml`.
D6-11 Existing deployable substrate: edge WAF YAML under `iac/edge-waf.yaml`.
D6-12 Existing deployable substrate: OpenBao policy under `iac/openbao-policy.hcl`.
D6-13 Existing deployable substrate: secret bindings under `iac/secret-bindings.yaml`.
D6-14 Existing deployable substrate: PQC certificate config under `iac/pqc-cert.yaml`.
D6-15 Existing deployable substrate is generic Kubernetes/Helm, not six-context deployment support.
D6-16 No context-specific README files exist.
D6-17 No context-specific variables files exist.
D6-18 No context-specific outputs files exist.
D6-19 No context-specific versions files exist.
D6-20 No context-specific state backend declarations exist.
D6-21 No tenant onboarding command sequence exists for any context.
D6-22 No context explicitly marked correctly not applicable.
D6-23 Product docs assume Kubernetes as runtime (`ARCHITECTURE.md:585-588`).
D6-24 Kubernetes runtime can be valid in all six contexts, but the docs do not show context-specific substrate details.
D6-25 The on-prem/colo story appears only as prose in multi-region/compliance documents, not deployable modules.
D6-26 Cloud vendor APIs directly from business logic were not found because no business logic source exists.
D6-27 Direct cloud provider API coupling is therefore unproven rather than present.
D6-28 Cloud-specific bill ingestion is explicitly delegated to `cloud-iac` via OpenTofu modules (`PRD.md:59-60`).
D6-29 Delegation to cloud-iac is acceptable only if local deployment context still exists for finops-portal.
D6-30 No proof exists that finops-portal can be deployed in guest-on-AWS while remaining provider-agnostic.
D6-31 No proof exists that finops-portal can be deployed in guest-on-OCI.
D6-32 No proof exists that finops-portal can be deployed on-prem.
D6-33 No proof exists that finops-portal can be deployed in colo.
D6-34 No proof exists that finops-portal can run when Oyatie itself is the cloud provider.
D6-35 No proof exists that public-cloud deployment state uses the required state backend.
D6-36 No proof exists that guest-on-AWS deployment state uses the required state backend.
D6-37 No proof exists that guest-on-OCI deployment state uses OCI Object Storage/Autonomous DB.
D6-38 No proof exists that on-prem/colo deployment state uses the required S3-compatible or customer-held backend.
D6-39 No proof exists that tenant-owned AWS/OCI credentials are treated only as provider configuration, not business logic coupling.
D6-40 Multi-context support table: all six unsupported by deployable IaC evidence.
D6-41 Multi-context support can be recovered without product redesign because Helm and K8s primitives exist.
D6-42 Severity: P1 because ADR-0328 marks missing context IaC as a cross-cutting blocker for non-P0 services.
D6-43 Correctly not applicable contexts: none documented.
D6-44 If the service is truly deployable only in shared Oyatie cloud, the PRD/manifest must say so and justify out-of-scope contexts; they do not.
D6-45 Current deployable context assumption from user instruction remains all six unless audit finds otherwise; this audit finds all six intended but not evidenced.
D6-46 Dimension verdict: not supported.
D6-47 Remediation: create `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`.
D6-48 Remediation: each context needs OpenTofu module files, state backend, signing, README, and tenant onboarding commands.
D6-49 Remediation: add a deployment matrix table in README or manifest after modules exist.
D6-50 Stop condition: no context module exists, so deeper line reading cannot prove support.

### §3.7 Dimension 7 - OpenTofu IaC coverage

D7-01 Required engine: OpenTofu (`specs/master-plan-sequencing.json:747-776`).
D7-02 Forbidden engines: Terraform, Pulumi, CloudFormation, ARM templates (`specs/master-plan-sequencing.json:747-776`).
D7-03 Required forbidden pattern checks: `null_resource`, `local-exec`, SSH provisioners, hand-edited state, unsigned modules (`ADR-0328:3897-3939`).
D7-04 Existing IaC root files: `ech-config.yaml`, `edge-waf.yaml`, `k8s-network-policy.yaml`, `openbao-policy.hcl`, `pqc-cert.yaml`, `secret-bindings.yaml`, `terraform-module.tf`.
D7-05 Existing Helm chart files: Chart, values, overlays, deployment, service, HPA, networkpolicy, prometheusrule, servicemonitor, helpers.
D7-06 No `iac/<context>/main.tf` exists.
D7-07 No `iac/<context>/variables.tf` exists.
D7-08 No `iac/<context>/outputs.tf` exists.
D7-09 No `iac/<context>/versions.tf` exists.
D7-10 No context README exists.
D7-11 No context state backend file exists.
D7-12 `iac/terraform-module.tf:1` starts with `terraform {`.
D7-13 `iac/terraform-module.tf:2` requires version `>= 1.7.0`, which is Terraform phrasing and not the canonical OpenTofu pin.
D7-14 Provider sources in `iac/terraform-module.tf:4-5` use `hashicorp/kubernetes` and `hashicorp/helm`.
D7-15 The file uses `kubernetes_namespace` and `helm_release`, which are plausible OpenTofu resources but not context-specific modules (`iac/terraform-module.tf:19-40`).
D7-16 Grep found no `pulumi` references in the target path.
D7-17 Grep found no `CloudFormation` references in the target path.
D7-18 Grep found no `ARM template` references in the target path.
D7-19 Grep found no `null_resource` references in the target path.
D7-20 Grep found no `local-exec` references in the target path.
D7-21 Grep found no SSH provisioner references in the target path.
D7-22 Grep found no hand-edited tfstate reference in the target path.
D7-23 Grep found `sigstore_fulcio: true` only in `iac/pqc-cert.yaml:11`, not context module signing wiring.
D7-24 Compliance claims cosign signature/provenance for image promotion (`compliance.md:695-699`).
D7-25 Compliance claim is not enough to satisfy ADR-0039/OpenTofu module signing wiring.
D7-26 No `cosign` command or module signature manifest exists in IaC context files.
D7-27 No OpenTofu lock file or provider version pin per context exists.
D7-28 No remote state backend per context exists.
D7-29 No tenant onboarding `tofu init` command exists in service docs.
D7-30 No tenant onboarding `tofu plan` command exists in service docs.
D7-31 No tenant onboarding `tofu apply` command exists in service docs.
D7-32 README does not mention OpenTofu context deployment; it mentions only Helm (`README.md:46`).
D7-33 PRD delegates provider bill ingestion to cloud-iac via OpenTofu, but not service deployment (`PRD.md:59-60`).
D7-34 The Helm chart can be nested under OpenTofu modules, but today it is standalone.
D7-35 Existing `iac/helm` is useful substrate, not sufficient IaC coverage.
D7-36 Existing `iac/openbao-policy.hcl` is policy material, not context deployment.
D7-37 Existing `iac/edge-waf.yaml` is edge config, not context deployment.
D7-38 Existing `iac/ech-config.yaml` is not context deployment.
D7-39 Existing `iac/secret-bindings.yaml` is not context deployment.
D7-40 Existing `iac/pqc-cert.yaml` is not context deployment.
D7-41 P1 finding: OpenTofu context module matrix is absent.
D7-42 P1 finding: Terraform-shaped file name and block conflict with current canonical naming and should be remediated.
D7-43 P2 finding: sigstore/cosign evidence is prose, not wiring.
D7-44 P2 finding: state backend design is absent.
D7-45 P2 finding: tenant onboarding command path is absent.
D7-46 No forbidden local-exec or SSH provisioner pattern was found, which limits the blast radius.
D7-47 IaC verdict: drifted-fixable, not compliant.
D7-48 Remediation: split generic Helm module from six OpenTofu context wrappers.
D7-49 Remediation: add module signatures, pinned versions, and state backend declarations.
D7-50 Stop condition: current path cannot pass OpenTofu audit until files are created.

### §3.8 Dimension 8 - OS support matrix

D8-01 Required manifest: per-microservice OS support manifest (`specs/master-plan-sequencing.json:777-816`).
D8-02 Manifest status: no `supported-oses.json` file exists under finops-portal.
D8-03 Manifest status: no `supported_oses` field exists in `manifest.json`.
D8-04 tenant_class-1 Talos Linux status: not declared.
D8-05 tenant_class-1 RHEL 9.x+ status: not declared.
D8-06 tenant_class-1 Oracle Linux 9.x+ status: not declared.
D8-07 tenant_class-1 SLES 15 SP6+ status: not declared.
D8-08 tenant_class-1 Ubuntu 24.04 LTS+ status: not declared.
D8-09 tenant_class-1 Debian 13+ status: not declared.
D8-10 tenant_class-1 Rocky Linux 9.x+ status: not declared.
D8-11 tenant_class-1 AlmaLinux 9.x+ status: not declared.
D8-12 tenant_class-1 CentOS Stream 10+ status: not declared.
D8-13 tenant_class-1 Amazon Linux 2023+ status: not declared.
D8-14 tenant_class-1 Flatcar status: not declared.
D8-15 tenant_class-1 Photon OS 5.x+ status: not declared.
D8-16 tenant_class-1 macOS Apple Silicon M5+ status: not declared.
D8-17 tenant_class-2 ppc64le status: not declared as test-only.
D8-18 tenant_class-2 s390x status: not declared as test-only.
D8-19 Out-of-scope Intel macOS status: not explicitly declared out-of-scope.
D8-20 Out-of-scope pre-M5 Apple Silicon status: not explicitly declared out-of-scope.
D8-21 Out-of-scope FreeBSD status: not explicitly declared out-of-scope.
D8-22 Out-of-scope OpenBSD status: not explicitly declared out-of-scope.
D8-23 Out-of-scope Windows Server status: not explicitly declared out-of-scope.
D8-24 Out-of-scope Solaris status: not explicitly declared out-of-scope.
D8-25 Package format RPM: not declared.
D8-26 Package format DEB: not declared.
D8-27 Package format `.pkg`: not declared.
D8-28 Package format Homebrew: not declared.
D8-29 Package format Talos extension: not declared.
D8-30 Package format Flatcar extension: not declared.
D8-31 Container image format: implied by Helm/Kubernetes, not declared in OS matrix.
D8-32 CI lane for Talos: not declared.
D8-33 CI lane for RHEL: not declared.
D8-34 CI lane for Oracle Linux: not declared.
D8-35 CI lane for SLES: not declared.
D8-36 CI lane for Ubuntu: not declared.
D8-37 CI lane for Debian: not declared.
D8-38 CI lane for Rocky: not declared.
D8-39 CI lane for AlmaLinux: not declared.
D8-40 CI lane for CentOS Stream: not declared.
D8-41 CI lane for Amazon Linux: not declared.
D8-42 CI lane for Flatcar: not declared.
D8-43 CI lane for Photon: not declared.
D8-44 CI lane for macOS Apple Silicon M5+: not declared.
D8-45 Architecture matrix x86_64/arm64: not declared.
D8-46 tenant_class-2 architecture ppc64le/s390x test-only policy: not declared.
D8-47 Existing Helm chart is OS-agnostic at Kubernetes layer but does not satisfy OS matrix proof.
D8-48 Severity: P1, per OS support memory and ADR-0328.
D8-49 Dimension verdict: missing.
D8-50 Remediation: add `supported-oses.json` with per-OS build/test/package status and explicit out-of-scope entries.

### §3.9 Dimension 9 - Rust-strict language coverage

D9-01 Rust-strict source scan checked extensions: `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, `.fsx`.
D9-02 Result: no forbidden source files found under `microservices/finops-portal/`.
D9-03 Rust source scan checked `.rs` and `Cargo.toml`.
D9-04 Result: no service-local Rust source files found.
D9-05 Result: no service-local Cargo manifest found.
D9-06 Authorized non-Rust docs and schemas present: Markdown, YAML, JSON, proto, OpenSLO YAML, Cedar, HCL, and `.tf`.
D9-07 `.tf` extension is allowed by language policy as infrastructure code, but engine/terminology must be OpenTofu.
D9-08 `iac/terraform-module.tf` is therefore not a language-policy violation by extension.
D9-09 `iac/terraform-module.tf` is an IaC policy violation by naming and engine framing.
D9-10 Proto contract is authorized (`contracts/cost-allocation-policy-internal.proto`).
D9-11 YAML OpenAPI contract is authorized (`contracts/tenant-invoice-public.openapi.yaml`).
D9-12 YAML AsyncAPI contract is authorized (`contracts/focus-export-internal.asyncapi.yaml`).
D9-13 OpenSLO YAML files are authorized.
D9-14 Cedar policy files are authorized.
D9-15 JSON dashboards and schema files are authorized.
D9-16 Markdown docs are authorized.
D9-17 HCL policy file is not named in the strict allowlist, but it is OpenBao policy material rather than runtime code; treat as authorized security/IaC adjacent artifact needing explicit policy allowance.
D9-18 No Swift frontend files found.
D9-19 No Kotlin frontend files found.
D9-20 No WinUI3 frontend files found.
D9-21 No Leptos/Rust frontend files found.
D9-22 No `frontend/<platform>/` directory found.
D9-23 Frontend product exists only as docs and Grafana embed plans.
D9-24 Build invocation required by ADR-0328 §D-20.104 is `cargo build --workspace --release --all-features --locked`.
D9-25 No service-local doc proves that invocation has been run for finops-portal.
D9-26 Existing reference implementation is Markdown containing Rust snippets, not a crate (`reference-implementations/cost-query-rust-sdk.md`).
D9-27 Existing tutorial commands assume CLI availability but do not ship Rust code (`tutorials/build-chargeback-dashboard.md` inventory row).
D9-28 Existing SDK reference authority says OpenAPI 3.1 even though the OpenAPI contract is 3.2.0 (`sdk-reference.md:5`, `contracts/tenant-invoice-public.openapi.yaml:1`).
D9-29 No generated SDK output was found.
D9-30 No unauthorized generated SDK output was found.
D9-31 No JavaScript app was found.
D9-32 No TypeScript custom node or frontend was found.
D9-33 No Python tooling was found.
D9-34 No Go tooling was found.
D9-35 No Java/Scala/Groovy build tooling was found.
D9-36 No Ruby/PHP/F# source was found.
D9-37 Rust-strict severity: no P1 forbidden-language finding.
D9-38 Rust-strict buildability severity: P2 because implementation proof is absent.
D9-39 Language-policy interaction with OpenTofu: `.tf` is allowed, but `terraform` engine claims must be corrected.
D9-40 Language-policy interaction with frontend: Swift/Kotlin/WinUI3/Leptos are allowed only in proper frontend paths, but no frontend exists.
D9-41 The product's Grafana-embedded dashboard strategy is not itself a forbidden frontend.
D9-42 The service should decide whether the UI is Rust/Leptos, Swift/Kotlin/WinUI3 client, or Grafana-only.
D9-43 Without that decision, frontend parity against Vantage/Cloudability/CloudHealth remains a docs-only claim.
D9-44 Rust backend architecture references exist in manifest crate names and IPs.
D9-45 The absence of source files means no forbidden language drift was introduced by this service pack.
D9-46 The absence of source files also means no compile/test evidence can be claimed.
D9-47 Dimension verdict: aligned on forbidden-language absence, incomplete on Rust implementation proof.
D9-48 Remediation: add or link service-owned Rust crates in the workspace with cargo build/test evidence.
D9-49 Remediation: add frontend path policy if a native or web frontend will exist.
D9-50 Stop condition: grep evidence is complete for the target path.

## §4 Findings summary

| severity | dimension | short description | citation | remediation hint |
|---|---|---|---|---|
| P1 | D4/D6/D7 | Six required deployment contexts have no service OpenTofu modules. | `specs/master-plan-sequencing.json:704-776`; `iac/` inventory | Add six `iac/<context>/` OpenTofu wrappers. |
| P1 | D7 | IaC file is Terraform-shaped rather than OpenTofu-context-shaped. | `microservices/finops-portal/iac/terraform-module.tf:1-6` | Rename/reframe as OpenTofu and add versions/state/signing. |
| P1 | D8 | Service has no supported OS manifest. | `specs/master-plan-sequencing.json:777-816`; `manifest.json:1-158` | Add `supported-oses.json`. |
| P1 | D4/D6 | OCI Always Free module and demo_trial reconciliation are absent. | `ADR-0328:3666-3678`; `ADR-0330 and ADR-0331 tenant_class model:15-22` | Add `iac/oci-guest/always-free/` and OCI Always Free demo_trial tenant_class. |
| P1 | D1/D5 | FOCUS version conflict between tenant_class adoption matrix and rest of service. | `ADR-0330 and ADR-0331 tenant_class model:25-27`; `PRD.md:13`; `ADR-FIN-001:55-58` | Normalize all service docs to FOCUS 1.3. |
| P1 | D3 | Service is not buildable from local path: no Rust source or Cargo manifest. | inventory absence; `manifest.json:15-67` | Add/link crates and cargo evidence. |
| P2 | D1/D2 | README inventory is stale for IPs, runbooks, dashboards, policies, contracts. | `README.md:38-47`; inventory table | Refresh README counts and versions. |
| P2 | D1 | Missing deployment rollback runbook referenced by incident/failure/multi-region docs. | `incident-playbook.md:56`; `failure-modes.md:57-64`; `multi-region-strategy.md:72` | Add runbook or retarget references. |
| P2 | D1/D3 | Benchmark harness and results directories are absent. | `benchmarks/...:108-119` | Add harness/results or label numbers as targets. |
| P2 | D1/D2 | Evidence paths are referenced but service-local evidence directory is absent. | `PRD.md:106-107`; `manifest.json:96-97`; `dpia.md:86` | Add evidence artifacts or remove claims. |
| P2 | D1/D2 | Capability tenant_class adoption matrix references non-existent contract paths. | `ADR-0330 and ADR-0331 tenant_class model:121-126` | Point to current `contracts/*` files. |
| P2 | D2 | Manifest omits expanded upstreams `cloud-billing`, `payments`, `analytics`, `intelligence`, `secrets`, `chaos-substrate`. | `manifest.json:69-74`; `faqs/finops-engineer-faq.md:89-96`; `ARCHITECTURE.md:640-642` | Add dependency/handoff matrix. |
| P2 | D2 | No `cross-microservice-handoffs.md` despite many cross-service references. | inventory absence | Add local handoff ledger. |
| P2 | D3 | Thin IP-016..IP-026 do not match original IP substance. | inventory rows for IP-016..IP-026 | Expand plans with data model, APIs, tests. |
| P2 | D5 | Counterpart union gaps remain for VQL, budget hierarchy/import, business mapping, scorecards, automation, sustainability. | public docs cited in §3.5 | Add roadmap and contracts. |
| P2 | D9 | Rust build proof absent even though no forbidden source files exist. | inventory absence | Add cargo build lane and source paths. |
| P3 | D2 | PRD competitor list is stale for current Vantage/Cloudability/CloudHealth audit tuple. | `PRD.md:77-95`; chat lines 15698-15702 | Update competitor section after Wave 14. |
| P3 | D3 | Architecture has broad repeated content-pass blocks. | `ARCHITECTURE.md:43-89` | Consolidate after executable proofs land. |
| P3 | D7 | Sigstore/cosign appears in prose, not module wiring. | `compliance.md:695-699`; `iac/pqc-cert.yaml:11` | Add signed module metadata. |

Severity counts: P0=0; P1=6; P2=10; P3=3.

## §5 Open questions for Wave 14 aggregation

1. Should `cloud-billing` replace `billing-rails` and supplement `cloud-iac` in the canonical finops-portal dependency graph?
2. Should the service continue Grafana-embedded UX only, or should it own a Leptos/native frontend path under the current frontend allowlist?
3. Should demo_trial be a universal tenant_class or split into demo_trial OCI Always Free and non-OCI Always Free demo_trial?
4. Should Vantage/Cloudability/CloudHealth become the PRD competitor set, replacing the older hyperscaler-native-only PRD list?
5. Should the existing benchmark numbers be retained as target numbers or invalidated until a real harness/result set lands?
6. Should `iac/terraform-module.tf` be rewritten into context-specific OpenTofu modules or kept as a shared generic Helm wrapper after renaming?
7. Should service-local evidence paths be materialized, or should evidence be centralized under repo-level evidence with explicit path rules?
8. Should IP-016..IP-026 be promoted into the primary phase plan, or should they remain later roadmap slices outside the original P00-P04 plan?
9. Should manifest `tenant_class_adoption` use limited/minimal risk labels or demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating product tiers?
10. Should this service own automated optimization actions, or should CloudHealth-style stop/start/resize actions belong to cloud-iac with finops-portal as recommender only?

<!-- ORCHESTRATOR REPORT
  microservice: finops-portal
  deliverables_landed: microservices/finops-portal/coherence-audit-2026-05-20.md (747 lines); microservices/finops-portal/feature-parity-matrix-2026-05-20.md (430 lines); microservices/finops-portal/performance-benchmark-numbers-2026-05-20.md (312 lines); microservices/finops-portal/capability-adoption-deltas-vs-counterparts-2026-05-20.md (431 lines)
  inventory_files_seen: 161
  inventory_lines_read: 27245
  chat_history_matches_processed: 10
  findings_p0: 0
  findings_p1: 6
  findings_p2: 10
  findings_p3: 3
  top_3_counterparts_confirmed: Vantage / Cloudability (IBM) / CloudHealth (VMware)
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1920
-->
