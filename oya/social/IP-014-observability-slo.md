---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-014-observability-slo
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + ops-sre-reliability
acceptance_lanes: [openslo-validate, dashboard-json-validate, prometheus-rule-test]
---

# IP-014: Observability and SLO wiring

## A. Problem
Social cannot make promotion or counterpart parity claims without measurable SLOs and dashboards for feed, post, profile, follow, search, moderation, CSAM, notification, and minor-protection paths.

## B. Approach
Bind every existing OpenSLO file to Grafana dashboards, Prometheus rules, runbooks, and PRD performance targets. Keep target numbers as target evidence unless source implementation and benchmark evidence exist.

## C. Deliverables
| Artifact | Role |
|---|---|
| `slos/*.openslo.yaml` | SLO definitions for social paths. |
| `dashboards/*.json` | Dashboard evidence for feed, moderation, abuse, CSAM, federation, and minor protection. |
| `iac/helm/social/templates/prometheusrule.yaml` and `servicemonitor.yaml` | Runtime alert and scrape bindings. |
| `performance-benchmark-numbers-2026-05-20.md` | Target benchmark source. |

## D. Ordered implementation steps
1. Validate every OpenSLO file.
2. Validate every dashboard JSON file.
3. Cross-link SLO names to PrometheusRule expressions.
4. Check every PRD performance target has an SLO or documented follow-up.
5. Link every SLO alert to a runbook.
6. Mark missing runtime measurements as targets, not production proof.
7. Run chart-rendered alert validation.

## E. Acceptance
- OpenSLO validation passes for all files under `microservices/social/slos/`.
- JSON validation passes for all files under `microservices/social/dashboards/`.
- `helm lint microservices/social/iac/helm/social` passes.
- Every PrometheusRule alert references a known SLO/runbook.
- `performance-benchmark-numbers-2026-05-20.md` and `PRD.md` targets remain consistent.

## F. Evidence
- SLOs: `slos/`.
- Dashboards: `dashboards/`.
- IaC: `iac/helm/social/templates/prometheusrule.yaml`, `servicemonitor.yaml`.
- Benchmarks: `performance-benchmark-numbers-2026-05-20.md`.

## G. Counterpart comparison
Public counterparts rarely expose SLOs to customers. X, Instagram, TikTok, and Snapchat prove user-scale pressure; Oyatie's differentiator is promotion gated by explicit SLO files, dashboards, and runbook-backed burn alerts.

## H. Foundation delivery expansion
- Deliverable detail: each OpenSLO file maps to a Prometheus rule, dashboard panel, runbook, and PRD target.
- Deliverable detail: dashboards cover feed, moderation, abuse, CSAM, federation, minor protection, and experience health.
- Deliverable detail: Prometheus rules distinguish burn-rate, correctness, backlog, latency, and availability alerts.
- Deliverable detail: ServiceMonitor scrape labels align with rendered Helm workloads.
- Deliverable detail: missing runtime measurements are explicitly marked as targets until implementation exists.
- Deliverable detail: benchmark numbers distinguish load target, measured result, and evidence source.
- Deliverable detail: runbook links include trigger name and first diagnostic query.
- Deliverable detail: Slack community uptime and moderation responsiveness are counterpart pressure for SLO visibility.

## I. Acceptance expansion
- Acceptance detail: OpenSLO validation must parse every `slos/*.openslo.yaml`.
- Acceptance detail: dashboard JSON validation must parse every social dashboard.
- Acceptance detail: chart lint must validate PrometheusRule and ServiceMonitor templates.
- Acceptance detail: alert-to-runbook scan must reject alerts without runbook links.
- Acceptance detail: performance benchmark review must not label targets as measured production facts.
- Acceptance detail: PRD target mapping must include feed, post, notification, moderation, search, and minor-protection surfaces.
- Acceptance detail: remediation notes must record the social foundation IP repair count.
- Acceptance detail: Slack, X, Instagram, TikTok, and Snapchat comparisons must remain scale/operability pressure, not proof.

## J. Evidence expansion
- Evidence detail: capture OpenSLO validation output.
- Evidence detail: capture dashboard JSON validation output.
- Evidence detail: capture Helm lint output for observability templates.
- Evidence detail: cite `performance-benchmark-numbers-2026-05-20.md`.
- Evidence detail: cite `dashboards/moderation-and-safety.json` and `dashboards/feed-experience.json`.
- Evidence detail: cite `iac/helm/social/templates/prometheusrule.yaml` and `servicemonitor.yaml`.
- Evidence detail: cite Slack as community operations pressure requiring transparent SLO/runbook links.
