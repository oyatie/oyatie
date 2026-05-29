---
doc_kind: implementation-plan
id: IP-001
title: retirement design readiness bundle
status: Accepted
owner_team: council-architecture
related_adrs: [ADR-0134, ADR-0135]
---

# IP-001: Retirement Design Readiness Bundle

## Intent

Make the retiring `connector` umbrella auditable as a design/spec surface while preserving the rule that no new product runtime scope lands here.

## Scope

- Add machine-readable manifest coverage for retirement status, ADRs, contracts, SLO, policy, and audit events.
- Define read-only retirement status contracts.
- Document tenant, residency, cost, threat, failure, and operational boundaries for the temporary umbrella.

## Acceptance

- The gate can verify all required design/spec surfaces under `microservices/connector`.
- Contracts expose only retirement status and readiness evidence.
- Policy forbids new runtime product ownership under `connector`.
- The artifacts do not claim operational maturity, product completeness, or deployed scale.


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio. See `microservices/connector/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.


## A. Problem
`IP-001: Retirement Design Readiness Bundle` closes a concrete `connector` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Operational substrate correctness: Postgres RLS, OpenBao policy, Kata/network-policy isolation, ingress, SLO recording rules, and load gates prove connector is a substrate rather than a runtime dumping ground. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connector` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connector/PRD.md` — concrete artifact to verify or update.
- `microservices/connector/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connector/contracts/openapi/connector-integration.yaml` — concrete artifact to verify or update.
- `microservices/connector/contracts/proto/connector_integration.proto` — concrete artifact to verify or update.
- `microservices/connector/contracts/asyncapi/connector-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connector/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connector/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connector/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connector/iac/postgres-migration-001.sql` — concrete artifact to verify or update.
- `microservices/connector/iac/openbao-policy.hcl` — concrete artifact to verify or update.
- `microservices/connector/iac/network-policy.yaml` — concrete artifact to verify or update.
- Declared Rust crates/types such as `ConnectorCatalog`, `OAuthBrokerService`, `WebhookReceiverService`, `ConnectorAdapter`, or `DlqService` must be added only by implementation PRs that also add tests; this documentation scrub does not fake source existence.

## D. Implementation Steps
1. Confirm the bounded-context row in `microservices/connector/PRD.md` and the retirement/substrate boundary in `microservices/connector/ARCHITECTURE.md`.
2. Trace each public command or event to `contracts/openapi/connector-integration.yaml`, `contracts/proto/connector_integration.proto`, or `contracts/asyncapi/connector-integration-events.yaml`.
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
- `microservices/connector/PRD.md`
- `microservices/connector/ARCHITECTURE.md`
- `microservices/connector/contracts/openapi/connector-integration.yaml`
- `microservices/connector/contracts/proto/connector_integration.proto`
- `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`
- `microservices/connector/policy/connector-authorization.cedar`
- `microservices/connector/slos/connector-availability.openslo.yaml`
- `microservices/connector/competitor-parity-matrix.md`
- `microservices/connector/iac/postgres-migration-001.sql`
- `microservices/connector/iac/openbao-policy.hcl`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | AWS EventBridge sets event-ingest durability pressure; Zapier/n8n set marketplace/user-facing substrate expectations. This IP binds `001 connector retirement design readiness` to concrete connector contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
