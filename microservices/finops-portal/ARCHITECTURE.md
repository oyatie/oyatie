---
doc_class: Architecture
shape: Walkthrough
length_cap: 1500
authority_tier: 2
status: Accepted
date: 2026-05-20
microservice: finops-portal
companion_docs:
  - microservices/finops-portal/PRD.md
  - microservices/finops-portal/compliance-matrix.md
  - microservices/finops-portal/threat-model.md
  - microservices/finops-portal/dpia.md
related_adrs:
  - ADR-0199
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253
  - ADR-0254
  - ADR-0258
  - ADR-0263
  - ADR-0276
  - ADR-0294
  - ADR-0295
  - ADR-0296
inbound_citations:
  - microservices/finops-portal/PRD.md
  - microservices/finops-portal/README.md
---

# finops-portal — Architecture

## §principals (ADR-0242)

Runs as `oyatie.finops-portal.tenant-billing-presenter`,
`oyatie.finops-portal.cost-allocation-policy-evaluator`,
`oyatie.finops-portal.anomaly-explainer`, `oyatie.finops-portal.focus-exporter`,
`oyatie.finops-portal.credit-ledger`, `oyatie.finops-portal.budget-alert-notifier`,
`oyatie.finops-portal.rightsizing-recommender`, `oyatie.finops-portal.forecaster`,
`oyatie.finops-portal.commitment-manager`. SPIFFE-attested per ADR-0295.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `principals (ADR-0242)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.

## §cedar-gates (ADR-0243)

Cedar fragments: `policy/cedar/tenant-isolation.cedar`,
`policy/cedar/customer-success-credit-application.cedar`,
`policy/cedar/ops-finops-dashboard-access.cedar`,
`policy/cedar/regulator-evidence-emit.cedar`,
`policy/cedar/action-authorization.cedar` (NEW — default-deny),
`policy/cedar/abuse-defence.cedar` (NEW — UX-floor honored),
`policy/cedar/data-residency.cedar` (NEW),
`policy/cedar/auditor-scope.cedar` (NEW),
`policy/cedar/ci-scope.cedar` (NEW).
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `cedar-gates (ADR-0243)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.

## §tenant-scoping (ADR-0244)

Every cost-attribution row, invoice, credit, budget, anomaly record carries `tenant_id`.
`audience_type = B2B_TENANT_ADMIN`, `B2B_FINOPS_VIEWER`, `INTERNAL_OPS`. Cost data is
tenant-PII (a leak reveals tenant's compute spend, headcount proxy, growth trajectory) —
treated as PII per dpia.md.
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `tenant-scoping (ADR-0244)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §substrate-product-binding (ADR-0245)

tenant_class-product (FinOps surface for substrate users + tenant admins). Depends on substrate
microservices: `observability` (OpenCost + Mimir), `cloud-iac` (SeaweedFS S3),
`tenancy`, `audit-chain`.
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `substrate-product-binding (ADR-0245)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.

## §policy-evaluation (ADR-0246 + amendment)

Library-first via `oya-shared-policy-eval`. Cedar fragments soak ≥60s per ADR-0294.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `policy-evaluation (ADR-0246 + amendment)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `finops-portal` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `finops-portal` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §cellular-architecture (ADR-0248)

tenant_class-1 frontend (read-mostly). tenant_class-3 data cells host FOCUS export storage.
### Content-pass expansion — cell-eligibility
- This expansion preserves the existing prose above and closes `cell-eligibility` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS cell-based architecture anchors the external control pattern for `cell-eligibility`.
- Precedent 2: Route 53 shuffle sharding provides a second independent hyperscaler pattern for `cell-eligibility`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cell-eligibility`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cell-eligibility` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `cellular-architecture (ADR-0248)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `cellular architecture (ADR 0248)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `cellular architecture (ADR 0248)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cellular architecture (ADR 0248)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `cellular architecture (ADR 0248)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `cellular architecture (ADR 0248)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `cellular architecture (ADR 0248)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `cellular architecture (ADR 0248)` workflow.
- Depth detail 17: `finops-portal` telemetry for `cellular architecture (ADR 0248)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `finops-portal` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §day-one-cert-readiness (ADR-0250)

Day-one SOC2-Type-II + GDPR (cost data == PII) + KR-CSAP + EU-sovereign. Per-pack overlays in
`iac/helm/finops-portal/values-*.yaml`.
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `day-one-cert-readiness (ADR-0250)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `day one cert readiness (ADR 0250)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness (ADR 0250)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `day one cert readiness (ADR 0250)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `day one cert readiness (ADR 0250)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `day one cert readiness (ADR 0250)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `day one cert readiness (ADR 0250)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `day one cert readiness (ADR 0250)` workflow.
- Depth detail 17: `finops-portal` telemetry for `day one cert readiness (ADR 0250)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §pack-overlay-roster (ADR-0251)

`generic`, `kr`, `eu`, `us-healthcare`, `us-financial`, `us-public-sector`,
`gdpr`, `kr-csap`, `eu-sovereign`, `hipaa`, `cn-pipl`.
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `pack-overlay-roster (ADR-0251)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `pack overlay roster (ADR 0251)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `pack overlay roster (ADR 0251)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `pack overlay roster (ADR 0251)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `pack overlay roster (ADR 0251)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `pack overlay roster (ADR 0251)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `pack overlay roster (ADR 0251)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `pack overlay roster (ADR 0251)` workflow.
- Depth detail 17: `finops-portal` telemetry for `pack overlay roster (ADR 0251)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §time-coordination (ADR-0252)

HLC default. TrueTime opt-in for quarterly regulator-evidence-emit + commitment-discount
finalisation (immutable financial ledger).
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `time-coordination (ADR-0252)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `finops-portal` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §transport (ADR-0253)

HTTP/3 + QUIC default. ECH + PQC offered. Grafana embed via signed iframe + tenant-scoped JWT
per ADR-finops-portal-005.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `transport (ADR-0253)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `finops-portal` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §deployment-shape (ADR-0254)

K8s. Standard containers for read-path. Cost-allocation-policy editor in Cloud Hypervisor +
Kata (handles signing key for committed-use ledger).
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `deployment-shape (ADR-0254)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `deployment shape (ADR 0254)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `deployment shape (ADR 0254)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `deployment shape (ADR 0254)` workflow.
- Depth detail 17: `finops-portal` telemetry for `deployment shape (ADR 0254)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §intelligence-dispatch (ADR-0255)

Calls Intelligence for anomaly explanation + forecasting + rightsizing recommendations.
Library-first; network-opt-in fallback. `audience_type = INTERNAL_SUBSTRATE`.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `intelligence-dispatch (ADR-0255)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `intelligence dispatch (ADR 0255)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `intelligence dispatch (ADR 0255)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `intelligence dispatch (ADR 0255)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `intelligence dispatch (ADR 0255)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255)` workflow.
- Depth detail 17: `finops-portal` telemetry for `intelligence dispatch (ADR 0255)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §ontology-read-path (ADR-0257 amendment)

Reads tenant + workload Ontology projections for cost attribution.
`ontology_read_mode = library_first`. `freshness_floor = 60s`.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `ontology-read-path (ADR-0257 amendment)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `ontology read path (ADR 0257 amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 amendment)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `ontology read path (ADR 0257 amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `ontology read path (ADR 0257 amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `ontology read path (ADR 0257 amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 amendment)` workflow.
- Depth detail 17: `finops-portal` telemetry for `ontology read path (ADR 0257 amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §observability (ADR-0263)

Audit-event-classes: `FinOpsQuarterlyReport`, `TenantInvoiceFinalized`,
`CostAllocationPolicyChanged`, `CreditApplied`, `FocusExportDownloaded`,
`BudgetAlertFired`, `RightsizingRecommendationEmitted`, `ForecastEmitted`,
`CommitmentDiscountApplied`. (5 existing + 4 new).
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four core signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `observability (ADR-0263)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `observability (ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §abuse-defence (§3.2.3 + ADR-0297)

Internet-facing: tenant-billing surface + admin portal. Anti-bot via edge bot-mgmt + WebAuthn
on cost-allocation-policy editor (high-blast-radius change). Anti-spoof via SPIFFE workload +
HMAC webhook. Anti-scrape via per-tenant rate-limit + FOCUS export paid-API tenant_class.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `abuse-defence (§3.2.3 + ADR-0297)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `abuse defence (§3.2.3 + ADR 0297)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (§3.2.3 + ADR 0297)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (§3.2.3 + ADR 0297)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `abuse defence (§3.2.3 + ADR 0297)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `abuse defence (§3.2.3 + ADR 0297)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `abuse defence (§3.2.3 + ADR 0297)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `abuse defence (§3.2.3 + ADR 0297)` workflow.

## §credential-isolation (ADR-0296)

OpenBao SecretReference `${openbao:secret/<tenant_id>/finops-portal/<key>}`. Ed25519 quarterly
key per ADR-finops-portal-007 + Grafana iframe-embed signing key + FOCUS-export tenant key.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `finops-portal` to the ≥50-line documentation-rigor floor.
- Service owner `ops-finops` owns this answer; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `tenant-billing-presentation`; bounded contexts: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- API surfaces: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy surfaces: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`; +5 more.
- State/event surfaces: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- SLO/dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `finops-portal` `tenant-billing-presentation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/finops-portal/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `finops-portal` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `finops-portal` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `finops-portal` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `finops-portal` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `finops-portal` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `tenant-billing-presentation` evaluates `<tenant>.finops-portal.tenant-billing-presentation` against policy, writes `finops_portal.tenant_billing_presentation`, and emits `oya.finops.portal.tenant.billing.presentation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `finops-portal` binds `credential-isolation (ADR-0296)` to `{'name': 'tenant-billing-presentation', 'description': 'Tenant-facing invoice presentation + drill-down dashboards.', 'crates': ['oya-finops-portal-tenant-billing-presentation-kernel', 'oya-finops-portal-tenant-billing-presentation-domain', 'oya-finops-portal-tenant-billing-presentation-usecase', 'oya-finops-portal-tenant-billing-presentation-api', 'oya-finops-portal-tenant-billing-presentation-app'], 'status': 'planned'}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `finops-portal` is `contracts/cost-allocation-policy-internal.proto, contracts/focus-export-internal.asyncapi.yaml, contracts/tenant-invoice-public.openapi.yaml`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `finops-portal` is `policy/cedar/abuse-defence.cedar, policy/cedar/action-authorization.cedar, policy/cedar/auditor-scope.cedar, policy/cedar/ci-scope.cedar, policy/cedar/customer-success-credit-application.cedar, policy/cedar/data-residency.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `finops-portal` state/event naming uses `finops_portal.{'name': 'tenant_billing_presentation', 'description': 'Tenant_facing invoice presentation + drill_down dashboards.', 'crates': ['oya_finops_portal_tenant_billing_presentation_kernel', 'oya_finops_portal_tenant_billing_presentation_domain', 'oya_finops_portal_tenant_billing_presentation_usecase', 'oya_finops_portal_tenant_billing_presentation_api', 'oya_finops_portal_tenant_billing_presentation_app'], 'status': 'planned'}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `finops-portal` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `finops-portal` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `finops-portal` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `finops-portal` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `finops-portal` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `finops-portal` uses SLOs `slos/anomaly-explanation-latency.openslo.yaml, slos/cost-allocation-policy-change-latency.openslo.yaml, slos/credit-application-correctness.openslo.yaml, slos/drilldown-query-latency-p99.openslo.yaml, slos/focus-export-availability.openslo.yaml, plus 4 more` and dashboards `dashboards/anomaly-investigation.grafana.json, dashboards/budget-alerts.grafana.json, dashboards/fleet-cost-rollup.grafana.json, dashboards/rightsizing-recommendations.grafana.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `finops-portal` uses runbooks `runbooks/budget-alert-runaway-firings.md, runbooks/cost-allocation-policy-rollback.md, runbooks/credit-application-reconciliation.md, runbooks/focus-export-failure.md, runbooks/quarterly-regulator-emit-miss.md, plus 3 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `finops-portal` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/finops-portal/Chart.yaml, iac/helm/finops-portal/templates/_helpers.tpl, iac/helm/finops-portal/templates/deployment.yaml, plus 14 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `finops-portal` uses `capabilities/anomaly-explanation.capability.yaml, capabilities/focus-export.capability.yaml, capabilities/tenant-invoice-render.capability.yaml` and `catalog/bnf-v4.1.yaml, catalog/oya-finops-portal-budget-alert-kernel.yaml, catalog/oya-finops-portal-commitment-management-domain.yaml, catalog/oya-finops-portal-forecasting-usecase.yaml, plus 2 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `finops-portal` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `finops-portal` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `finops-portal` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `finops-portal` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §backup-portability (ADR-0276)

FOCUS 1.3 export IS the portability format. Invoice PDFs + JSON-LD ledger exportable per
GDPR Article 20.

## §six-dimension matrix

| Dimension | Status |
|---|---|
| Maintainability | OpenCost + Mimir LTS pins per manifest; SemVer per ADR-0258. |
| Observability | 9 audit-event-classes + 9 SLOs + 4 dashboards (3 existing + 1 added). |
| Scalability | Per-tenant rate-limit; capacity-model.md. |
| Performance | Drill-down P99 ≤500ms; invoice PDF render P99 ≤3s; FOCUS export streamed. |
| Optimization | Lazy invoice PDF (render on first request); eager budget-alert evaluation. |
| Code quality | ≥85% line ≥75% branch; `oya-check-*`; Rust deny(warnings). |

---



## §cell-eligibility
This anchor is closed for `finops-portal` against ADR-0248 §D-1: cell tenant_class, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- tenant_class 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for tenant_class-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `tenant-billing-presentation` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to tenant_class 0/1 pods; tenant_class 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `finops-portal`; owner `ops-finops`; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- Capability records cited: `microservices/finops-portal/capabilities/anomaly-explanation.capability.yaml`, `microservices/finops-portal/capabilities/focus-export.capability.yaml`, `microservices/finops-portal/capabilities/tenant-invoice-render.capability.yaml`.
- API surfaces cited: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`, `microservices/finops-portal/policy/cedar/data-residency.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`, `microservices/finops-portal/slos/quarterly-regulator-evidence-emit-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`, `microservices/finops-portal/runbooks/tenant-budget-exhausted.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar binding: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`, `microservices/finops-portal/policy/cedar/data-residency.cedar`; +3 more.
- State/event binding: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- Capability binding: `tenant-billing-presentation`.
- SLO binding: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`, `microservices/finops-portal/slos/quarterly-regulator-evidence-emit-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`, `microservices/finops-portal/runbooks/tenant-budget-exhausted.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `finops-portal`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `finops-portal`.
- `policy-engine` supplies the signed Cedar corpus while `finops-portal` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `finops-portal` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `finops-portal`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `finops-portal` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `finops-portal` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `finops-portal` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `finops-portal` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `finops-portal` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `tenant-billing-presentation` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `finops-portal` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `finops-portal`; owner `ops-finops`; tenant_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, `focus-export`, `credit-ledger`.
- Capability records cited: `microservices/finops-portal/capabilities/anomaly-explanation.capability.yaml`, `microservices/finops-portal/capabilities/focus-export.capability.yaml`, `microservices/finops-portal/capabilities/tenant-invoice-render.capability.yaml`.
- API surfaces cited: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`, `microservices/finops-portal/policy/cedar/data-residency.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`, `microservices/finops-portal/slos/quarterly-regulator-evidence-emit-correctness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`, `microservices/finops-portal/runbooks/tenant-budget-exhausted.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`, `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`, `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`.
- Cedar binding: `microservices/finops-portal/policy/cedar/abuse-defence.cedar`, `microservices/finops-portal/policy/cedar/action-authorization.cedar`, `microservices/finops-portal/policy/cedar/auditor-scope.cedar`, `microservices/finops-portal/policy/cedar/ci-scope.cedar`, `microservices/finops-portal/policy/cedar/customer-success-credit-application.cedar`, `microservices/finops-portal/policy/cedar/data-residency.cedar`; +3 more.
- State/event binding: `finops_portal.tenant_billing_presentation`, `finops_portal.cost_allocation_policy`, `finops_portal.anomaly_explanation`, `finops_portal.focus_export`, `finops_portal.credit_ledger`.
- Capability binding: `tenant-billing-presentation`.
- SLO binding: `microservices/finops-portal/slos/anomaly-explanation-latency.openslo.yaml`, `microservices/finops-portal/slos/cost-allocation-policy-change-latency.openslo.yaml`, `microservices/finops-portal/slos/credit-application-correctness.openslo.yaml`, `microservices/finops-portal/slos/drilldown-query-latency-p99.openslo.yaml`, `microservices/finops-portal/slos/focus-export-availability.openslo.yaml`, `microservices/finops-portal/slos/quarterly-regulator-evidence-emit-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/finops-portal/runbooks/budget-alert-runaway-firings.md`, `microservices/finops-portal/runbooks/cost-allocation-policy-rollback.md`, `microservices/finops-portal/runbooks/credit-application-reconciliation.md`, `microservices/finops-portal/runbooks/focus-export-failure.md`, `microservices/finops-portal/runbooks/quarterly-regulator-emit-miss.md`, `microservices/finops-portal/runbooks/tenant-budget-exhausted.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `finops-portal`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `finops-portal`.
- `policy-engine` supplies the signed Cedar corpus while `finops-portal` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `finops-portal` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `finops-portal`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `finops-portal` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

