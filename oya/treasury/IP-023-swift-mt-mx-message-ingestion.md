---
doc_class: ImplementationPlan
ip_id: IP-023
microservice: treasury
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0319]
journey_id: j106-multi-currency-cross-border-payment
journey_link: docs/user-journeys/j106-multi-currency-cross-border-payment/story.md
status: Accepted
date: 2026-05-20
owner: axis-treasury
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [TRM-CM Bank Statement, TRM-TM SWIFT Confirmation, TRM-RM Settlement Risk Feeds]
---

# IP-023: SWIFT MT/MX message ingestion

## Intent
Implement treasury ingestion for SWIFT MT and ISO 20022 MX bank messages with validation, normalization, idempotency, and downstream handoffs.
The feature supports statement, confirmation, acknowledgement, and exception messages that treasury needs for cash, payments, debt, and risk workflows.
The feature displaces SAP Bank Communication Management file monitors, SWIFT connector parsing, and selected TRM confirmation ingestion surfaces.
The implementation must accept already-delivered files or payloads from bank connectivity; it does not establish SWIFT network connectivity itself.
The implementation must normalize MT and MX into canonical treasury message events.
The implementation must preserve the raw message, parser result, validation findings, and evidence hash.
The implementation must enforce tenant, bank profile, message type, and data-residency policy before processing.
The implementation must emit ADR-0263 audit events for ingestion, parsing, rejection, replay, and downstream delivery.
The implementation must support safe replay without duplicate downstream events.
The implementation must be intern-buildable from schema, endpoints, parser boundaries, and fixtures.

## Context
Why: treasury cannot rely on payment and cash-position state unless bank acknowledgements, statements, and confirmations are ingested consistently.
Why: SAP BCM and SWIFT connector monitors hide parsing and retry behavior behind technical queues that are hard to test.
Why: Oyatie needs a canonical message inbox that can feed bank-statement, payments, risk, and audit services.
Journey leg: j106 bank operations receives MT940 statements, camt.053 statements, pain.002 acknowledgements, and MT300 confirmations during the same cross-border payment window.
Named persona: Aicha Diallo, Cash Operations Analyst at WAFRIA Energy, investigates a rejected bank acknowledgement before releasing the next payment batch.
Supporting persona: Kenji Sato, Treasury Integration Engineer, maps a new bank's camt.053 variant without changing core treasury logic.
Pain point: MT940 and camt.053 contain similar statement concepts but produce different downstream records.
Pain point: duplicate bank files arrive after retry storms and currently create duplicate statement lines.
Pain point: malformed message payloads are quarantined without enough evidence for operations to fix bank profiles.
SAP parity: SAP BCM bank communication monitor, SWIFT connector, electronic bank statement import, and TRM-TM confirmation ingestion.
Product outcome: operators see every bank message lifecycle from received to parsed to delivered or rejected.
Non-goal: SWIFT Alliance Lite2 connectivity and EBICS transport remain in bank-connectivity services.
Non-goal: outbound payment generation remains in IP-017 and IP-025.
Non-goal: sanctions screening remains in compliance.
Invariant: raw payload hash plus bank profile plus tenant produces idempotent ingestion.
Invariant: a rejected message never emits downstream business events.
Invariant: replay reuses the original raw payload and creates a new parser attempt row.
Invariant: parser profiles are versioned and effective dated.
Acceptance anchor: an intern can implement tables, parser adapter interface, APIs, policies, workflows, fixtures, and replay tests from this file.

## Data Model Deltas
Table `treasury.bank_message_profile`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `bank_id UUID NOT NULL`.
Column `profile_code TEXT NOT NULL`.
Column `message_family TEXT NOT NULL CHECK (message_family IN ('SWIFT-MT','ISO20022-MX'))`.
Column `supported_message_types TEXT[] NOT NULL`.
Column `parser_version TEXT NOT NULL`.
Column `data_residency_region TEXT NOT NULL`.
Column `active BOOLEAN NOT NULL DEFAULT true`.
Column `effective_from DATE NOT NULL`.
Column `effective_to DATE`.
Constraint `UNIQUE (tenant_id, bank_id, profile_code, effective_from)`.
Table `treasury.bank_message_inbox`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `profile_id UUID NOT NULL REFERENCES treasury.bank_message_profile(id)`.
Column `received_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
Column `source_channel TEXT NOT NULL CHECK (source_channel IN ('SWIFTNet','EBICS','SFTP','API','ManualUpload'))`.
Column `message_type TEXT NOT NULL`.
Column `message_reference TEXT`.
Column `raw_payload_hash TEXT NOT NULL`.
Column `raw_payload_uri TEXT NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Received','Parsed','Rejected','Delivered','Quarantined','Replayed'))`.
Column `idempotency_key TEXT NOT NULL`.
Column `evidence_hash TEXT NOT NULL`.
Column `cedar_decision_id UUID NOT NULL`.
Constraint `UNIQUE (tenant_id, idempotency_key)`.
Index `ix_bank_message_inbox_status` on `(tenant_id, received_at DESC, status)`.
Table `treasury.bank_message_parse_attempt`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `inbox_id UUID NOT NULL REFERENCES treasury.bank_message_inbox(id)`.
Column `attempt_number INTEGER NOT NULL`.
Column `parser_version TEXT NOT NULL`.
Column `started_at TIMESTAMPTZ NOT NULL`.
Column `finished_at TIMESTAMPTZ`.
Column `status TEXT NOT NULL CHECK (status IN ('Running','Succeeded','Failed'))`.
Column `validation_error_count INTEGER NOT NULL DEFAULT 0`.
Column `canonical_event_count INTEGER NOT NULL DEFAULT 0`.
Column `failure_code TEXT`.
Column `failure_message TEXT`.
Constraint `UNIQUE (inbox_id, attempt_number)`.
Table `treasury.bank_message_canonical_event`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `inbox_id UUID NOT NULL REFERENCES treasury.bank_message_inbox(id)`.
Column `parse_attempt_id UUID NOT NULL REFERENCES treasury.bank_message_parse_attempt(id)`.
Column `canonical_event_type TEXT NOT NULL CHECK (canonical_event_type IN ('BankStatement','PaymentAck','PaymentNack','FxConfirmation','DebtConfirmation','ServiceMessage'))`.
Column `external_reference TEXT`.
Column `bank_account_id UUID`.
Column `currency CHAR(3)`.
Column `amount NUMERIC(22,4)`.
Column `value_date DATE`.
Column `payload_json JSONB NOT NULL`.
Column `delivered_to TEXT[] NOT NULL DEFAULT '{}'`.
Column `delivery_status TEXT NOT NULL CHECK (delivery_status IN ('Pending','Delivered','Failed'))`.
Constraint `UNIQUE (tenant_id, inbox_id, canonical_event_type, external_reference)`.
Table `treasury.bank_message_validation_finding`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `parse_attempt_id UUID NOT NULL REFERENCES treasury.bank_message_parse_attempt(id)`.
Column `severity TEXT NOT NULL CHECK (severity IN ('Info','Warning','Blocking'))`.
Column `field_path TEXT`.
Column `code TEXT NOT NULL`.
Column `message TEXT NOT NULL`.
Column `raw_excerpt_hash TEXT`.
Storage rule: raw payloads are stored in object storage through content-addressed URI; database stores hash and URI only.
Partitioning rule: inbox and canonical events partition by tenant cell and received month.
Retention rule: raw payload and parse evidence retained according to financial-record retention policy, minimum ten years for delivered financial messages.

## API Endpoints
REST `POST /v1/treasury/bank-messages`.
Request example:
```json
{
  "profile_id": "3a5d8a1f-1111-4f33-9222-333344445555",
  "source_channel": "SFTP",
  "message_type": "camt.053.001.08",
  "message_reference": "BANKREF-20260520-001",
  "raw_payload_hash": "sha256:6f12...",
  "raw_payload_uri": "object://treasury-bank-messages/2026/05/20/6f12"
}
```
Response example:
```json
{
  "inbox_id": "4ac4db67-2222-4f61-b333-555566667777",
  "status": "Received",
  "idempotent_replay": false
}
```
REST `POST /v1/treasury/bank-messages/{inbox_id}/parse`.
Parse response returns parse attempt id, status, canonical event count, and validation finding count.
REST `POST /v1/treasury/bank-messages/{inbox_id}/deliver`.
Deliver response returns downstream targets and delivered event ids.
REST `POST /v1/treasury/bank-messages/{inbox_id}/replay`.
Replay request includes parser version override and reason code.
REST `GET /v1/treasury/bank-messages/{inbox_id}` returns raw metadata, attempts, findings, and canonical events.
REST `GET /v1/treasury/bank-messages?message_type=MT940&status=Rejected`.
gRPC `TreasuryBankMessageIngestionService.Ingest(IngestBankMessageRequest) returns (BankMessageInboxRecord)`.
gRPC `TreasuryBankMessageIngestionService.Parse(ParseBankMessageRequest) returns (BankMessageParseResult)`.
Error `409 BANK_MESSAGE_DUPLICATE` returns existing inbox id for same idempotency key.
Error `422 BANK_MESSAGE_TYPE_UNSUPPORTED` when profile does not support message type.
Error `403 BANK_MESSAGE_PROFILE_POLICY_DENIED` when Cedar blocks profile or region.

## Cedar Policy Hooks
Principal shape: `ServiceOrUser::{ id, tenant_id, roles, bank_profile_scope, residency_region_scope }`.
Action `Action::"ingest_bank_message"`.
Action `Action::"parse_bank_message"`.
Action `Action::"deliver_bank_message_events"`.
Action `Action::"replay_bank_message"`.
Resource `BankMessageInbox::{ tenant_id, profile_id, message_type, source_channel, data_residency_region, status }`.
Context `BankMessageContext::{ now, payload_hash, object_region, parser_version, replay_reason, request_origin }`.
Permit bank-connectivity service principals to ingest when profile scope includes profile id.
Permit treasury integration engineers to replay quarantined messages with reason code.
Forbid ingest when object region differs from profile data residency region.
Forbid parse when message type is not supported by profile.
Forbid deliver unless inbox status is Parsed and parse attempt succeeded.
Forbid replay of Delivered messages unless principal has role `treasury-message-replay-supervisor`.
Emit `SwiftMtMxMessagePolicyDenied` for every deny.
Policy fixture `policy/swift-mx-residency-deny.json`.
Policy fixture `policy/swift-mx-unsupported-type-deny.json`.
Policy fixture `policy/swift-mx-delivered-replay-deny.json`.

## Ontology Projection
SAP BCM message monitor entry maps to `Oyatie::Treasury::BankMessageInbox`.
SAP electronic bank statement item maps to canonical event `BankStatement`.
SAP payment medium ack maps to canonical event `PaymentAck` or `PaymentNack`.
SAP TRM-TM confirmation maps to canonical event `FxConfirmation` or `DebtConfirmation`.
SWIFT MT940 maps to bank statement canonical payload.
SWIFT MT942 maps to intraday statement canonical payload.
SWIFT MT300 maps to FX confirmation canonical payload.
ISO 20022 camt.053 maps to bank statement canonical payload.
ISO 20022 pain.002 maps to payment acknowledgement canonical payload.
ISO 20022 camt.054 maps to debit or credit notification canonical payload.
Ontology field `BankMessage.messageType` maps from `message_type`.
Ontology field `BankMessage.rawPayloadHash` maps from `raw_payload_hash`.
Ontology field `BankMessage.status` maps from `status`.
Ontology field `CanonicalBankEvent.eventType` maps from `canonical_event_type`.
Ontology field `CanonicalBankEvent.externalReference` maps from `external_reference`.
Ontology edge `MESSAGE_PARSED_IN_ATTEMPT` connects inbox to parse attempt.
Ontology edge `MESSAGE_PRODUCED_CANONICAL_EVENT` connects inbox to canonical event.
Ontology edge `CANONICAL_EVENT_DELIVERED_TO_SERVICE` connects event to downstream service.
Ontology edge `MESSAGE_PROFILE_ACCEPTS_TYPE` connects profile to supported message type.
Projection must hide raw payload URI from users without sensitive topology permission.

## Workflow Steps
Workflow `treasury.bank_message.ingest`.
Node `load_message_profile` validates active profile and effective date.
Node `cedar_ingest_check` validates tenant, bank profile, and residency region.
Node `compute_idempotency_key` uses tenant, profile, raw payload hash, and message reference.
Node `persist_inbox_record` creates Received record or returns duplicate.
Node `emit_message_received`.
Workflow `treasury.bank_message.parse`.
Node `load_raw_payload` reads object by URI and verifies hash.
Node `select_parser` chooses MT or MX parser by profile and message type.
Node `parse_raw_message` returns canonical candidate records.
Node `validate_canonical_events` checks required fields, BIC, IBAN, amount signs, and value date.
Node `persist_parse_attempt` records parser version and findings.
Node `persist_canonical_events` writes canonical events for successful attempts.
Node `mark_inbox_parsed_or_rejected`.
Node `emit_message_parsed_or_rejected`.
Branch `blocking_validation_findings` marks inbox Rejected and skips delivery.
Branch `parser_exception` marks attempt Failed and inbox Quarantined.
Workflow `treasury.bank_message.deliver`.
Node `load_pending_canonical_events`.
Node `route_events_to_downstream_services`.
Node `publish_bank_statement_events`.
Node `publish_payment_ack_events`.
Node `publish_confirmation_events`.
Node `mark_events_delivered`.
Node `emit_message_delivered`.

## Audit Events
Audit event class `TreasurySwiftMtMxMessageReceived`.
Audit event class `TreasurySwiftMtMxMessageDuplicateReceived`.
Audit event class `TreasurySwiftMtMxMessageParseStarted`.
Audit event class `TreasurySwiftMtMxMessageParsed`.
Audit event class `TreasurySwiftMtMxMessageRejected`.
Audit event class `TreasurySwiftMtMxMessageQuarantined`.
Audit event class `TreasurySwiftMtMxCanonicalEventCreated`.
Audit event class `TreasurySwiftMtMxCanonicalEventDelivered`.
Audit event class `TreasurySwiftMtMxMessageReplayRequested`.
Audit event class `TreasurySwiftMtMxMessagePolicyDenied`.
Audit payload must include tenant id, profile id, inbox id, message type, raw payload hash, and parser version.
Audit payload for rejection must include validation finding codes and blocking count.
Audit payload for delivery must include downstream target names and canonical event ids.
Audit payload for replay must include replay reason and previous attempt id.
Audit retention class is `TreasuryBankMessageEvidence`.
Audit ordering key is `tenant_id:profile_id:raw_payload_hash`.

## SLO Targets
p50 ingest latency excluding object upload: 70 ms.
p95 ingest latency excluding object upload: 250 ms.
p99 ingest latency excluding object upload: 600 ms.
p50 parse latency for 5 MB camt.053: 500 ms.
p95 parse latency for 5 MB camt.053: 1800 ms.
p99 parse latency for 5 MB camt.053: 3500 ms.
p50 delivery latency for 100 canonical events: 120 ms.
p95 delivery latency for 100 canonical events: 500 ms.
p99 delivery latency for 100 canonical events: 1200 ms.
Throughput target: 1000 messages ingested per minute per cell.
Throughput target: 100000 canonical events delivered per minute per cell.
Availability target for ingest API: 99.95 percent monthly.
Availability target for message read API: 99.99 percent monthly.
Rationale: file arrival bursts happen at bank cutoffs and must not block close.
Rationale: read API availability supports operations investigation and audit.
Rationale: parse p99 must remain low enough for replay workflows during close windows.

## Failure Modes + Recovery
Failure `RAW_PAYLOAD_HASH_MISMATCH`: detect object hash mismatch; recover by quarantining and requesting re-delivery.
Failure `UNSUPPORTED_MESSAGE_TYPE`: detect profile mismatch; recover by updating profile or rejecting message.
Failure `PARSER_VERSION_MISSING`: detect parser registry miss; recover by routing to integration engineer.
Failure `BLOCKING_VALIDATION_FINDING`: detect required field or schema violation; recover by bank profile fix and replay.
Failure `DUPLICATE_MESSAGE`: detect idempotency conflict; recover by returning existing inbox without delivery duplication.
Failure `DOWNSTREAM_DELIVERY_FAILED`: detect publish failure; recover by retrying pending canonical event delivery.
Failure `DATA_RESIDENCY_DENY`: detect object region mismatch; recover by re-uploading payload to approved region.
Failure `RAW_PAYLOAD_UNAVAILABLE`: detect object storage read failure; recover by retrying or marking Quarantined.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain error; recover by aborting state transition and retrying.
Failure `PARTIAL_CANONICAL_EVENT_WRITE`: prevent with transaction; repair by marking attempt Failed and replaying.
Recovery worker `treasury.bank_message.delivery_retry` retries failed downstream delivery with idempotency keys.
Runbook entry `runbooks/swift-mt-mx-message-ingestion-failure.md` should cover quarantine, replay, and profile updates.

## Migration Notes
Source vendor surface: SAP BCM bank communication monitor.
Source vendor surface: SAP SWIFT connector inbound queues.
Source vendor surface: SAP electronic bank statement import.
Source vendor surface: SAP TRM-TM confirmation matching.
Source vendor surface: Kyriba bank connectivity message archive.
Source vendor surface: GTreasury bank file monitor.
Source vendor surface: FIS Quantum confirmation feeds.
Migration maps SAP bank communication channel to bank message profile.
Migration maps SAP message id to message reference.
Migration maps archived bank files to raw payload URI and hash.
Migration maps SAP parse error logs to validation finding rows when available.
Migration imports only message metadata and raw payload pointers for historical messages unless full replay is required.
Migration dry-run report lists unsupported MT and MX message types.
Migration dry-run report lists payloads missing raw archive files.
Migration acceptance requires replay of representative MT940, MT942, MT300, camt.053, camt.054, and pain.002 fixtures.

## Cross-microservice Handoffs
Handoff to `bank-connectivity`: receive payload URI, hash, channel, and bank profile.
Handoff to `bank-statement`: deliver canonical bank statement events.
Handoff to `payments`: deliver payment ack and nack events.
Handoff to `debt`: deliver debt confirmation events.
Handoff to `fx-transaction`: deliver FX confirmation events.
Handoff to `compliance`: provide suspicious message metadata when validation patterns indicate tampering.
Handoff to `workflow`: orchestrate ingest, parse, deliver, quarantine, and replay.
Handoff to `ontology`: project bank message and canonical event graph.
Handoff to `audit-chain`: seal received, parsed, rejected, delivered, and replay events.
Handoff to `ops-dashboard`: expose queue depth, rejection count, parser errors, and replay count.

## Build Notes
Add database migration for profile, inbox, parse attempt, canonical event, and finding tables.
Add parser adapter interface `BankMessageParser`.
Add MT parser implementation for MT940, MT942, and MT300 fixtures.
Add MX parser implementation for camt.053, camt.054, and pain.002 fixtures.
Add idempotency service based on tenant, profile, raw hash, and message reference.
Add Cedar schema for bank message inbox and context.
Add REST handlers for ingest, parse, deliver, replay, and read.
Add gRPC handlers for ingest and parse.
Add contract tests for duplicate message, unsupported type, residency deny, and replay.
Add workflow tests for parser exception, blocking validation, and downstream delivery retry.
Add load fixture with 1000 messages and 100000 canonical events.
Add migration fixture with SAP BCM message monitor export.
Add dashboard panels for ingest latency, parse latency, rejection rate, quarantine depth, and replay success.
