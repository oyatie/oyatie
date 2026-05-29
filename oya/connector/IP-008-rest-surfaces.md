---
ip_id: IP-008
title: "IP-008: REST surfaces (catalog-rest, oauth-broker-rest, webhook-receiver-rest)"
microservice: connector
bounded_context: connector-catalog + oauth-broker + webhook-receiver
layers: [api, rest]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0243, ADR-0253, ADR-0258, ADR-0263]
companion_docs:
  - microservices/connector/contracts/openapi/connector-integration.yaml
  - microservices/connector/catalog/oya-connector-catalog-api.yaml
  - microservices/connector/iac/ingress-production.yaml
doc_status: published
---

# IP-008: REST surfaces (catalog-rest, oauth-broker-rest, webhook-receiver-rest)

## Purpose

Wire the domain services from IP-002 through IP-004 into Axum-based REST handlers conforming to the OpenAPI 3.2.0 contract at `contracts/openapi/connector-integration.yaml`. HTTP/3 + Alt-Svc advertisement per ADR-0253.

## Routes

| Handler | Method | Path | Domain service |
|---|---|---|---|
| list_connectors | GET | /v1/catalog | ConnectorCatalogService::query |
| get_connector | GET | /v1/catalog/{connector_name} | ConnectorCatalogService::get_by_id |
| initiate_oauth | POST | /v1/oauth/grants/initiate | OAuthBrokerService::initiate_grant |
| oauth_callback | GET | /v1/oauth/callback | OAuthBrokerService::handle_callback |
| revoke_oauth | DELETE | /v1/oauth/grants/{grant_id} | OAuthBrokerService::revoke_grant |
| list_grants | GET | /v1/oauth/grants | OAuthBrokerService::list_grants |
| register_webhook | POST | /v1/webhooks/endpoints | WebhookReceiverService::register_endpoint |
| receive_webhook | POST | /v1/webhooks/{endpoint_id} | WebhookReceiverService::receive |
| replay_dlq | POST | /v1/dlq/{entry_id}/replay | DlqService::replay |
| list_dlq | GET | /v1/dlq | DlqService::list_entries |

## Acceptance criteria

1. All routes return `Content-Type: application/json`; errors use RFC 7807 Problem JSON.
2. `Alt-Svc: h3=":443"; ma=86400` header on every response per ADR-0253.
3. SPIFFE workload identity validated on every µservice-to-µservice call (not end-user calls).
4. Webhook receiver route (`/v1/webhooks/{endpoint_id}`) returns `200 OK` within ≤200ms p99; signature verification on async path does not block ack.
5. OpenAPI contract validation: all response shapes validated by `utoipa` or equivalent at compile-time.
6. `X-Request-ID` correlated through all spans per ADR-0263 trace shape.

## Definition of done

- [ ] Integration test: round-trip for each route against mock domain services
- [ ] `cargo clippy -- -D warnings` passes
- [ ] OpenAPI contract matches `contracts/openapi/connector-integration.yaml` (diff = empty)


## A. Problem
`IP-008: REST surfaces (catalog-rest, oauth-broker-rest, webhook-receiver-rest)` is not a generic implementation packet; it closes the `008 rest surfaces` gap for `connector` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Integration substrate boundaries keep connector execution outside workflow orchestration while preserving tenant, credential, schema, and policy evidence. The implementation must keep the µservice boundary intact: contracts remain under `microservices/connector/contracts/openapi/connector-integration.yaml` / `microservices/connector/contracts/proto/connector_integration.proto`, policy decisions remain in `microservices/connector/policy/connector-authorization.cedar`, operational proof remains in `microservices/connector/slos/connector-availability.openslo.yaml`, and the parity claim is checked against `microservices/connector/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/connector/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/contracts/openapi/connector-integration.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/contracts/proto/connector_integration.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/contracts/asyncapi/connector-integration-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/policy/connector-authorization.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/slos/connector-availability.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/runbooks/connector-cascade-failure.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/catalog/oya-connector-catalog-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/catalog/connectors/github.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/catalog/connectors/gitlab.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/connector/PRD.md` and `microservices/connector/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `connector`.
2. Diff the declared contract in `microservices/connector/contracts/openapi/connector-integration.yaml` and `microservices/connector/contracts/proto/connector_integration.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/connector/policy/connector-authorization.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/connector/slos/connector-availability.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/connector/catalog/oya-connector-catalog-domain.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/connector/PRD.md`, `microservices/connector/ARCHITECTURE.md`, `microservices/connector/contracts/openapi/connector-integration.yaml`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/slos/connector-availability.openslo.yaml`, and `microservices/connector/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/connector/PRD.md`
- `microservices/connector/ARCHITECTURE.md`
- `microservices/connector/contracts/openapi/connector-integration.yaml`
- `microservices/connector/contracts/proto/connector_integration.proto`
- `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`
- `microservices/connector/policy/connector-authorization.cedar`
- `microservices/connector/slos/connector-availability.openslo.yaml`
- `microservices/connector/runbooks/connector-cascade-failure.md`
- `microservices/connector/catalog/oya-connector-catalog-domain.yaml`
- `microservices/connector/competitor-parity-matrix.md`
- `microservices/connector/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n/Workato define connector breadth; Stripe/Salesforce/Slack/GitHub/GitLab/HubSpot/Notion/Linear/Snowflake/Twilio adapters define early correctness probes; AWS EventBridge defines event-ingest durability pressure. This IP closes the relevant gap by binding `008 rest surfaces` to concrete `connector` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
