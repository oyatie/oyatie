---
doc_class: ImplementationPlan
ip_id: IP-023
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
journey_id: j102-raw-material-purchase-with-quality-attestation
journey_link: docs/user-journeys/j102-raw-material-purchase-with-quality-attestation/story.md
status: Accepted
date: 2026-05-20
owner: axis-global-trade
tenant_class_eligibility: [demo_trial, paid]
sap_submodule_equivalents:
  - SAP GTS-PI preference-processing-origin-determination
  - SAP GTS-CC commodity-and-origin-compliance
  - SAP GTS-EM preference-result-distribution
---

# IP-023: Preferential trade agreement origin determination logic

## Context With Why
- This IP builds preferential trade agreement origin determination logic for products, components, and transaction lines.
- The why is margin and compliance: preference claims reduce duty only when origin rules are satisfied and evidence is defensible.
- Origin determination must be explainable across bill of materials, supplier declarations, HS classification, value content, and tariff shift rules.
- The journey leg starts when a product or shipment line asks whether a trade agreement preference can be claimed.
- The journey leg ends when origin determination is qualified, not qualified, needs evidence, or expired with audit evidence.
- Named persona: Hana, a preference program specialist, needs to qualify products for USMCA and EU-KR preference without manually rebuilding rule calculations.
- Hana can approve supplier evidence and review rule outcomes, but she cannot bypass required origin inputs.
- This IP maps SAP GTS-PI preference processing origin determination into Oyatie.
- SAP GTS-CC is relevant because HS classification and compliance controls are required inputs.
- SAP GTS-EM is relevant because preference results must be distributed to certificate, customs, and broker channels.
- The implementation must not own supplier master, product master, or bill of materials master data.
- The implementation stores origin determination decisions, rule calculations, evidence refs, and agreement versions.
- ADR-0105 keeps origin rules in domain logic and source imports in adapters.
- ADR-0243 requires provenance for agreement rules, BOM, supplier declarations, classification, and cost inputs.
- ADR-0244 requires tenant and sub-scope isolation.
- ADR-0253 requires Cedar default-deny for determine, approve, override, export, and evidence read.
- ADR-0263 requires chainable origin determination events.
- ADR-0304 requires ontology projection of OriginDetermination, TradeAgreementRule, and OriginEvidence.
- ADR-0315 sets SAP GTS parity for preference processing.
- Intern build target: one origin determination aggregate, one agreement rule evaluator, one evidence collector, one approval workflow, and one result export worker.

## Scope Boundaries
- In scope: agreement version loading, rule-of-origin evaluation, regional value content, tariff shift, de minimis, accumulation, and supplier evidence checks.
- In scope: qualified, not_qualified, needs_evidence, expired, overridden, and superseded states.
- In scope: product-level and shipment-line-level determination.
- In scope: output for certificate generation, customs declaration, broker filing, and HS preference attach.
- Out of scope: supplier declaration solicitation workflow.
- Out of scope: product BOM master ownership.
- Out of scope: accounting cost rollup ownership.
- Out of scope: final customs duty calculation.
- Boundary rule: origin decisions reference source BOM, supplier, and cost snapshots.
- Boundary rule: approved origin decision can be used by IP-016 preference attachment and IP-018 certificate generation.
- Boundary rule: expired agreement rules invalidate new claims but do not erase historical decisions.
- Boundary rule: manual override requires separate reason, approver, and audit event.

## Data Model Deltas
- Table: `global_trade_pta_agreement_rule_version`.
- Column: `tenant_id uuid not null`.
- Column: `agreement_rule_version_id uuid primary key`.
- Column: `agreement_code text not null`.
- Column: `jurisdiction_pair text not null`.
- Column: `hs_code_prefix text not null`.
- Column: `rule_of_origin_code text not null`.
- Column: `rule_type text not null check rule_type in ('tariff_shift','regional_value_content','wholly_obtained','specific_process','de_minimis','accumulation')`.
- Column: `rule_payload jsonb not null`.
- Column: `effective_from date not null`.
- Column: `effective_until date null`.
- Column: `rule_hash text not null`.
- Column: `rule_state text not null check rule_state in ('staged','active','retired','superseded')`.
- Table: `global_trade_origin_determination`.
- Column: `tenant_id uuid not null`.
- Column: `origin_determination_id uuid primary key`.
- Column: `product_ref text not null`.
- Column: `source_line_ref text null`.
- Column: `agreement_code text not null`.
- Column: `origin_country text not null`.
- Column: `destination_country text not null`.
- Column: `hs_code text not null`.
- Column: `determination_state text not null check determination_state in ('qualified','not_qualified','needs_evidence','expired','overridden','superseded')`.
- Column: `agreement_rule_version_id uuid not null`.
- Column: `calculation_summary text not null`.
- Column: `confidence_score numeric(5,4) not null`.
- Column: `valid_from date not null`.
- Column: `valid_until date null`.
- Column: `idempotency_key text not null`.
- Column: `policy_bundle_version text not null`.
- Column: `ontology_version text not null`.
- Column: `audit_chain_ref text not null`.
- Unique: `gt_origin_determination_idempotency_uq` on `(tenant_id, idempotency_key)`.
- Table: `global_trade_origin_rule_calculation`.
- Column: `tenant_id uuid not null`.
- Column: `calculation_id uuid primary key`.
- Column: `origin_determination_id uuid not null references global_trade_origin_determination(origin_determination_id)`.
- Column: `calculation_type text not null`.
- Column: `input_snapshot_ref text not null`.
- Column: `input_snapshot_hash text not null`.
- Column: `calculation_result jsonb not null`.
- Column: `passed boolean not null`.
- Table: `global_trade_origin_evidence_bundle`.
- Column: `tenant_id uuid not null`.
- Column: `origin_evidence_bundle_id uuid primary key`.
- Column: `origin_determination_id uuid not null`.
- Column: `evidence_type text not null check evidence_type in ('supplier_declaration','bom_snapshot','cost_snapshot','classification','manufacturing_step','prior_determination')`.
- Column: `source_ref text not null`.
- Column: `source_event_ref text not null`.
- Column: `evidence_hash text not null`.
- Column: `accepted_for_origin boolean not null default false`.
- Retention: origin determinations are immutable after approval; changes create superseding decisions.
- Retention: rule versions are immutable after activation.

## API Endpoints
- REST: `POST /v1/global-trade/origin-determinations:determine`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "usr_hana_preference",
  "idempotency_key": "origin-2026-05-20-501",
  "product_ref": "sku-PUMP-88",
  "source_line_ref": "shipment:880045:line-1",
  "agreement_code": "USMCA",
  "origin_country": "MX",
  "destination_country": "US",
  "hs_code": "8413.70.2004",
  "evidence_refs": [
    "supplier-declaration:mx-supplier-4:2026",
    "bom:sku-PUMP-88:rev-7",
    "cost:sku-PUMP-88:2026q2"
  ]
}
```
- REST response example:
```json
{
  "origin_determination_id": "orig_01jytg_pta_023",
  "determination_state": "qualified",
  "agreement_rule_version_id": "usmca-8413-2026-v4",
  "rule_of_origin_code": "RVC-NC-50",
  "calculation_summary": "Regional value content net cost method passed at 61.4 percent.",
  "valid_until": "2026-12-31",
  "audit_event_class": "EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-QUALIFIED"
}
```
- REST: `POST /v1/global-trade/origin-determinations/{origin_determination_id}:approve`.
- REST: `POST /v1/global-trade/origin-determinations/{origin_determination_id}:override`.
- REST: `GET /v1/global-trade/origin-determinations/{origin_determination_id}`.
- REST: `GET /v1/global-trade/origin-determinations?product_ref={product_ref}&agreement_code={agreement_code}`.
- gRPC: `DetermineOrigin(DetermineOriginRequest) returns (DetermineOriginResult)`.
- gRPC: `ApproveOriginDetermination(ApproveOriginDeterminationRequest) returns (ApproveOriginDeterminationResult)`.
- gRPC: `ExportOriginResult(ExportOriginResultRequest) returns (ExportOriginResultResult)`.
- Worker command: `global-trade.origin.import-agreement-rules`.
- Worker command: `global-trade.origin.expire-determinations`.
- Error envelope: `POLICY_DENIED`, `AGREEMENT_RULE_NOT_FOUND`, `EVIDENCE_MISSING`, `RULE_CALCULATION_FAILED`, `OVERRIDE_NOT_ALLOWED`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"usr_hana_preference"`.
- Action: `GlobalTrade::Action::"DetermineOrigin"`.
- Resource: `GlobalTrade::OriginDetermination::"orig_01jytg_pta_023"`.
- Context field: `tenant_id`.
- Context field: `agreement_code`.
- Context field: `origin_country`.
- Context field: `destination_country`.
- Context field: `hs_code`.
- Context field: `evidence_refs`.
- Approve action: `GlobalTrade::Action::"ApproveOriginDetermination"`.
- Override action: `GlobalTrade::Action::"OverrideOriginDetermination"`.
- Evidence action: `GlobalTrade::Action::"ReadOriginEvidence"`.
- Export action: `GlobalTrade::Action::"ExportOriginResult"`.
- Allow rule intent: preference specialists can determine origin for assigned agreements.
- Allow rule intent: compliance approvers can approve qualified determinations with complete evidence.
- Deny rule intent: origin cannot be approved when required supplier declaration is missing.
- Deny rule intent: override cannot convert not_qualified to qualified without supervisor permission.
- Deny rule intent: broker principals can read final result only, not cost snapshots.
- Audit on allow: include agreement, rule version, calculation summary, and evidence hashes.
- Audit on deny: include blocked action and missing evidence classes without exposing costs.

## Ontology Projection Field Mapping
- Ontology node: `OriginDetermination`.
- `origin_determination_id` maps to `OriginDetermination.id`.
- `product_ref` maps to `OriginDetermination.productRef`.
- `agreement_code` maps to `OriginDetermination.agreementCode`.
- `origin_country` maps to `OriginDetermination.originCountry`.
- `destination_country` maps to `OriginDetermination.destinationCountry`.
- `hs_code` maps to `OriginDetermination.hsCode`.
- `determination_state` maps to `OriginDetermination.state`.
- `calculation_summary` maps to `OriginDetermination.summary`.
- Ontology node: `TradeAgreementRule`.
- `agreement_rule_version_id` maps to `TradeAgreementRule.versionId`.
- `rule_of_origin_code` maps to `TradeAgreementRule.ruleCode`.
- `rule_type` maps to `TradeAgreementRule.ruleType`.
- `rule_hash` maps to `TradeAgreementRule.hash`.
- Ontology node: `OriginEvidence`.
- `origin_evidence_bundle_id` maps to `OriginEvidence.id`.
- `evidence_type` maps to `OriginEvidence.type`.
- `source_ref` maps to `OriginEvidence.sourceRef`.
- `evidence_hash` maps to `OriginEvidence.hash`.
- Projection mode: project rule version after activation, determination after decision, evidence edges after acceptance.
- Projection guard: cost snapshots are projected by hash and result summary, not raw cost details.

## Workflow Steps
- Node `ReceiveDetermineRequest`: validate tenant, agreement, origin, destination, HS code, evidence refs, and idempotency.
- Node `LoadAgreementRuleVersion`: find active rule by agreement, jurisdiction pair, HS prefix, and date.
- Branch `NoRuleVersion`: return rule not found and create rule maintenance task.
- Node `CollectEvidenceSnapshots`: load supplier declaration, BOM, cost, classification, manufacturing step, and prior determination refs.
- Branch `EvidenceMissing`: create needs_evidence state and workflow task.
- Node `EvaluateTariffShift`: compare finished good HS and non-originating component HS codes.
- Node `EvaluateRegionalValueContent`: compute RVC using approved cost snapshot method.
- Node `EvaluateDeMinimis`: apply threshold when components fail tariff shift.
- Node `EvaluateAccumulation`: include eligible partner-country inputs when rule allows.
- Branch `RulePassed`: create qualified determination.
- Branch `RuleFailed`: create not_qualified determination with calculation details.
- Node `RunCedarAuthorization`: check determine, approve, override, evidence read, and export actions.
- Node `SealAuditEvent`: append ADR-0263 origin determination event.
- Node `ProjectOntology`: project OriginDetermination, TradeAgreementRule, and OriginEvidence.
- Node `NotifyDownstream`: send result to HS preference attach, certificate generation, customs declaration, and broker filing.
- Branch `OverrideRequested`: create supervisor workflow and keep original state visible.
- Branch `DeterminationExpired`: mark expired and notify downstream consumers.

## Audit Events
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-REQUESTED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-RULE_LOADED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-EVIDENCE_COLLECTED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-EVIDENCE_MISSING_HELD`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-QUALIFIED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-NOT_QUALIFIED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-APPROVED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-OVERRIDDEN`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-EXPIRED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-ORIGIN_DETERMINATION-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `origin_determination_id`, `agreement_code`, `rule_version`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: calculation result hashes are included for qualified and not-qualified decisions.
- Event rule: override events include original state, new state, approver, and reason.

## SLO Targets
- Availability target: 99.95 percent monthly for determine and read endpoints.
- Throughput target: 200 origin determinations per second per region.
- p50 latency target: 120 ms for product-level determination with cached rule and evidence.
- p95 latency target: 900 ms for BOM and RVC calculation up to 100 components.
- p99 latency target: 3000 ms for tariff shift plus RVC plus accumulation evaluation.
- Rule import target: validated agreement rule version active within 15 minutes.
- Expiration target: expired determinations marked within 5 minutes of rule or evidence expiration.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Rationale: origin determination can be computationally heavier than lookup, but users need same-session feedback for shipment planning.
- Burn alert: page when p99 exceeds 5 seconds or evidence-missing rate doubles baseline for 30 minutes.

## Failure Modes And Recovery
- Failure: agreement rule missing for HS prefix; recovery: create rule maintenance task and return rule not found.
- Failure: supplier declaration expired; recovery: needs_evidence state and notify supplier evidence owner.
- Failure: BOM snapshot hash mismatch; recovery: reject evidence and require fresh snapshot.
- Failure: cost snapshot unavailable; recovery: skip RVC only if rule does not require it, otherwise needs_evidence.
- Failure: tariff shift and RVC produce conflicting result; recovery: record both calculations and require reviewer decision.
- Failure: override requested by same preparer; recovery: Cedar deny under separation-of-duties rule.
- Failure: ontology projection fails; recovery: retry projection and block export until projected.
- Failure: audit chain seal fails; recovery: rollback determination and retry from command.
- Failure: downstream certificate generation reads expired result; recovery: return expired state and require redetermination.
- Failure: rule import has malformed payload; recovery: reject staged version and keep prior active rule.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS preference processing origin determinations.
- SAP source: SAP GTS preference agreements, rule of origin data, and supplier declaration links.
- SAP source: SAP GTS electronic messaging preference result outputs.
- Oracle source: GTM trade agreement qualification and origin determination exports.
- Descartes source: FTA qualification and origin management records.
- Amber Road source: preferential origin and supplier declaration data.
- Tenant source: BOM, cost, and supplier declaration spreadsheets.
- Migration step: import agreement rules as staged rule versions.
- Migration step: hash rule payloads and source agreement refs.
- Migration step: import historical determinations as immutable legacy decisions.
- Migration step: map supplier declaration refs to origin evidence bundles.
- Migration step: mark determinations expired when source rule version is no longer active.
- Migration step: replay a sample of qualified and not-qualified products before enabling export.

## Cross-Microservice Handoffs
- From HS classification: approved HS code and classification basis.
- From product and BOM source systems: product refs, component refs, and BOM snapshot refs.
- From supplier evidence source: supplier declaration refs and validity windows.
- To IP-016 preference attach: qualified origin result and rule-of-origin code.
- To certificate generation: origin evidence and preference statement values.
- To customs-declaration: preference claim eligibility and evidence bundle ref.
- To broker-filing: exportable preference result packet.
- To workflow-engine: missing evidence, reviewer decision, and override tasks.
- To ontology: OriginDetermination, TradeAgreementRule, and OriginEvidence projections.
- To audit-chain: ADR-0263 event append and chain verification.
- To observability: determination latency, evidence missing rate, rule import age, and seal failures.

## Implementation Checklist
- Add aggregate `OriginDetermination`.
- Add entity `PtaAgreementRuleVersion`.
- Add entity `OriginRuleCalculation`.
- Add entity `OriginEvidenceBundle`.
- Add value object `AgreementCode`.
- Add value object `RuleOfOriginCode`.
- Add value object `JurisdictionPair`.
- Add repository for agreement rule versions.
- Add repository for origin determinations.
- Add repository for evidence bundles.
- Add tariff shift evaluator.
- Add regional value content evaluator.
- Add de minimis evaluator.
- Add accumulation evaluator.
- Add evidence collector port.
- Add command handler for determine.
- Add command handler for approve.
- Add command handler for override.
- Add command handler for export result.
- Add command handler for rule import activation.
- Add Cedar checks for determine, approve, override, evidence read, and export.
- Add REST route for determine.
- Add REST route for approve.
- Add REST route for override.
- Add REST route for read.
- Add REST route for product query.
- Add gRPC methods for internal origin determination.
- Add worker for agreement rule import.
- Add worker for determination expiration.
- Add ontology projection writer.
- Add audit appender with ADR-0263 event classes.
- Add fixture for USMCA qualified RVC result.
- Add fixture for tariff shift failure.
- Add fixture for missing supplier declaration.
- Add fixture for expired agreement rule.
- Add fixture for override approval.
- Add unit tests for tariff shift rules.
- Add unit tests for RVC calculations.
- Add unit tests for evidence expiration.
- Add policy tests for preference specialist, approver, broker, auditor, and CI.
- Add contract tests for determine endpoint.
- Add replay tests for rule version changes.
- Add migration tests for SAP GTS preference processing history.
- Add performance test for 100-component BOM determinations.
- Add dashboard panels for p50, p95, p99, throughput, evidence missing rate, rule age, and audit seal failures.
- Add acceptance evidence referencing this IP id.
