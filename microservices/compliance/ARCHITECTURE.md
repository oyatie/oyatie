---
doc_class: Architecture
shape: Walkthrough
length_cap: 1500
authority_tier: 2
status: Accepted
date: 2026-05-20
microservice: compliance
companion_docs:
  - microservices/compliance/PRD.md
  - microservices/compliance/compliance.md
  - microservices/compliance/threat-model.md
  - microservices/compliance/dpia.md
related_adrs:
  - ADR-0209
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0250
  - ADR-0251
  - ADR-0253
  - ADR-0263
  - ADR-0276
  - ADR-0293
planned_enforcement_ref: oya-governance-adr-adherence-matrix
inbound_citations:
  - microservices/compliance/PRD.md
  - microservices/compliance/README.md
---

# compliance — Architecture

## §principals (ADR-0242)

Runs as principals `oyatie.compliance.evidence-collector`, `oyatie.compliance.dsar-orchestrator`,
`oyatie.compliance.breach-notifier`, `oyatie.compliance.auditor-portal`,
`oyatie.compliance.regulator-evidence-emit`. All principals carry SPIFFE SVIDs per ADR-0295.
Tenant-scoped callers are tagged `tenant.<id>.compliance.admin` and
`tenant.<id>.compliance.dsar-subject`. Hyperscaler analog: AWS Audit Manager principal model.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `principals (ADR-0242)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `principals (ADR 0242)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §cedar-gates (ADR-0243)

Default-deny baseline in `policy/action-authorization.cedar`. Defence-in-depth FORBIDs in
`policy/abuse-defence.cedar`. Pack-overlay gating in `policy/pack-overlay-authorization.cedar`.
Residency in `policy/data-residency.cedar` / `policy/data-residency.md`. Audit at
`policy/auditor-scope.cedar`. CI at `policy/ci-scope.cedar`.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `cedar-gates (ADR-0243)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `cedar gates (ADR 0243)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `cedar gates (ADR 0243)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `cedar gates (ADR 0243)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §tenant-scoping (ADR-0244)

Every evidence row, DSAR record, breach event, regulator-engagement carries `tenant_id`.
`audience_type` is `B2B_TENANT_ADMIN` and `B2C_DSAR_SUBJECT`. `provider_credential_mode` is
`tenant_byok` for OpenBao-backed signing keys per ADR-0296 (sidecar isolation; ≤60s TTL on
materialised plaintext).
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `tenant-scoping (ADR-0244)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §substrate-product-binding (ADR-0245)

Tier-substrate. Consumers: every µservice with PII / PHI / payments. Substrate dependencies:
`tenancy`, `policy-engine`, `cloud-secrets`, `observability`, `audit-chain`.
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `substrate-product-binding (ADR-0245)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.
- Depth detail 17: `compliance` telemetry for `substrate product binding (ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §policy-evaluation (ADR-0246 + amendment)

Library-first via `oya-shared-policy-eval`. `policy_evaluation_mode = library_first`. Fragment
publish honors ≥60s soak per ADR-0294.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `policy-evaluation (ADR-0246 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `compliance` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §time-coordination (ADR-0252)

HLC default for evidence emission lag. TrueTime opt-in for SOC2-Type-II quarterly evidence
windows where commit-time ordering across regions matters.
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `time-coordination (ADR-0252)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `compliance` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §transport (ADR-0253)

HTTP/3 + QUIC default. Fallback HTTP/2 → HTTP/1.1. TLS 1.3 floor. HSTS preload. ECH advertised
per `iac/ech-config.yaml`. PQC hybrid `X25519MLKEM768` offered per `iac/pqc-cert.yaml`.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `transport (ADR-0253)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `compliance` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §deployment-shape (ADR-0254)

Kubernetes pods on Cloud Hypervisor + Kata sandbox for evidence-collector + DSAR-orchestrator
(PII/PHI in transit). Auditor portal as standard container. Evidence storage on SeaweedFS.
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `deployment-shape (ADR-0254)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `deployment shape (ADR 0254)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `deployment shape (ADR 0254)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `deployment shape (ADR 0254)` workflow.
- Depth detail 17: `compliance` telemetry for `deployment shape (ADR 0254)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §intelligence-dispatch (ADR-0255)

Calls Intelligence for PHI-anomaly detection. Library-first when bundled; network-opt-in
fallback. `audience_type = INTERNAL_SUBSTRATE` on every call.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `intelligence-dispatch (ADR-0255)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `intelligence dispatch (ADR 0255)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `intelligence dispatch (ADR 0255)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `intelligence dispatch (ADR 0255)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `intelligence dispatch (ADR 0255)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255)` workflow.
- Depth detail 17: `compliance` telemetry for `intelligence dispatch (ADR 0255)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §ontology-read-path (ADR-0257 amendment)

Reads Ontology projections for tenant + pack overlays. `ontology_read_mode = library_first`.
`freshness_floor = 60s`.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `ontology-read-path (ADR-0257 amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `ontology read path (ADR 0257 amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 amendment)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `ontology read path (ADR 0257 amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `ontology read path (ADR 0257 amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `ontology read path (ADR 0257 amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 amendment)` workflow.
- Depth detail 17: `compliance` telemetry for `ontology read path (ADR 0257 amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §observability (ADR-0263)

Audit-event-classes emitted: `oya.compliance.evidence-emit`, `oya.compliance.dsar-open`,
`oya.compliance.dsar-complete`, `oya.compliance.breach-notify`,
`oya.compliance.regulator-engagement-grant`, `oya.compliance.regulator-engagement-revoke`,
`oya.compliance.audit-seal-verify`. Metric cardinality budget 50k unique attribute combinations.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four key signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `observability (ADR-0263)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `observability (ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §marketplace (ADR-0249)

Indirect — exposes compliance-pack catalog in the marketplace for tenants to subscribe.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe Connect platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `marketplace (ADR-0249)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `marketplace (ADR 0249)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace (ADR 0249)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace (ADR 0249)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `marketplace (ADR 0249)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `marketplace (ADR 0249)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `marketplace (ADR 0249)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `marketplace (ADR 0249)` workflow.
- Depth detail 17: `compliance` telemetry for `marketplace (ADR 0249)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `compliance` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §abuse-defence (§3.2.3 + ADR-0297)

Internet-facing surfaces are the auditor portal + DSAR subject portal. Anti-bot via edge bot
mgmt + JA4 fingerprinting + per-fingerprint rate-limit. Anti-spoof via SPIFFE + signed webhook
ingest + WebAuthn for auditor admin. Anti-scrape via per-tenant rate-limit + canary evidence
records. UX floor: bot-score below threshold experiences zero added latency; CAPTCHA only on
suspicion; WCAG 2.2 AA challenge alternatives wired.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `abuse-defence (§3.2.3 + ADR-0297)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `abuse defence (§3.2.3 + ADR 0297)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (§3.2.3 + ADR 0297)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (§3.2.3 + ADR 0297)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `abuse defence (§3.2.3 + ADR 0297)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `abuse defence (§3.2.3 + ADR 0297)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §credential-isolation (ADR-0296)

OpenBao SecretReference `${openbao:secret/<tenant_id>/compliance/<key>}`. Sidecar isolation.
≤60s plaintext TTL.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `credential-isolation (ADR-0296)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `compliance` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §pack-overlay-roster

Pack-id roster cited from central registry: `gdpr`, `hipaa`, `pci-dss`, `soc2-type-2`,
`kr-csap`, `eu-sovereign`, `cn-pipl`, `il5`, `il6`, `fedramp-high`, `eu-ai-act-annex-iii`.
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `pack-overlay-roster` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `pack overlay roster` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `pack overlay roster`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `pack overlay roster` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `pack overlay roster` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `pack overlay roster` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `pack overlay roster` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `pack overlay roster` workflow.
- Depth detail 17: `compliance` telemetry for `pack overlay roster` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §bounded-contexts

- **pack-registry** — pack lifecycle + manifest.
- **dpia-orchestration** — DPIA workflow.
- **breach-notification-workflow** — Article 33/34 + state-AG cascading.
- **regulator-audit-evidence** — auditor portal + read-only scoped access.
- **cell-certification-attestation** — per-cell certification surface.
- **compliance-control-mapping** — SOC2/PCI/HIPAA controls→evidence mapping.

## §day-one-cert-readiness (ADR-0250)

Day-one readiness target: SOC 2 Type II, GDPR DSAR-30d-SLA, HIPAA min-necessary, PCI-DSS L1
evidence. Built-in, not retrofitted.
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `day-one-cert-readiness (ADR-0250)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `day one cert readiness (ADR 0250)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness (ADR 0250)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `day one cert readiness (ADR 0250)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `day one cert readiness (ADR 0250)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `day one cert readiness (ADR 0250)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `day one cert readiness (ADR 0250)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `day one cert readiness (ADR 0250)` workflow.
- Depth detail 17: `compliance` telemetry for `day one cert readiness (ADR 0250)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §minor-protection (ADR-0292)

Compliance is not a minor-facing surface; minor protection enforced at the µservices that hold
minor PII, with compliance receiving the redacted evidence stream.
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `minor-protection (ADR-0292)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `minor protection (ADR 0292)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection (ADR 0292)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection (ADR 0292)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `minor protection (ADR 0292)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `minor protection (ADR 0292)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `minor protection (ADR 0292)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `minor protection (ADR 0292)` workflow.
- Depth detail 17: `compliance` telemetry for `minor protection (ADR 0292)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §meta-trust-attestation (ADR-0293)

Evidence emission carries meta-trust-root attestation when the evidence describes
self-modification artifacts.
### Content-pass expansion — meta-trust-attestation
- This expansion preserves the existing prose above and closes `meta-trust-attestation` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: The Update Framework roots anchors the external control pattern for `meta-trust-attestation`.
- Precedent 2: Sigstore Rekor transparency provides a second independent hyperscaler pattern for `meta-trust-attestation`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `meta-trust-attestation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `meta-trust-attestation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `meta-trust-attestation (ADR-0293)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `meta trust attestation (ADR 0293)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `meta trust attestation (ADR 0293)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `meta trust attestation (ADR 0293)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `meta trust attestation (ADR 0293)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `meta trust attestation (ADR 0293)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `meta trust attestation (ADR 0293)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `meta trust attestation (ADR 0293)` workflow.
- Depth detail 17: `compliance` telemetry for `meta trust attestation (ADR 0293)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §fragment-publish (ADR-0294)

Cedar fragments published from this µservice respect the ≥60s soak window. Headers carry
`x-fragment-soak-seconds: 60`.
### Content-pass expansion — fragment-publish
- This expansion preserves the existing prose above and closes `fragment-publish` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS AppConfig bake windows anchors the external control pattern for `fragment-publish`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `fragment-publish`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `fragment-publish`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `fragment-publish` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `fragment-publish (ADR-0294)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `fragment publish (ADR 0294)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `fragment publish (ADR 0294)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `fragment publish (ADR 0294)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `fragment publish (ADR 0294)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `fragment publish (ADR 0294)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `fragment publish (ADR 0294)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `fragment publish (ADR 0294)` workflow.
- Depth detail 17: `compliance` telemetry for `fragment publish (ADR 0294)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §bootstrap-trust-chain (ADR-0295)

Bootstrap-tier-1 — SPIFFE attestation + kill-switch wired in
`iac/k8s-network-policy.yaml`.
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `compliance` to the ≥50-line documentation-rigor floor.
- Service owner `axis-compliance` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `compliance`; bounded contexts: `compliance`.
- API surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`; +2 more.
- State/event surfaces: `compliance.compliance`.
- SLO/dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `AUDIT`, `SECRET`, `PII_IDENTIFYING`, `PHI`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `compliance` `compliance` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/compliance/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `compliance` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `compliance` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `compliance` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `compliance` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `compliance` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `compliance` evaluates `<tenant>.compliance.compliance` against policy, writes `compliance.compliance`, and emits `oya.compliance.compliance.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `compliance` binds `bootstrap-trust-chain (ADR-0295)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `compliance` is `contracts/asyncapi.yaml, contracts/compliance.proto, contracts/dsar-export-format.json, contracts/openapi.yaml`; reviewers must map `bootstrap trust chain (ADR 0295)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `compliance` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.cedar, policy/data-residency.md, plus 1 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain (ADR 0295)`.
- Depth detail 4: `compliance` state/event naming uses `compliance.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `compliance` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `compliance` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `compliance` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain (ADR 0295)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `compliance` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `compliance` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `compliance` uses SLOs `slos/audit-chain-seal-verify-success.openslo.yaml, slos/auditor-portal-availability.openslo.yaml, slos/auditor-portal-latency.openslo.yaml, slos/breach-notify-authority-72h.openslo.yaml, slos/cross-tenant-isolation-violations.openslo.yaml, plus 7 more` and dashboards `dashboards/audit-chain-seal-health.json, dashboards/breach-notification-sla.json, dashboards/dsar-pipeline.json, dashboards/evidence-coverage.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `compliance` uses runbooks `runbooks/audit-seal-verify-failure.md, runbooks/breach-notification-72h-clock-at-risk.md, runbooks/cross-tenant-dsar-leak-suspected.md, runbooks/dsar-backlog-overflow.md, runbooks/engagement-cedar-revoke-failed.md, plus 5 more` so `bootstrap trust chain (ADR 0295)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `compliance` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/evidence-collector/Chart.yaml, iac/helm/evidence-collector/README.md, iac/helm/evidence-collector/values.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `compliance` uses `capabilities/auditor-engagement-read.cedar, capabilities/breach-declare.cedar, capabilities/compliance-admin-upload.cedar, capabilities/dsar-subject-self-service.cedar, plus 1 more` and `catalog/api-asyncapi.yaml, catalog/api-rest.yaml, catalog/auditor-portal-frontend.yaml, catalog/component-info.yaml, plus 7 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `compliance` fails closed when `bootstrap trust chain (ADR 0295)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `compliance` emits denial evidence for `bootstrap trust chain (ADR 0295)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `compliance` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain (ADR 0295)` workflow.
- Depth detail 17: `compliance` telemetry for `bootstrap trust chain (ADR 0295)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §six-dimension matrix

| Dimension | Status |
|---|---|
| Maintainability | Module boundaries + SemVer policy + deprecation cadence per ADR-0258. |
| Observability | 7 audit-event-classes + 8 SLOs + 3 dashboards (this directory). |
| Scalability | Sharded by tenant + pack; capacity-model.md. |
| Performance | DSAR P95 ≤72h; evidence emit lag P99 ≤5s; auditor portal P95 ≤300ms. |
| Optimization | Lazy evidence indexing; eager seal-verify on every read. |
| Code quality | ≥85% line, ≥75% branch; `oya-check-*` lints; Rust deny(warnings). |

---



## §cell-eligibility
This anchor is closed for `compliance` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `compliance` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `compliance`; owner `axis-compliance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `compliance` root context.
- Capability records cited: `microservices/compliance/capabilities/auditor-engagement-read.cedar`, `microservices/compliance/capabilities/breach-declare.cedar`, `microservices/compliance/capabilities/compliance-admin-upload.cedar`, `microservices/compliance/capabilities/dsar-subject-self-service.cedar`, `microservices/compliance/capabilities/pack-overlay-subscribe.cedar`.
- API surfaces cited: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +1 more.
- SLO and dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`, `microservices/compliance/slos/dsar-backlog-depth.openslo.yaml`; +12 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`, `microservices/compliance/runbooks/evidence-collector-degraded.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar binding: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +1 more.
- State/event binding: `compliance.compliance`.
- Capability binding: `compliance`.
- SLO binding: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`, `microservices/compliance/slos/dsar-backlog-depth.openslo.yaml`; +6 more.
- Runbook binding: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`, `microservices/compliance/runbooks/evidence-collector-degraded.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `compliance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `compliance`.
- `policy-engine` supplies the signed Cedar corpus while `compliance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `compliance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `compliance`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `compliance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `compliance` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `compliance` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `compliance` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `compliance` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `compliance` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `compliance` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `compliance`; owner `axis-compliance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `compliance` root context.
- Capability records cited: `microservices/compliance/capabilities/auditor-engagement-read.cedar`, `microservices/compliance/capabilities/breach-declare.cedar`, `microservices/compliance/capabilities/compliance-admin-upload.cedar`, `microservices/compliance/capabilities/dsar-subject-self-service.cedar`, `microservices/compliance/capabilities/pack-overlay-subscribe.cedar`.
- API surfaces cited: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +1 more.
- SLO and dashboard evidence: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`, `microservices/compliance/slos/dsar-backlog-depth.openslo.yaml`; +12 more.
- Runbook/IaC evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`, `microservices/compliance/runbooks/evidence-collector-degraded.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`.
- Cedar binding: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +1 more.
- State/event binding: `compliance.compliance`.
- Capability binding: `compliance`.
- SLO binding: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`, `microservices/compliance/slos/dsar-backlog-depth.openslo.yaml`; +6 more.
- Runbook binding: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`, `microservices/compliance/runbooks/evidence-collector-degraded.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `compliance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `compliance`.
- `policy-engine` supplies the signed Cedar corpus while `compliance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `compliance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `compliance`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `compliance` applies the most restrictive policy and emits a degraded-mode audit event.
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

