---
doc_class: ImplementationPlan
ip_id: IP-022
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
journey_id: j124-supply-chain-disruption-emergency-coordination
journey_link: docs/user-journeys/j124-supply-chain-disruption-emergency-coordination/story.md
status: Accepted
date: 2026-05-20
owner: axis-global-trade
tenant_class_eligibility: [demo_trial, paid]
sap_submodule_equivalents:
  - SAP GTS-CC embargo-check
  - SAP GTS-EM embargo-status-distribution
---

# IP-022: Embargo event audit chain anchor

## Context With Why
- This IP builds an embargo event path that anchors embargo decisions into the ADR-0263 audit chain.
- The why is legal defensibility: embargo blocks must be explainable, immutable, and replayable across shipments, customers, brokers, and declarations.
- Embargo checks differ from denied-party checks because the blocked dimension can be country, route, commodity, party role, or jurisdiction program.
- The journey leg starts when a transaction line, shipment, broker filing, or declaration requires embargo evaluation.
- The journey leg ends when the embargo result is anchored, projected, and handed off as a block, release, or review event.
- Named persona: Nadia, a regional compliance officer, needs to prove why a shipment to a restricted destination was blocked.
- Nadia can view embargo rule evidence for her region, but she cannot alter the event chain after anchoring.
- This IP maps SAP GTS-CC embargo checks and SAP GTS-EM embargo status distribution into Oyatie.
- The implementation must not own sanctions list screening; it can consume denied-party status as evidence.
- The implementation must not own route planning; it evaluates route country evidence supplied by logistics sources.
- ADR-0105 keeps embargo rule evaluation in domain logic and source imports in adapters.
- ADR-0243 requires provenance for embargo rule source, route, party, commodity, and transaction refs.
- ADR-0244 requires tenant and sub-scope isolation.
- ADR-0253 requires Cedar default-deny for evaluate, view evidence, override, release, and export.
- ADR-0263 is central: every embargo result must have event class, prior hash, current hash, and anchor ref.
- ADR-0304 requires ontology projection of EmbargoDecision, EmbargoRule, and AuditAnchor.
- ADR-0315 sets SAP GTS parity for embargo control.
- Intern build target: one embargo decision aggregate, one rule version loader, one anchor writer, one override workflow, and one status export worker.

## Scope Boundaries
- In scope: embargo rule evaluation by country, route, commodity, party role, jurisdiction, and transaction purpose.
- In scope: block, clear, review, override-requested, overridden, and released states.
- In scope: audit-chain anchoring for every embargo decision and override.
- In scope: distribution of embargo status to customs-declaration, broker-filing, workflow, and notification.
- Out of scope: denied party fuzzy matching.
- Out of scope: product classification decisioning.
- Out of scope: logistics route optimization.
- Boundary rule: embargo event anchoring is append-only.
- Boundary rule: override creates a new event and never mutates the original block.
- Boundary rule: rule versions are staged before activation and cannot be edited in place.
- Boundary rule: downstream services consume embargo status by event ref and anchor ref.

## Data Model Deltas
- Table: `global_trade_embargo_rule_version`.
- Column: `tenant_id uuid not null`.
- Column: `embargo_rule_version_id uuid primary key`.
- Column: `rule_source_ref text not null`.
- Column: `jurisdiction text not null`.
- Column: `program_code text not null`.
- Column: `restricted_country text null`.
- Column: `restricted_route_country text null`.
- Column: `restricted_hs_prefix text null`.
- Column: `restricted_party_role text null`.
- Column: `effective_from date not null`.
- Column: `effective_until date null`.
- Column: `rule_hash text not null`.
- Column: `rule_state text not null check rule_state in ('staged','active','retired','superseded')`.
- Table: `global_trade_embargo_decision`.
- Column: `tenant_id uuid not null`.
- Column: `embargo_decision_id uuid primary key`.
- Column: `source_transaction_ref text not null`.
- Column: `source_line_ref text null`.
- Column: `decision_state text not null check decision_state in ('clear','blocked','review','override_requested','overridden','released','superseded')`.
- Column: `jurisdiction text not null`.
- Column: `program_code text not null`.
- Column: `matched_rule_version_id uuid null`.
- Column: `decision_reason text not null`.
- Column: `route_countries jsonb not null default '[]'`.
- Column: `party_refs jsonb not null default '[]'`.
- Column: `commodity_refs jsonb not null default '[]'`.
- Column: `policy_bundle_version text not null`.
- Column: `ontology_version text not null`.
- Column: `audit_chain_ref text not null`.
- Column: `anchor_ref text not null`.
- Column: `idempotency_key text not null`.
- Unique: `gt_embargo_decision_idempotency_uq` on `(tenant_id, idempotency_key)`.
- Table: `global_trade_embargo_audit_anchor`.
- Column: `tenant_id uuid not null`.
- Column: `anchor_ref text primary key`.
- Column: `embargo_decision_id uuid not null references global_trade_embargo_decision(embargo_decision_id)`.
- Column: `event_class text not null`.
- Column: `event_hash text not null`.
- Column: `prev_event_hash text not null`.
- Column: `anchor_provider text not null`.
- Column: `anchored_at timestamptz not null`.
- Table: `global_trade_embargo_override`.
- Column: `tenant_id uuid not null`.
- Column: `override_id uuid primary key`.
- Column: `embargo_decision_id uuid not null`.
- Column: `requested_by_principal text not null`.
- Column: `approved_by_principal text null`.
- Column: `override_reason text not null`.
- Column: `override_state text not null check override_state in ('requested','approved','rejected','expired')`.
- Retention: rule versions and decisions are immutable after activation or anchoring.
- Retention: override approvals append new decision states and do not alter original block event.

## API Endpoints
- REST: `POST /v1/global-trade/embargo-decisions:evaluate`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "svc_customs_declaration",
  "idempotency_key": "embargo-2026-05-20-900",
  "source_transaction_ref": "shipment:880045",
  "source_line_ref": "line-1",
  "jurisdiction": "US",
  "destination_country": "CU",
  "route_countries": ["US", "PA", "CU"],
  "party_refs": ["customer:88401", "consignee:19002"],
  "commodity_refs": ["hs:8413.70.2004"]
}
```
- REST response example:
```json
{
  "embargo_decision_id": "emb_01jytg_022",
  "decision_state": "blocked",
  "program_code": "US-EMBARGO-CU",
  "matched_rule_version_id": "emb_rule_us_cu_2026_v2",
  "anchor_ref": "audit-anchor:global-trade:emb_01jytg_022",
  "audit_event_class": "EVT-GLOBAL_TRADE-EMBARGO-BLOCKED_ANCHORED"
}
```
- REST: `POST /v1/global-trade/embargo-decisions/{embargo_decision_id}:request-override`.
- REST: `POST /v1/global-trade/embargo-decisions/{embargo_decision_id}:approve-override`.
- REST: `POST /v1/global-trade/embargo-rules:activate`.
- REST: `GET /v1/global-trade/embargo-decisions/{embargo_decision_id}`.
- REST: `GET /v1/global-trade/embargo-decisions/{embargo_decision_id}/anchor`.
- gRPC: `EvaluateEmbargo(EvaluateEmbargoRequest) returns (EvaluateEmbargoResult)`.
- gRPC: `RequestEmbargoOverride(RequestEmbargoOverrideRequest) returns (RequestEmbargoOverrideResult)`.
- gRPC: `ApproveEmbargoOverride(ApproveEmbargoOverrideRequest) returns (ApproveEmbargoOverrideResult)`.
- Worker command: `global-trade.embargo.import-rule-version`.
- Worker command: `global-trade.embargo.distribute-status`.
- Error envelope: `POLICY_DENIED`, `RULE_VERSION_UNAVAILABLE`, `ANCHOR_WRITE_FAILED`, `OVERRIDE_NOT_ALLOWED`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"svc_customs_declaration"`.
- Action: `GlobalTrade::Action::"EvaluateEmbargo"`.
- Resource: `GlobalTrade::EmbargoDecision::"emb_01jytg_022"`.
- Context field: `tenant_id`.
- Context field: `jurisdiction`.
- Context field: `destination_country`.
- Context field: `route_countries`.
- Context field: `party_roles`.
- Context field: `hs_codes`.
- Override action: `GlobalTrade::Action::"RequestEmbargoOverride"`.
- Approve action: `GlobalTrade::Action::"ApproveEmbargoOverride"`.
- Evidence action: `GlobalTrade::Action::"ReadEmbargoRuleEvidence"`.
- Allow rule intent: service principals can evaluate embargo for assigned tenant flows.
- Allow rule intent: regional compliance officers can view rule evidence for assigned jurisdiction.
- Deny rule intent: override approval requires a different principal from requester.
- Deny rule intent: hard embargo programs can disallow override entirely.
- Deny rule intent: broker principals can read block status but cannot see full rule evidence unless granted.
- Audit on allow: include rule version, program, route, source transaction, and anchor ref.
- Audit on deny: include action and resource with redacted party and route details.

## Ontology Projection Field Mapping
- Ontology node: `EmbargoDecision`.
- `embargo_decision_id` maps to `EmbargoDecision.id`.
- `source_transaction_ref` maps to `EmbargoDecision.sourceTransactionRef`.
- `decision_state` maps to `EmbargoDecision.state`.
- `jurisdiction` maps to `EmbargoDecision.jurisdiction`.
- `program_code` maps to `EmbargoDecision.programCode`.
- `decision_reason` maps to `EmbargoDecision.reason`.
- Ontology node: `EmbargoRule`.
- `embargo_rule_version_id` maps to `EmbargoRule.versionId`.
- `rule_source_ref` maps to `EmbargoRule.sourceRef`.
- `rule_hash` maps to `EmbargoRule.hash`.
- `effective_from` maps to `EmbargoRule.effectiveFrom`.
- `effective_until` maps to `EmbargoRule.effectiveUntil`.
- Ontology node: `AuditAnchor`.
- `anchor_ref` maps to `AuditAnchor.id`.
- `event_hash` maps to `AuditAnchor.eventHash`.
- `prev_event_hash` maps to `AuditAnchor.previousHash`.
- `anchor_provider` maps to `AuditAnchor.provider`.
- `anchored_at` maps to `AuditAnchor.anchoredAt`.
- Projection mode: project EmbargoDecision and AuditAnchor atomically after audit seal.
- Projection guard: party_refs are projected as refs only, not expanded party data.

## Workflow Steps
- Node `ReceiveEmbargoEvaluation`: validate tenant, source transaction, jurisdiction, route, party, and commodity refs.
- Node `LoadActiveEmbargoRules`: load active rule versions for jurisdiction and date.
- Branch `NoActiveRules`: fail closed or review by tenant policy.
- Node `EvaluateCountryRules`: compare destination, origin, and route countries.
- Node `EvaluateCommodityRules`: compare HS prefix and export control refs.
- Node `EvaluatePartyRoleRules`: compare consignee, end user, broker, bank, and carrier roles.
- Branch `NoMatch`: create clear decision and anchor event.
- Branch `PotentialMatch`: create review decision and workflow task.
- Branch `HardMatch`: create blocked decision and downstream hold.
- Node `RunCedarAuthorization`: check evaluate, evidence read, override, and approve actions.
- Node `SealAuditEvent`: append ADR-0263 embargo event with current and prior hash.
- Node `WriteAuditAnchor`: persist anchor ref and provider response.
- Node `ProjectOntology`: project EmbargoDecision, EmbargoRule, and AuditAnchor.
- Node `DistributeStatus`: notify customs-declaration, broker-filing, workflow, and notification.
- Branch `AnchorFailure`: rollback decision visibility and retry anchor write.
- Branch `OverrideRequested`: create override task and keep block active.
- Branch `OverrideApproved`: append release event and distribute release status.

## Audit Events
- `EVT-GLOBAL_TRADE-EMBARGO-EVALUATION_REQUESTED`.
- `EVT-GLOBAL_TRADE-EMBARGO-RULE_VERSION_LOADED`.
- `EVT-GLOBAL_TRADE-EMBARGO-CLEAR_ANCHORED`.
- `EVT-GLOBAL_TRADE-EMBARGO-REVIEW_ANCHORED`.
- `EVT-GLOBAL_TRADE-EMBARGO-BLOCKED_ANCHORED`.
- `EVT-GLOBAL_TRADE-EMBARGO-OVERRIDE_REQUESTED`.
- `EVT-GLOBAL_TRADE-EMBARGO-OVERRIDE_APPROVED`.
- `EVT-GLOBAL_TRADE-EMBARGO-OVERRIDE_REJECTED`.
- `EVT-GLOBAL_TRADE-EMBARGO-RELEASED_ANCHORED`.
- `EVT-GLOBAL_TRADE-EMBARGO-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-EMBARGO-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `embargo_decision_id`, `program_code`, `anchor_ref`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: every clear, review, block, override, and release event has an anchor ref.
- Event rule: hard embargo override denial records policy id and rule version.

## SLO Targets
- Availability target: 99.97 percent monthly for evaluate endpoint.
- Throughput target: 700 embargo evaluations per second per region.
- p50 latency target: 50 ms for cached rule clear decision.
- p95 latency target: 250 ms for route and commodity evaluation.
- p99 latency target: 800 ms for multi-program evaluation with anchor write.
- Anchor freshness target: 99 percent of decisions anchored within 5 seconds.
- Rule feed activation target: validated rule version active within 10 minutes.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Rationale: embargo checks sit on transaction release paths, so clear and block answers must be fast and defensible.
- Burn alert: page when anchor lag exceeds 60 seconds or evaluate p99 exceeds 1200 ms for 10 minutes.

## Failure Modes And Recovery
- Failure: active rule version unavailable; recovery: fail closed to review state and create rule-feed task.
- Failure: route country list missing; recovery: evaluate destination only and create route evidence warning.
- Failure: anchor provider unavailable; recovery: retry anchor and block downstream visibility until anchored.
- Failure: override requested for non-overridable program; recovery: Cedar deny and emit deny event.
- Failure: party refs cannot be resolved; recovery: mark review and request source evidence.
- Failure: commodity refs lack HS code; recovery: mark review and request classification.
- Failure: audit chain seal fails; recovery: rollback decision and retry from source command.
- Failure: ontology projection fails; recovery: retry projection and keep decision queryable by source endpoint.
- Failure: downstream status distribution fails; recovery: retry SAP GTS-EM equivalent worker.
- Failure: rule import has overlapping effective dates; recovery: reject staged version and keep prior active rule.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS embargo check results and rule versions.
- SAP source: SAP GTS electronic messaging embargo status distribution logs.
- Oracle source: GTM embargo and trade control screening records.
- Descartes source: embargo screening history and country control lists.
- Amber Road source: embargo and restricted country workflows.
- Tenant source: manual country block lists and route restriction spreadsheets.
- Migration step: import rule versions as staged and hash every rule row.
- Migration step: import historical embargo decisions with source transaction refs.
- Migration step: create anchor records for migrated decisions using migration event class.
- Migration step: map legacy block and release statuses to decision_state values.
- Migration step: do not auto-approve legacy overrides without approver provenance.
- Migration step: replay five blocked and five clear historical decisions before activation.

## Cross-Microservice Handoffs
- From customs-declaration: transaction, line, route, and destination refs.
- From HS classification: commodity HS refs for embargo commodity rules.
- From denied-party screening: party role refs and hit status where relevant.
- To broker-filing: block, release, or review status with anchor ref.
- To workflow-engine: review, override, and missing evidence tasks.
- To ontology: EmbargoDecision, EmbargoRule, and AuditAnchor projections.
- To audit-chain: ADR-0263 event append and external anchor verification.
- To notification: block and release notices for compliance and operations.
- To observability: evaluation latency, anchor lag, rule feed age, and deny spikes.
- To marketplace: embargo capability entitlement check only; no settlement ownership.

## Implementation Checklist
- Add aggregate `EmbargoDecision`.
- Add entity `EmbargoRuleVersion`.
- Add entity `EmbargoAuditAnchor`.
- Add entity `EmbargoOverride`.
- Add value object `ProgramCode`.
- Add value object `RouteCountrySet`.
- Add repository for embargo rules.
- Add repository for embargo decisions.
- Add repository for audit anchors.
- Add rule evaluation service.
- Add audit anchor writer port.
- Add status distribution port.
- Add command handler for evaluate.
- Add command handler for request override.
- Add command handler for approve override.
- Add command handler for rule activation.
- Add Cedar checks for evaluate, evidence read, override request, override approval, and export.
- Add REST route for evaluate.
- Add REST route for override request.
- Add REST route for override approval.
- Add REST route for rule activation.
- Add REST route for anchor read.
- Add gRPC methods for internal embargo decisions.
- Add worker for rule import.
- Add worker for status distribution.
- Add ontology projection writer.
- Add audit appender with ADR-0263 event classes.
- Add fixture for clear decision.
- Add fixture for hard block.
- Add fixture for review decision.
- Add fixture for override denied.
- Add fixture for anchor failure retry.
- Add unit tests for country, route, commodity, and party role rules.
- Add policy tests for service principal, compliance officer, broker, auditor, and CI.
- Add contract tests for evaluate endpoint.
- Add replay tests for migrated historical decisions.
- Add migration tests for SAP GTS embargo history.
- Add performance test for 700 evaluations per second.
- Add dashboard panels for p50, p95, p99, anchor lag, rule feed age, and audit seal failures.
- Add acceptance evidence referencing this IP id.
