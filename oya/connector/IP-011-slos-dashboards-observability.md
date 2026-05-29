---
ip_id: IP-011
title: "IP-011: SLOs, dashboards, and observability wiring"
microservice: connector
bounded_context: cross-cutting
layers: [slo, dashboard, observability]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0139, ADR-0263]
companion_docs:
  - microservices/connector/slos/connector-availability.openslo.yaml
  - microservices/connector/slos/oauth-token-health.openslo.yaml
  - microservices/connector/slos/webhook-receiver-throughput.openslo.yaml
  - microservices/connector/slos/dlq-overflow-prevention.openslo.yaml
  - microservices/connector/dashboards/connector-usage-by-tenant.json
  - microservices/connector/dashboards/oauth-token-health.md
  - microservices/connector/dashboards/webhook-receiver-throughput.json
  - microservices/connector/dashboards/dlq-state.json
doc_status: published
---

# IP-011: SLOs, dashboards, and observability wiring

## Purpose

Author the 4 OpenSLO manifests and 4 Grafana dashboards for the connector microservice; wire all audit-event-class emissions per ADR-0263; register metrics cardinality budget; register all dashboards in Grafana provisioning config.

## SLO targets

| SLO | Target | Burn-rate alert |
|---|---|---|
| connector-availability | 99.9% | 14× / 1h fast-burn |
| oauth-token-health | 99.5% | 14× / 1h fast-burn |
| webhook-receiver-throughput | 99.5% | 14× / 1h fast-burn |
| dlq-overflow-prevention | 99.0% | 10× / 1h fast-burn |

## Metrics cardinality budget

| Metric | Labels | Max series |
|---|---|---|
| `oya_connector_action_total` | tenant_id, connector, result | 10,000 × 500 × 3 = 15M → use recording rules to aggregate by connector; per-tenant series capped at 500 |
| `oya_connector_dlq_depth` | tenant_id, connector | 10,000 × 500 = 5M → per-tenant aggregate preferred |
| `oya_connector_webhook_received_total` | tenant_id, connector, result | 10,000 × 500 × 3 = 15M → aggregate |
| `oya_connector_oauth_grants_active` | tenant_id, connector | 10,000 × 500 = 5M |

All per-tenant per-connector series pre-aggregated via recording rules in `iac/helm-values-connector.yaml` Prometheus rules block. Raw series available for ≤1h (hot storage); aggregated series retained 30d.

## Acceptance criteria

1. All 4 SLO manifests pass `openslo validate`.
2. All 4 dashboards importable into Grafana 10.x without errors.
3. Recording rules registered and tested against a Prometheus instance with sample data.
4. Grafana provisioning config references all 4 dashboard UIDs.
5. ADR-0263 audit-event-class list in `ARCHITECTURE.md §observability` matches all `emit_audit_event!()` call sites in the codebase.

## Definition of done

- [ ] `openslo validate slos/*.openslo.yaml` passes
- [ ] `grafana-dash-import --validate dashboards/*.json` passes
- [ ] CI audit-emission coverage lane green


## A. Problem
`IP-011: SLOs, dashboards, and observability wiring` is not a generic implementation packet; it closes the `011 slos dashboards observability` gap for `connector` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Operational readiness is proven through Postgres/OpenBao/Kata/network-policy manifests plus SLO and load-test gates for connector, OAuth, webhook, and DLQ paths. The implementation must keep the µservice boundary intact: contracts remain under `microservices/connector/contracts/openapi/connector-integration.yaml` / `microservices/connector/contracts/proto/connector_integration.proto`, policy decisions remain in `microservices/connector/policy/connector-authorization.cedar`, operational proof remains in `microservices/connector/slos/connector-availability.openslo.yaml`, and the parity claim is checked against `microservices/connector/competitor-parity-matrix.md`.

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
- `microservices/connector/iac/network-policy.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/iac/helm-values-connector.yaml` — verify/update as the authoritative artifact for this IP.
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
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n/Workato define connector breadth; Stripe/Salesforce/Slack/GitHub/GitLab/HubSpot/Notion/Linear/Snowflake/Twilio adapters define early correctness probes; AWS EventBridge defines event-ingest durability pressure. This IP closes the relevant gap by binding `011 slos dashboards observability` to concrete `connector` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
