---
doc_class: ImplementationPlan
ip_id: IP-020
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
journey_id: j106-multi-currency-cross-border-payment
journey_link: docs/user-journeys/j106-multi-currency-cross-border-payment/story.md
status: Accepted
date: 2026-05-20
owner: axis-global-trade
tenant_class_eligibility: [demo_trial, paid]
sap_submodule_equivalents:
  - SAP GTS-CC customs-duty-management
  - SAP GTS-EM drawback-claim-message-support
---

# IP-020: Duty drawback claim workflow

## Context With Why
- This IP builds the workflow for identifying, assembling, approving, and submitting duty drawback claims.
- The why is financial: tenants can recover duties when imported goods are exported, destroyed, or otherwise qualify under jurisdiction rules.
- Drawback evidence is high-risk because claim amounts connect customs declarations, inventory movements, exports, and broker filings.
- The journey leg starts when import duty payment evidence and export or disposition evidence become matchable.
- The journey leg ends when a drawback claim packet is approved, submitted, reconciled, or rejected with audit-chain evidence.
- Named persona: Elena, a trade finance manager, needs to recover eligible duties without creating unsupported claims.
- Elena can approve claim packets, but she cannot fabricate import-export matches or change customs payment facts.
- This IP maps SAP GTS-CC customs duty management and SAP GTS-EM claim messaging into an Oyatie workflow.
- The implementation must not own accounts receivable, tax ledger, or cash posting.
- The implementation must not recalculate customs duty from first principles; it consumes paid duty evidence.
- ADR-0105 keeps drawback domain logic separate from broker adapters and finance handoffs.
- ADR-0243 requires provenance for import entries, export declarations, inventory links, and payment refs.
- ADR-0244 requires tenant and sub-scope isolation for every claim.
- ADR-0253 requires Cedar default-deny for candidate creation, claim approval, evidence export, and submission.
- ADR-0263 requires claim lifecycle events to be chainable.
- ADR-0304 requires ontology projection of DutyDrawbackClaim, ClaimLine, and RecoveryEvidence.
- ADR-0315 sets the SAP GTS parity bar for customs recovery workflows.
- Intern build target: one claim aggregate, one candidate matcher, one workflow integration, one broker submission port, and one finance handoff event.

## Scope Boundaries
- In scope: drawback candidate matching, claim line assembly, eligibility checks, approval workflow, submission packet, status reconciliation.
- In scope: import entry refs, export declaration refs, duty payment refs, inventory consumption refs, and broker claim refs.
- In scope: claim states for candidate, evidence_needed, ready_for_review, approved, submitted, accepted, rejected, paid, withdrawn.
- In scope: partial claims, split exports, and claim period cutoff validation.
- Out of scope: duty payment execution.
- Out of scope: general ledger posting.
- Out of scope: customs declaration creation.
- Out of scope: inventory master ownership.
- Boundary rule: source duty and export evidence are referenced and snapshotted, not mutated.
- Boundary rule: claim amount is derived from source evidence and jurisdiction rules, then reviewed.
- Boundary rule: submission happens through broker-filing or electronic messaging, not direct customs ownership in this IP.
- Boundary rule: rejected claims remain in history and can be superseded by corrected claims.

## Data Model Deltas
- Table: `global_trade_duty_drawback_claim`.
- Column: `tenant_id uuid not null`.
- Column: `drawback_claim_id uuid primary key`.
- Column: `claim_number text not null`.
- Column: `jurisdiction text not null`.
- Column: `claim_period_start date not null`.
- Column: `claim_period_end date not null`.
- Column: `claim_state text not null check claim_state in ('candidate','evidence_needed','ready_for_review','approved','submitted','accepted','rejected','paid','withdrawn','superseded')`.
- Column: `claimed_amount numeric(18,4) not null`.
- Column: `currency_code text not null`.
- Column: `source_system_ref text not null`.
- Column: `broker_partner_ref text null`.
- Column: `idempotency_key text not null`.
- Column: `policy_bundle_version text not null`.
- Column: `ontology_version text not null`.
- Column: `audit_chain_ref text not null`.
- Unique: `gt_drawback_claim_number_uq` on `(tenant_id, jurisdiction, claim_number)`.
- Table: `global_trade_duty_drawback_claim_line`.
- Column: `tenant_id uuid not null`.
- Column: `claim_line_id uuid primary key`.
- Column: `drawback_claim_id uuid not null references global_trade_duty_drawback_claim(drawback_claim_id)`.
- Column: `import_declaration_ref text not null`.
- Column: `import_line_ref text not null`.
- Column: `export_declaration_ref text not null`.
- Column: `export_line_ref text not null`.
- Column: `product_ref text not null`.
- Column: `hs_code text not null`.
- Column: `origin_country text null`.
- Column: `paid_duty_amount numeric(18,4) not null`.
- Column: `eligible_recovery_amount numeric(18,4) not null`.
- Column: `match_confidence numeric(5,4) not null`.
- Column: `eligibility_state text not null check eligibility_state in ('eligible','needs_evidence','ineligible','overridden')`.
- Index: `gt_drawback_claim_line_claim_idx` on `(tenant_id, drawback_claim_id)`.
- Table: `global_trade_duty_drawback_evidence`.
- Column: `tenant_id uuid not null`.
- Column: `evidence_id uuid primary key`.
- Column: `claim_line_id uuid not null references global_trade_duty_drawback_claim_line(claim_line_id)`.
- Column: `evidence_type text not null check evidence_type in ('import_entry','duty_payment','export_entry','inventory_movement','destruction_certificate','broker_ack')`.
- Column: `source_ref text not null`.
- Column: `source_event_ref text not null`.
- Column: `evidence_hash text not null`.
- Column: `accepted_for_claim boolean not null default false`.
- Table: `global_trade_duty_drawback_status`.
- Column: `tenant_id uuid not null`.
- Column: `status_id uuid primary key`.
- Column: `drawback_claim_id uuid not null`.
- Column: `external_status_code text not null`.
- Column: `status_received_at timestamptz not null`.
- Column: `status_source_ref text not null`.
- Retention: claim lines are immutable after submission; corrections create superseding claims.
- Retention: paid status can be reconciled with finance, but finance posting remains outside this service.

## API Endpoints
- REST: `POST /v1/global-trade/duty-drawback/claims:candidate`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "usr_elena_trade_finance",
  "idempotency_key": "drawback-2026-05-20-001",
  "jurisdiction": "US",
  "claim_period_start": "2026-01-01",
  "claim_period_end": "2026-03-31",
  "source_system_ref": "trade-ledger:na",
  "selection": {
    "product_refs": ["sku-PUMP-88"],
    "export_declaration_refs": ["export:US:9921"]
  }
}
```
- REST response example:
```json
{
  "drawback_claim_id": "ddb_01jytg_claim_020",
  "claim_state": "evidence_needed",
  "claimed_amount": "12840.55",
  "currency_code": "USD",
  "line_count": 12,
  "missing_evidence": ["duty_payment", "broker_ack"],
  "audit_event_class": "EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CANDIDATE_CREATED"
}
```
- REST: `POST /v1/global-trade/duty-drawback/claims/{drawback_claim_id}:approve`.
- REST: `POST /v1/global-trade/duty-drawback/claims/{drawback_claim_id}:submit`.
- REST: `POST /v1/global-trade/duty-drawback/claims/{drawback_claim_id}:withdraw`.
- REST: `GET /v1/global-trade/duty-drawback/claims/{drawback_claim_id}`.
- REST: `GET /v1/global-trade/duty-drawback/claims?claim_state={claim_state}`.
- gRPC: `CreateDrawbackCandidate(CreateDrawbackCandidateRequest) returns (CreateDrawbackCandidateResult)`.
- gRPC: `ApproveDrawbackClaim(ApproveDrawbackClaimRequest) returns (ApproveDrawbackClaimResult)`.
- gRPC: `SubmitDrawbackClaim(SubmitDrawbackClaimRequest) returns (SubmitDrawbackClaimResult)`.
- Worker command: `global-trade.drawback.match-candidates`.
- Worker command: `global-trade.drawback.reconcile-status`.
- Error envelope: `POLICY_DENIED`, `EVIDENCE_MISSING`, `CLAIM_PERIOD_CLOSED`, `DUPLICATE_CLAIM`, `SUBMISSION_FAILED`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"usr_elena_trade_finance"`.
- Action: `GlobalTrade::Action::"CreateDrawbackCandidate"`.
- Resource: `GlobalTrade::DrawbackClaim::"ddb_01jytg_claim_020"`.
- Context field: `tenant_id`.
- Context field: `jurisdiction`.
- Context field: `claim_period_start`.
- Context field: `claim_period_end`.
- Context field: `claimed_amount`.
- Context field: `tenant_class`.
- Approval action: `GlobalTrade::Action::"ApproveDrawbackClaim"`.
- Submit action: `GlobalTrade::Action::"SubmitDrawbackClaim"`.
- Evidence action: `GlobalTrade::Action::"ReadDrawbackEvidence"`.
- Allow rule intent: trade finance managers can create candidate claims for assigned jurisdictions.
- Allow rule intent: compliance approvers can approve claims when all required evidence is accepted.
- Deny rule intent: preparer and approver cannot be the same principal when claimed amount exceeds tenant threshold.
- Deny rule intent: claims with missing evidence cannot be submitted.
- Deny rule intent: broker principals can update external status but cannot change claim amount.
- Audit on allow: include jurisdiction, claim period, amount, and evidence hash summary.
- Audit on deny: include action, reason, and claim id without exposing supplier data.

## Ontology Projection Field Mapping
- Ontology node: `DutyDrawbackClaim`.
- `drawback_claim_id` maps to `DutyDrawbackClaim.id`.
- `claim_number` maps to `DutyDrawbackClaim.claimNumber`.
- `jurisdiction` maps to `DutyDrawbackClaim.jurisdiction`.
- `claim_state` maps to `DutyDrawbackClaim.lifecycleState`.
- `claimed_amount` maps to `DutyDrawbackClaim.claimedAmount`.
- `currency_code` maps to `DutyDrawbackClaim.currency`.
- Ontology node: `ClaimLine`.
- `claim_line_id` maps to `ClaimLine.id`.
- `import_declaration_ref` maps to `ClaimLine.importDeclarationRef`.
- `export_declaration_ref` maps to `ClaimLine.exportDeclarationRef`.
- `product_ref` maps to `ClaimLine.productRef`.
- `eligible_recovery_amount` maps to `ClaimLine.eligibleRecoveryAmount`.
- `eligibility_state` maps to `ClaimLine.eligibilityState`.
- Ontology node: `RecoveryEvidence`.
- `evidence_id` maps to `RecoveryEvidence.id`.
- `evidence_type` maps to `RecoveryEvidence.type`.
- `source_ref` maps to `RecoveryEvidence.sourceRef`.
- `evidence_hash` maps to `RecoveryEvidence.hash`.
- Projection mode: claim header first, claim lines second, evidence edges third.
- Projection guard: finance payment details are referenced by source ref, not duplicated.

## Workflow Steps
- Node `ReceiveCandidateRequest`: validate tenant, jurisdiction, period, source, and idempotency.
- Node `LoadImportDutyEvidence`: load paid duty refs from customs-declaration or broker-filing.
- Node `LoadExportEvidence`: load export declaration and shipment evidence.
- Node `MatchImportToExport`: match by product, HS code, quantity, lot, and inventory movement.
- Branch `NoMatch`: record ineligible line and explanation.
- Branch `LowConfidenceMatch`: create evidence-needed state and review task.
- Branch `EligibleMatch`: compute eligible recovery amount.
- Node `AssembleClaimPacket`: create claim lines and evidence refs.
- Node `RunCedarAuthorization`: check candidate, approve, evidence read, and submit actions.
- Node `ApproveClaim`: lock claim amount and approver identity.
- Node `SealAuditEvent`: append ADR-0263 claim event.
- Node `ProjectOntology`: project claim, lines, and evidence.
- Node `SubmitViaBrokerFiling`: transmit claim packet through broker-filing or EM worker.
- Node `ReconcileExternalStatus`: ingest accepted, rejected, paid, or correction statuses.
- Branch `RejectedByCustoms`: create correction workflow and keep original claim immutable.
- Branch `Paid`: hand payment reference to finance as a read-only recovery event.
- Branch `WithdrawRequested`: mark withdrawn and emit event.

## Audit Events
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CANDIDATE_REQUESTED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CANDIDATE_CREATED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-EVIDENCE_MISSING_HELD`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CLAIM_APPROVED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CLAIM_SUBMITTED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CLAIM_ACCEPTED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CLAIM_REJECTED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CLAIM_PAID`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-CLAIM_WITHDRAWN`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-DUTY_DRAWBACK-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `drawback_claim_id`, `claim_amount`, `currency_code`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: claim amount changes only through supersession events.
- Event rule: payment reconciliation references finance event ids without owning ledger entries.

## SLO Targets
- Availability target: 99.9 percent monthly for claim workflow endpoints.
- Throughput target: 80 candidate claim requests per second per region.
- p50 latency target: 200 ms for claim header creation.
- p95 latency target: 1500 ms for candidate matching up to 100 claim lines.
- p99 latency target: 5000 ms for candidate matching up to 1000 claim lines.
- Worker throughput target: 50,000 import-export line matches per hour.
- Status freshness target: 99 percent of broker status updates reconciled within 5 minutes.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Rationale: drawback is workflow-heavy and finance-sensitive, so correctness and auditability outweigh sub-second batch matching.
- Burn alert: page when status reconciliation lag exceeds 30 minutes or audit seal failure exceeds 0.1 percent.

## Failure Modes And Recovery
- Failure: import duty evidence missing; recovery: mark evidence_needed and assign source evidence task.
- Failure: export declaration is not final; recovery: keep candidate open and retry after declaration finalization.
- Failure: duplicate claim period and product set; recovery: block duplicate and suggest existing claim id.
- Failure: preparer attempts approval above threshold; recovery: Cedar deny and create approver task.
- Failure: broker submission fails; recovery: keep approved state and retry submission worker.
- Failure: customs rejects claim; recovery: record rejection status and create correction workflow.
- Failure: paid status lacks payment reference; recovery: hold finance handoff until broker status is complete.
- Failure: ontology projection fails; recovery: retry projection and block submission until projected.
- Failure: audit chain seal fails; recovery: rollback state transition and retry.
- Failure: inventory movement evidence conflicts; recovery: mark line ineligible and require override approval.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS customs duty and drawback claim records.
- SAP source: SAP GTS electronic messaging claim submissions and responses.
- Oracle source: GTM duty drawback and customs duty recovery exports.
- Descartes source: drawback claim workflow and broker submission history.
- Amber Road source: duty recovery claim data and status feeds.
- Finance source: ERP payment evidence and recovery receivable references.
- Migration step: import historical claims as immutable lifecycle snapshots.
- Migration step: map claim statuses to Oyatie claim_state values.
- Migration step: hash every import, export, payment, and broker evidence artifact.
- Migration step: do not reopen paid claims unless tenant requests correction migration.
- Migration step: create supersession links for corrected claims.
- Migration step: reconcile a sample of paid claims against finance before bulk activation.

## Cross-Microservice Handoffs
- From customs-declaration: import declarations, export declarations, and duty payment evidence refs.
- From broker-filing: broker submission and external claim status.
- From inventory or fulfillment source: inventory movement evidence refs through source-system lineage.
- To workflow-engine: evidence-needed, approval, correction, and withdrawal tasks.
- To ontology: DutyDrawbackClaim, ClaimLine, and RecoveryEvidence projections.
- To audit-chain: ADR-0263 event append and chain verification.
- To finance: read-only recovery event with payment reference after paid status.
- To notification: claim approval, rejection, paid, and evidence-needed alerts.
- To observability: candidate matching latency, submission backlog, status lag, and audit seal failures.
- To marketplace: entitlement check for drawback capability only; no settlement ownership.

## Implementation Checklist
- Add aggregate `DutyDrawbackClaim`.
- Add entity `DutyDrawbackClaimLine`.
- Add entity `DutyDrawbackEvidence`.
- Add entity `DutyDrawbackStatus`.
- Add value object `ClaimPeriod`.
- Add value object `RecoveryAmount`.
- Add repository for drawback claims.
- Add repository for claim evidence.
- Add matcher service for import-export lines.
- Add eligibility rule service by jurisdiction.
- Add broker submission port.
- Add finance handoff event port.
- Add command handler for candidate creation.
- Add command handler for approval.
- Add command handler for submission.
- Add command handler for withdrawal.
- Add command handler for status reconciliation.
- Add Cedar checks for candidate, approval, evidence read, submit, and withdraw.
- Add REST route for candidate.
- Add REST route for approve.
- Add REST route for submit.
- Add REST route for withdraw.
- Add REST route for read.
- Add gRPC methods for internal claim workflow.
- Add worker for candidate matching.
- Add worker for status reconciliation.
- Add ontology projection writer.
- Add audit appender with ADR-0263 event classes.
- Add fixture for eligible claim.
- Add fixture for evidence-needed claim.
- Add fixture for rejected claim.
- Add fixture for paid claim.
- Add fixture for duplicate claim period.
- Add unit tests for claim amount derivation.
- Add unit tests for match confidence thresholds.
- Add policy tests for preparer, approver, broker, finance, and auditor principals.
- Add contract tests for candidate and submit endpoints.
- Add replay tests for status reconciliation.
- Add migration tests for SAP GTS drawback history.
- Add performance test for 50,000 line matches per hour.
- Add dashboard panels for p50, p95, p99, throughput, status lag, and seal failures.
- Add acceptance evidence referencing this IP id.
