---
id: ADR-MS-001
title: Customer record mutation and revenue lineage contract for crm
status: Proposed
date: 2026-05-20
microservice: crm
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters
  - ADR-0128-hyperscaler-architecture-invariants
  - ADR-0131-per-microservice-flat-layout
decision_owner: axis-crm + council-revenue
---

# ADR-MS-001: Customer record mutation and revenue lineage contract for crm

## Context

- Pressure name: customer-record authority pressure.
- `crm` owns account master, opportunity, quote, service case, campaign, and loyalty ledger mutations.
- Customer records are shared by revenue, support, marketing, marketplace, payments, intelligence, and Ontology consumers.
- A loose CRM write model would fragment account truth and weaken revenue lineage.
- The service PRD requires tenant scope, Cedar action, Ontology projection, workflow handoff, audit-chain seal, pack, tier, and replay fixture evidence in the same trace.
- The OpenAPI contract exposes `POST /v1/crm/account-master`.
- The OpenAPI contract exposes `POST /v1/crm/opportunity`.
- The OpenAPI contract exposes `POST /v1/crm/quote`.
- The OpenAPI contract exposes `POST /v1/crm/service-case`.
- The OpenAPI contract exposes `POST /v1/crm/campaign`.
- The OpenAPI contract exposes `POST /v1/crm/loyalty-ledger`.
- The AsyncAPI contract emits `AccountMasterChanged`, `OpportunityChanged`, `QuoteChanged`, `ServiceCaseChanged`, `CampaignChanged`, and `LoyaltyLedgerChanged`.
- Local policy files include `account-master-authorization.cedar`, `opportunity-authorization.cedar`, `quote-authorization.cedar`, `service-case-authorization.cedar`, `campaign-authorization.cedar`, and `loyalty-ledger-authorization.cedar`.
- Local policy files also include `pack-overlay-authorization.cedar`, `abuse-defence.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, `data-residency.md`, and `tenant-isolation.md`.
- Local SLOs include `crm-availability`, `crm-latency-p99`, `crm-throughput`, and `account-master-success-rate`.
- Local dashboards include `crm-overview.json`, `account-master-health.json`, and `opportunity-residency.md`.
- Constraint name: one account master per tenant graph.
- Account hierarchy and customer identity must remain stable across quotes, campaigns, service cases, and loyalty events.
- Constraint name: quote and discount approval pressure.
- Quote line pricing and discount approval need explainable policy evidence before a commercial commitment is emitted.
- Constraint name: service-case SLA pressure.
- Service-case escalation and SLA timers need customer, entitlement, severity, and contact state in one trace.
- Constraint name: campaign-to-revenue attribution pressure.
- Campaign attribution must connect marketing spend, opportunity influence, quote outcome, and loyalty impact without inventing duplicate customer ids.
- Constraint name: regulated revenue data.
- CRM payloads can contain personal data, corporate confidential data, pricing terms, and financial commitments.
- ADR-0008 requires data class and purpose to travel with every mutation.
- ADR-0003 requires mutation evidence that can be replayed and audited.

## Decision

- Decision name: customer aggregate mutation envelope.
- `crm` will use one mutation envelope for all six CRM aggregates.
- The six aggregates are `account-master`, `opportunity`, `quote`, `service-case`, `campaign`, and `loyalty-ledger`.
- Each mutation envelope must include tenant id, principal id, aggregate id, aggregate kind, action, purpose, data class, pack overlay, tier, idempotency key, trace context, and audit target.
- Each mutation envelope must include `source_system` and `source_confidence` when imported from Salesforce, SAP, Dynamics, NetSuite, or connector-backed systems.
- Each mutation envelope must include `ontology_projection_id` when the mutation writes Customer, Account, Opportunity, Quote, ServiceCase, Campaign, or LoyaltyLedger objects.
- Each mutation envelope must include `workflow_handoff` when approval, escalation, dunning, renewal, or campaign journey orchestration follows.
- Account master creates and merges require deterministic duplicate checks before persistence.
- Account hierarchy edges must carry `relationship_type`, `effective_from`, `effective_to`, `confidence`, and `source_system`.
- Opportunity stage changes must preserve prior stage, next stage, amount, currency, close date, probability, and approval reason.
- Quote changes must preserve line item ids, product refs, price book id, discount basis, approver id, tax context, and expiration.
- Service-case changes must preserve severity, entitlement id, SLA clock, customer contact ref, channel, and escalation state.
- Campaign changes must preserve spend bucket, target segment, consent basis, attribution model, and linked opportunities.
- Loyalty ledger changes must be append-only and must preserve earn, redeem, adjust, expire, and reversal reason.
- Mutating actions are `create`, `amend`, `approve`, `reverse`, `archive`, `import`, `export`, and `read` where read is evidence-bound for protected views.
- Every mutation emits the matching `*Changed` event after Cedar approval and audit target validation.
- Account merges emit one `AccountMasterChanged` event with `change_kind=merge` and both source account ids.
- Quote approval emits `QuoteChanged` with `change_kind=approval` and policy evidence.
- Service-case escalation emits `ServiceCaseChanged` with `change_kind=escalation` and SLA evidence.
- Campaign attribution emits `CampaignChanged` with `change_kind=attribution_update`.
- Loyalty reversals emit `LoyaltyLedgerChanged` with `change_kind=reversal`.
- Availability target is 0.999.
- P99 latency target is 0.99 for endpoint-specific good events.
- Throughput target is 0.995 for accepted CRM mutations.
- Account-master success-rate target is 0.999.
- Account merge duplicate-detection false negative rate must be <=0.1% in canonicalen fixtures.
- Quote approval policy latency must stay within the service policy-decision budget.
- Metrics must not expose raw customer names, emails, phone numbers, or account ids.

## Alternatives Considered

### Alternative 1: Separate APIs and event semantics per CRM aggregate

- Pros: each aggregate can evolve independently.
- Pros: narrower schemas per endpoint.
- Cons: policy, audit, replay, and Ontology projection logic diverge quickly.
- Cons: cross-aggregate revenue lineage becomes difficult to reconstruct.
- Cons: SDK clients must learn six incompatible mutation shapes.
- Rejected because the aggregates need one evidence envelope.

### Alternative 2: Treat CRM as imported read-only data from external systems

- Pros: avoids owning customer master writes.
- Pros: works for tenants already standardized on Salesforce or SAP.
- Cons: Oyatie cannot guarantee account hierarchy, campaign attribution, or quote approval evidence.
- Cons: SMB and greenfield tenants still need native CRM state.
- Cons: external systems differ on data class and audit shape.
- Rejected because CRM is a durable product microservice, not just a connector mirror.

### Alternative 3: Use event sourcing for every CRM aggregate

- Pros: strong history and replay model.
- Pros: easy to inspect all mutations.
- Cons: high operational complexity for first implementation.
- Cons: read model rebuilds for large tenants are expensive.
- Cons: loyalty ledger needs append-only semantics, but not every aggregate does.
- Rejected because append-only is required only where business semantics demand it.

### Alternative 4: Single generic `crm.object.mutate` endpoint

- Pros: minimal route surface.
- Pros: easy for generic SDK generation.
- Cons: hides aggregate-specific policy and schema constraints.
- Cons: weakens OpenAPI clarity and per-aggregate SLOs.
- Cons: makes quote, case, and loyalty semantics too implicit.
- Rejected because explicit endpoints are more reviewable while sharing one envelope.

### Alternative 5: Push all customer objects into Ontology as the write authority

- Pros: one graph model for all customer data.
- Pros: strong relationship traversal.
- Cons: Ontology is not the CRM domain write owner.
- Cons: domain invariants like quote approval and SLA escalation need CRM semantics.
- Cons: graph writes alone do not provide commercial action evidence.
- Rejected because CRM writes project to Ontology, not the reverse.

## Consequences

### Positive

- SDKs can use one mutation envelope across all CRM aggregates.
- Audit-chain evidence becomes uniform for customer, revenue, support, campaign, and loyalty changes.
- Account hierarchy and opportunity lineage can be joined without guessing source identity.
- Quote approvals and service-case escalations become policy-visible.
- Campaign attribution can connect spend to opportunity and quote outcome.
- Loyalty ledger remains append-only without forcing event sourcing on every aggregate.
- External CRM imports remain possible through source-system metadata.
- Dashboards can track customer master health and opportunity residency.

### Negative

- The shared envelope is stricter than a minimal CRUD API.
- Aggregate-specific validation still requires distinct domain modules.
- Import pipelines need mapping and confidence metadata.
- Account merge and duplicate detection can be controversial for tenants.
- Quote approval policies must be maintained with finance and revenue operations.
- Campaign attribution can create high-cardinality relationship graphs.
- Service-case SLA failures can become customer-facing incidents.

### Neutral

- External CRM systems remain supported through `connector` adapters.
- Ontology remains the projection and query graph, not the write authority.
- Workflow remains the approval and escalation orchestrator, not the data owner.
- Revenue dashboards may consume CRM events but cannot mutate CRM state directly.
- Read paths can be optimized separately from mutation evidence.

### Follow-up work

- Add duplicate account fixture suite with conglomerate and subsidiary cases.
- Add quote discount policy examples for finance approval.
- Add service-case SLA escalation replay tests.
- Add campaign attribution lineage graph dashboard.
- Add loyalty ledger reversal playbook.
- Add import confidence thresholds for Salesforce, SAP, Dynamics, and NetSuite mappings.

## Implementation Notes

### Data Shapes

- `CrmMutationEnvelope` fields: `tenant_id`, `principal_id`, `aggregate_kind`, `aggregate_id`, `action`, `purpose`, `data_class`, `pack_overlay`, `tier`, `idempotency_key`, `traceparent`, `audit_target`.
- `SourceDescriptor` fields: `source_system`, `external_object_id`, `source_confidence`, `mapping_version`, `imported_at`, `connector_wiring_id`.
- `AccountMaster` fields: `account_id`, `legal_name_ref`, `display_name`, `tenant_scope`, `parent_account_id`, `industry_code`, `residency_label`, `data_class`.
- `AccountHierarchyEdge` fields: `from_account_id`, `to_account_id`, `relationship_type`, `effective_from`, `effective_to`, `confidence`, `source_system`.
- `Opportunity` fields: `opportunity_id`, `account_id`, `stage`, `amount`, `currency`, `close_date`, `probability`, `owner_id`, `campaign_refs`.
- `Quote` fields: `quote_id`, `opportunity_id`, `line_items`, `price_book_id`, `discount_basis`, `approval_state`, `tax_context`, `expires_at`.
- `ServiceCase` fields: `case_id`, `account_id`, `contact_ref`, `severity`, `entitlement_id`, `sla_clock`, `channel`, `escalation_state`.
- `Campaign` fields: `campaign_id`, `target_segment_ref`, `consent_basis`, `spend_bucket`, `attribution_model`, `linked_opportunity_ids`.
- `LoyaltyLedgerEntry` fields: `entry_id`, `account_id`, `event_type`, `points_delta`, `monetary_value`, `reason_code`, `reversal_of`, `created_at`.
- `CrmChangedEvent` fields: `tenant_id_hash`, `aggregate_kind`, `aggregate_id`, `change_kind`, `policy_version`, `ontology_projection_id`, `workflow_handoff`, `evidence_id`.

### API Endpoints

- `POST /v1/crm/account-master` mutates account master and account hierarchy state.
- `POST /v1/crm/opportunity` mutates opportunity stage, owner, amount, or forecast state.
- `POST /v1/crm/quote` mutates quote line items, approvals, expirations, and reversals.
- `POST /v1/crm/service-case` mutates service case severity, SLA, escalation, and resolution.
- `POST /v1/crm/campaign` mutates campaign target, spend, consent, and attribution state.
- `POST /v1/crm/loyalty-ledger` appends or reverses loyalty ledger entries.
- Every endpoint accepts `CrmMutationEnvelope` plus aggregate-specific payload.
- Every endpoint returns `mutation_id`, `aggregate_id`, `change_kind`, `evidence_id`, and emitted event type.

### Cedar Policies

- `policy/account-master-authorization.cedar` governs customer master create, merge, import, and export.
- `policy/opportunity-authorization.cedar` governs stage movement, forecast mutation, and owner reassignment.
- `policy/quote-authorization.cedar` governs price, discount, approval, reversal, and export.
- `policy/service-case-authorization.cedar` governs SLA escalation, customer contact access, and case closure.
- `policy/campaign-authorization.cedar` governs segment targeting and consent-bound campaign changes.
- `policy/loyalty-ledger-authorization.cedar` governs append, adjust, expire, redeem, and reverse.
- `policy/pack-overlay-authorization.cedar` rejects pack-ineligible CRM mutations.
- `policy/tenant-isolation.md` documents tenant data boundaries.
- `policy/data-residency.md` binds CRM records to region and pack overlays.

### SLO Targets

- `crm-availability.openslo.yaml`: CRM availability target 0.999.
- `crm-latency-p99.openslo.yaml`: p99 latency good-event target 0.99.
- `crm-throughput.openslo.yaml`: CRM throughput target 0.995.
- `account-master-success-rate.openslo.yaml`: account master success target 0.999.
- Duplicate account false-negative rate must be <=0.1% in canonicalen fixtures.
- Quote approval policy latency must satisfy the service policy-decision latency budget.

## Verification

- Unit test `mutation_envelope_requires_tenant_principal_action_data_class_and_audit_target`.
- Unit test `account_hierarchy_edge_requires_effective_window`.
- Unit test `opportunity_stage_change_preserves_prior_stage`.
- Unit test `quote_discount_requires_approval_reason`.
- Unit test `service_case_escalation_requires_sla_clock`.
- Unit test `campaign_attribution_requires_consent_basis`.
- Unit test `loyalty_ledger_reversal_references_original_entry`.
- Property test `account_merge_is_idempotent_for_same_duplicate_set`.
- Property test `loyalty_ledger_balance_never_changes_without_entry`.
- Cedar test `account_master_denies_cross_tenant_merge`.
- Cedar test `opportunity_denies_unapproved_stage_regression`.
- Cedar test `quote_denies_discount_without_finance_approval`.
- Cedar test `service_case_denies_contact_read_without_entitlement`.
- Cedar test `campaign_denies_missing_consent_basis`.
- Cedar test `loyalty_ledger_denies_direct_delete`.
- Contract test `openapi-v1.yaml_contains_six_crm_mutation_paths`.
- Contract test `asyncapi-v1.yaml_contains_six_changed_events`.
- Integration test `account_master_mutate_emits_account_master_changed`.
- Integration test `opportunity_stage_change_projects_to_ontology`.
- Integration test `quote_approval_hands_off_to_workflow`.
- Integration test `service_case_escalation_emits_sla_evidence`.
- Integration test `campaign_attribution_links_opportunity`.
- Integration test `loyalty_reversal_is_append_only`.
- Import test `salesforce_account_import_preserves_source_descriptor`.
- Import test `dynamics_opportunity_import_respects_pack_overlay`.
- Replay test `quote_approval_replay_does_not_duplicate_commitment`.
- Replay test `service_case_replay_preserves_sla_clock`.
- Load test `crm_latency_p99_meets_target_under_mutation_mix`.
- Load test `crm_throughput_target_holds_for_six_aggregate_mix`.
- Metric `crm-availability-sli`.
- Metric `crm-latency-p99-sli`.
- Metric `crm-throughput-sli`.
- Metric `account-master-success-rate-sli`.
- Metric `oya_crm_quote_approval_denied_total`.
- Metric `oya_crm_service_case_sla_breach_total`.
- Dashboard `dashboards/crm-overview.json`.
- Dashboard `dashboards/account-master-health.json`.
- Dashboard `dashboards/opportunity-residency.md`.
- Dashboard panel `Quote approvals by policy version`.
- Dashboard panel `Service-case SLA burn by pack`.
- Runbook check `runbooks/account-merge-rollback.md` covers merge reversal.
- Runbook check `runbooks/quote-approval-regression.md` covers policy rollback.
- Promotion gate blocks if any CRM mutation lacks evidence id.
- Promotion gate blocks if account master success rate is below 0.999.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0104: Ecosystem expansion toolchain and adapters.
- Oyatie ADR-0128: Hyperscaler architecture invariants.
- Oyatie ADR-0131: Per-microservice flat layout.
- RFC 6902: JSON Patch.
- RFC 7396: JSON Merge Patch.
- RFC 9110: HTTP Semantics.
- W3C Trace Context Recommendation.
- CloudEvents specification.
- Salesforce REST API and Platform Events documentation.
- SAP Sales Cloud and SAP Cloud for Customer documentation.
- Microsoft Dataverse Web API documentation.
- Oracle CX Sales documentation.
- NetSuite SuiteTalk REST Web Services documentation.
- Google SRE Workbook: SLOs and error budget policy.
- Cedar policy language documentation.
