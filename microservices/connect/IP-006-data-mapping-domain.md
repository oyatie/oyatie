---
ip_id: IP-006
title: "IP-006: data-mapping domain crate"
microservice: connect
bounded_context: data-mapping
layers: [domain]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0244, ADR-0255, ADR-0257]
companion_docs:
  - microservices/connect/catalog/oya-connect-data-mapping-domain.yaml
doc_status: published
---

# IP-006: data-mapping domain crate

## Purpose

Implement `oya-connect-data-mapping-domain` — the field mapping engine that transforms vendor-specific event shapes into the oyatie canonical event schema. Supports visual field-mapper config, schema-drift detection, per-field data-class tagging, and AI-powered auto-suggestions via Intelligence library-first dispatch.

## Acceptance criteria

1. `DataMapper::map(vendor_payload, mapping_config)` returns `CanonicalEvent` applying per-field transformations defined in `MappingConfig`.
2. `MappingConfig` supports: direct field copy, JSON path extraction, constant injection, Jinja2-style template expressions, type coercions (string→datetime, string→number, etc.).
3. Schema-drift detection: `SchemaDriftDetector::diff(known_schema, incoming_sample)` returns `Vec<SchemaDiff>` enumerating added/removed/type-changed fields; emits `SchemaDriftDetected` audit event.
4. Per-field data-class tagging: `FieldDataClass` (PII_EMAIL, PII_PHONE, FINANCIAL_TRANSACTION, INTERNAL_ONLY, etc.) propagated to `CanonicalEvent` for downstream DLP gating.
5. AI auto-suggest: `DataMappingAssistant::suggest(source_schema, target_schema)` calls Intelligence library-first per ADR-0255 amendment; audience tag `OYATIE_INTERNAL_CONNECT_DATA_MAPPING`.
6. PII fields from connector YAML (`pii_fields`) respected: DLP hook checks these fields before canonicalization; `PIILeakViaConnector` detection signal emitted if PII reaches unexpected destination.

## Definition of done

- [ ] Unit test: map Salesforce Lead → canonical Contact event; verify PII field tagging
- [ ] Unit test: schema-drift detection catches field removal and type change
- [ ] Integration test: AI auto-suggest (mocked Intelligence response)
- [ ] `cargo clippy -- -D warnings` passes


## A. Problem
`IP-006: data-mapping domain crate` closes a concrete `connect` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Data mapping correctness: source schema samples become canonical events with per-field data classes, schema-drift alerts, and DLP labels before downstream workflow use. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connect` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connect/PRD.md` — concrete artifact to verify or update.
- `microservices/connect/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — concrete artifact to verify or update.
- `microservices/connect/contracts/proto/connect_integration.proto` — concrete artifact to verify or update.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connect/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connect/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connect/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connect/catalog/oya-connect-data-mapping-domain.yaml` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/salesforce.yaml` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/hubspot.yaml` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/snowflake.yaml` — concrete artifact to verify or update.
- Declared Rust crates/types such as `ConnectorCatalog`, `OAuthBrokerService`, `WebhookReceiverService`, `ConnectorAdapter`, or `DlqService` must be added only by implementation PRs that also add tests; this documentation scrub does not fake source existence.

## D. Implementation Steps
1. Confirm the bounded-context row in `microservices/connect/PRD.md` and the retirement/substrate boundary in `microservices/connect/ARCHITECTURE.md`.
2. Trace each public command or event to `contracts/openapi/connect-integration.yaml`, `contracts/proto/connect_integration.proto`, or `contracts/asyncapi/connect-integration-events.yaml`.
3. Check the relevant Cedar policy before adding publish, OAuth, webhook, invoke, replay, or catalog mutation behavior.
4. Bind credentials through `iac/openbao-policy.hcl` and never through raw tenant tokens in docs, tests, or examples.
5. Attach an SLO, dashboard, runbook, or audit-event class for every failure mode named in this IP.
6. Run the IP-specific cargo/gate/contract/load command when source exists; otherwise record the missing crate as implementation debt.

## E. Acceptance
- Artifact links above resolve in this checkout.
- Vendor-specific probes include at least one real connector catalog entry, not a hypothetical vendor.
- Credential, webhook, and DLQ paths have policy plus audit evidence before runtime claims.
- The counterpart matrix row is updated when parity changes.

## F. Evidence
- `microservices/connect/PRD.md`
- `microservices/connect/ARCHITECTURE.md`
- `microservices/connect/contracts/openapi/connect-integration.yaml`
- `microservices/connect/contracts/proto/connect_integration.proto`
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`
- `microservices/connect/policy/connector-authorization.cedar`
- `microservices/connect/slos/connector-availability.openslo.yaml`
- `microservices/connect/competitor-parity-matrix.md`
- `microservices/connect/catalog/oya-connect-data-mapping-domain.yaml`
- `microservices/connect/catalog/connectors/salesforce.yaml`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Salesforce and HubSpot custom fields plus Snowflake warehouse schemas are the probe set; Workato/Boomi/MuleSoft set visual-mapper parity pressure. This IP binds `006 data mapping domain` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
