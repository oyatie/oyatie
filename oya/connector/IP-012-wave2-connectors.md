---
ip_id: IP-012
title: "IP-012: Wave-2 connector catalog seed (50+ additional connectors)"
microservice: connector
bounded_context: connector-catalog
layers: [catalog]
acceptance_status: backlog
date: 2026-05-20
related_adrs: [ADR-0249]
companion_docs:
  - microservices/connector/IP-009-connector-catalog-seed.md
doc_status: published
---

# IP-012: Wave-2 connector catalog seed (50+ additional connectors)

## Purpose

Expand the connector catalog from the Tier-1 seed (IP-009, 30 connectors) to ≥80 connectors covering:

## Target connectors (Wave-2)

| Category | Connectors |
|---|---|
| Project Management | Monday.com, Basecamp, Teamwork, Smartsheet |
| Storage | Box, OneDrive, SharePoint |
| Developer Tools | Bitbucket, Azure DevOps, CircleCI, Jenkins |
| Email | Mailgun, Postmark, Amazon SES, Brevo |
| Payments | WeChat Pay, LINE Pay, Razorpay, Mollie, Adyen |
| Messaging | WhatsApp Business, LINE, KakaoTalk, Microsoft Teams, Zoom |
| Social | X/Twitter, Instagram, TikTok, Facebook, LinkedIn |
| Advertising | Facebook Ads, Google Ads, TikTok Ads |
| Analytics | GA4, Amplitude, PostHog, Heap |
| Data Warehouse | Databricks, Redshift, Azure Synapse |
| Databases | Postgres (direct), MySQL (direct), Redis (counterpart-fact external connector), Elasticsearch |
| Cloud Services | AWS SNS, AWS S3, AWS Lambda, GCP Pub/Sub, Azure Service Bus |
| Incident Management | OpsGenie, VictorOps |
| Customer Support | Zendesk, Intercom, Freshdesk |
| HR | BambooHR, Workday, Greenhouse |
| Finance | QuickBooks, Xero, NetSuite |

## Acceptance criteria

1. All connectors: `yamllint` passes; required fields present.
2. Korean-domestic connectors (LINE Pay, WeChat Pay via KR): `pack_allow_list` correct.
3. WhatsApp Business: `critical_path.emergency_services_adjacent: true` (crisis-line class per §3.2.5 row 6).
4. All advertising connectors: `behavioral_analytics_consent_required: true`; CCPA/GDPR pack overlays declared.

## Status

Backlog; authoring planned for M01-P4 sprint after Tier-1 seed is loaded and queryable.


## A. Problem
`IP-012: Wave-2 connector catalog seed (50+ additional connectors)` closes a concrete `connector` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Connector catalog correctness: YAML connector records drive category, auth, action, webhook, PII, pack, emergency-services, and rate-limit behavior before adapter code can invoke a vendor. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connector` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connector/PRD.md` — concrete artifact to verify or update.
- `microservices/connector/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connector/contracts/openapi/connector-integration.yaml` — concrete artifact to verify or update.
- `microservices/connector/contracts/proto/connector_integration.proto` — concrete artifact to verify or update.
- `microservices/connector/contracts/asyncapi/connector-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connector/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connector/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connector/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connector/catalog/connectors/salesforce.yaml` — concrete artifact to verify or update.
- `microservices/connector/catalog/connectors/slack.yaml` — concrete artifact to verify or update.
- `microservices/connector/catalog/connectors/stripe.yaml` — concrete artifact to verify or update.
- `microservices/connector/catalog/connectors/github.yaml` — concrete artifact to verify or update.
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
- `microservices/connector/catalog/connectors/salesforce.yaml`
- `microservices/connector/catalog/connectors/slack.yaml`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n define breadth; Salesforce/Slack/Stripe/GitHub/Snowflake define early adapter probes; the trait keeps marketplace adapters compatible. This IP binds `012 wave2 connectors` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
