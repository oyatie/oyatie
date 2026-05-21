---
ip_id: IP-013
title: "IP-013: connector-adapter-trait contract + connector SDK"
microservice: connect
bounded_context: connector-adapter
layers: [contracts, sdk]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0145, ADR-0243, ADR-0249, ADR-0258]
companion_docs:
  - microservices/connect/contracts/connector-adapter-trait.md
  - microservices/connect/sdk-plan.md
doc_status: published
---

# IP-013: connector-adapter-trait contract + connector SDK

## Purpose

Define the stable `ConnectorAdapter` Rust trait that all first-party and third-party connector implementations MUST implement. This is the public ABI boundary for the connect plugin system. Also author the connector SDK (thin wrapper around the trait + sidecar client + test harness).

## Trait definition

```rust
#[async_trait::async_trait]
pub trait ConnectorAdapter: Send + Sync + 'static {
    /// Static metadata — must be idempotent and allocation-free
    fn metadata(&self) -> &ConnectorMetadata;

    /// Invoked by connector-adapter-domain for every action call
    async fn invoke(
        &self,
        ctx: &InvokeContext,
        action: &ActionName,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, AdapterError>;

    /// Returns the JSON Schema for each supported action's payload
    fn action_schema(&self, action: &ActionName) -> Option<serde_json::Value>;

    /// Called by schema-drift-monitor; returns current vendor schema fingerprint
    async fn vendor_schema_fingerprint(&self) -> Result<String, AdapterError>;
}
```

## SDK surface

- `oya-connect-sdk`: re-exports `ConnectorAdapter` trait + `InvokeContext` + `AdapterError`
- `oya-connect-sdk-test-harness`: mock `InvokeContext` + mock sidecar for unit testing adapters
- Documentation: `sdk-plan.md` §SDK usage guide

## SemVer policy (ADR-0258)

- Minor version bump required for new optional trait methods (default impl provided).
- Major version bump required for breaking changes to `invoke` signature.
- Deprecation notice ≥6 months before breaking change; sunset ADR required.

## Acceptance criteria

1. Three first-party adapters implement the trait: `SalesforceAdapter`, `SlackAdapter`, `PagerDutyAdapter`.
2. `PagerDutyAdapter::invoke` never returns `AdapterError::CircuitOpen` for `triggerIncident` action (emergency-services hard rule).
3. `oya-connect-sdk-test-harness` enables offline adapter unit tests without real vendor credentials.
4. `cargo doc --no-deps` produces complete API documentation.
5. Trait version pinned in `Cargo.toml` as `connect-adapter-sdk = "1.0"`.

## Definition of done

- [ ] Trait stabilized; `#[non_exhaustive]` attributes on enums
- [ ] 3 first-party adapter implementations pass CI
- [ ] SDK published to internal crates registry
- [ ] SemVer policy documented in `CHANGELOG.md`


## A. Problem
`IP-013: connector-adapter-trait contract + connector SDK` closes a concrete `connect` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Connector catalog correctness: YAML connector records drive category, auth, action, webhook, PII, pack, emergency-services, and rate-limit behavior before adapter code can invoke a vendor. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connect` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connect/PRD.md` — concrete artifact to verify or update.
- `microservices/connect/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — concrete artifact to verify or update.
- `microservices/connect/contracts/proto/connect_integration.proto` — concrete artifact to verify or update.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connect/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connect/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connect/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/salesforce.yaml` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/slack.yaml` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/stripe.yaml` — concrete artifact to verify or update.
- `microservices/connect/catalog/connectors/github.yaml` — concrete artifact to verify or update.
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
- `microservices/connect/catalog/connectors/salesforce.yaml`
- `microservices/connect/catalog/connectors/slack.yaml`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n define breadth; Salesforce/Slack/Stripe/GitHub/Snowflake define early adapter probes; the trait keeps marketplace adapters compatible. This IP binds `013 connector adapter trait` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
