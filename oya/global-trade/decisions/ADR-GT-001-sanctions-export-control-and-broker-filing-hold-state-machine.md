---
id: ADR-GT-001
title: sanctions-export-control-and-broker-filing-hold-state-machine
status: inventory-provenance-planned-only
date: 2026-05-20
microservice: global-trade
related_oyatie_adrs:
  - docs/decisions/ADR-0002-tenant-and-identity-kernel.md
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0005-eventing-backbone-outbox-pattern.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0010-regional-pack-architecture.md
  - docs/decisions/ADR-0011-cross-microservice-contract-registry.md
  - docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md
  - docs/decisions/ADR-0042-observability-stack-otel-and-in-house-ui.md
decision_owner: global-trade-platform-architecture
current_authority: specs/microservices/global-trade.json
authority_status: metadata-only PRD
inventory_status: inventory/provenance/planned-only
---

> Current authority: specs/microservices/global-trade.json metadata-only PRD; inventory/provenance/planned-only. This document is historical/proposed design provenance only; it is not current implementation, runtime, cloud, API, SLO, dashboard, runbook, filing, broker, Workflow, or audit-chain authority.


# ADR-GT-001: Sanctions, Export Control, And Broker Filing Hold State Machine

## Context

- Architectural pressure: Regulated Shipment Stop-Ship Pressure.
- Global Trade owns customs declarations, sanctions screening, export control classification, trade documents, denied-party hits, and broker filing evidence.
- The service cannot let order, fulfillment, procurement, or finance workflows ship around an unresolved trade hold.
- The service also cannot become a generic document store because declaration, screening, and filing decisions carry different legal retention and release semantics.
- Sanctions screening has to reconcile vendor list updates, tenant overrides, jurisdiction overlays, and manual adjudication without losing the original match basis.
- Export control classification has to bind ECCN, HS code, country of origin, destination, end use, end user, and preferential trade agreement evidence.
- Broker filing has to expose deterministic handoff state even when external broker or customs endpoints are delayed, partially acknowledged, or manually corrected.
- Customs declaration values must remain reproducible from invoice, packing, origin, and classification inputs.
- Denied-party hits must be held separately from low-risk fuzzy matches so the operational path can preserve shipment velocity.
- The trade document subsystem must emit certificate, commercial invoice, packing list, and proof-of-origin artifacts with immutable hash references.
- Named constraint: Stop-Ship Invariant.
- The Stop-Ship Invariant requires every shipment export, customs declaration, and broker filing attempt to consult the current hold state before release.
- Named constraint: Jurisdiction Overlay Constraint.
- The Jurisdiction Overlay Constraint requires policy selection by export country, import country, tenant region pack, and product classification.
- Named constraint: Evidence Replay Constraint.
- The Evidence Replay Constraint requires the service to reconstruct a screening or classification decision from immutable inputs and referenced list versions.
- Named constraint: Broker Acknowledgement Constraint.
- The Broker Acknowledgement Constraint treats submitted, acknowledged, rejected, corrected, and cancelled filing states as distinct legal facts.
- Named constraint: False Positive Budget Constraint.
- The False Positive Budget Constraint caps low-risk fuzzy match escalation at 3 percent of screened parties per tenant per day unless a watchlist delta overrides it.
- Named constraint: Filing Deadline Constraint.
- The Filing Deadline Constraint requires pre-export declarations to reach broker-submitted or explicit-hold state at least 30 minutes before planned departure.
- Named constraint: Cross-Service Contract Constraint.
- The Cross-Service Contract Constraint requires procurement, warehouse, supply-chain-planning, and finance integrations to consume stable hold events instead of reading internal tables.
- Named constraint: Data Residency Constraint.
- The Data Residency Constraint requires sanctions evidence, end-user declarations, and customs documents to stay inside the selected regional pack unless a broker connector explicitly permits transfer.
- Named constraint: Audit Chain Constraint.
- The Audit Chain Constraint requires every automatic hold, manual release, classification override, and broker resubmission to emit an audit event before side effects leave the service.
- Current IP coverage names customs-declaration, sanctions-screening, export-control-classification, trade-document, denied-party-hit, and broker-filing as independent slices.
- Without a single hold state machine, those slices could independently pass their own checks while the shipment remains unsafe to release.
- The PRD benchmark set includes SAP GTS, Oracle GTM, Descartes, and Amber Road, all of which treat trade compliance as a blocking operational control.
- Oyatie needs the same stop-ship semantics while preserving a flat internal catalog and bounded microservice ownership.
- The service must support high-volume screening for ordinary parties and low-volume manual workflows for high-risk hits.
- The design must survive list update races, broker callback retries, duplicate event delivery, and delayed customs responses.
- The decision must be expressible in API contracts, AsyncAPI events, Cedar policy, dashboards, and runbooks.

## Decision

- We will implement the Deterministic Trade Hold State Machine for all outbound shipment release decisions owned by global-trade.
- The named pattern is an append-only compliance ledger plus normalized state projection plus transactional outbox.
- The named technology choice is service-local Postgres for ledger and projection, Cedar for release authorization, and AsyncAPI CloudEvents for cross-service hold publication.
- The state machine will make TradeHold the single release gate for shipment, declaration, broker filing, and trade document workflows.
- A TradeHold can be in clear, pending_screening, pending_classification, pending_broker_ack, blocked_sanctions, blocked_export_control, blocked_customs, blocked_documentation, manual_review, released, cancelled, or expired.
- The projection must compute one current hold state per tenant_id, shipment_id, export_country, import_country, and product_bundle_id.
- Sanctions screening runs must attach a watchlist_version, match_algorithm_version, match_score, normalized_party_hash, and adjudication_state.
- Exact list matches with score >= 0.95 move the hold to blocked_sanctions.
- Fuzzy matches with score >= 0.80 and < 0.95 move the hold to manual_review.
- Fuzzy matches with score < 0.80 may clear only when no jurisdiction pack marks the list as strict.
- Export classifications must attach hs_code, eccn_code, origin_country, destination_country, end_use_code, end_user_risk_tier, and classifier_version.
- Any controlled ECCN with destination_country in a restricted policy set moves the hold to blocked_export_control until license evidence is attached.
- Customs declarations must attach declaration_number, declared_value_minor, currency, incoterm, line_count, broker_id, and filing_deadline_at.
- Broker filing callbacks must be idempotent by broker_id, external_filing_id, callback_sequence, and callback_digest.
- A rejected broker filing moves the hold to blocked_customs when rejection_code severity is high.
- A corrected broker filing remains pending_broker_ack until the corrected filing has a broker acknowledgement or explicit customs acceptance.
- Trade document generation must attach document_kind, content_hash, template_version, signer_principal, and retention_class.
- A missing certificate of origin for a preferential-origin claim moves the hold to blocked_documentation.
- Manual release is permitted only for manual_review, blocked_documentation, and low-severity blocked_customs.
- Manual release is not permitted for blocked_sanctions unless the sanction authority policy marks the hit as false positive and requires two approvers.
- Manual release is not permitted for blocked_export_control unless license evidence references a valid license number, license issuer, expiry date, and product line coverage.
- The release API will enforce a p95 authorization decision latency of 75 ms and a p99 latency of 200 ms for cached policy packs.
- Sanctions screening ingestion will target p95 completion within 2 seconds for single-party checks and p99 within 10 seconds for 100-party shipment batches.
- Export classification projection will target p95 completion within 500 ms per line and p99 within 2 seconds for 500-line declarations.
- Broker filing callback processing will target p95 acknowledgement projection within 1 second and p99 within 5 seconds after receipt.
- Hold publication will target p99 event emission within 3 seconds after a committed state transition.
- Filing deadline guardrails will page the global-trade on-call if pending_broker_ack remains within 30 minutes of departure for priority shipments.
- The service will reject direct shipment release reads from internal tables; consumers must use the TradeHold projection API or subscribe to hold events.
- Every state transition will write TradeHoldTransition and GlobalTradeAuditEvent in the same transaction.
- The outbox worker will emit global_trade.trade_hold.v1, global_trade.screening_result.v1, global_trade.classification_result.v1, and global_trade.broker_filing_status.v1 events.
- Each emitted event must include traceparent, tenant_id, region_pack, policy_pack_version, evidence_hash, and decision_epoch.

## Alternatives Considered

- Alternative 1: Broker-Centric Compliance Gate.
- Pros: Broker systems already know customs filing state and can reduce internal workflow complexity.
- Pros: External acknowledgement can be treated as a simple release token.
- Cons: Sanctions, export control, and document readiness are not always broker-owned.
- Cons: Tenant-specific jurisdiction overlays would become invisible to Oyatie policy and audit chains.
- Cons: Broker outages would block internal adjudication even when a false positive is resolvable.
- Rejection reason: Broker state is an input to the hold, not the authority for the whole release decision.

- Alternative 2: Per-Domain Independent Gates.
- Pros: Sanctions, classification, declarations, documents, and broker filing could evolve independently.
- Pros: Each IP slice could expose small APIs with limited coupling.
- Cons: A shipment could appear clear in one domain while another domain still blocks release.
- Cons: Consumers would need to compose five gates correctly every time.
- Cons: Audit reconstruction would require cross-table and cross-event joins during an incident.
- Rejection reason: The stop-ship invariant requires one current release decision.

- Alternative 3: Generic Workflow Engine For Trade Holds.
- Pros: A shared workflow engine could express approvals, timers, and retries.
- Pros: Business operators might configure new paths without code changes.
- Cons: Export control and sanctions transitions require hard-coded legal invariants and list-version replay.
- Cons: Generic workflow configuration increases the risk of unsafe local overrides.
- Cons: State replay would depend on workflow engine history rather than service-owned ledger semantics.
- Rejection reason: Global trade needs deterministic domain transitions before configurable workflow breadth.

- Alternative 4: Event-Only Hold Derivation.
- Pros: Consumers could derive release status from published screening, classification, and filing events.
- Pros: The service would avoid a current-state projection table.
- Cons: Every consumer would need the exact same event ordering and conflict-resolution logic.
- Cons: Late broker callbacks and list-update corrections would create inconsistent release views.
- Cons: Support teams would lack a single dashboard row for current operational status.
- Rejection reason: Cross-service release safety requires a service-owned projection.

- Alternative 5: Full External GTS Delegation.
- Pros: SAP GTS or Oracle GTM can provide mature classification and screening features.
- Pros: External delegation could accelerate parity for multinational enterprises.
- Cons: It would conflict with Oyatie's in-house domain ownership and audit-chain requirements.
- Cons: Tenant pack policy, Cedar authorization, and product ontology evidence would be fragmented.
- Cons: Cost, integration latency, and data residency constraints would vary by customer.
- Rejection reason: External systems may be connectors, not the source of Oyatie release authority.

## Consequences

- Positive consequence: Shipment release has one explainable state that procurement, warehouse, finance, and supply-chain-planning can consume.
- Positive consequence: A denied-party hit, export-control license gap, customs rejection, or missing certificate creates a named hold reason instead of an opaque failure.
- Positive consequence: Audit reconstruction can start from TradeHoldTransition and follow immutable evidence hashes.
- Positive consequence: Regional pack behavior is visible in the policy_pack_version attached to each transition.
- Positive consequence: Broker retries and duplicate callbacks are contained by idempotency keys.
- Positive consequence: False-positive adjudication becomes measurable through match score bands and manual review outcomes.
- Positive consequence: Product classification debt is surfaced before filing rather than after customs rejection.
- Negative consequence: The state machine becomes a high-importance operational dependency for shipping workflows.
- Negative consequence: Incorrect threshold configuration can create either excessive manual review or unsafe automatic release.
- Negative consequence: Broker connector behavior must be normalized carefully across jurisdictions and vendors.
- Negative consequence: Historical replay requires preserving list versions, classifier versions, and document template versions.
- Negative consequence: Manual release UX must explain why some states are non-releasable by humans.
- Neutral consequence: The global-trade service now owns the current hold projection even though some evidence is imported from external systems.
- Neutral consequence: Cross-service consumers must update to TradeHold projection contracts and stop duplicating trade logic.
- Neutral consequence: The implementation aligns global-trade with the same ledger and outbox posture used by other regulated Oyatie services.
- Follow-up work: GT-FW-001 will add watchlist delta ingestion and version pinning tests.
- Follow-up work: GT-FW-002 will define broker-specific callback adapters for Descartes, SAP GTS, and manual broker upload.
- Follow-up work: GT-FW-003 will add preferential origin evidence validation for FTA claims.
- Follow-up work: GT-FW-004 will add a release-safety dashboard tile to the global-trade overview.
- Follow-up work: GT-FW-005 will add migration tooling for existing declaration records into TradeHold projections.

## Implementation Notes

- Data shape: TradeHold.
- TradeHold fields: hold_id, tenant_id, shipment_id, export_country, import_country, product_bundle_id, current_state, reason_code, severity, decision_epoch.
- TradeHold fields: policy_pack_version, region_pack, evidence_hash, released_by, released_at, expires_at, created_at, updated_at.
- Data shape: TradeHoldTransition.
- TradeHoldTransition fields: transition_id, hold_id, from_state, to_state, trigger_kind, trigger_ref, actor_principal, cedar_decision_id.
- TradeHoldTransition fields: idempotency_key, transition_reason, evidence_hash, occurred_at, traceparent.
- Data shape: SanctionsScreeningRun.
- SanctionsScreeningRun fields: screening_id, tenant_id, shipment_id, party_id, normalized_party_hash, list_provider, watchlist_version.
- SanctionsScreeningRun fields: match_score, match_basis, algorithm_version, adjudication_state, adjudicated_by, adjudicated_at.
- Data shape: DeniedPartyHit.
- DeniedPartyHit fields: hit_id, screening_id, list_name, entry_id, matched_name, matched_country, score, risk_tier, false_positive_reason.
- Data shape: ExportControlClassification.
- ExportControlClassification fields: classification_id, tenant_id, product_id, hs_code, eccn_code, origin_country, destination_country.
- ExportControlClassification fields: end_use_code, end_user_risk_tier, license_required, license_ref, classifier_version.
- Data shape: CustomsDeclaration.
- CustomsDeclaration fields: declaration_id, tenant_id, shipment_id, declaration_number, declared_value_minor, currency, incoterm.
- CustomsDeclaration fields: broker_id, filing_deadline_at, declaration_state, rejection_code, correction_sequence.
- Data shape: BrokerFiling.
- BrokerFiling fields: broker_filing_id, broker_id, external_filing_id, declaration_id, submitted_at, acknowledged_at, rejected_at.
- BrokerFiling fields: callback_sequence, callback_digest, broker_status, customs_status, retry_count.
- Data shape: TradeDocument.
- TradeDocument fields: document_id, shipment_id, document_kind, content_hash, template_version, signer_principal, retention_class, issued_at.
- Data shape: PreferentialOriginClaim.
- PreferentialOriginClaim fields: claim_id, shipment_id, agreement_code, origin_rule, certificate_document_id, claimant_principal, expires_at.
- API endpoint: POST /v1/global-trade/screenings creates a sanctions screening run and returns the current hold projection.
- API endpoint: POST /v1/global-trade/classifications resolves export control classification for product lines.
- API endpoint: POST /v1/global-trade/customs-declarations creates or revises a declaration with line-level evidence.
- API endpoint: POST /v1/global-trade/broker-filings submits a declaration to a broker adapter.
- API endpoint: POST /v1/global-trade/broker-callbacks/{broker_id} receives idempotent broker callbacks.
- API endpoint: GET /v1/global-trade/trade-holds/{hold_id} returns the current projection and last 20 transitions.
- API endpoint: POST /v1/global-trade/trade-holds/{hold_id}/release requests a manual or evidence-backed release.
- API endpoint: POST /v1/global-trade/trade-documents generates certificate, invoice, packing, or origin documents.
- API endpoint: GET /v1/global-trade/shipments/{shipment_id}/release-decision returns clear, hold, or expired with reason codes.
- Event: global_trade.trade_hold.v1 publishes every committed hold transition.
- Event: global_trade.screening_result.v1 publishes sanctions result summaries without leaking full list payloads to unauthorized consumers.
- Event: global_trade.classification_result.v1 publishes classification outcome and evidence hash.
- Event: global_trade.broker_filing_status.v1 publishes submitted, acknowledged, rejected, corrected, and cancelled broker states.
- Cedar policy: sanctions-screening-authorization.cedar permits screening requests only for principals with trade_compliance_read or shipment_release duties.
- Cedar policy: export-control-classification-authorization.cedar permits classification override only to trade_compliance_admin and export_control_officer roles.
- Cedar policy: denied-party-hit-authorization.cedar prevents non-compliance personas from reading full denied-party match evidence.
- Cedar policy: customs-declaration-authorization.cedar requires broker_filing_operator or customs_manager for declaration submission.
- Cedar policy: broker-filing-authorization.cedar requires region_pack compatibility between broker connector and tenant residency settings.
- Cedar policy: trade-document-authorization.cedar requires document_kind-specific signer authority before issuing certificates.
- Cedar policy: trade-hold-release.cedar denies release for blocked_sanctions without two distinct approvers and false_positive_reason.
- Cedar policy: trade-hold-release.cedar denies release for blocked_export_control without license_ref and non-expired license coverage.
- Cedar policy: auditor-scope.cedar permits read-only access to transitions, evidence hashes, and policy decisions for audit principals.
- Cedar policy: data-residency.md maps broker transfer permissions to region_pack and destination_country.
- SLO target: release decision read availability is 99.95 percent monthly.
- SLO target: release decision read latency is p95 75 ms and p99 200 ms for cached projections.
- SLO target: sanctions screening single-party completion is p95 2 seconds and p99 10 seconds.
- SLO target: 100-party shipment batch screening completion is p95 8 seconds and p99 30 seconds.
- SLO target: export classification line projection is p95 500 ms and p99 2 seconds.
- SLO target: broker callback projection is p95 1 second and p99 5 seconds after receipt.
- SLO target: outbox publication lag is p99 under 3 seconds for hold transitions.
- SLO target: audit event completeness is 100 percent for state transitions in production.
- Dashboard: global-trade-overview.json shows hold counts by state, jurisdiction, and severity.
- Dashboard: customs-declaration-health.json shows declaration acceptance, rejection, correction, and broker callback latency.
- Dashboard: sanctions-screening-residency.md shows watchlist version freshness and cross-region transfer exceptions.
- Runbook: denied-party-hit adjudication describes false positive evidence capture and two-approver release.
- Runbook: broker-filing retry describes callback dedupe, corrected filing submission, and customs rejection escalation.
- Runbook: preferential-origin gap describes certificate regeneration and FTA claim invalidation.

## Verification

- Test: trade_hold_state_machine_exact_match_blocks_release creates a score 0.98 denied-party hit and asserts blocked_sanctions.
- Test: trade_hold_state_machine_fuzzy_match_manual_review creates a score 0.87 match and asserts manual_review.
- Test: trade_hold_state_machine_low_score_clears_when_policy_allows creates score 0.62 evidence and asserts clear under non-strict policy.
- Test: export_control_restricted_destination_blocks creates controlled ECCN plus restricted destination and asserts blocked_export_control.
- Test: export_control_license_releases_when_valid attaches license_ref with product coverage and asserts released.
- Test: broker_callback_idempotency_replays_same_digest submits duplicate callback_sequence and asserts one transition.
- Test: broker_rejection_high_severity_blocks_customs maps rejection severity high to blocked_customs.
- Test: missing_origin_certificate_blocks_documentation creates preferential claim without certificate and asserts blocked_documentation.
- Test: manual_release_denied_for_sanctions_without_two_approvers asserts Cedar denies single-approver release.
- Test: release_projection_consumer_contract validates OpenAPI response for /shipments/{shipment_id}/release-decision.
- Test: outbox_emits_hold_transition_once validates CloudEvents idempotency key and traceparent propagation.
- Test: evidence_replay_reconstructs_transition verifies list version, classifier version, and document hash replay.
- Test: broker_deadline_guard_pages_before_departure simulates pending_broker_ack within 30 minutes of departure.
- Test: data_residency_denies_broker_transfer_without_pack_allowance asserts Cedar denies incompatible broker route.
- Metric: global_trade_hold_current_total by tenant_id, current_state, export_country, import_country, and severity.
- Metric: global_trade_screening_duration_seconds by provider, watchlist_version, match_band, and tenant_id.
- Metric: global_trade_classification_duration_seconds by classifier_version and line_count_bucket.
- Metric: global_trade_broker_callback_lag_seconds by broker_id and callback_status.
- Metric: global_trade_hold_transition_outbox_lag_seconds by event_type and region_pack.
- Metric: global_trade_manual_release_total by state, reason_code, approver_count, and outcome.
- Metric: global_trade_false_positive_rate by list_provider, watchlist_version, and tenant_id.
- Metric: global_trade_declaration_rejection_total by broker_id, rejection_code, and severity.
- Metric: global_trade_evidence_replay_failure_total by evidence_kind and version.
- Dashboard: Global Trade Release Safety shows blocked_sanctions, blocked_export_control, and manual_review burn.
- Dashboard: Broker Filing Deadline Risk shows pending_broker_ack within 60, 30, and 10 minutes of departure.
- Dashboard: Watchlist Freshness shows provider list age, failed imports, and strict-list fallback mode.
- Dashboard: Trade Document Completeness shows missing certificates, stale template versions, and signer failures.
- Dashboard: Export Control Classification Quality shows override rate, license gaps, and classifier drift.
- Alert: GlobalTradeHoldOutboxLagHigh fires when p99 outbox lag exceeds 3 seconds for 10 minutes.
- Alert: GlobalTradeBrokerDeadlineRisk fires when priority shipments stay pending within 30 minutes of departure.
- Alert: GlobalTradeWatchlistStale fires when strict watchlist provider age exceeds 6 hours.
- Alert: GlobalTradeAuditCompletenessBroken fires on any transition without an audit event.
- Promotion gate: run contract tests for OpenAPI, AsyncAPI, and protobuf surfaces before dev promotion.
- Promotion gate: run replay tests for 1000 historical screening and filing events before regional rollout.
- Promotion gate: run Cedar policy tests for sanctions release, export license release, broker transfer, and auditor read scope.
- Promotion gate: run load test at 200 screening requests per second and 100 broker callbacks per second.

## References

- OFAC, Specially Designated Nationals And Blocked Persons List documentation.
- U.S. Bureau of Industry and Security, Export Administration Regulations and Commerce Control List.
- World Customs Organization, Harmonized Commodity Description and Coding System guidance.
- World Trade Organization, Rules of Origin Agreement.
- European Commission, TARIC integrated tariff documentation.
- United Nations Security Council Consolidated List documentation.
- SAP Global Trade Services product documentation.
- Oracle Global Trade Management Cloud documentation.
- Descartes Global Logistics Network and customs filing documentation.
- CloudEvents Specification 1.0.2.
- OpenAPI Specification 3.1.0.
- AsyncAPI Specification 3.0.0.
- RFC 9110, HTTP Semantics.
- W3C Trace Context Recommendation.
