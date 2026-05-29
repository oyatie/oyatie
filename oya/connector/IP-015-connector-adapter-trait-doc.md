---
ip_id: IP-015
title: "IP-015: Load tests, property tests, and CI lane wiring"
microservice: connector
bounded_context: cross-cutting
layers: [tests, ci]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0139, ADR-0263]
companion_docs:
  - microservices/connector/IP-011-slos-dashboards-observability.md
  - microservices/connector/slos/connector-availability.openslo.yaml
doc_status: published
---

# IP-015: Load tests, property tests, and CI lane wiring

## Purpose

Author the load test set (k6 + Locust), property-based tests (proptest), and CI lane definitions for the connector microservice. Gates SLO-gated promotion per ADR-0139.

## Load tests (k6)

| Test | Target | SLO gate |
|---|---|---|
| `connector-action-volume.js` | 50,000 actions/s for 60s; p99 ≤500ms | connector-availability |
| `webhook-ingest-volume.js` | 10,000 webhooks/s for 60s; p99 ≤200ms ack | webhook-receiver-throughput |
| `oauth-token-health-volume.js` | 1,000 token-fetches/s for 60s; p99 ≤500ms | oauth-token-health |
| `dlq-overflow-pressure.js` | Fill 9,000 entries per tenant; verify cap alert fires | dlq-overflow-prevention |
| `pagerduty-emergency-bypass.js` | PagerDuty triggerIncident at 10× normal rate; verify zero blocks | Emergency-services hard rule |

## Property tests (proptest)

| Property | Crate |
|---|---|
| HMAC constant-time verify: wall-time distribution over 10,000 iterations not bimodal | webhook-receiver-domain |
| OAuth CSRF nonce: state nonce always validated before code exchange | oauth-broker-domain |
| DLQ retry schedule: retry delay always within [backoff × 0.75, backoff × 1.25] | retry-dlq-domain |
| Connector catalog pack filter: emergency-services connector always in result | connector-catalog-domain |
| Idempotency-key dedup: identical key within 5min window returns same entry_id | webhook-receiver-domain |

## CI lanes

| Lane | Type | Gate |
|---|---|---|
| `oya-connector-unit-tests` | Unit | Per PR |
| `oya-connector-integration-tests` | Integration | Per PR |
| `oya-connector-property-tests` | Property/fuzz | Per PR |
| `oya-connector-load-tests` | Load | Per SLO-gated promotion |
| `oya-connector-openapi-contract` | Contract | Per PR (diff = empty) |
| `oya-governance-adr-adherence-matrix` | Compliance | Advisory until 2026-07-15 |
| `oya-governance-abuse-defence-ux-floor` | UX | Per PR (≤2ms p99 gate) |
| `oya-governance-emergency-services-chaos-test` | Chaos | Quarterly |

## Acceptance criteria

1. All 5 k6 load tests pass at targets above.
2. All 5 property tests pass with 10,000 cases each.
3. `pagerduty-emergency-bypass.js` zero blocks confirmed.
4. CI lanes registered in `.github/workflows/connector.yml` or Foundry pipeline equivalent.

## Definition of done

- [ ] Load tests authored and passing in CI
- [ ] Property tests authored and passing
- [ ] CI lane config merged
- [ ] SLO-gated promotion gate `connector-availability` = eligible


## A. Problem
`IP-015: Load tests, property tests, and CI lane wiring` is not a generic implementation packet; it closes the `015 connector adapter trait doc` gap for `connector` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Catalog-backed connector dispatch: connector YAML defines auth, action, webhook, PII, pack, and rate-limit posture; domain code loads and filters those entries before adapter invocation. The implementation must keep the µservice boundary intact: contracts remain under `microservices/connector/contracts/openapi/connector-integration.yaml` / `microservices/connector/contracts/proto/connector_integration.proto`, policy decisions remain in `microservices/connector/policy/connector-authorization.cedar`, operational proof remains in `microservices/connector/slos/connector-availability.openslo.yaml`, and the parity claim is checked against `microservices/connector/competitor-parity-matrix.md`.

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
- `microservices/connector/catalog/connectors/salesforce.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/catalog/connectors/slack.yaml` — verify/update as the authoritative artifact for this IP.
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
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n/Workato define connector breadth; Stripe/Salesforce/Slack/GitHub/GitLab/HubSpot/Notion/Linear/Snowflake/Twilio adapters define early correctness probes; AWS EventBridge defines event-ingest durability pressure. This IP closes the relevant gap by binding `015 connector adapter trait doc` to concrete `connector` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
