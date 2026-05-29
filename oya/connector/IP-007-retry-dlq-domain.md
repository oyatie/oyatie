---
ip_id: IP-007
title: "IP-007: retry-and-DLQ domain crate"
microservice: connector
bounded_context: retry-and-DLQ
layers: [domain]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0145, ADR-0243, ADR-0263]
companion_docs:
  - microservices/connector/catalog/oya-connector-retry-dlq-domain.yaml
  - microservices/connector/runbooks/dlq-overflow.md
  - microservices/connector/dashboards/dlq-state.json
  - microservices/connector/slos/dlq-overflow-prevention.openslo.yaml
doc_status: published
---

# IP-007: retry-and-DLQ domain crate

## Purpose

Implement `oya-connector-retry-dlq-domain` — exponential backoff retry orchestration, DLQ persistence, per-tenant depth cap enforcement, replay surface, and quarantine APIs.

## Acceptance criteria

1. `DlqService::enqueue(tenant_id, connector, action, payload_digest, error_class)` persists entry in `connector.dlq_entries`; emits `DLQEntryAdded` audit event.
2. Retry schedule: 1s → 2s → 4s → 8s → 16s × 5 attempts (configurable per connector); non-retryable errors enqueued immediately without retry.
3. Per-tenant depth cap: default 10,000 entries; when cap approached (≥90%), emit `DLQCapApproaching` alert; at cap, reject new entries with `DlqError::TenantCapExceeded` (caller routes to operator notification).
4. Replay: `DlqService::replay(entry_id)` re-invokes the original action; on success, marks entry `success`; on failure, increments `retry_count`.
5. Quarantine: `DlqService::quarantine(entry_id, reason)` marks entry `quarantined`; quarantined entries excluded from auto-replay; require explicit operator clearance.
6. DLQ depth metric: `oya_connector_dlq_depth{tenant_id, connector}` gauge; updated on every enqueue/replay/abandon/quarantine.
7. ADR-0145 §invariant-1: DLQ accepts overflow without blocking new connector actions; DLQ and action-invoke paths are decoupled.

## Failure modes

1. **DB write failure on enqueue** → return `DlqError::PersistenceFailed`; caller MUST surface error to tenant; action result is `error` not `success`.
2. **Replay triggers same error** → increment retry_count; schedule next backoff; max retries → `abandoned`.
3. **Cap exceeded during incident** → `DLQCapExceeded` alert; ops-dashboard surfaces tenant; manual quarantine/abandon required.

## Definition of done

- [ ] Unit test: retry schedule matches exponential backoff formula with jitter bounds
- [ ] Unit test: depth cap enforcement (per-tenant isolation)
- [ ] Integration test: enqueue → auto-replay → success → entry marked success
- [ ] `cargo clippy -- -D warnings` passes; ≥85% line coverage


## A. Problem
`IP-007: retry-and-DLQ domain crate` is not a generic implementation packet; it closes the `007 retry dlq domain` gap for `connector` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Retry/DLQ as a tenant-scoped control surface: bounded backoff, digest-only payload handling, replay idempotency, and dashboarded cap pressure. The implementation must keep the µservice boundary intact: contracts remain under `microservices/connector/contracts/openapi/connector-integration.yaml` / `microservices/connector/contracts/proto/connector_integration.proto`, policy decisions remain in `microservices/connector/policy/connector-authorization.cedar`, operational proof remains in `microservices/connector/slos/connector-availability.openslo.yaml`, and the parity claim is checked against `microservices/connector/competitor-parity-matrix.md`.

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
- `microservices/connector/catalog/oya-connector-retry-dlq-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/connector/dashboards/dlq-state.json` — verify/update as the authoritative artifact for this IP.
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
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Zapier/n8n/Workato define connector breadth; Stripe/Salesforce/Slack/GitHub/GitLab/HubSpot/Notion/Linear/Snowflake/Twilio adapters define early correctness probes; AWS EventBridge defines event-ingest durability pressure. This IP closes the relevant gap by binding `007 retry dlq domain` to concrete `connector` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
