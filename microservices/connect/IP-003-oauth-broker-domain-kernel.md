---
ip_id: IP-003
title: "IP-003: oauth-broker domain + kernel crates"
microservice: connect
bounded_context: oauth-broker
layers: [domain, kernel]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0243, ADR-0244, ADR-0255, ADR-0296]
companion_docs:
  - microservices/connect/catalog/oya-connect-oauth-broker-domain.yaml
  - microservices/connect/catalog/oya-connect-oauth-broker-kernel.yaml
  - microservices/connect/policy/oauth-broker-authorization.cedar
doc_status: published
---

# IP-003: oauth-broker domain + kernel crates

## Purpose

Implement `oya-connect-oauth-broker-domain` and `oya-connect-oauth-broker-kernel` — OAuth 2.0 + OIDC authorization code + PKCE flow initiation, callback, refresh, and revocation with per-tenant provider-BYOK credential isolation via OpenBao sidecars.

## Acceptance criteria

1. `OAuthBrokerService::initiate_grant(tenant_id, connector_name, scopes, flow_type)` returns `AuthorizationUrl` and persists CSRF state nonce in OpenBao (TTL 10min).
2. `OAuthBrokerService::handle_callback(code, state)` validates nonce, exchanges code for tokens, stores `refresh_token` in OpenBao at `secret/<tenant_id>/connect/oauth/<connector>/<grant_id>`, stores `access_token` at `secret/<tenant_id>/connect/oauth/access-token/<grant_id>` (TTL 60s).
3. Sidecar refresh: `OAuthSidecar::refresh_access_token(grant_id)` exchanges refresh_token for new access_token; stores new token; never exposes refresh_token to caller.
4. `OAuthBrokerService::revoke_grant(grant_id)` marks grant `revoked` in DB, deletes OpenBao secrets, emits `OAuthGrantRevoked` audit event.
5. Cedar gate `oauth-broker-authorization.cedar` consulted for `initiate`, `revoke`, `list_grants` actions.
6. `provider_credential_mode` honored: `byok` reads client_id from `secret/<tenant_id>/connect/oauth-clients/<connector>`; `oyatie_shared` uses platform-level client (deprecated path, logs deprecation warning).
7. Refresh token rotation: if vendor supports rotation, old refresh token deleted on successful rotation.

## Key types

```rust
pub enum OAuthFlowType { AuthorizationCodePkce, ClientCredentials, JwtBearer }

pub struct GrantId(pub Uuid);

pub struct OAuthGrant {
    pub grant_id: GrantId,
    pub tenant_id: TenantId,
    pub connector_name: ConnectorName,
    pub scopes: Vec<String>,
    pub status: GrantStatus,
    pub refresh_token_ref: SecretReference, // never raw token
}

impl OAuthBrokerService {
    pub async fn initiate_grant(&self, req: InitiateGrantRequest) -> Result<AuthorizationUrl, OAuthError>;
    pub async fn handle_callback(&self, req: CallbackRequest) -> Result<OAuthGrant, OAuthError>;
    pub async fn revoke_grant(&self, grant_id: GrantId, reason: &str) -> Result<(), OAuthError>;
}
```

## Failure modes

1. **OpenBao unavailable during callback** → return `OAuthError::CredentialStoreUnavailable`; do NOT persist partial grant; emit `OAuthGrantCallbackFailed` audit event.
2. **State nonce mismatch (CSRF)** → return 400; emit `OAuthCsrfMismatch` audit event; do NOT exchange code.
3. **Vendor /token endpoint 5xx** → return `OAuthError::ProviderError`; do NOT store partial tokens.
4. **Refresh token rotation fails** → retain old token; log warning; schedule retry.

## Definition of done

- [ ] Integration test: mock Salesforce OAuth server → full initiate → callback → refresh → revoke flow
- [ ] Property test: CSRF nonce is always validated before code exchange
- [ ] `cargo clippy -- -D warnings` passes
- [ ] ≥85% line coverage on oauth-broker-domain


## A. Problem
`IP-003: oauth-broker domain + kernel crates` closes a concrete `connect` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
OAuth grant correctness: PKCE state, tenant/provider credential selection, OpenBao refresh-token isolation, short-lived sidecar access tokens, revoke propagation, and sealed grant events. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connect` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connect/PRD.md` — concrete artifact to verify or update.
- `microservices/connect/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — concrete artifact to verify or update.
- `microservices/connect/contracts/proto/connect_integration.proto` — concrete artifact to verify or update.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connect/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connect/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connect/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connect/policy/oauth-broker-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connect/capabilities/oauth-grant-initiate.yaml` — concrete artifact to verify or update.
- `microservices/connect/iac/openbao-policy.hcl` — concrete artifact to verify or update.
- `microservices/connect/catalog/oya-connect-oauth-broker-domain.yaml` — concrete artifact to verify or update.
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
- `microservices/connect/policy/oauth-broker-authorization.cedar`
- `microservices/connect/capabilities/oauth-grant-initiate.yaml`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Salesforce/HubSpot/Slack OAuth grants provide the correctness probe; Workato and MuleSoft set the enterprise governance bar; oyatie adds provider-BYOK isolation. This IP binds `003 oauth broker domain kernel` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
