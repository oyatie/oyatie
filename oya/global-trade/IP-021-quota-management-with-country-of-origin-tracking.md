---
doc_class: ImplementationPlan
ip_id: IP-021
microservice: global-trade
related_adrs:
  - ADR-0105
  - ADR-0243
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0304
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_id: j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow
journey_link: docs/user-journeys/j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow/README.md
status: Accepted
date: 2026-05-20
owner: axis-global-trade
tenant_class_eligibility: [demo_trial, paid]
sap_submodule_equivalents:
  - SAP GTS-CC customs-quota-management
  - SAP GTS-PI origin-and-preference-support
  - SAP GTS-EM quota-status-distribution
---

# IP-021: Quota management with country-of-origin tracking

## Context With Why
- This IP builds quota management tied to country-of-origin tracking for import and export trade controls.
- The why is risk and margin: quota exhaustion can create unexpected duties, blocked declarations, or wrong preference claims.
- Quota decisions must depend on origin evidence, classification, jurisdiction, and time window.
- The journey leg starts when a classified product line with origin evidence is planned for import or export.
- The journey leg ends when quota availability is reserved, consumed, released, or escalated with audit evidence.
- Named persona: Samir, an import planner, needs to know whether a steel component shipment can use remaining quota before booking freight.
- Samir can reserve quota for planned shipments, but he cannot consume quota until customs declaration evidence exists.
- This IP maps SAP GTS-CC customs quota management and SAP GTS-PI origin support into Oyatie.
- SAP GTS-EM is relevant because quota status must be distributed to broker and planning surfaces.
- The implementation must not own inventory planning or purchase order planning.
- The implementation stores quota programs, quota balances, origin evidence refs, reservations, and consumption events.
- ADR-0105 keeps quota domain logic separate from external quota feed adapters.
- ADR-0243 requires provenance for quota feed, origin evidence, and declaration consumption.
- ADR-0244 requires tenant and sub-scope isolation on quota reservations and balances.
- ADR-0253 requires Cedar default-deny for reserve, consume, release, override, and export.
- ADR-0263 requires chainable quota reservation and consumption events.
- ADR-0304 requires ontology projection of QuotaProgram, OriginEvidence, and QuotaReservation.
- ADR-0315 sets SAP GTS parity for quota and origin workflows.
- Intern build target: one quota program aggregate, one reservation aggregate, one origin evidence reference model, one allocation service, and one broker status worker.

## Scope Boundaries
- In scope: quota program setup, quota feed import, origin evidence linkage, reservation, consumption, release, and override workflow.
- In scope: country of origin, HS code, quota period, quantity unit, reserved quantity, consumed quantity, and remaining quantity.
- In scope: quota warnings for customs-declaration, broker-filing, and certificate generation.
- In scope: split-line handling when partial quota remains.
- Out of scope: production planning, procurement optimization, and supplier compliance scoring.
- Out of scope: final duty calculation.
- Out of scope: supplier origin solicitation workflows beyond evidence refs.
- Boundary rule: quota reservation is a planning hold; customs declaration consumption is the authoritative usage event.
- Boundary rule: origin evidence must be approved before quota can be consumed.
- Boundary rule: quota feed imports are staged before active balance update.
- Boundary rule: over-quota override requires separate Cedar action and audit event.

## Data Model Deltas
- Table: `global_trade_quota_program`.
- Column: `tenant_id uuid not null`.
- Column: `quota_program_id uuid primary key`.
- Column: `program_code text not null`.
- Column: `jurisdiction text not null`.
- Column: `hs_code text not null`.
- Column: `origin_country text not null`.
- Column: `quota_period_start date not null`.
- Column: `quota_period_end date not null`.
- Column: `quantity_limit numeric(18,4) not null`.
- Column: `quantity_unit text not null`.
- Column: `feed_version_ref text not null`.
- Column: `program_state text not null check program_state in ('staged','active','closed','superseded')`.
- Unique: `gt_quota_program_uq` on `(tenant_id, program_code, jurisdiction, hs_code, origin_country, quota_period_start)`.
- Table: `global_trade_quota_balance`.
- Column: `tenant_id uuid not null`.
- Column: `quota_balance_id uuid primary key`.
- Column: `quota_program_id uuid not null references global_trade_quota_program(quota_program_id)`.
- Column: `reserved_quantity numeric(18,4) not null default 0`.
- Column: `consumed_quantity numeric(18,4) not null default 0`.
- Column: `released_quantity numeric(18,4) not null default 0`.
- Column: `remaining_quantity numeric(18,4) not null`.
- Column: `balance_version integer not null`.
- Table: `global_trade_quota_reservation`.
- Column: `tenant_id uuid not null`.
- Column: `quota_reservation_id uuid primary key`.
- Column: `quota_program_id uuid not null`.
- Column: `source_line_ref text not null`.
- Column: `product_ref text not null`.
- Column: `origin_evidence_ref text not null`.
- Column: `reserved_quantity numeric(18,4) not null`.
- Column: `reservation_state text not null check reservation_state in ('reserved','consumed','released','expired','overridden')`.
- Column: `expires_at timestamptz not null`.
- Column: `idempotency_key text not null`.
- Column: `policy_bundle_version text not null`.
- Column: `audit_chain_ref text not null`.
- Table: `global_trade_origin_evidence_ref`.
- Column: `tenant_id uuid not null`.
- Column: `origin_evidence_ref text primary key`.
- Column: `product_ref text not null`.
- Column: `origin_country text not null`.
- Column: `evidence_source_ref text not null`.
- Column: `evidence_hash text not null`.
- Column: `approval_state text not null check approval_state in ('pending','approved','rejected','expired')`.
- Retention: balance changes are append-driven; direct balance correction requires override event.
- Retention: origin evidence refs are immutable snapshots and can expire.

## API Endpoints
- REST: `POST /v1/global-trade/quotas:reserve`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "usr_samir_import",
  "idempotency_key": "quota-res-2026-05-20-77",
  "jurisdiction": "US",
  "hs_code": "7308.90.9590",
  "origin_country": "KR",
  "product_ref": "sku-STEEL-BRACKET-9",
  "source_line_ref": "po-line:8844:1",
  "origin_evidence_ref": "origin:sku-STEEL-BRACKET-9:KR:2026",
  "quantity": "2400.0000",
  "quantity_unit": "KG"
}
```
- REST response example:
```json
{
  "quota_reservation_id": "quotares_01jytg_021",
  "reservation_state": "reserved",
  "quota_program_id": "quota_steel_kr_2026",
  "reserved_quantity": "2400.0000",
  "remaining_quantity": "18400.0000",
  "audit_event_class": "EVT-GLOBAL_TRADE-QUOTA-RESERVED"
}
```
- REST: `POST /v1/global-trade/quotas/reservations/{quota_reservation_id}:consume`.
- REST: `POST /v1/global-trade/quotas/reservations/{quota_reservation_id}:release`.
- REST: `POST /v1/global-trade/quotas/reservations/{quota_reservation_id}:override`.
- REST: `GET /v1/global-trade/quotas?jurisdiction={jurisdiction}&hs_code={hs_code}`.
- REST: `GET /v1/global-trade/quotas/reservations/{quota_reservation_id}`.
- gRPC: `ReserveQuota(ReserveQuotaRequest) returns (ReserveQuotaResult)`.
- gRPC: `ConsumeQuota(ConsumeQuotaRequest) returns (ConsumeQuotaResult)`.
- gRPC: `ReleaseQuota(ReleaseQuotaRequest) returns (ReleaseQuotaResult)`.
- Worker command: `global-trade.quota.import-feed`.
- Worker command: `global-trade.quota.expire-reservations`.
- Error envelope: `POLICY_DENIED`, `QUOTA_NOT_FOUND`, `ORIGIN_EVIDENCE_NOT_APPROVED`, `QUOTA_EXHAUSTED`, `BALANCE_CONFLICT`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"usr_samir_import"`.
- Action: `GlobalTrade::Action::"ReserveQuota"`.
- Resource: `GlobalTrade::QuotaProgram::"quota_steel_kr_2026"`.
- Context field: `tenant_id`.
- Context field: `jurisdiction`.
- Context field: `hs_code`.
- Context field: `origin_country`.
- Context field: `quantity`.
- Context field: `source_line_ref`.
- Consume action: `GlobalTrade::Action::"ConsumeQuota"`.
- Release action: `GlobalTrade::Action::"ReleaseQuota"`.
- Override action: `GlobalTrade::Action::"OverrideQuota"`.
- Allow rule intent: import planners can reserve quota for assigned product families.
- Allow rule intent: customs declaration service can consume quota after declaration approval.
- Deny rule intent: reservations with unapproved origin evidence are denied.
- Deny rule intent: over-quota consumption is denied unless override action is explicitly allowed.
- Deny rule intent: broker principals can read quota status but cannot reserve or override quota.
- Audit on allow: include program code, origin country, quantity, and balance version.
- Audit on deny: include requested quantity and reason without exposing supplier data.

## Ontology Projection Field Mapping
- Ontology node: `QuotaProgram`.
- `quota_program_id` maps to `QuotaProgram.id`.
- `program_code` maps to `QuotaProgram.programCode`.
- `jurisdiction` maps to `QuotaProgram.jurisdiction`.
- `hs_code` maps to `QuotaProgram.hsCode`.
- `origin_country` maps to `QuotaProgram.originCountry`.
- `quantity_limit` maps to `QuotaProgram.limitQuantity`.
- `remaining_quantity` maps to `QuotaProgram.remainingQuantity`.
- Ontology node: `OriginEvidence`.
- `origin_evidence_ref` maps to `OriginEvidence.id`.
- `product_ref` maps to `OriginEvidence.productRef`.
- `origin_country` maps to `OriginEvidence.country`.
- `evidence_hash` maps to `OriginEvidence.hash`.
- `approval_state` maps to `OriginEvidence.approvalState`.
- Ontology node: `QuotaReservation`.
- `quota_reservation_id` maps to `QuotaReservation.id`.
- `source_line_ref` maps to `QuotaReservation.sourceLineRef`.
- `reserved_quantity` maps to `QuotaReservation.reservedQuantity`.
- `reservation_state` maps to `QuotaReservation.state`.
- Projection mode: project QuotaProgram and balance after active feed, OriginEvidence after approval, QuotaReservation after reserve or state change.
- Projection guard: balance projection must include balance version to prevent stale UI reads.

## Workflow Steps
- Node `ReceiveReserveRequest`: validate tenant, product, HS code, origin, quantity, and idempotency.
- Node `LoadOriginEvidence`: verify approved origin evidence and expiration.
- Branch `OriginEvidenceMissing`: deny reservation and create evidence task.
- Node `FindQuotaProgram`: locate active quota program by jurisdiction, HS code, origin, and period.
- Branch `NoQuotaProgram`: return no-quota response and allow declaration to proceed without reservation if policy allows.
- Node `CheckRemainingBalance`: compare requested quantity with remaining quantity.
- Branch `QuotaAvailable`: create reservation and decrement remaining available balance.
- Branch `PartialQuotaAvailable`: split reservation and create workflow decision.
- Branch `QuotaExhausted`: return exhausted status and notify planner.
- Node `RunCedarAuthorization`: enforce reserve, consume, release, or override.
- Node `SealAuditEvent`: append ADR-0263 quota event.
- Node `ProjectOntology`: project quota program, balance, and reservation.
- Node `NotifyDownstream`: publish status to customs-declaration, broker-filing, and planning source.
- Node `ConsumeOnDeclaration`: consume reserved quantity after customs declaration approval.
- Node `ReleaseOnCancellation`: release reservation when source line is cancelled or expired.
- Branch `BalanceConflict`: retry with optimistic lock and then create exception task.

## Audit Events
- `EVT-GLOBAL_TRADE-QUOTA-FEED_IMPORTED`.
- `EVT-GLOBAL_TRADE-QUOTA-PROGRAM_ACTIVATED`.
- `EVT-GLOBAL_TRADE-QUOTA-RESERVE_REQUESTED`.
- `EVT-GLOBAL_TRADE-QUOTA-RESERVED`.
- `EVT-GLOBAL_TRADE-QUOTA-PARTIAL_RESERVED`.
- `EVT-GLOBAL_TRADE-QUOTA-EXHAUSTED`.
- `EVT-GLOBAL_TRADE-QUOTA-CONSUMED`.
- `EVT-GLOBAL_TRADE-QUOTA-RELEASED`.
- `EVT-GLOBAL_TRADE-QUOTA-OVERRIDDEN`.
- `EVT-GLOBAL_TRADE-QUOTA-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-QUOTA-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `quota_program_id`, `quota_reservation_id`, `quantity`, `balance_version`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: balance changes include prior and next balance versions.
- Event rule: override events include override reason and approver principal.

## SLO Targets
- Availability target: 99.95 percent monthly for reserve, consume, and release.
- Throughput target: 400 quota operations per second per region.
- p50 latency target: 70 ms for reserve with cached active quota program.
- p95 latency target: 350 ms for reserve with origin evidence lookup.
- p99 latency target: 1200 ms for split-line partial quota handling.
- Feed promotion target: validated quota feed active within 10 minutes for 250,000 rows.
- Reservation expiration target: expired reservations released within 2 minutes.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Rationale: planners and declaration flows need fast quota answers; feed ingestion can run asynchronously.
- Burn alert: page when balance conflict retries exceed 2 percent or reservation expiration lag exceeds 10 minutes.

## Failure Modes And Recovery
- Failure: quota feed import has invalid period overlap; recovery: reject staged feed and keep prior active programs.
- Failure: origin evidence is pending or expired; recovery: deny reserve and create origin evidence workflow task.
- Failure: remaining balance is insufficient; recovery: return partial or exhausted result by tenant policy.
- Failure: optimistic balance conflict; recovery: retry reservation with latest balance version.
- Failure: consume request arrives without reservation; recovery: attempt direct consume if policy allows or create exception.
- Failure: release arrives after consumption; recovery: reject release and emit conflict event.
- Failure: override requested by unauthorized principal; recovery: Cedar deny and emit deny event.
- Failure: ontology projection fails; recovery: retry projection and mark reservation visible only through source endpoint.
- Failure: audit chain seal fails; recovery: rollback balance change and retry from command.
- Failure: broker reads stale quota status; recovery: require balance version in broker response.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS customs quota management records.
- SAP source: SAP GTS preference and origin evidence records.
- SAP source: SAP GTS electronic messaging quota status outputs.
- Oracle source: GTM quota management and country of origin records.
- Descartes source: quota tracking and origin qualification exports.
- Amber Road source: quota and origin control data.
- Tenant source: quota spreadsheets and customs broker quota advisories.
- Migration step: import quota programs as staged feed versions.
- Migration step: validate period overlaps before activation.
- Migration step: map origin evidence refs to existing classification and certificate evidence.
- Migration step: import open reservations with expiration timestamps.
- Migration step: reconcile consumed quantities against customs declaration history.
- Migration step: create balance adjustment events for approved historical corrections.

## Cross-Microservice Handoffs
- From HS classification: HS code and product classification refs.
- From origin determination or certificate generation: approved origin evidence refs.
- To customs-declaration: quota reservation and consumption status.
- To broker-filing: quota status packet and release notice.
- To workflow-engine: partial quota, evidence missing, override, and feed validation tasks.
- To ontology: QuotaProgram, OriginEvidence, and QuotaReservation projections.
- To audit-chain: ADR-0263 event append and chain verification.
- To notification: quota exhaustion, reservation expiration, and override alerts.
- To observability: reserve latency, balance conflict rate, feed age, and projection lag.
- To marketplace: quota capability entitlement check only; no settlement ownership.

## Implementation Checklist
- Add aggregate `QuotaProgram`.
- Add entity `QuotaBalance`.
- Add aggregate `QuotaReservation`.
- Add entity `OriginEvidenceRef`.
- Add value object `QuotaPeriod`.
- Add value object `QuantityWithUnit`.
- Add repository for quota programs.
- Add repository for quota balances.
- Add repository for quota reservations.
- Add quota allocation service.
- Add origin evidence lookup port.
- Add quota feed import port.
- Add command handler for reserve.
- Add command handler for consume.
- Add command handler for release.
- Add command handler for override.
- Add command handler for feed activation.
- Add Cedar checks for reserve, consume, release, override, and read.
- Add REST route for reserve.
- Add REST route for consume.
- Add REST route for release.
- Add REST route for override.
- Add REST route for quota query.
- Add gRPC methods for internal quota operations.
- Add worker for quota feed import.
- Add worker for reservation expiration.
- Add ontology projection writer.
- Add audit appender with ADR-0263 event classes.
- Add fixture for successful reservation.
- Add fixture for partial quota.
- Add fixture for exhausted quota.
- Add fixture for origin evidence missing.
- Add fixture for balance conflict.
- Add unit tests for quota allocation.
- Add unit tests for reservation expiration.
- Add policy tests for planner, customs service, broker, approver, and auditor.
- Add contract tests for reserve and consume endpoints.
- Add replay tests for feed reactivation.
- Add migration tests for SAP GTS quota history.
- Add performance test for 250,000-row quota feed.
- Add dashboard panels for p50, p95, p99, throughput, conflict rate, feed freshness, and audit seal failures.
- Add acceptance evidence referencing this IP id.
