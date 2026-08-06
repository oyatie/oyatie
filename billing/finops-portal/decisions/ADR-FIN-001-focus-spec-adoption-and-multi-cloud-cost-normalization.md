---
id: ADR-FIN-001
title: FOCUS Spec Adoption and Multi-Cloud Cost Normalization
status: Proposed
date: 2026-05-20
microservice: finops-portal
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0704-k8s-port-live-apex.md
decision_owner: ops-finops
---

# ADR-FIN-001: FOCUS Spec Adoption and Multi-Cloud Cost Normalization

## Context

- Finops-portal is the tenant-facing product surface for invoice presentation, drill-down dashboards, cost-allocation policy, anomaly explanation, FOCUS export, credit ledger, and regulator evidence.
- Existing local ADR `ADR-finops-portal-001-focus-spec-version.md` pins FOCUS 1.3 for the export pipeline; this ADR expands the decision into the service-wide normalization architecture.
- The local manifest names `focus-export`, `tenant-billing-presentation`, `cost-allocation-policy`, `anomaly-explanation`, and `credit-ledger` bounded contexts.
- The PRD says the upstream data plane exists through OpenCost, Mimir, and FOCUS 1.3, while this microservice owns differentiated presentation and workflow.
- ADR-0174 makes cost attribution and chargeback a platform concern; this ADR decides how cloud and substrate bills become comparable tenant records.
- ADR-0244 requires every cost row, invoice, credit, anomaly, and export to carry tenant scope and audience type.
- Cost data is business-sensitive because spend patterns reveal scale, headcount proxies, customer growth, and operational incidents.
- Tenants expect AWS, GCP, Azure, OCI, Kubernetes, SaaS, and internal Oyatie substrate cost to be comparable without learning each provider's bill schema.
- Finance teams expect stable columns, period close semantics, corrections, credits, amortization, and committed-use treatment.
- SRE teams need cost per SLO, cost per incident, cost per workflow, and cost per tenant cell.
- Product teams need cost per feature or capability without reading provider billing exports.
- Customer-success teams need credit application and invoice explanation without mutating raw provider records.
- Auditors need signed quarterly cost reports and export records tied to audit-chain.
- Regulated tenants may require cost evidence to stay inside home region or pack-approved storage.
- Provider billing exports arrive at different cadences and with different late-adjustment semantics.
- Kubernetes cost allocation from OpenCost does not directly match cloud invoice line items without reconciliation.
- Internal substrate costs such as observability, audit-chain, intelligence dispatch, and workflow execution need allocation rules independent of cloud provider labels.
- FOCUS gives common cost and usage vocabulary, but provider exports still need source-specific adapters and quality checks.
- FOCUS 1.3 adds useful dimensions and commitment handling, but existing tenant finance pipelines may still consume older shapes.
- The service must publish FOCUS export without letting tenants infer other tenant spend from shared resource allocation.
- The service must keep raw provider bill rows immutable and write normalized facts as a derived dataset.
- The service must separate billed cost, effective cost, amortized cost, list cost, credits, taxes, and negotiated discounts.
- The service must make currency conversion explicit rather than hiding it in one amount field.
- The service must support cost allocation policy changes that are prospective by default and backdated only through audited correction runs.
- The service must preserve portability under ADR-0276 by giving tenants exportable cost evidence.
- The service must not let dashboards become the only source of truth; normalized FOCUS rows are the source.
- The service must keep Grafana embedding and drill-down dashboards downstream of authorization and row-level scoping.
- The service must include OpenCost for Kubernetes allocation but not treat OpenCost as the entire financial ledger.
- The service must document how FOCUS, OpenCost, provider bill exports, and Oyatie internal charge events fit together.

## Decision

- Adopt FOCUS 1.3 as the canonical normalized cost schema for finops-portal exports, invoice explanation, anomaly explanation, and tenant drill-downs.
- Treat FOCUS rows as derived normalized records, not as replacements for raw provider billing evidence.
- Store raw provider exports in immutable source tables keyed by `provider`, `billing_account_id`, `invoice_period`, `source_file_hash`, and `ingested_at`.
- Store normalized FOCUS 1.3 rows in tenant-scoped warehouse tables keyed by `tenant_id`, `billing_period`, `provider`, `service_category`, and `charge_id`.
- Use source adapters for AWS CUR, GCP BigQuery Billing Export, Azure Cost Management export, OCI cost reports, OpenCost allocation, and internal Oyatie charge events.
- Use OpenCost for Kubernetes workload allocation and join it back to provider infrastructure line items through cluster, namespace, workload, node, and time window.
- Use ADR-0174 cost-center and workload-class tags as required normalization inputs.
- Use `EffectiveCost` for chargeback and tenant invoice presentation where credits and discounts apply.
- Use `BilledCost` for provider invoice reconciliation.
- Use `ListCost` for savings and discount explanation.
- Use `ContractedCost` or FOCUS commitment datasets where provider exports expose commitment terms.
- Use ISO 4217 currency codes and store conversion rate id, conversion timestamp, and source currency on every converted row.
- Normalize all tenant-facing exports to tenant invoice currency while preserving original provider currency.
- Apply shared substrate allocation using signed cost-allocation policy versions, not hard-coded percentages.
- Allocate shared observability, audit-chain, workflow-engine, and intelligence substrate costs by declared allocation drivers.
- Use allocation drivers: `request_count`, `trace_span_count`, `audit_event_count`, `workflow_step_count`, `model_token_count`, `storage_gib_hours`, and `seat_count`.
- Reject cost rows missing tenant resolution unless they are routed to an explicit `unallocated` suspense tenant for investigation.
- Keep `unallocated` below 0.5 percent of monthly effective cost as an SLO.
- Use append-only correction rows rather than mutating normalized rows after a period closes.
- Close invoice periods on day 5 after month end by default and allow late provider corrections for 90 days.
- Emit audit events for source ingest, normalization completion, allocation policy change, credit application, export download, and invoice finalization.
- Gate every cost read and export through Cedar; tenant admins read only their tenant rows, internal FinOps reads require purpose and time-bound scope.
- Publish FOCUS exports as Parquet and CSV, with Parquet canonical for machine pipelines and CSV as compatibility projection.
- Version export endpoint as `/v1/finops/focus/exports` with `schema_version=1.3` default.
- Keep the older local ADR's FOCUS 1.3 pin as a narrow export-version precedent and supersede its scope only where this ADR defines normalization.
- Build dashboard queries from normalized FOCUS views and never directly from raw provider exports.
- Use signed quarterly evidence bundles that include source file hashes, adapter versions, policy versions, and export manifests.
- Preserve an upgrade path to FOCUS 1.4 or later by adding a parallel translator and six-month deprecation window.
- Treat provider-specific fields as extension columns under a namespaced map, not as first-class portable dimensions.
- Treat cost data as `PII_QUASI` plus business-confidential for access, logging, and export review.

## Alternatives Considered

### Provider-native schemas only

- Pros: no loss of provider-specific detail.
- Pros: easiest reconciliation against each provider invoice.
- Pros: avoids waiting for all providers to support FOCUS consistently.
- Cons: tenants must understand multiple cloud billing vocabularies.
- Cons: cross-cloud dashboards become bespoke joins and mappings.
- Cons: chargeback policy becomes provider-specific and hard to audit.
- Rejected because finops-portal exists to normalize cost across providers and substrates.

### OpenCost as the canonical schema

- Pros: excellent fit for Kubernetes workload allocation.
- Pros: open-source and aligned with cloud-native cost monitoring.
- Pros: already useful for cluster and namespace allocation.
- Cons: does not cover every cloud, SaaS, provider invoice, commitment, tax, and credit concept.
- Cons: finance teams need billing dataset semantics beyond cluster allocation.
- Cons: tenant exports should match the broader FinOps FOCUS vocabulary.
- Rejected as the canonical schema; retained as the Kubernetes allocation input.

### Internal proprietary cost schema

- Pros: can model every Oyatie-specific allocation and billing nuance.
- Pros: easy to optimize for current dashboards.
- Pros: no external version migration pressure.
- Cons: weak tenant portability and poor finance-system interoperability.
- Cons: violates the product promise that tenants can export and reason about their own cost data.
- Cons: future provider adapters would become permanent custom mapping work.
- Rejected because FOCUS is the industry-aligned interchange vocabulary.

### Wait for every provider to emit native FOCUS

- Pros: reduces adapter code and source-specific mapping risk.
- Pros: makes provider conformance someone else's problem.
- Pros: simplifies validation if provider exports are already normalized.
- Cons: delays a required tenant product surface.
- Cons: providers will still differ in late adjustments, credits, and metadata quality.
- Cons: internal Oyatie substrate costs still need normalization.
- Rejected because adapter-based normalization is needed now and remains useful later.

### Dashboard-only FinOps with no exportable normalized dataset

- Pros: fastest visible UX.
- Pros: fewer storage and schema governance obligations.
- Pros: easier to change charts during product discovery.
- Cons: tenants cannot integrate with finance pipelines.
- Cons: audit and regulator evidence becomes screenshot-driven and weak.
- Cons: dashboard queries can drift from invoice computation.
- Rejected because normalized rows, not charts, must be the source of truth.

## Consequences

- Positive: tenants get a portable FOCUS 1.3 dataset instead of provider-specific bill fragments.
- Positive: finance, SRE, and product teams can discuss cost with one vocabulary.
- Positive: raw provider evidence remains available for reconciliation and audit.
- Positive: OpenCost can do what it is good at without becoming the entire ledger.
- Positive: cost-allocation policy becomes versioned and auditable.
- Positive: late corrections are explicit rows rather than hidden mutations.
- Positive: quarterly evidence bundles can be independently traced to source exports.
- Negative: source adapters require ongoing maintenance as provider exports evolve.
- Negative: FOCUS version upgrades will require parallel translators and tenant communication.
- Negative: normalized rows can be misunderstood if users ignore billed versus effective versus list cost.
- Negative: shared-cost allocation can create tenant disputes even when math is consistent.
- Negative: currency conversion and tax treatment require careful finance review.
- Neutral: provider-native exports remain immutable evidence, not tenant UX.
- Neutral: Grafana dashboards remain a projection and can evolve independently from source rows.
- Neutral: FOCUS 1.3 is a current pin, not a permanent ceiling.
- Neutral: unallocated cost becomes a visible operational queue.
- Follow-up: build a FOCUS validation job and block export on schema violations.
- Follow-up: add cost allocation policy review UI backed by signed policy versions.
- Follow-up: add an unallocated-cost burn-down dashboard.
- Follow-up: add a six-month tenant communication template for future FOCUS upgrades.
- Follow-up: add adapter conformance fixtures for AWS, GCP, Azure, OCI, OpenCost, and internal events.

## Implementation Notes

- Data shape `RawCostSourceFile`: `{source_file_id, provider, billing_account_id, invoice_period, object_ref, sha256, ingested_at, adapter_version}`.
- Data shape `ProviderBillLine`: `{source_file_id, provider_line_id, provider_service, provider_sku, usage_start, usage_end, amount, currency, raw_payload_ref}`.
- Data shape `FocusCostRow`: `{tenant_id, billing_period, provider, service_category, service_name, billed_cost, effective_cost, list_cost, charge_class, currency}`.
- Data shape `FocusCostRow` also includes `{resource_id, region_id, availability_zone, pricing_category, commitment_id, invoice_issuer, sub_account_id}`.
- Data shape `AllocationPolicyVersion`: `{policy_id, version, tenant_id, driver, selector, effective_from, effective_to, signed_by, audit_event_id}`.
- Data shape `SharedCostAllocation`: `{source_line_id, allocation_policy_version, target_tenant_id, allocated_cost, driver_quantity, explanation_code}`.
- Data shape `CreditLedgerEntry`: `{tenant_id, credit_id, amount, currency, reason_code, applied_period, approved_by, audit_event_id}`.
- Data shape `FocusExportManifest`: `{export_id, tenant_id, period, schema_version, row_count, sha256, format, generated_at, downloaded_by}`.
- REST endpoint `POST /v1/finops/sources/ingest` registers immutable provider exports.
- REST endpoint `POST /v1/finops/normalization/jobs` runs provider-to-FOCUS translation.
- REST endpoint `GET /v1/finops/invoices/{period}` returns tenant invoice view from normalized rows.
- REST endpoint `POST /v1/finops/allocation-policies` creates a signed prospective policy version.
- REST endpoint `POST /v1/finops/credits` applies customer-success credits after Cedar and audit checks.
- REST endpoint `GET /v1/finops/focus/exports?period=YYYY-MM&schema_version=1.3` returns export metadata.
- REST endpoint `POST /v1/finops/focus/exports` generates Parquet or CSV export.
- Async event `finops.source.ingested.v1` carries source hash, provider, period, and adapter version.
- Async event `finops.normalization.completed.v1` carries row count, validation status, and unallocated cost.
- Async event `finops.allocation_policy.changed.v1` carries policy diff and effective date.
- Async event `finops.focus_export.downloaded.v1` carries export id and tenant scope.
- Cedar permit `finops_portal::cost::read` requires same tenant, finance role, and period scope.
- Cedar permit `finops_portal::focus_export::download` requires tenant admin or auditor engagement scope.
- Cedar permit `finops_portal::allocation_policy::write` requires FinOps admin and prospective effective date.
- Cedar forbid `finops_portal::allocation_policy::backdate` blocks backdating unless correction workflow is approved.
- Cedar permit `finops_portal::credit::apply` requires customer-success approval and finance dual control.
- SLO target `focus_export_availability`: 99.9 percent monthly.
- SLO target `drilldown_query_latency_p99`: below 1 second on normalized tenant views.
- SLO target `normalization_freshness`: 95 percent provider exports normalized within 6 hours of source availability.
- SLO target `unallocated_cost_ratio`: below 0.5 percent of monthly effective cost.
- Dashboard `fleet-cost-rollup.grafana.json` queries normalized FOCUS rows only.
- Dashboard `budget-alerts.grafana.json` uses effective cost and committed-use policy views.
- Dashboard `anomaly-investigation.grafana.json` links anomalies to source file hashes and allocation policy versions.
- Export storage path `s3://finops-exports/<tenant_id>/<period>/focus-1.3/<export_id>.parquet`.
- Source storage path `s3://finops-raw/<provider>/<billing_account_id>/<period>/<source_file_hash>`.
- Validation checks enforce FOCUS required columns, decimal precision, currency code, period boundaries, and tenant id presence.
- Currency conversion uses daily FX rates with source id and timestamp stored on each converted row.
- Late corrections append `charge_class="Correction"` rows and link to the original normalized row.
- Period close worker emits `TenantInvoiceFinalized` after source freshness, validation, and unallocated-ratio gates pass.
- Export worker emits signed audit-chain evidence before returning a downloadable link.
- PII and secret scrubbers prevent provider account owner email, card, or credential data from reaching tenant exports.

## Verification

- Unit test `focus_row_requires_tenant_id` rejects normalized rows without tenant scope.
- Unit test `effective_cost_includes_credit_application` validates invoice math.
- Unit test `billed_cost_matches_provider_source_total` validates reconciliation basis.
- Unit test `allocation_policy_backdate_denied_without_correction` validates Cedar gate.
- Unit test `currency_conversion_records_rate_source` validates auditability.
- Unit test `provider_extension_fields_namespaced` prevents non-portable top-level schema drift.
- Property test `raw_source_to_focus_totals_conserve_cost` generates provider rows and corrections.
- Property test `shared_cost_allocation_sums_to_source_line` validates allocation math.
- Property test `late_corrections_append_not_mutate` protects closed-period history.
- Integration test `aws_cur_to_focus_export_validates` covers AWS adapter.
- Integration test `gcp_billing_to_focus_export_validates` covers GCP adapter.
- Integration test `azure_export_to_focus_export_validates` covers Azure adapter.
- Integration test `opencost_allocation_joins_provider_node_cost` covers Kubernetes costs.
- Integration test `tenant_focus_export_excludes_other_tenant_rows` validates Cedar and SQL filters.
- Integration test `credit_application_emits_audit_chain_event` validates evidence.
- Load test `tenant_drilldown_p99_under_1s` validates dashboard query path.
- Load test `monthly_normalization_10m_rows_under_6h` validates freshness.
- Chaos test `provider_late_file_arrives_after_close` validates correction row flow.
- Chaos test `allocation_policy_bad_version_rolls_back` validates signed policy pointer rollback.
- Dashboard check `unallocated-cost` shows suspense tenant ratio and top source causes.
- Dashboard check `focus-export-health` shows validation failures by adapter and schema version.
- Metric check `finops_unallocated_cost_ratio` pages above 0.5 percent.
- Static check every export endpoint declares `schema_version`.
- Static check dashboards read normalized views, not raw provider tables.
- Oya VCS evidence must include line count, root ADR cite count, and reference count for this ADR.

## References

- FOCUS Specification 1.3: https://focus.finops.org/focus-specification/
- FOCUS project overview and GitHub repository from the FinOps Foundation.
- OpenCost Specification: https://opencost.io/docs/specification
- Microsoft Learn, FinOps Open Cost and Usage Specification overview.
- AWS Cost and Usage Report data dictionary.
- Google Cloud Billing Export to BigQuery documentation.
- Azure Cost Management export documentation.
- OCI Cost Analysis and usage reports documentation.
- FinOps Framework public cloud and allocation guidance.
- Cedar Policy Language authorization and schema documentation: https://docs.cedarpolicy.com/
- Local `ADR-finops-portal-001-focus-spec-version.md`.
- ADR-0174, ADR-0211, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, and ADR-0276.
