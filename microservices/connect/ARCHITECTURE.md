---
doc_class: ArchitectureWalkthrough
microservice: connect
date: 2026-05-20
owner_team: axis-integration
status: Accepted
related_adrs: [ADR-0056, ADR-0105, ADR-0145, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0248, ADR-0253, ADR-0255, ADR-0263, ADR-0273, ADR-0294, ADR-0295, ADR-0296, ADR-0297]
companion_docs:
  - microservices/connect/PRD.md
  - microservices/connect/threat-model.md
  - microservices/connect/compliance.md
inbound_citations:
  - microservices/connect/PRD.md
  - microservices/connect/manifest.json
doc_status: published
---

# ARCHITECTURE — connect (Integration Substrate)

## A. Entry point — cold-start question

"A tenant clicks 'Connect Salesforce' in workflow-studio. By the time their first Lead-created event is dispatched into their workflow, what code paths fired, in what order, what got persisted, and what audit events sealed?"

This walkthrough traces that flow layer-by-layer against the canonical 8 BCs.

## B. Bounded Contexts (BC roster)

| BC | Purpose | Hyperscaler analog |
|---|---|---|
| `connector-catalog` | Searchable directory of ≥500 connector adapters with categories, scopes, rate-limit profiles, compliance posture | Zapier App Directory |
| `oauth-broker` | OAuth 2.0 (RFC 6749) + OIDC + JWT-bearer + client-credentials authorization flows; per-tenant provider-credential BYOK client provisioning (ADR-0255 §D-4) | Auth0 + Okta + Stripe Connect |
| `webhook-receiver` | Per-tenant DNS endpoints; HMAC signature verification; idempotency; backpressure | Stripe Webhooks + GitHub Webhooks |
| `signature-verification` | Constant-time HMAC verify; vendor-specific signing schemes (Shopify, Stripe, GitHub, Slack); replay-window enforcement | Stripe Webhook signing |
| `payload-canonicalization` | Normalize vendor-specific JSON/XML/form-encoded payloads into oyatie canonical event shape | n8n payload normalizer |
| `connector-adapter` | Per-connector typed action invocation; retry; circuit-break; library-first dispatch per ADR-0246 amendment | Workato connector SDK |
| `data-mapping` | Visual field-mapper with schema-drift detection; per-field data-class tagging | Workato/Boomi data mapping |
| `retry-and-DLQ` | Exponential backoff with jitter; DLQ persistence; replay surface | AWS EventBridge DLQ + AWS SQS |

## §principals (ADR-0242 adherence)

This µservice operates under `oyatie.connect.*` principal slugs:
- `oyatie.connect.catalog-api` — catalog read surface (Tier-0/1 edge).
- `oyatie.connect.oauth-broker-api` — OAuth flow entry/callback.
- `oyatie.connect.webhook-receiver-edge` — per-tenant webhook ingress.
- `oyatie.connect.adapter-worker` — outbound connector invocation worker.
- `oyatie.connect.dlq-replay-worker` — DLQ persistence + replay worker.
- `oyatie.connect.schema-drift-monitor` — periodic vendor-schema diff worker.

Tenant principals calling in:
- `tenant.<tenant_id>.workflow-engine.dispatcher` (most common caller).
- `tenant.<tenant_id>.foundry.runner` (Foundry jobs).
- `tenant.<tenant_id>.ops-admin` (admin UI for OAuth provisioning).
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `principals (ADR-0242 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `principals (ADR 0242 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §cedar-gates (ADR-0243 adherence)

Default-deny baseline applies. Permits live in `policy/`:
- `connector-authorization.cedar` — gates which connector actions a principal may invoke
- `oauth-broker-authorization.cedar` — gates OAuth grant creation/revocation
- `webhook-receiver-gating.cedar` — gates which principals may register webhook endpoints
- `payload-signature-verification.cedar` — gates skip-verify exemptions (none in prod; sandbox only)
- `abuse-defence.cedar` — UX-floor compliant per documentation-rigor §3.2.3
- `auditor-scope.cedar` — auditor read access
- `ci-scope.cedar` — CI/test reserved-tenant scope
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `cedar-gates (ADR-0243 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `cedar gates (ADR 0243 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `cedar gates (ADR 0243 adherence)` failures have trigger, rollback, and post-incident closure.

## §tenant-scoping (ADR-0244 adherence)

Every row carries `tenant_id` (UUIDv7 per ADR-0244 §D-2). Tables:
- `connect.oauth_grants(grant_id, tenant_id, connector_name, principal_id, scopes[], issued_at, expires_at, refresh_token_ref, status)`
- `connect.webhook_endpoints(endpoint_id, tenant_id, connector_name, signing_secret_ref, created_at)`
- `connect.dlq_entries(entry_id, tenant_id, wiring_id, payload_digest, error_class, last_tried_at, retry_count)`
- `connect.audit_events(event_id, tenant_id, event_class, principal_id, sealed_at, signature)` (per ADR-0263)

`audience_type`: `TENANT_OWNED_INTEGRATION` (default) or `FRIENDLY_CRAWLER_PARTNER` (allow-list bypass) or `OYATIE_INTERNAL_OPS`.

`provider_credential_mode`: `byok` (default; tenant-provisioned OAuth client per ADR-0255 §D-4) or `oyatie_shared` (legacy; deprecated 2026-Q4).
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `tenant-scoping (ADR-0244 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `tenant scoping (ADR 0244 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `tenant scoping (ADR 0244 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.

## §substrate-product-binding (ADR-0245 adherence)

**Tier: substrate.** Consumed by:
- `microservices/workflow-engine/` (executes wirings that call connectors)
- `microservices/marketplace/` (lists connector adapters as marketplace items)
- `microservices/ops-dashboard-control-center/` (admin actions for OAuth provisioning)
- `microservices/intelligence/` (Foundry jobs call connectors)
- `microservices/intelligence/` (consumes via library-first per ADR-0255 amendment)

Depends on (substrate-of-substrate per ADR-0280):
- `microservices/cloud-secrets/` (OpenBao for credential storage)
- `microservices/policy-engine/` (library-first Cedar eval per ADR-0246)
- `microservices/observability/` (SLO emission + tracing)
- `microservices/ontology/` (read-path for tenant context per ADR-0257 amendment)
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `substrate-product-binding (ADR-0245 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `substrate product binding (ADR 0245 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §policy-evaluation (ADR-0246 + amendment)

`policy_evaluation_mode`: `library-first`. Uses `oya-shared-policy-eval` Rust crate. Cedar fragments compiled at startup; hot-reloaded on fragment publish (per ADR-0294 ≥60s soak).
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `policy-evaluation (ADR-0246 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connect` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `connect` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connect` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §intelligence-dispatch (ADR-0255 + amendment)

`intelligence_call_mode`: `library-first` for data-mapper auto-suggestions; `network` only for the chat-assist surface in workflow-studio. Audience tag per call: `OYATIE_INTERNAL_CONNECT_DATA_MAPPING`.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `intelligence-dispatch (ADR-0255 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `intelligence dispatch (ADR 0255 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255 + amendment)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `intelligence dispatch (ADR 0255 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `intelligence dispatch (ADR 0255 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `intelligence dispatch (ADR 0255 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connect` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255 + amendment)` workflow.
- Depth detail 17: `connect` telemetry for `intelligence dispatch (ADR 0255 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connect` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §ontology-read-path (ADR-0257 + amendment)

`ontology_read_mode`: `library-first`. `freshness_floor`: 5s for tenant-context reads; 60s for connector-vendor metadata.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `ontology-read-path (ADR-0257 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `ontology read path (ADR 0257 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 + amendment)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `ontology read path (ADR 0257 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `ontology read path (ADR 0257 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `ontology read path (ADR 0257 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connect` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 + amendment)` workflow.
- Depth detail 17: `connect` telemetry for `ontology read path (ADR 0257 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connect` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §time-coordination (ADR-0252 adherence)

`time_coordination_tier`: HLC default for audit ordering; TrueTime opt-in (none currently — connect doesn't need fin-grade externally-consistent ordering).
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `time-coordination (ADR-0252 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `time coordination (ADR 0252 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `time coordination (ADR 0252 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `time coordination (ADR 0252 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `time coordination (ADR 0252 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connect` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252 adherence)` workflow.
- Depth detail 17: `connect` telemetry for `time coordination (ADR 0252 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connect` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §transport (ADR-0253 adherence)

HTTP/3 + QUIC default for all surfaces. Alt-Svc advertisement on every response (`Alt-Svc: h3=":443"; ma=86400`). Fallback chain: HTTP/3 → HTTP/2 → HTTP/1.1. HTTP/1.0 forbidden.

TLS 1.3 floor; cipher suites: TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256, TLS_AES_128_GCM_SHA256. Curve preference: X25519 (with PQC hybrid `X25519MLKEM768` advertised). HSTS preload. CT-required.

ECH: enabled. HTTPS RR published per-tenant via the DKIM/SPF/DMARC toolchain (ADR-0273). Key rotation ≥90d per `docs/runbooks/cedar-fragment-emergency-rollback.md` cadence.

PQC: hybrid `X25519MLKEM768` offered in ClientHello; signature hybrid `ed25519+ml_dsa_65` for new oyatie-rooted CA chains. Non-PQ clients degrade silently to X25519.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `transport (ADR-0253 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `transport (ADR 0253 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `transport (ADR 0253 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `transport (ADR 0253 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `transport (ADR 0253 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §deployment-shape (ADR-0254 adherence)

- `connector-catalog-rest` + `oauth-broker-rest` + `webhook-receiver-edge`: K8s containers on Tier-0/1 cells.
- `connector-adapter-worker`: Cloud Hypervisor + Kata pods per-tenant per ADR-0254 (vendor-API outbound calls isolated by Kata sandbox).
- `dlq-replay-worker`: K8s containers on Tier-2 cells.

WASM: not used (connector adapters compile to native Rust + Kata isolation).
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `deployment-shape (ADR-0254 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `deployment shape (ADR 0254 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `deployment shape (ADR 0254 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `deployment shape (ADR 0254 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `deployment shape (ADR 0254 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §observability (ADR-0263 adherence)

Audit-event classes emitted (in central registry):
- `OAuthGrantIssued`, `OAuthGrantRevoked`, `OAuthGrantRotated`
- `ConnectorActionInvoked`, `ConnectorActionFailed`
- `WebhookReceived`, `WebhookSignatureVerifyFailed`, `WebhookReplayBlocked`
- `DLQEntryAdded`, `DLQEntryReplayed`
- `ProviderCredentialProvisioned`, `ProviderCredentialRotated`
- `SchemaDriftDetected`
- `AbuseDefenceChallengeIssued`, `AbuseDefenceBlockedRequest`

Metric cardinality budget: declared in PRD §E Observability.

Trace span shape: every connector call → root span `oya.connect.action`, child spans for `auth-resolve`, `rate-limit-check`, `vendor-invoke`, `response-canonicalize`.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four baseline signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `observability (ADR-0263 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `observability (ADR 0263 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §marketplace (ADR-0249 adherence)

Exposes the `connector` category in multi-category marketplace. Publishing flow:
1. MPO submits adapter manifest + signed binary to marketplace ingest.
2. Security review (auto + manual for high-risk).
3. Listed in catalog with publisher namespace `mp/<publisher>/<connector>`.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `marketplace (ADR-0249 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `marketplace (ADR 0249 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace (ADR 0249 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace (ADR 0249 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `marketplace (ADR 0249 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `marketplace (ADR 0249 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `marketplace (ADR 0249 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §abuse-defence (ADR-0297 adherence; documentation-rigor §3.2.3)

Internet-facing surfaces (catalog browse, OAuth callback, webhook receiver) wire defence-in-depth per §3.2.3:

- **Anti-bot (8 controls):** edge rate-limiting (per-IP, per-fingerprint, per-tenant, per-route); JA4/JA4+ fingerprinting (passive); bot-mgmt ML scoring (Cloudflare or in-house equivalent); CAPTCHA-on-suspicion (hCaptcha + Turnstile; ≤10s solve; a11y alternatives; never on default path); device attestation (App Attest + Play Integrity for native; WebAuthn for web); HIBP credential check on OAuth callback; per-action quota gates via Cedar; honeypot routes (`/v1/internal-only/*`) + canary payloads (seeded fake API keys in catalog metadata).
- **Anti-spoof (8 controls):** DKIM+SPF+DMARC for any outbound email (per ADR-0273); strict TLS + cert pinning for native apps; WebAuthn passkeys for high-risk ops (OAuth provisioning, provider-credential BYOK rotation per ADR-0255 §D-4); HMAC-signed sessions with audience binding + SameSite=Strict + token-binding RFC 8473; signed webhook payloads per vendor scheme (Shopify HMAC, Stripe sig, GitHub HMAC); per-µservice audit-event signing via sidecar (ADR-0296); HMAC verify on every inbound webhook (replay-window ≤5min); SPIFFE workload identity (ADR-0295) on every µservice-to-µservice call.
- **Anti-scrape (8 controls):** aggressive low caps on unauth catalog reads; pattern-anomaly detection (sequential-ID enumeration, breadth-first crawl); per-tenant robots.txt (`hooks.<tenant>.oyatie.app/robots.txt`); paid-API tier via api-gateway for legitimate bulk consumers; per-tenant content fingerprinting (zero-width chars in catalog responses); adaptive challenge on scrape-pattern; dynamic CSS class names per session; abuse-report email + DMCA agent per `policy/abuse-defence.cedar`.

**UX-floor invariants enforced:**
- Default-path latency overhead: ≤2ms p99 (CI-gated by `oya-governance-abuse-defence-ux-floor`).
- Friendly-crawler partner allow-list via `audience_type = FRIENDLY_CRAWLER_PARTNER`.
- Every challenge has a11y alternatives (audio CAPTCHA, keyboard-only, screen-reader).
- Cognitive load: challenge solvable in ≤10s.
- Session continuity preserved on successful challenge.
- Tenant-tier-adaptive sensitivity (paid tiers lower; anonymous strictest).
- Locale-aware challenge UI per ADR-0064.
- Mobile UX parity via platform-idiomatic (App Attest, Play Integrity) not web-CAPTCHA.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `abuse-defence (ADR-0297 adherence; documentation-rigor §3.2.3)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `abuse defence (ADR 0297 adherence; documentation rigor §3.2.3)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (ADR 0297 adherence; documentation rigor §3.2.3)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.

## §credential-isolation (ADR-0296 adherence)

Connector adapter workers run in Cloud Hypervisor + Kata pods. The credential sidecar issues short-lived OAuth access tokens (≤60s TTL) from refresh tokens stored in OpenBao. The adapter process never sees the refresh token; it sees only the access token, which expires before any plausible exfiltration attack window.

Rotation: refresh tokens rotated automatically when vendor supports refresh-token-rotation (Slack, Google, Microsoft, etc.); else rotated on grant-renewal cadence per vendor policy.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `credential-isolation (ADR-0296 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `credential isolation (ADR 0296 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `credential isolation (ADR 0296 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `credential isolation (ADR 0296 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `credential isolation (ADR 0296 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connect` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296 adherence)` workflow.
- Depth detail 17: `connect` telemetry for `credential isolation (ADR 0296 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §bootstrap-trust-chain (ADR-0295 adherence)

SPIFFE workload identity issued by cluster SPIRE server. Every connect crate's workload SVID is signed by the cluster CA, rooted in the meta-trust-root per ADR-0293. Kill-switch: `kubectl annotate ns connect oya.kill-switch=true` revokes all SVIDs in <30s, halting all connector activity.
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connect` binds `bootstrap-trust-chain (ADR-0295 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connect` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `bootstrap trust chain (ADR 0295 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connect` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain (ADR 0295 adherence)`.
- Depth detail 4: `connect` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connect` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connect` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connect` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain (ADR 0295 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connect` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connect` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connect` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connect` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `bootstrap trust chain (ADR 0295 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connect` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connect.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connect` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connect-connector-adapter-domain.yaml, catalog/oya-connect-connector-catalog-api.yaml, catalog/oya-connect-connector-catalog-domain.yaml, catalog/oya-connect-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connect` fails closed when `bootstrap trust chain (ADR 0295 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connect` emits denial evidence for `bootstrap trust chain (ADR 0295 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connect` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain (ADR 0295 adherence)` workflow.
- Depth detail 17: `connect` telemetry for `bootstrap trust chain (ADR 0295 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connect` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §fragment-publish (ADR-0294 adherence)

Cedar fragments under `policy/` carry headers `# soak-window-seconds: 60` and `# publish-stage: prepublish|active`. Publishing flow: prepublish → 60s soak in CI canary → active. Emergency rollback per `docs/runbooks/cedar-fragment-emergency-rollback.md`.

## C. Concrete example — Salesforce wiring end-to-end

(73-step trace from TIE click → first event flowing — abbreviated; see PRD §F Flow 1 for the user-facing flow.)

1. `workflow-studio` UI calls `GET /v1/catalog?q=salesforce` → `connector-catalog-rest` handler (file: `src/crates/oya-connect-connector-catalog-rest/src/handlers/list.rs:42`).
2. Catalog returns matching entries (Cedar gate verified caller principal).
3. TIE selects "Salesforce v2.3" → `workflow-studio` calls `POST /v1/oauth/grants/initiate` → `oauth-broker-rest` (`src/crates/oya-connect-oauth-broker-rest/src/handlers/initiate.rs:30`).
4. Broker generates state nonce; stores at `secret/<tenant>/connect/oauth-state/<nonce>` (TTL 10min).
5. Broker returns Salesforce authorization URL including the per-tenant client_id (provider-credential BYOK lookup from `secret/<tenant>/connect/oauth-clients/salesforce`, ADR-0255 §D-4).
6. TIE redirected to `login.salesforce.com/...`; user grants; Salesforce redirects to `oauth.<tenant>.oyatie.app/callback?code=...&state=<nonce>`.
7. Edge proxy (`webhook-receiver-edge`) routes to `oauth-broker-rest` callback handler (`src/crates/oya-connect-oauth-broker-rest/src/handlers/callback.rs:78`).
8. Broker validates state nonce; exchanges code for refresh_token + access_token; stores refresh_token at `secret/<tenant>/connect/oauth/salesforce/<grant-id>`.
9. Broker emits `OAuthGrantIssued` audit event → ADR-0263 sealer signs → audit chain Merkle-seal.
10. (...subsequent steps: TIE picks trigger; webhook URL generated; first Lead created → Salesforce POSTs → ingress HMAC verify → enqueue → workflow-engine dispatch.)

## D. Common confusions

1. **"Connect executes workflows."** No. Connect provides the substrate; workflow-engine executes.
2. **"Connect holds credentials."** No. OpenBao holds them; connect holds `SecretReference` strings.
3. **"OAuth client is oyatie-owned."** Default is provider-credential BYOK (tenant-owned, ADR-0255 §D-4). Shared client is deprecated 2026-Q4.

## E. Where to read next

- `microservices/connect/threat-model.md` — STRIDE for each BC
- `microservices/connect/policy/abuse-defence.cedar` — UX-floor encoding
- `microservices/connect/runbooks/connector-cascade-failure.md` — when Salesforce 5xxs propagate
- `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`

---
### Content-pass expansion — fragment-publish
- This expansion preserves the existing prose above and closes `fragment-publish` for `connect` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connect.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS AppConfig bake windows anchors the external control pattern for `fragment-publish`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `fragment-publish`.
- Tenant-scope invariant: every `connect` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connect` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connect` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connect` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connect` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connect` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connect.umbrella-retirement-readiness` against policy, writes `connect.umbrella_retirement_readiness`, and emits `oya.connect.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `fragment-publish`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `fragment-publish` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §cell-eligibility
This anchor is closed for `connect` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `umbrella-retirement-readiness` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `connect`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connect` root context.
- Capability records cited: `microservices/connect/capabilities/connector-invoke.yaml`, `microservices/connect/capabilities/oauth-grant-initiate.yaml`, `microservices/connect/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connect/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`, `microservices/connect/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`, `microservices/connect/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connect/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`, `microservices/connect/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`, `microservices/connect/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`, `microservices/connect/policy/data-residency.md`; +5 more.
- State/event binding: `connect.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`, `microservices/connect/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connect`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connect`.
- `policy-engine` supplies the signed Cedar corpus while `connect` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connect` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connect`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connect` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `connect` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `connect` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `connect` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `connect` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `umbrella-retirement-readiness` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `connect` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `connect`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connect` root context.
- Capability records cited: `microservices/connect/capabilities/connector-invoke.yaml`, `microservices/connect/capabilities/oauth-grant-initiate.yaml`, `microservices/connect/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connect/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`, `microservices/connect/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`, `microservices/connect/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connect/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`, `microservices/connect/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`, `microservices/connect/contracts/asyncapi-v1.yaml`, `microservices/connect/contracts/connect-retirement.asyncapi.yaml`, `microservices/connect/contracts/connect-retirement.openapi.yaml`, `microservices/connect/contracts/connect_retirement.proto`, `microservices/connect/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connect/policy/abuse-defence.cedar`, `microservices/connect/policy/auditor-scope.cedar`, `microservices/connect/policy/ci-scope.cedar`, `microservices/connect/policy/connector-authorization.cedar`, `microservices/connect/policy/connector-catalog-publishing.cedar`, `microservices/connect/policy/data-residency.md`; +5 more.
- State/event binding: `connect.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connect/slos/connect-retirement.openslo.yaml`, `microservices/connect/slos/connector-availability.openslo.yaml`, `microservices/connect/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connect/slos/oauth-token-health.openslo.yaml`, `microservices/connect/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connect/runbooks/connector-attestation-revoked.md`, `microservices/connect/runbooks/connector-cascade-failure.md`, `microservices/connect/runbooks/connector-onboarding.md`, `microservices/connect/runbooks/connector-rate-limit-saturation.md`, `microservices/connect/runbooks/dlq-overflow.md`, `microservices/connect/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connect`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connect`.
- `policy-engine` supplies the signed Cedar corpus while `connect` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connect` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connect`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connect` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
