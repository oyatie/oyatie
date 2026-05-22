---
ip_id: IP-011
title: "IP-011: SLOs, dashboards, and observability wiring"
microservice: connect
bounded_context: cross-cutting
layers: [slo, dashboard, observability]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0139, ADR-0263]
companion_docs:
  - microservices/connect/slos/connector-availability.openslo.yaml
  - microservices/connect/slos/oauth-token-health.openslo.yaml
  - microservices/connect/slos/webhook-receiver-throughput.openslo.yaml
  - microservices/connect/slos/dlq-overflow-prevention.openslo.yaml
  - microservices/connect/dashboards/connector-usage-by-tenant.json
  - microservices/connect/dashboards/oauth-token-health.md
  - microservices/connect/dashboards/webhook-receiver-throughput.json
  - microservices/connect/dashboards/dlq-state.json
doc_status: published
---

# IP-011: SLOs, dashboards, and observability wiring

## Purpose

Author the 4 OpenSLO manifests and 4 Grafana dashboards for the connect microservice; wire all audit-event-class emissions per ADR-0263; register metrics cardinality budget; register all dashboards in Grafana provisioning config.

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
| `oya_connect_action_total` | tenant_id, connector, result | 10,000 × 500 × 3 = 15M → use recording rules to aggregate by connector; per-tenant series capped at 500 |
| `oya_connect_dlq_depth` | tenant_id, connector | 10,000 × 500 = 5M → per-tenant aggregate preferred |
| `oya_connect_webhook_received_total` | tenant_id, connector, result | 10,000 × 500 × 3 = 15M → aggregate |
| `oya_connect_oauth_grants_active` | tenant_id, connector | 10,000 × 500 = 5M |

All per-tenant per-connector series pre-aggregated via recording rules in `iac/helm-values-connect.yaml` Prometheus rules block. Raw series available for ≤1h (hot storage); aggregated series retained 30d.

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
`IP-011: SLOs, dashboards, and observability wiring` is not a generic implementation packet; it closes the `011 slos dashboards observability` gap for `connect` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Operational readiness is proven through Postgres/OpenBao/Kata/network-policy manifests plus SLO and load-test gates for connector, OAuth, webhook, and DLQ paths. The implementation must keep the µservice boundary intact: contracts remain under `microservices/connect/contracts/openapi/connect-integration.yaml` / `microservices/connect/contracts/proto/connect_integration.proto`, policy decisions remain in `microservices/connect/policy/connector-authorization.cedar`, operational proof remains in `microservices/connect/slos/connector-availability.openslo.yaml`, and the parity claim is checked against `microservices/connect/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/connect/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/contracts/proto/connect_integration.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/policy/connector-authorization.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/slos/connector-availability.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/runbooks/connector-cascade-failure.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/iac/network-policy.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connect/iac/helm-values-connect.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/connect/PRD.md` and `microservices/connect/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `connect`.
2. Diff the declared contract in `microservices/connect/contracts/openapi/connect-integration.yaml` and `microservices/connect/contracts/proto/connect_integration.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/connect/policy/connector-authorization.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/connect/slos/connector-availability.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/connect/PRD.md`, `microservices/connect/ARCHITECTURE.md`, `microservices/connect/contracts/openapi/connect-integration.yaml`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/slos/connector-availability.openslo.yaml`, and `microservices/connect/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/connect/PRD.md`
- `microservices/connect/ARCHITECTURE.md`
- `microservices/connect/contracts/openapi/connect-integration.yaml`
- `microservices/connect/contracts/proto/connect_integration.proto`
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`
- `microservices/connect/policy/connector-authorization.cedar`
- `microservices/connect/slos/connector-availability.openslo.yaml`
- `microservices/connect/runbooks/connector-cascade-failure.md`
- `microservices/connect/catalog/oya-connect-connector-catalog-domain.yaml`
- `microservices/connect/competitor-parity-matrix.md`
- `microservices/connect/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n/Workato define connector breadth; Stripe/Salesforce/Slack/GitHub/GitLab/HubSpot/Notion/Linear/Snowflake/Twilio adapters define early correctness probes; AWS EventBridge defines event-ingest durability pressure. This IP closes the relevant gap by binding `011 slos dashboards observability` to concrete `connect` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
