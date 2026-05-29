---
ip_id: IP-009
title: "IP-009: Connector catalog seed — 30 tier-1 connectors"
microservice: connector
bounded_context: connector-catalog
layers: [catalog]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0249, ADR-0263]
companion_docs:
  - microservices/connector/catalog/connectors/
doc_status: published
---

# IP-009: Connector catalog seed — 30 tier-1 connectors

## Purpose

Author the initial seed of ≥30 `catalog/connectors/*.yaml` entries for the highest-priority connectors across all categories. Each entry carries: auth flows, webhook config, rate-limit profile, action list, compliance annotations, PII field list, pack_allow_list, and data-residency posture.

## Connector roster (Tier-1 seed)

| Category | Connectors |
|---|---|
| Messaging | Slack, Discord |
| Email | Gmail, Outlook, SendGrid, Mailgun |
| Payments | Stripe, Toss Payments, KakaoPay |
| CRM | Salesforce, HubSpot |
| Developer Tools | GitHub, GitLab, Bitbucket |
| Project Management | Jira, Linear, Asana, Trello, ClickUp |
| Productivity | Notion, Airtable, Google Sheets, Google Drive, Dropbox |
| Communications | Twilio |
| Observability | Datadog, PagerDuty, Sentry, LaunchDarkly |
| Analytics | Segment, Mixpanel |
| Data Warehouse | Snowflake, BigQuery |

## Acceptance criteria

1. Each `.yaml` passes `yamllint` with no errors.
2. `pagerduty.yaml` has `critical_path.emergency_services: true` and `critical_path.never_throttle: true`.
3. `toss-payments.yaml` and `kakaopay.yaml` have `pack_allow_list: [pack-kr]` only.
4. All connectors with `pii_fields` populated have `pii_excluded_from_dlq: true`.
5. All connectors with `webhook.signing_algorithm` set have `webhook.replay_window_seconds: 300`.
6. `catalog/connectors/` directory is loadable by `ConnectorCatalog::from_dir()` from IP-002.

## Status

Seed authored at IP-009 time. 30 connectors written. Additional connectors (Monday.com, Box, OneDrive, Bitbucket, Mailgun, WhatsApp Business, LINE Pay, WeChat Pay, Postgres, MySQL, Redis (counterpart-fact external connector), ElasticSearch, AWS services, GCP services, Azure services, Databricks, OpsGenie, Facebook Ads, Google Ads, X/Twitter, Instagram, TikTok, GA4) tracked in backlog for IP-012 (wave-2 connectors).


## A. Problem
`IP-009: Connector catalog seed — 30 tier-1 connectors` closes a concrete `connector` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

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
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n define breadth; Salesforce/Slack/Stripe/GitHub/Snowflake define early adapter probes; the trait keeps marketplace adapters compatible. This IP binds `009 connector catalog seed` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
