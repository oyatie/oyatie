---
id: ADR-MS-001
title: Connector broker, webhook receiver, and DLQ replay contract for connect
status: Proposed
date: 2026-05-20
microservice: connector
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0043-secrets-management-openbao-and-hsm-per-cell
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters
  - ADR-0128-hyperscaler-architecture-invariants
decision_owner: axis-connector + council-integrations
---

# ADR-MS-001: Connector broker, webhook receiver, and DLQ replay contract for connect

## Context

- Pressure name: integration sprawl pressure.
- Workflow, marketplace, foundry, ops-dashboard-control-center, intelligence, and product surfaces need third-party SaaS calls.
- Without a shared integration substrate, every consumer would reimplement OAuth, webhook HMAC verification, adapter retry, schema drift, and DLQ replay.
- The service PRD names `connector` as integration substrate, not a product endpoint.
- The PRD requires a searchable catalog of at least 500 connectors at GA.
- The PRD requires OAuth authorization-code, client-credentials, JWT-bearer, and refresh-token flows.
- The PRD requires webhook receiver registration, HMAC verification, and spoof denial.
- The PRD requires connector adapters that handle auth, retry, rate limit, circuit break, and observability.
- The PRD requires retry and DLQ with replay UI and per-wiring policy.
- The PRD requires provider-BYOK credential provisioning through OpenBao references.
- The PRD requires library-first dispatch when Foundry jobs and connector adapters are collocated.
- Local OpenAPI exposes `/catalog/connectors` and `/catalog/connectors/{connector_id}`.
- Local OpenAPI exposes `/oauth/grants/initiate`, `/oauth/grants/callback`, and `/oauth/grants/{grant_id}`.
- Local OpenAPI exposes `/webhook-endpoints` and `/webhook-endpoints/{endpoint_id}/rotate-secret`.
- Local OpenAPI exposes `/actions/invoke`, `/dlq`, `/dlq/{entry_id}/replay`, and `/schema-drift/{wiring_id}`.
- Local AsyncAPI emits `OAuthGrantIssued`, `OAuthGrantRevoked`, `WebhookReceived`, `WebhookSignatureVerifyFailed`, `ConnectorActionInvoked`, `ConnectorActionFailed`, `DLQEntryAdded`, `DLQEntryReplayed`, `SchemaDriftDetected`, and `AbuseDefenceChallengeIssued`.
- Constraint name: credential sidecar isolation.
- The substrate must never store raw tenant-owned provider credentials in service-local tables.
- Credential reads use OpenBao `SecretReference` values and <=60 second sidecar TTL behavior.
- Constraint name: webhook spoof pressure.
- Vendor webhooks are public ingress and must be HMAC verified before workflow handoff.
- A spoofed payload returns 401 and emits audit evidence.
- Constraint name: vendor-tail pressure.
- Connector action calls depend on vendor rate limits, 429 retry headers, vendor 5xx behavior, and long-tail latency.
- The PRD allows three retries with jitter for 5xx before DLQ.
- The PRD calls for hedged requests for read-only connector calls after p95 budget.
- Constraint name: pack-aware catalog pressure.
- Tenant pack, jurisdiction code, vendor eligibility, and cell egress posture determine which connectors appear.
- Tier-3 data cells with no internet egress must see internal-protocol connectors only.
- The catalog cannot present a connector a tenant cannot lawfully or physically invoke.

## Decision

- Decision name: tenant-wiring connector control contract.
- `connector` will treat `ConnectorWiring` as the canonical unit of integration.
- `ConnectorWiring` joins tenant, connector version, auth grant, webhook endpoint, mapping version, rate budget, and DLQ policy.
- The connector catalog will list only connectors whose policy, pack, cell, and credential mode match the requesting tenant.
- OAuth grants will store refresh tokens and client secrets only as cloud-secrets references.
- OAuth callback validation will bind `state`, tenant, connector id, requested scopes, redirect id, and trace context.
- Provider-BYOK client identity will override any shared Oyatie connector identity when configured by tenant admin.
- Webhook endpoint registration will create a tenant-specific endpoint id and signing secret reference.
- Webhook receive will verify HMAC before parsing domain payloads.
- Webhook receive will emit `WebhookSignatureVerifyFailed` before returning 401 on spoof or mismatch.
- Connector action invocation will require a `ConnectorActionRequest` with tenant, wiring id, action, input schema version, idempotency key, and trace context.
- Connector action invocation will use vendor-specific adapter manifests but one shared action contract.
- Vendor 429 responses must respect `Retry-After` and consume the wiring rate budget.
- Vendor 5xx responses retry three times with exponential backoff and jitter before DLQ.
- Read-only connector actions may hedge after p95 budget when idempotent and vendor terms allow duplicate reads.
- Non-idempotent write actions must not hedge.
- DLQ entries retain full context for 7 days by default and 30 days maximum when compliance pack permits.
- DLQ replay must re-evaluate Cedar, connector version, mapping version, and credential state before dispatch.
- Schema drift detection must mark impacted wirings and block unsafe automatic replay.
- Connector adapters may be loaded lazily and cached by content-addressed marketplace artifact digest.
- Connector adapter version deprecation uses 90-day notice and 180-day sunset unless security revocation forces faster removal.
- Availability target for connector dispatch is 99.95% for managed substrate paths.
- Availability target for OAuth broker is 99.9%.
- Availability target for webhook receive path is 99.95%.
- Webhook receiver p99 throughput path must acknowledge valid events within the SLO budget.
- OAuth token health SLO target is 0.995.
- DLQ headroom target is 0.99.
- Per-tenant abuse false-positive rate must remain <=0.1% for legitimate vendor calls.
- Metrics must cap connector cardinality by connector id, action, status, tenant pack, and bucketed cell tier.
- Raw vendor payloads may be stored only under the tenant data class and retention policy required by the wiring.

## Alternatives Considered

### Alternative 1: Let each consumer own its own connectors

- Pros: consumer teams can move independently.
- Pros: no shared integration queue during early buildout.
- Cons: OAuth, webhook, retry, DLQ, and observability duplicate across consumers.
- Cons: vendor credentials sprawl across services.
- Cons: tenants see inconsistent pack and eligibility behavior.
- Cons: incident response cannot reconstruct end-to-end integration evidence.
- Rejected because integration behavior is a substrate concern.

### Alternative 2: Use Zapier, Workato, or n8n as the runtime authority

- Pros: huge existing connector libraries.
- Pros: lower adapter authoring burden.
- Cons: tenant credential custody leaves Oyatie control.
- Cons: pack-specific residency and audit semantics become vendor-dependent.
- Cons: Foundry and Workflow cannot rely on library-first dispatch.
- Cons: per-call cost and throttling are outside Oyatie control.
- Rejected because external tools can be migration inputs, not the authority.

### Alternative 3: Webhook-only integration model

- Pros: simpler ingress shape.
- Pros: avoids OAuth broker complexity for some event sources.
- Cons: cannot support outbound connector actions.
- Cons: cannot handle SaaS APIs that require pull, write, or mutation operations.
- Cons: schema drift and DLQ replay still need a connector registry.
- Rejected because workflows require both triggers and actions.

### Alternative 4: One global OAuth client per connector

- Pros: easier connector setup for small tenants.
- Pros: fewer tenant admin steps.
- Cons: violates provider-BYOK expectations.
- Cons: noisy tenant can exhaust shared vendor quota.
- Cons: revocation and liability are harder to scope.
- Rejected because per-tenant client identity is required where vendors support it.

### Alternative 5: Synchronous-only connector invocation

- Pros: easiest API contract for callers.
- Pros: immediate success or failure response.
- Cons: vendor tail latency would leak into Workflow and Foundry.
- Cons: retry and DLQ become caller-specific.
- Cons: high-volume connectors cannot scale safely.
- Rejected because connector actions need async evidence, retry, and replay semantics.

## Consequences

### Positive

- Tenants get one catalog, one OAuth view, one webhook receiver, and one DLQ surface.
- Consumers call connector actions without owning provider-specific credential storage.
- Marketplace publishers can ship adapters under governed version and sunset rules.
- Workflow and Foundry can call connectors through the same action contract.
- Webhook spoof attempts become signed audit events.
- Schema drift can be detected once and surfaced to every affected wiring.
- Vendor outages can be isolated by connector circuit breaker.
- Pack-aware filtering avoids presenting unusable connectors to tenants.

### Negative

- The service becomes a dependency for many product and automation paths.
- OAuth callback and webhook ingress require continuous abuse-defence attention.
- Connector adapter marketplace loading adds supply-chain review burden.
- Vendor sandbox tests are expensive and sometimes flaky.
- A catalog policy mistake can hide valid connectors or expose invalid ones.
- DLQ replay correctness depends on saved context and fresh policy evaluation.
- High-cardinality connector metrics need strict label budgets.

### Neutral

- Consumer services may still implement first-party domain actions.
- External connector products may be migration sources or adapter targets.
- Internal protocol connectors can remain available in no-internet cells.
- Marketplace economics remain outside this ADR except adapter version and custody.
- Vendor payload retention follows tenant data policy, not one global retention period.

### Follow-up work

- Add top-50 vendor sandbox test matrix for OAuth and webhook flows.
- Add connector action idempotency registry.
- Add HMAC verification property tests for each signature scheme.
- Add schema drift quarantine dashboard for affected wirings.
- Add adapter digest attestation and rollback playbook.
- Add pack-specific connector eligibility export for auditors.

## Implementation Notes

### Data Shapes

- `ConnectorCatalogEntry` fields: `connector_id`, `version`, `publisher_id`, `categories`, `auth_modes`, `actions`, `triggers`, `data_classes`, `eligible_packs`, `cell_egress_class`, `deprecation_state`.
- `ConnectorWiring` fields: `wiring_id`, `tenant_id_hash`, `connector_id`, `connector_version`, `auth_grant_id`, `webhook_endpoint_id`, `mapping_version`, `rate_budget_id`, `dlq_policy_id`, `created_by`.
- `OAuthGrant` fields: `grant_id`, `tenant_id_hash`, `connector_id`, `subject_principal_id`, `scopes`, `refresh_token_ref`, `client_secret_ref`, `expires_at`, `revoked_at`.
- `WebhookEndpoint` fields: `endpoint_id`, `tenant_id_hash`, `connector_id`, `wiring_id`, `url_path`, `signing_secret_ref`, `verify_algorithm`, `active`.
- `ConnectorActionRequest` fields: `tenant_id`, `wiring_id`, `connector_id`, `action`, `input_schema_version`, `payload_ref`, `idempotency_key`, `traceparent`.
- `DLQEntry` fields: `entry_id`, `wiring_id`, `action`, `failure_reason`, `attempt_count`, `payload_ref`, `policy_version`, `mapping_version`, `created_at`, `expires_at`.
- `SchemaDriftFinding` fields: `wiring_id`, `connector_id`, `vendor_schema_version`, `detected_at`, `field_changes`, `severity`, `quarantine_state`.
- `ConnectorActionInvoked` fields: `tenant_id_hash`, `connector_id`, `action`, `status`, `vendor_request_id`, `latency_ms`, `evidence_id`.
- `DLQEntryReplayed` fields: `entry_id`, `wiring_id`, `old_policy_version`, `new_policy_version`, `result`, `evidence_id`.

### API Endpoints

- `GET /catalog/connectors` searches the tenant-filtered connector catalog.
- `GET /catalog/connectors/{connector_id}` returns trigger, action, auth, rate, data class, and sunset detail.
- `POST /oauth/grants/initiate` starts an OAuth grant with tenant-bound state.
- `GET /oauth/grants/callback` validates callback, stores token references, and emits grant evidence.
- `GET /oauth/grants/{grant_id}` reads current grant state.
- `DELETE /oauth/grants/{grant_id}` revokes a grant and emits `OAuthGrantRevoked`.
- `POST /webhook-endpoints` creates webhook URL and signing secret reference.
- `POST /webhook-endpoints/{endpoint_id}/rotate-secret` rotates webhook signing material.
- `POST /actions/invoke` dispatches a connector action with retry and rate policy.
- `GET /dlq` lists DLQ entries filtered by tenant and wiring.
- `POST /dlq/{entry_id}/replay` replays after fresh policy and schema checks.
- `GET /schema-drift/{wiring_id}` returns drift findings and quarantine state.

### Cedar Policies

- `policy/connector-authorization.cedar` authorizes connector action invocation.
- `policy/oauth-broker-authorization.cedar` authorizes grant creation, callback finalization, and revocation.
- `policy/webhook-receiver-gating.cedar` authorizes endpoint registration and receive behavior.
- `policy/payload-signature-verification.cedar` rejects unverifiable webhook payloads.
- `policy/connector-catalog-publishing.cedar` governs publisher version changes.
- `policy/no-new-runtime-scope.cedar` prevents adapter updates from silently adding scopes.
- `policy/abuse-defence.cedar` challenges abusive webhook or OAuth traffic.
- `policy/tenant-isolation.md` documents tenant isolation expectations for connector payloads.
- `policy/data-residency.md` binds payload and credential handling to tenant pack.

### SLO Targets

- `connector-availability.openslo.yaml`: connector action availability target 0.999.
- `oauth-token-health.openslo.yaml`: OAuth token fetch health target 0.995.
- `webhook-receiver-throughput.openslo.yaml`: webhook ack latency target 0.995.
- `dlq-overflow-prevention.openslo.yaml`: DLQ headroom target 0.99.
- `connector-retirement.openslo.yaml`: retirement workflow target 0.99.
- Abuse false-positive rate for legitimate vendor calls must stay <=0.1%.
- Catalog search median time-to-first-wiring target is <=5 minutes from tenant signup.

## Verification

- Unit test `connector_wiring_requires_tenant_connector_auth_and_dlq_policy`.
- Unit test `oauth_state_binds_tenant_connector_scope_and_trace`.
- Unit test `webhook_endpoint_generates_secret_reference_not_raw_secret`.
- Unit test `action_request_requires_idempotency_key`.
- Unit test `dlq_replay_rejects_schema_drift_quarantine`.
- Property test `hmac_signature_roundtrip_all_supported_algorithms`.
- Property test `oauth_callback_parser_rejects_tampered_state`.
- Fuzz test `webhook_payload_deserializer_never_panics`.
- Fuzz test `connector_adapter_manifest_rejects_unknown_runtime_scope`.
- Cedar test `connector_authorization_denies_cross_tenant_wiring`.
- Cedar test `oauth_broker_denies_scope_expansion_without_approval`.
- Cedar test `webhook_receiver_denies_unverified_signature`.
- Cedar test `catalog_filters_connector_by_pack_and_cell`.
- Cedar test `publisher_cannot_add_runtime_scope_without_review`.
- Contract test `connector-integration.yaml_paths_match_router`.
- Contract test `connector-integration-events.yaml_messages_match_event_codec`.
- Integration test `initiate_oauth_grant_stores_no_raw_tokens`.
- Integration test `oauth_callback_emits_grant_issued`.
- Integration test `webhook_spoof_returns_401_and_emits_failure_event`.
- Integration test `connector_429_respects_retry_after`.
- Integration test `connector_5xx_retries_three_times_then_dlq`.
- Integration test `dlq_replay_re_evaluates_policy_and_mapping`.
- Integration test `schema_drift_marks_affected_wiring`.
- Load test `one_million_actions_per_minute_sustained`.
- Load test `ten_million_webhooks_per_day_per_tenant_path`.
- Load test `oauth_grants_100k_concurrent_state_machine`.
- Chaos test `vendor_outage_trips_connector_circuit_breaker`.
- Chaos test `openbao_unavailable_blocks_credential_resolution`.
- Metric `oya_connector_action_total{connector,action,status}`.
- Metric `oya_connector_oauth_grant_total{connector,outcome}`.
- Metric `oya_connector_webhook_receive_total{connector,verify_outcome}`.
- Metric `oya_connector_dlq_depth{wiring_id}`.
- Metric `oya_connector_schema_drift_total{connector,severity}`.
- Dashboard `dashboards/connector-usage-by-tenant.json`.
- Dashboard `dashboards/dlq-state.json`.
- Dashboard `dashboards/webhook-receiver-throughput.json`.
- Dashboard `dashboards/oauth-token-health.md`.
- Runbook check `runbooks/oauth-callback-failure.md` covers state mismatch.
- Runbook check `runbooks/vendor-outage-dlq-surge.md` covers circuit breaker and replay.
- Promotion gate blocks if top-50 connector sandbox suite is stale.
- Promotion gate blocks if any adapter adds a new OAuth scope without policy review.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0043: Secrets management OpenBao and HSM per cell.
- Oyatie ADR-0104: Ecosystem expansion toolchain and adapters.
- Oyatie ADR-0128: Hyperscaler architecture invariants.
- RFC 2104: HMAC Keyed-Hashing for Message Authentication.
- RFC 6749: The OAuth 2.0 Authorization Framework.
- RFC 6750: OAuth 2.0 Bearer Token Usage.
- RFC 7009: OAuth 2.0 Token Revocation.
- RFC 7636: OAuth 2.0 PKCE.
- RFC 8252: OAuth 2.0 for Native Apps.
- RFC 9110: HTTP Semantics.
- CloudEvents specification.
- AsyncAPI specification.
- OpenAPI Specification.
- Dean and Barroso: The Tail at Scale.
- Zapier, Workato, n8n, and AWS EventBridge connector documentation.
- Cedar policy language documentation.
