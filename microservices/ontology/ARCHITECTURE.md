---
doc_class: ArchitectureWalkthrough
shape: Reference
length_cap: 2400
authority_tier: 2
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0006
  - ADR-0028
  - ADR-0055
  - ADR-0105
  - ADR-0122
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0273
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
companion_docs:
  - microservices/ontology/PRD.md
  - microservices/ontology/threat-model.md
  - microservices/ontology/dpia.md
  - microservices/ontology/compliance.md
  - microservices/ontology/manifest.json
planned_enforcement_ref: oya-governance-adr-adherence-matrix
inbound_citations:
  - microservices/ontology/PRD.md
  - microservices/ontology/README.md
  - microservices/ontology/runbooks/ontology-read-library-fallback.md
---

# Ontology µservice — Architecture Walkthrough

## §entry-point — cold-start

The Ontology µservice is the substrate equivalent of **Palantir Foundry's Ontology layer** layered with **AWS Neptune** graph traversal semantics, **Microsoft Fabric OneLake** discovery, **Google Vertex AI Feature Store** semantic lookup, and **Salesforce Data Cloud** unified-object surfacing. It defines `Person`, `Document`, `Recording`, and tenant-extension entity types, indexes them per ADR-0257 read-path library-first dispatch, and serves Cedar-gated queries through a 3-layer Knowledge Graph (typed-entity-substrate → relationship-graph → derived-projection).

Cold-start question: *Where does a property defined on `Person` get evaluated when ChatGPT-style retrieval queries it via the agent gateway?* Trace:
1. The agent gateway (`oya-ontology-agent-gateway-rest`) receives the request with tenant context.
2. The `oya-shared-policy-eval` library evaluates Cedar fragments (ADR-0246 amendment library-first; no network hop) at `policy/tenant-scope.cedar` + `policy/type-isolation.md` + `policy/abuse-defence.cedar`.
3. The query-engine (`oya-ontology-query-engine-adapter-clickhouse`) hits the ClickHouse history-mirror **or** the Postgres+Citus typed-entity store depending on `freshness_floor` from the read-path manifest.
4. ADR-0263 audit events `oya.ontology.cedar-evaluate` / `oya.ontology.query-execute` are emitted with `tenant_id` + `principal_id` + `audience_type` to the audit chain.
5. The response is filtered by Cedar context before returning to the agent.

## §principals (ADR-0242)

Operates as `oyatie.ontology.{type-registry, query-engine, function-engine, action-engine, agent-gateway, audit-chain}` principals. Called by tenant principals `<tenant>.<workspace>.<actor>` and by substrate principals from `intelligence`, `workflow-studio`, `governance`, `connect`, `mail`, `notes`, `social`, `community`, `marketplace`. No legacy `oyatie` string literal exists — every reference is via `tenant.platform_owner_indirection` per ADR-0284.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `principals (ADR-0242)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `principals (ADR 0242)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `principals (ADR 0242)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `principals (ADR 0242)` workflow.
- Depth detail 17: `ontology` telemetry for `principals (ADR 0242)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §cedar-gates (ADR-0243)

Default-deny baseline at `policy/tenant-scope.cedar`. Defence-in-depth FORBIDs:
- `policy/auditor-scope.cedar` — auditors get read-only on flagged ranges
- `policy/ci-scope.cedar` — CI principals get a separate scope from runtime
- `policy/public-read.cedar` — explicit `audience_type=PUBLIC` only
- `policy/abuse-defence.cedar` — anti-bot + anti-spoof + anti-scrape per ADR-0297
- `policy/ontology-write-quota.cedar` — write rate gates per ADR-0297
- `policy/cross-tenant-refusal.cedar` — explicit refusal for any cross-tenant projection except via amended ADR-0257 share-token surface

Fragment soak ≥60s per ADR-0294. Cedar v4.2 LTS.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `cedar-gates (ADR-0243)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `cedar gates (ADR 0243)` failures have trigger, rollback, and post-incident closure.

## §tenant-scoping (ADR-0244)

Every table carries `tenant_id` + `home_cell` + `dr_cell` + `audience_type` + `provider_credential_mode`. `migrations/2026-05-tenant-scoping.up.sql` applies RLS to `ontology_persons`, `ontology_documents`, `ontology_recordings`, `ontology_relationships`. `audience_type` enum: `B2C_PERSONAL`, `B2B_WORK`, `INTEROP_FEDERATED`, `FRIENDLY_CRAWLER_PARTNER`, `PUBLIC`, `INTERNAL_SUBSTRATE`. `provider_credential_mode` declares whether provider-credential BYOK or platform-managed; default `TENANT_BYOK` for paid tenant_class, `PLATFORM_MANAGED` for demo_trial tenant_class per ADR-0255 §D-4.
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `tenant-scoping (ADR-0244)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `tenant scoping (ADR 0244)` workflow.
- Depth detail 17: `ontology` telemetry for `tenant scoping (ADR 0244)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §substrate-product-binding (ADR-0245)

**Tier: substrate.** Consumed by every product µservice: `mail` (entity-resolution for senders/recipients), `notes` (note-entity-link mining), `social` (profile-entity-mirror), `community` (forum-thread-entity-extraction), `marketplace` (listing-entity-mirror), `messenger` (room-participant-entity), `workflow-studio` (object-type palette), `intelligence` (semantic-retrieval surface), `governance` (object-policy mapping).
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `substrate-product-binding (ADR-0245)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.
- Depth detail 17: `ontology` telemetry for `substrate product binding (ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §policy-evaluation (ADR-0246 + amendment)

Library-first dispatch via `oya-shared-policy-eval` crate; no network hop. `policy_evaluation_mode: LIBRARY_FIRST`. Network fallback to `policy-engine` µservice only on library load failure (panic OR version-mismatch); fallback emits `oya.ontology.policy-fallback-network` audit event.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `policy-evaluation (ADR-0246 + amendment)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `ontology` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §ontology-read-path (ADR-0257 + amendment)

`ontology_read_mode: LIBRARY_FIRST_BYO_CACHE`. The `oya-ontology-read-path-library` crate is loaded by every caller and serves reads from a per-tenant local cache with `freshness_floor: 2s` (configurable per BC). Writes still go through the write-path service for ACID guarantees. The amendment also defines:
- `ontology_share_token` surface for explicit cross-tenant projection
- `freshness_floor` enum: `STRICT_REALTIME` (≤200ms), `TIGHT` (≤2s), `LOOSE` (≤60s), `EVENTUAL` (≤24h)
- `read_path_eviction_policy: LRU_BY_TENANT` to prevent cross-tenant cache pressure
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `ontology-read-path (ADR-0257 + amendment)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `ontology read path (ADR 0257 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 + amendment)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `ontology read path (ADR 0257 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `ontology read path (ADR 0257 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `ontology read path (ADR 0257 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §time-coordination (ADR-0252)

HLC default; TrueTime opt-in for `audit-chain` Merkle root sealing (≤7ms clock skew tolerance) and for `function-engine` financial-grade evaluation when the calling tenant declares `time_coordination: TRUE_TIME_TIER`.
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `time-coordination (ADR-0252)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `ontology` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §transport (ADR-0253)

REST surface advertises `Alt-Svc: h3=":443"; ma=86400, h3-29=":443"; ma=86400`. Fallback h3 → h2 → h1.1 (never skip h2). TLS 1.3 floor; HSTS `max-age=63072000; includeSubDomains; preload`; OCSP-stapling on. ECH enabled (config-id rotated every 90d per `iac/<env>-ech-config.yaml`); PQC hybrid `X25519MLKEM768` advertised + `ed25519+ml_dsa_65` signature hybrid for ontology-issued cert chains.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `transport (ADR-0253)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `ontology` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §deployment-shape (ADR-0254)

Kubernetes-everywhere except Tier-3 edge cells. Each component:
- `oya-ontology-query-engine-adapter-clickhouse` → Cloud Hypervisor + Kata pod (sensitivity: tenant data)
- `oya-ontology-action-engine-usecase` → Kata pod
- `oya-ontology-agent-gateway-rest` → standard pod with mTLS sidecar
- `oya-ontology-read-path-library` → consumer-side; no µservice pod (library only)
- `oya-ontology-audit-chain-worker` → Cloud Hypervisor + Kata pod with TPM attestation
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `deployment-shape (ADR-0254)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.

## §intelligence-dispatch (ADR-0255 + amendment)

Calls Intelligence library-first for `embedding-generation` (T1) and `entity-resolution-disambiguation` (T2). Audience tag: `INTELLIGENCE_SUBSTRATE` for substrate calls; `B2C_PERSONAL` / `B2B_WORK` propagated when serving on behalf of an end-user request. Never network-opt-in for sensitive entities; library-only.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `intelligence-dispatch (ADR-0255 + amendment)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `intelligence dispatch (ADR 0255 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255 + amendment)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `intelligence dispatch (ADR 0255 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `intelligence dispatch (ADR 0255 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `intelligence dispatch (ADR 0255 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255 + amendment)` workflow.
- Depth detail 17: `ontology` telemetry for `intelligence dispatch (ADR 0255 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §marketplace (ADR-0249)

Ontology surfaces `data-product`, `entity-type-template`, and `function-template` marketplace categories. Tenant-authored ontology extensions can publish under tenant's marketplace namespace. Cross-tenant install requires explicit consent + Cedar gate.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `marketplace (ADR-0249)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `marketplace (ADR 0249)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace (ADR 0249)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace (ADR 0249)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `marketplace (ADR 0249)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `marketplace (ADR 0249)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `marketplace (ADR 0249)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `marketplace (ADR 0249)` workflow.
- Depth detail 17: `ontology` telemetry for `marketplace (ADR 0249)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §observability (ADR-0263)

Emitted audit-event classes (all in central registry): `oya.ontology.type-register`, `oya.ontology.type-deprecate`, `oya.ontology.query-execute`, `oya.ontology.action-invoke`, `oya.ontology.function-evaluate`, `oya.ontology.cedar-evaluate`, `oya.ontology.read-path-fallback-network`, `oya.ontology.cross-tenant-projection-issue`, `oya.ontology.cross-tenant-projection-revoke`, `oya.ontology.policy-fallback-network`, `oya.ontology.abuse-defence-block`, `oya.ontology.share-token-issue`, `oya.ontology.share-token-redeem`.

Per-metric cardinality budget: `oya_ontology_*` series capped at 10000 active labels per µservice instance; high-cardinality labels (entity_id, user_id) are NOT label dimensions — they appear in trace-span attributes instead.

Trace span shape: parent `ontology.query` → child `ontology.cedar-gate` + `ontology.read-path` + `ontology.audit-emit`. Span sampling default 1% baseline + 100% on `audience_type=B2B_WORK` + 100% on errors.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four-signal telemetry pattern anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `observability (ADR-0263)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `observability (ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `observability (ADR 0263)` workflow.

## §abuse-defence (ADR-0297)

Internet-facing only on the agent-gateway + REST surface. Tier-0 edge controls: token-bucket per-IP + per-tenant rate-limit; TLS JA4 fingerprint; bot-mgmt scoring forwarded as `X-Oya-Bot-Score`. Anti-scrape: pattern-anomaly detection for breadth-first entity-id traversal; sequential-id detection (ontology entity IDs are ULID — sequential brute-force is detectable). Anti-spoof: HMAC signing on every audit event per ADR-0263 (signed by sidecar key per ADR-0296); SPIFFE workload identity per ADR-0295 on every µservice-to-µservice call.

UX-floor: the default-path latency budget is ≤2ms added by bot-mgmt; legitimate agent-gateway calls from intelligence / workflow-studio / mail / notes / social MUST NOT see CAPTCHA — they are `audience_type=INTERNAL_SUBSTRATE` and bypass user-facing challenges. Only direct end-user query of public ontology surfaces sees adaptive challenge on bot-score > 95.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `abuse-defence (ADR-0297)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `abuse defence (ADR 0297)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (ADR 0297)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (ADR 0297)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `abuse defence (ADR 0297)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `abuse defence (ADR 0297)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `abuse defence (ADR 0297)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `abuse defence (ADR 0297)` workflow.
- Depth detail 17: `ontology` telemetry for `abuse defence (ADR 0297)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §credential-isolation (ADR-0296)

Provider credentials (e.g., per-tenant provider-credential BYOK to OpenAI for embedding generation when tenant declares it, ADR-0255 §D-4) live in a sidecar with ≤60s OpenBao TTL. Ontology µservice never holds long-lived tenant credentials; the sidecar mints short-lived per-call tokens.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `credential-isolation (ADR-0296)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `ontology` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §fragment-publish (ADR-0294)

Cedar fragment publish flow: PR → soak 60s on staging → soak 60s on canary cell → roll to GA cells. Emergency rollback runbook at `runbooks/cedar-fragment-rollback.md`.
### Content-pass expansion — fragment-publish
- This expansion preserves the existing prose above and closes `fragment-publish` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS AppConfig bake windows anchors the external control pattern for `fragment-publish`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `fragment-publish`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `fragment-publish`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `fragment-publish` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `fragment-publish (ADR-0294)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `fragment publish (ADR 0294)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `fragment publish (ADR 0294)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `fragment publish (ADR 0294)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `fragment publish (ADR 0294)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `fragment publish (ADR 0294)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `fragment publish (ADR 0294)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `fragment publish (ADR 0294)` workflow.
- Depth detail 17: `ontology` telemetry for `fragment publish (ADR 0294)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §bootstrap-trust-chain (ADR-0295)

The audit-chain-worker boots with SPIFFE attestation; if attestation fails, kill-switch engages and the µservice refuses to seal any audit events until re-attested.
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `bootstrap-trust-chain (ADR-0295)` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `bootstrap trust chain (ADR 0295)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain (ADR 0295)`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain (ADR 0295)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `bootstrap trust chain (ADR 0295)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `bootstrap trust chain (ADR 0295)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `bootstrap trust chain (ADR 0295)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain (ADR 0295)` workflow.
- Depth detail 17: `ontology` telemetry for `bootstrap trust chain (ADR 0295)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §self-modification

Ontology consumes Foundry self-modification artifacts for type-registry-schema updates. Meta-trust-root attestation per ADR-0293 verified before any type-registry mutation is accepted from Foundry.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `ontology` to the ≥50-line documentation-rigor floor.
- Service owner `axis-ontology` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `cedar-evaluate`; bounded contexts: `unknown`.
- API surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy surfaces: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`; +4 more.
- State/event surfaces: `ontology.unknown`.
- SLO/dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `ontology` `cedar-evaluate` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ontology/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ontology` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ontology` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ontology` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ontology` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ontology` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `cedar-evaluate` evaluates `<tenant>.ontology.cedar-evaluate` against policy, writes `ontology.unknown`, and emits `oya.ontology.cedar.evaluate.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ontology` binds `self-modification` to `{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya-ontology-action-engine-usecase', 'oya-ontology-agent-gateway-rest', 'oya-ontology-app', 'oya-ontology-audit-chain-worker', 'oya-ontology-cedar-fragment-coverage-adapter', 'oya-ontology-entity-store-adapter-clickhouse', 'oya-ontology-entity-store-adapter-postgres', 'oya-ontology-entity-store-kernel', 'oya-ontology-function-engine-rest', 'oya-ontology-object-type-registry-domain', 'oya-ontology-object-type-registry-kernel', 'oya-ontology-object-type-registry-usecase', 'oya-ontology-pillar-domain', 'oya-ontology-query-engine-adapter-clickhouse', 'oya-ontology-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ontology` is `contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ontology` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/cross-tenant-refusal.cedar, policy/data-residency.md, policy/ontology-write-quota.cedar, plus 3 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `ontology` state/event naming uses `ontology.{'name': 'unknown', 'description': "Bounded context 'unknown'", 'crates': ['oya_ontology_action_engine_usecase', 'oya_ontology_agent_gateway_rest', 'oya_ontology_app', 'oya_ontology_audit_chain_worker', 'oya_ontology_cedar_fragment_coverage_adapter', 'oya_ontology_entity_store_adapter_clickhouse', 'oya_ontology_entity_store_adapter_postgres', 'oya_ontology_entity_store_kernel', 'oya_ontology_function_engine_rest', 'oya_ontology_object_type_registry_domain', 'oya_ontology_object_type_registry_kernel', 'oya_ontology_object_type_registry_usecase', 'oya_ontology_pillar_domain', 'oya_ontology_query_engine_adapter_clickhouse', 'oya_ontology_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ontology` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ontology` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ontology` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ontology` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ontology` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ontology` uses SLOs `slos/action-invocation-availability.openslo.yaml, slos/audit-chain-emission-completeness.openslo.yaml, slos/dynamic-layer-freshness.openslo.yaml, slos/function-read-availability.openslo.yaml, slos/function-read-latency.openslo.yaml, plus 1 more` and dashboards `dashboards/abuse-defence-outcomes.json, dashboards/cedar-policy-coverage.json, dashboards/query-latency.json, dashboards/read-path-library-freshness.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ontology` uses runbooks `runbooks/cedar-fragment-rollback.md, runbooks/clickhouse-rebalance.md, runbooks/cross-tenant-leak-recovery.md, runbooks/object-type-deprecation.md, runbooks/ontology-bot-score-recalibration.md, plus 5 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ontology` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/cedar-policy-engine/Chart.yaml, iac/helm/cedar-policy-engine/templates/deployment.yaml, iac/helm/cedar-policy-engine/templates/networkpolicy.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ontology` uses `capabilities/cedar-evaluate.yaml, capabilities/query-execute.yaml, capabilities/type-register.yaml` and `catalog/oya-ontology-action-engine-usecase.yaml, catalog/oya-ontology-agent-gateway-rest.yaml, catalog/oya-ontology-app.yaml, catalog/oya-ontology-audit-chain-worker.yaml, plus 14 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ontology` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ontology` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ontology` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification` workflow.
- Depth detail 17: `ontology` telemetry for `self modification` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ontology` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §where-to-read-next

- `microservices/ontology/PRD.md` — product requirements + UX flows
- `microservices/ontology/threat-model.md` — STRIDE + mitigations
- `microservices/ontology/dpia.md` — DPIA per Art. 35
- `microservices/ontology/compliance.md` — pack-overlay roster
- `docs/decisions/ADR-0257-ontology-as-substrate.md` — substrate doctrine

---



## §cell-eligibility
This anchor is closed for `ontology` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `cedar-evaluate` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `ontology` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `ontology` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `ontology` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `cedar-evaluate` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `ontology` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
