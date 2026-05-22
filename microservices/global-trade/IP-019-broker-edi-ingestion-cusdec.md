---
doc_class: ImplementationPlan
ip_id: IP-019
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
  - SAP GTS-CC customs-declaration-control
  - SAP GTS-EM customs-electronic-message-ingestion
---

# IP-019: Broker EDI ingestion CUSDEC

## Context With Why
- This IP builds inbound broker EDI ingestion for customs declaration messages, with CUSDEC as the named first message family.
- The why is operational: brokers often send declaration acknowledgements, rejections, corrections, and release notices outside the tenant ERP.
- The service must convert broker EDI into canonical customs declaration events without letting brokers mutate domain state directly.
- The journey leg starts when a broker sends a CUSDEC-related EDI message through managed transport, SFTP, AS2, API, or message bus.
- The journey leg ends when the message is parsed, validated, linked to a declaration, audited, and handed to workflow or customs-declaration.
- Named persona: Owen, a broker integration engineer, needs to onboard a new customs broker without building tenant-specific parsing code.
- Owen can map partner envelopes and qualifiers, but he cannot bypass tenant policy or overwrite declaration data.
- This IP maps SAP GTS-EM electronic messaging and SAP GTS-CC customs declaration control into Oyatie broker ingestion.
- The implementation must not own outbound filing business approval; it ingests inbound broker facts and canonicalizes them.
- The implementation must not write directly into customs-declaration tables; it emits validated handoff events.
- ADR-0105 keeps EDI parsing adapters separate from domain ingestion decisions.
- ADR-0243 requires source message provenance, checksum, and transport metadata.
- ADR-0244 requires tenant-scoped broker partner resolution.
- ADR-0253 requires Cedar checks before message accept, replay, view, and handoff.
- ADR-0263 requires every ingest, parse, reject, and replay to create chainable audit events.
- ADR-0304 requires ontology projection of BrokerMessage, CustomsDeclarationMessage, and MessageLineage.
- ADR-0315 sets SAP GTS parity for customs electronic messaging.
- Intern build target: one inbound message aggregate, one parser adapter, one canonical mapper, one validation workflow, and one replay worker.

## Scope Boundaries
- In scope: inbound CUSDEC family message receipt, envelope validation, parsing, canonical mapping, deduplication, replay, and handoff.
- In scope: broker partner configuration lookup and tenant-specific qualifier maps.
- In scope: message states for received, parsed, rejected, accepted, handed_off, and replayed.
- In scope: support for original, correction, cancellation, rejection, acknowledgement, and release notice variants.
- Out of scope: outbound customs filing creation.
- Out of scope: broker commercial contract management.
- Out of scope: customs duty payment execution.
- Boundary rule: raw EDI payloads are immutable and stored by object ref plus hash.
- Boundary rule: parsed canonical data can be superseded by replay, but raw payload remains unchanged.
- Boundary rule: broker messages can propose declaration state changes; customs-declaration owns acceptance.
- Boundary rule: tenant partner mapping is required before any message is accepted.

## Data Model Deltas
- Table: `global_trade_broker_edi_message`.
- Column: `tenant_id uuid not null`.
- Column: `broker_message_id uuid primary key`.
- Column: `broker_partner_ref text not null`.
- Column: `message_family text not null check message_family in ('CUSDEC','CUSRES','customs_ack','customs_release','broker_status')`.
- Column: `message_variant text not null`.
- Column: `transport_channel text not null check transport_channel in ('sftp','as2','api','message_bus','manual_upload')`.
- Column: `source_message_ref text not null`.
- Column: `payload_object_ref text not null`.
- Column: `payload_sha256 text not null`.
- Column: `received_at timestamptz not null`.
- Column: `ingest_state text not null check ingest_state in ('received','parsed','rejected','accepted','handed_off','replayed')`.
- Column: `idempotency_key text not null`.
- Column: `policy_bundle_version text not null`.
- Column: `audit_chain_ref text not null`.
- Unique: `gt_broker_edi_dedupe_uq` on `(tenant_id, broker_partner_ref, payload_sha256)`.
- Table: `global_trade_broker_edi_parse_result`.
- Column: `tenant_id uuid not null`.
- Column: `parse_result_id uuid primary key`.
- Column: `broker_message_id uuid not null references global_trade_broker_edi_message(broker_message_id)`.
- Column: `parser_version text not null`.
- Column: `canonical_schema_version text not null`.
- Column: `parse_state text not null check parse_state in ('valid','invalid','partial','unsupported')`.
- Column: `validation_errors jsonb not null default '[]'`.
- Column: `canonical_payload jsonb not null`.
- Column: `declaration_ref text null`.
- Column: `line_count integer not null`.
- Table: `global_trade_broker_partner_mapping`.
- Column: `tenant_id uuid not null`.
- Column: `broker_partner_ref text not null`.
- Column: `external_partner_id text not null`.
- Column: `message_family text not null`.
- Column: `qualifier_map jsonb not null`.
- Column: `active_from date not null`.
- Column: `active_until date null`.
- Primary key: `(tenant_id, broker_partner_ref, message_family, active_from)`.
- Table: `global_trade_broker_edi_handoff`.
- Column: `tenant_id uuid not null`.
- Column: `handoff_id uuid primary key`.
- Column: `broker_message_id uuid not null`.
- Column: `target_context text not null check target_context in ('customs_declaration','broker_filing','workflow_exception')`.
- Column: `target_ref text not null`.
- Column: `handoff_state text not null check handoff_state in ('pending','sent','accepted','rejected','retrying')`.
- Retention: raw payload object refs are immutable.
- Retention: replay creates new parse and handoff rows linked to the original broker message.

## API Endpoints
- REST: `POST /v1/global-trade/broker-edi/messages:ingest`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "svc_broker_gateway",
  "idempotency_key": "edi-cusdec-2026-05-20-778",
  "broker_partner_ref": "broker:atlantic-customs",
  "message_family": "CUSDEC",
  "message_variant": "import_declaration_response",
  "transport_channel": "as2",
  "source_message_ref": "AS2-MSG-774455",
  "payload_object_ref": "obj://trade-inbound/774455.edi",
  "payload_sha256": "sha256:cusdec019"
}
```
- REST response example:
```json
{
  "broker_message_id": "edi_01jytg_cusdec_019",
  "ingest_state": "accepted",
  "parse_state": "valid",
  "declaration_ref": "custdec:US:880045",
  "handoff_state": "pending",
  "audit_event_class": "EVT-GLOBAL_TRADE-BROKER_EDI-CUSDEC_ACCEPTED"
}
```
- REST: `POST /v1/global-trade/broker-edi/messages/{broker_message_id}:replay`.
- REST: `GET /v1/global-trade/broker-edi/messages/{broker_message_id}`.
- REST: `GET /v1/global-trade/broker-edi/messages?broker_partner_ref={broker_partner_ref}`.
- REST: `POST /v1/global-trade/broker-edi/partner-mappings`.
- gRPC: `IngestBrokerEdi(IngestBrokerEdiRequest) returns (IngestBrokerEdiResult)`.
- gRPC: `ReplayBrokerEdi(ReplayBrokerEdiRequest) returns (ReplayBrokerEdiResult)`.
- gRPC: `ResolveBrokerPartner(ResolveBrokerPartnerRequest) returns (ResolveBrokerPartnerResult)`.
- Worker command: `global-trade.broker-edi.parse-cusdec`.
- Worker command: `global-trade.broker-edi.dispatch-handoff`.
- Error envelope: `POLICY_DENIED`, `BROKER_PARTNER_UNKNOWN`, `DUPLICATE_MESSAGE`, `UNSUPPORTED_MESSAGE_VARIANT`, `PARSE_FAILED`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"svc_broker_gateway"`.
- Action: `GlobalTrade::Action::"IngestBrokerEdi"`.
- Resource: `GlobalTrade::BrokerPartner::"broker:atlantic-customs"`.
- Context field: `tenant_id`.
- Context field: `broker_partner_ref`.
- Context field: `message_family`.
- Context field: `transport_channel`.
- Context field: `source_message_ref`.
- Context field: `data_class`.
- Replay action: `GlobalTrade::Action::"ReplayBrokerEdi"`.
- Mapping action: `GlobalTrade::Action::"MaintainBrokerPartnerMapping"`.
- Evidence action: `GlobalTrade::Action::"ReadBrokerEdiPayload"`.
- Allow rule intent: broker gateway service can ingest messages for mapped active broker partners.
- Allow rule intent: integration engineers can replay messages after parser or mapping changes.
- Deny rule intent: broker principals cannot maintain partner mappings.
- Deny rule intent: manual upload requires human principal and justification.
- Deny rule intent: raw payload read is auditor or integration-admin only.
- Audit on allow: include transport channel, partner ref, message family, and payload hash.
- Audit on deny: include partner ref and action without exposing raw payload.

## Ontology Projection Field Mapping
- Ontology node: `BrokerMessage`.
- `broker_message_id` maps to `BrokerMessage.id`.
- `broker_partner_ref` maps to `BrokerMessage.partnerRef`.
- `message_family` maps to `BrokerMessage.family`.
- `message_variant` maps to `BrokerMessage.variant`.
- `transport_channel` maps to `BrokerMessage.transportChannel`.
- `payload_sha256` maps to `BrokerMessage.payloadHash`.
- Ontology node: `CustomsDeclarationMessage`.
- `declaration_ref` maps to `CustomsDeclarationMessage.declarationRef`.
- `canonical_schema_version` maps to `CustomsDeclarationMessage.schemaVersion`.
- `canonical_payload` maps to `CustomsDeclarationMessage.redactedPayloadSummary`.
- `parse_state` maps to `CustomsDeclarationMessage.parseState`.
- Ontology node: `MessageLineage`.
- `source_message_ref` maps to `MessageLineage.sourceMessageRef`.
- `payload_object_ref` maps to `MessageLineage.payloadObjectRef`.
- `parser_version` maps to `MessageLineage.parserVersion`.
- `handoff_state` maps to `MessageLineage.handoffState`.
- Projection mode: BrokerMessage after receipt, CustomsDeclarationMessage after parse, MessageLineage after handoff.
- Projection guard: canonical payload projection is redacted and schema-bound, not a raw EDI dump.

## Workflow Steps
- Node `ReceiveTransportEnvelope`: validate tenant, partner, transport, idempotency, and payload hash.
- Node `ResolveBrokerPartner`: load active partner mapping and qualifier map.
- Branch `UnknownPartner`: reject and create onboarding workflow task.
- Node `StoreRawPayloadRef`: verify object ref and hash without mutating payload.
- Node `RunCedarAuthorization`: check ingest action and payload read constraints.
- Node `ParseCusdecMessage`: parse CUSDEC segments using active parser version.
- Branch `UnsupportedVariant`: reject and assign integration mapping task.
- Branch `ParseInvalid`: store validation errors and create exception queue item.
- Node `MapCanonicalDeclarationMessage`: normalize declaration ref, line refs, statuses, and broker remarks.
- Node `PersistParseResult`: write parse result and outbox record.
- Node `SealAuditEvent`: append ADR-0263 ingest or reject event.
- Node `ProjectOntology`: project BrokerMessage and message lineage.
- Node `DispatchHandoff`: publish canonical event to customs-declaration or broker-filing.
- Branch `DeclarationRefMissing`: route to workflow exception.
- Branch `HandoffRejected`: keep message accepted and retry or assign exception task.
- Branch `ReplayRequested`: create replay parse result linked to original message.

## Audit Events
- `EVT-GLOBAL_TRADE-BROKER_EDI-RECEIVED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-PARTNER_RESOLVED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-CUSDEC_PARSED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-CUSDEC_REJECTED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-CUSDEC_ACCEPTED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-HANDOFF_SENT`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-HANDOFF_REJECTED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-REPLAY_REQUESTED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-REPLAY_COMPLETED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-BROKER_EDI-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `broker_message_id`, `broker_partner_ref`, `payload_sha256`, `parser_version`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: raw EDI payload is referenced by hash and object ref, not embedded.
- Event rule: replay event includes original broker message id and replay reason.

## SLO Targets
- Availability target: 99.95 percent monthly for ingest endpoint.
- Throughput target: 300 broker EDI messages per second per region.
- p50 latency target: 100 ms for envelope acceptance and dedupe.
- p95 latency target: 700 ms for parse and canonical mapping.
- p99 latency target: 2000 ms for large CUSDEC payload with 500 lines.
- Handoff freshness target: 99 percent of valid messages handed off within 60 seconds.
- Replay target: 10,000 messages per hour per worker pool.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Rationale: broker messages are often batchy, but release and rejection notices must propagate quickly.
- Burn alert: page when handoff lag exceeds 5 minutes or parse rejection spikes above tenant baseline by 3x.

## Failure Modes And Recovery
- Failure: unknown broker partner; recovery: reject message and create partner onboarding workflow task.
- Failure: duplicate payload hash; recovery: return existing broker message id.
- Failure: parser cannot handle variant; recovery: store unsupported state and route mapping task.
- Failure: raw payload hash mismatch; recovery: reject and quarantine object ref.
- Failure: Cedar denies manual upload; recovery: no message row and deny audit event.
- Failure: declaration ref cannot be resolved; recovery: send to workflow exception queue.
- Failure: handoff target rejects canonical event; recovery: retry with backoff and preserve accepted parse result.
- Failure: ontology projection fails; recovery: retry projection without blocking customs handoff when audit is sealed.
- Failure: audit chain seal fails; recovery: rollback parse acceptance and retry from raw payload.
- Failure: replay creates different canonical output; recovery: mark replay diff and require integration approval before handoff.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS electronic messaging inbound and outbound logs.
- SAP source: SAP GTS customs declaration message status history.
- Oracle source: GTM broker message and customs response records.
- Descartes source: customs broker EDI transaction archive.
- Amber Road source: broker filing and customs message history.
- Broker source: AS2 MDN logs, SFTP file manifests, and API delivery receipts.
- Migration step: import raw message refs with payload hash and transport metadata.
- Migration step: map legacy partner identifiers to broker partner refs.
- Migration step: preserve legacy parse status but replay a sample through the new parser.
- Migration step: classify unsupported variants before bulk replay.
- Migration step: attach source checksums and parser versions to audit context.
- Migration step: do not hand off migrated historical messages unless tenant explicitly requests replay.

## Cross-Microservice Handoffs
- To customs-declaration: canonical customs declaration message and status change proposal.
- To broker-filing: broker packet acknowledgement, rejection, or release status.
- To workflow-engine: unknown partner, unsupported variant, parse error, and declaration matching tasks.
- To ontology: BrokerMessage, CustomsDeclarationMessage, and MessageLineage projections.
- To audit-chain: ADR-0263 event append and chain verification.
- To object storage: immutable payload refs and content hash checks.
- To notification: broker rejection and release notice alerts.
- To data-residency: raw payload region enforcement.
- To observability: ingest latency, parse errors, handoff lag, replay diffs, and deny spikes.
- To marketplace: partner connector entitlement check only; no settlement ownership.

## Implementation Checklist
- Add aggregate `BrokerEdiMessage`.
- Add entity `BrokerEdiParseResult`.
- Add entity `BrokerPartnerMapping`.
- Add entity `BrokerEdiHandoff`.
- Add value object `PayloadHash`.
- Add value object `MessageFamily`.
- Add value object `TransportChannel`.
- Add repository for broker EDI messages.
- Add repository for partner mappings.
- Add parser port for CUSDEC.
- Add canonical mapper for customs declaration messages.
- Add handoff publisher port.
- Add object ref verifier port.
- Add command handler for ingest.
- Add command handler for replay.
- Add command handler for partner mapping maintenance.
- Add Cedar checks for ingest, replay, payload read, and mapping maintenance.
- Add REST route for ingest.
- Add REST route for replay.
- Add REST route for broker message read.
- Add REST route for partner mapping.
- Add gRPC methods for internal ingestion.
- Add worker for CUSDEC parsing.
- Add worker for handoff dispatch.
- Add ontology projection writer.
- Add audit appender with ADR-0263 event classes.
- Add fixture for valid CUSDEC import response.
- Add fixture for unsupported variant.
- Add fixture for unknown broker partner.
- Add fixture for duplicate payload hash.
- Add fixture for replay diff.
- Add unit tests for envelope validation.
- Add unit tests for qualifier mapping.
- Add unit tests for parser error normalization.
- Add policy tests for broker gateway, integration engineer, auditor, and broker principal.
- Add contract tests for ingest request and response.
- Add replay tests for parser version changes.
- Add migration tests for SAP GTS electronic messaging history.
- Add performance test for 500-line CUSDEC payloads.
- Add dashboard panels for ingest throughput, parse rejection, handoff lag, replay diff count, and audit seal failures.
- Add acceptance evidence referencing this IP id.
