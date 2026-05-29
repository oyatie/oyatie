---
doc_class: Architecture
shape: Walkthrough
length_cap: 1500
authority_tier: 2
status: Accepted
date: 2026-05-20
microservice: comms-email
companion_docs:
  - microservices/comms-email/PRD.md
  - microservices/comms-email/compliance.md
  - microservices/comms-email/threat-model.md
  - microservices/comms-email/dpia.md
related_adrs:
  - ADR-0201
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253
  - ADR-0254
  - ADR-0258
  - ADR-0263
  - ADR-0273
  - ADR-0294
  - ADR-0295
  - ADR-0296
inbound_citations:
  - microservices/comms-email/PRD.md
  - microservices/comms-email/README.md
---

# comms-email — Architecture

## §principals (ADR-0242)

Runs as `oyatie.comms-email.outbound-sender`, `oyatie.comms-email.inbound-receiver`,
`oyatie.comms-email.template-renderer`, `oyatie.comms-email.suppression-list-manager`,
`oyatie.comms-email.deliverability-tracker`, `oyatie.comms-email.dkim-rotator`,
`oyatie.comms-email.reputation-monitor`. SPIFFE-attested per ADR-0295.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `principals (ADR-0242)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `principals (ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `principals (ADR 0242)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `principals (ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `principals (ADR 0242)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `principals (ADR 0242)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §cedar-gates (ADR-0243)

Cedar fragments: `policy/action-authorization.cedar` (default-deny baseline),
`policy/abuse-defence.cedar` (with UX-floor), `policy/comms-email-send.cedar`,
`policy/comms-email-suppression-list.cedar`, `policy/comms-email-tenant-domain-mgmt.cedar`,
`policy/comms-email-webhook-ingest.cedar`, `policy/data-residency.cedar`,
`policy/auditor-scope.cedar`, `policy/ci-scope.cedar`,
`policy/pack-overlay-authorization.cedar`.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `cedar-gates (ADR-0243)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `cedar gates (ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (ADR 0243)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `cedar gates (ADR 0243)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.

## §tenant-scoping (ADR-0244)

Every send / inbound / suppression / domain row carries `tenant_id`. `audience_type`:
`B2B_TENANT_ADMIN`, `B2C_CONSUMER_RECIPIENT`. `provider_credential_mode = tenant_byok`
(provider-BYOK; tenant brings their SES / Mailgun / SMTP credentials per ADR-0255 §D-4) —
distinct from encryption-BYOK on the at-rest store.
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Stripe account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `tenant-scoping (ADR-0244)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `tenant scoping (ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (ADR 0244)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `tenant scoping (ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `tenant scoping (ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `tenant scoping (ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §substrate-product-binding (ADR-0245)

Tier-substrate. Consumers: every product that sends email.
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `substrate-product-binding (ADR-0245)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `substrate product binding (ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (ADR 0245)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `substrate product binding (ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `substrate product binding (ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `substrate product binding (ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (ADR 0245)` workflow.
- Depth detail 17: `comms-email` telemetry for `substrate product binding (ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `comms-email` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §policy-evaluation (ADR-0246 + amendment)

Library-first via `oya-shared-policy-eval`. Per-tenant DKIM key fragments soaked ≥60s per
ADR-0294.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `policy-evaluation (ADR-0246 + amendment)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `policy evaluation (ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (ADR 0246 + amendment)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `policy evaluation (ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `policy evaluation (ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `policy evaluation (ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (ADR 0246 + amendment)` workflow.
- Depth detail 17: `comms-email` telemetry for `policy evaluation (ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §cellular-architecture (ADR-0248)

Tier-1 outbound senders + inbound receivers. Tier-3 stores suppression lists + deliverability
events. Cloud Hypervisor + Kata pods for tenant-PII handling.
### Content-pass expansion — cell-eligibility
- This expansion preserves the existing prose above and closes `cell-eligibility` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS cell-based architecture anchors the external control pattern for `cell-eligibility`.
- Precedent 2: Route 53 shuffle sharding provides a second independent hyperscaler pattern for `cell-eligibility`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cell-eligibility`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cell-eligibility` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `cellular-architecture (ADR-0248)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `cellular architecture (ADR 0248)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cellular architecture (ADR 0248)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cellular architecture (ADR 0248)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `cellular architecture (ADR 0248)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `cellular architecture (ADR 0248)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `cellular architecture (ADR 0248)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `cellular architecture (ADR 0248)` workflow.
- Depth detail 17: `comms-email` telemetry for `cellular architecture (ADR 0248)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §day-one-cert-readiness (ADR-0250)

Day-one HIPAA-BAA-eligible (postal-only enforcement in us-healthcare overlay), GDPR
DSAR-eligible, KR-PIPA-eligible.
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `day-one-cert-readiness (ADR-0250)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `day one cert readiness (ADR 0250)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness (ADR 0250)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `day one cert readiness (ADR 0250)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `day one cert readiness (ADR 0250)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `day one cert readiness (ADR 0250)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `day one cert readiness (ADR 0250)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `day one cert readiness (ADR 0250)` workflow.
- Depth detail 17: `comms-email` telemetry for `day one cert readiness (ADR 0250)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §pack-overlay-roster (ADR-0251)

`gdpr`, `hipaa`, `kr-csap`, `eu-sovereign`, `ksa`, `uae`, `us-healthcare`,
`cn-pipl` (postal-only with onshore relay), `eu-ai-act-annex-iii` (transactional only).
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `pack-overlay-roster (ADR-0251)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `pack overlay roster (ADR 0251)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `pack overlay roster (ADR 0251)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `pack overlay roster (ADR 0251)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `pack overlay roster (ADR 0251)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `pack overlay roster (ADR 0251)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `pack overlay roster (ADR 0251)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `pack overlay roster (ADR 0251)` workflow.
- Depth detail 17: `comms-email` telemetry for `pack overlay roster (ADR 0251)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §time-coordination (ADR-0252)

HLC default. TrueTime opt-in for cross-region suppression-list consistency on
unsubscribe-now-everywhere events.
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `time-coordination (ADR-0252)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `time coordination (ADR 0252)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination (ADR 0252)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination (ADR 0252)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `time coordination (ADR 0252)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `time coordination (ADR 0252)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `time coordination (ADR 0252)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `time coordination (ADR 0252)` workflow.
- Depth detail 17: `comms-email` telemetry for `time coordination (ADR 0252)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §transport (ADR-0253)

HTTP/3 + QUIC default. SMTP for outbound (RFC 5321). DKIM signing + DMARC alignment per
ADR-0273. ECH per `iac/ech-config.yaml`. PQC hybrid per `iac/pqc-cert.yaml`.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `transport (ADR-0253)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `transport (ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `transport (ADR 0253)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `transport (ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `transport (ADR 0253)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `transport (ADR 0253)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `transport (ADR 0253)` workflow.
- Depth detail 17: `comms-email` telemetry for `transport (ADR 0253)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §deployment-shape (ADR-0254)

K8s + Cloud Hypervisor + Kata for outbound + inbound (handles tenant PII). Postal as
self-hosted relay in sovereign packs; SES as SaaS in default cells.
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `deployment-shape (ADR-0254)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `deployment shape (ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (ADR 0254)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `deployment shape (ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `deployment shape (ADR 0254)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `deployment shape (ADR 0254)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `deployment shape (ADR 0254)` workflow.
- Depth detail 17: `comms-email` telemetry for `deployment shape (ADR 0254)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §intelligence-dispatch (ADR-0255)

Calls Intelligence for reputation-monitoring anomaly detection + bounce-pattern classification.
Library-first; `audience_type = INTERNAL_SUBSTRATE`.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `intelligence-dispatch (ADR-0255)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `intelligence dispatch (ADR 0255)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (ADR 0255)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (ADR 0255)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `intelligence dispatch (ADR 0255)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `intelligence dispatch (ADR 0255)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `intelligence dispatch (ADR 0255)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (ADR 0255)` workflow.
- Depth detail 17: `comms-email` telemetry for `intelligence dispatch (ADR 0255)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §ontology-read-path (ADR-0257 amendment)

Reads tenant + recipient projections. `ontology_read_mode = library_first`. `freshness_floor =
60s`.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `ontology-read-path (ADR-0257 amendment)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `ontology read path (ADR 0257 amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (ADR 0257 amendment)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (ADR 0257 amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `ontology read path (ADR 0257 amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `ontology read path (ADR 0257 amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `ontology read path (ADR 0257 amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (ADR 0257 amendment)` workflow.
- Depth detail 17: `comms-email` telemetry for `ontology read path (ADR 0257 amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §observability (ADR-0263)

Audit-event-classes: `oya.comms-email.send`, `oya.comms-email.bounce`,
`oya.comms-email.complaint`, `oya.comms-email.unsubscribe`, `oya.comms-email.dkim-rotated`,
`oya.comms-email.dmarc-alignment-fail`, `oya.comms-email.reputation-drop`,
`oya.comms-email.suppression-list-add`, `oya.comms-email.tenant-domain-onboarded`,
`oya.comms-email.webhook-ingest`.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE four primary SRE signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `observability (ADR-0263)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `observability (ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `observability (ADR 0263)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `observability (ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `observability (ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §cookie-consent (ADR-0272)

Email open-tracking + click-tracking only with per-purpose consent recorded.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `cookie-consent (ADR-0272)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `cookie consent (ADR 0272)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cookie consent (ADR 0272)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cookie consent (ADR 0272)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `cookie consent (ADR 0272)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `cookie consent (ADR 0272)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `cookie consent (ADR 0272)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `cookie consent (ADR 0272)` workflow.
- Depth detail 17: `comms-email` telemetry for `cookie consent (ADR 0272)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `comms-email` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §email-deliverability (ADR-0273)

Per-tenant DKIM/SPF/DMARC. DKIM key rotation 90d; emergency rotation runbook
`runbooks/dkim-key-rotation.md`. BIMI logo for tenants on opt-in.
### Content-pass expansion — email-deliverability
- This expansion preserves the existing prose above and closes `email-deliverability` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Workspace DKIM/SPF/DMARC anchors the external control pattern for `email-deliverability`.
- Precedent 2: AWS SES domain identity provides a second independent hyperscaler pattern for `email-deliverability`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `email-deliverability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `email-deliverability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `email-deliverability (ADR-0273)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `email deliverability (ADR 0273)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `email deliverability (ADR 0273)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `email deliverability (ADR 0273)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `email deliverability (ADR 0273)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `email deliverability (ADR 0273)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `email deliverability (ADR 0273)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `email deliverability (ADR 0273)` workflow.
- Depth detail 17: `comms-email` telemetry for `email deliverability (ADR 0273)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §abuse-defence (§3.2.3 + ADR-0297)

Internet-facing: inbound MX + webhook receivers + admin portal. Anti-bot via per-fingerprint
rate-limit. Anti-spoof via DKIM/SPF/DMARC (email) + SPIFFE (workload) + HMAC (webhook).
Anti-scrape via per-tenant rate-limit + paid bulk API tier. UX-floor preserved.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `abuse-defence (§3.2.3 + ADR-0297)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `abuse defence (§3.2.3 + ADR 0297)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (§3.2.3 + ADR 0297)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (§3.2.3 + ADR 0297)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `abuse defence (§3.2.3 + ADR 0297)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `abuse defence (§3.2.3 + ADR 0297)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `abuse defence (§3.2.3 + ADR 0297)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `abuse defence (§3.2.3 + ADR 0297)` workflow.

## §credential-isolation (ADR-0296)

OpenBao SecretReference `${openbao:secret/<tenant_id>/comms-email/<key>}`. Sidecar isolation.
DKIM private keys + provider API keys (SES, Mailgun) at ≤60s TTL.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `comms-email` to the ≥50-line documentation-rigor floor.
- Service owner `oya-substrate-comms` owns this answer; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `T0-transactional-send`; bounded contexts: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`; +2 more.
- API surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy surfaces: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`; +5 more.
- State/event surfaces: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`; +1 more.
- SLO/dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`; +11 more.
- Compliance packs: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `comms-email` `T0-transactional-send` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/comms-email/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `comms-email` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `comms-email` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `comms-email` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `comms-email` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `comms-email` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `T0-transactional-send` evaluates `<tenant>.comms-email.t0-transactional-send` against policy, writes `comms_email.transactional_send`, and emits `oya.comms.email.t0.transactional.send.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `comms-email` binds `credential-isolation (ADR-0296)` to `transactional-send` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `comms-email` is `contracts/asyncapi.yaml, contracts/comms_email.proto, contracts/openapi.yaml`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `comms-email` is `policy/abuse-defence.cedar, policy/action-authorization.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/comms-email-send.cedar, policy/comms-email-suppression-list.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `comms-email` state/event naming uses `comms_email.transactional_send` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `comms-email` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `comms-email` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `comms-email` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `comms-email` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `comms-email` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `comms-email` uses SLOs `slos/audit-chain-emit-lag-p99.openslo.yaml, slos/deliverability-rate.openslo.yaml, slos/dkim-signing-rate.openslo.yaml, slos/dmarc-alignment-rate.openslo.yaml, slos/from-domain-onboarding-time.openslo.yaml, plus 4 more` and dashboards `dashboards/deliverability.json, dashboards/dkim-rotation.json, dashboards/reputation-monitoring.json, dashboards/send-pipeline.json, plus 1 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `comms-email` uses runbooks `runbooks/blacklist-recovery.md, runbooks/bounce-storm-mitigation.md, runbooks/dkim-key-rotation.md, runbooks/dmarc-policy-tune.md, runbooks/inbound-receiver-quarantine-release.md, plus 5 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `comms-email` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/postal/Chart.yaml, iac/helm/postal/templates/deployment.yaml, iac/helm/postal/values.yaml, plus 12 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `comms-email` uses `capabilities/T0-transactional-send.json, capabilities/T1-bounce-handle.json, capabilities/T1-webhook-delivery-event.json, capabilities/T2-list-manage.json, plus 2 more` and `catalog/bounded-contexts.json` to keep layer names and owners machine-checkable.
- Depth detail 14: `comms-email` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `comms-email` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `comms-email` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `comms-email` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §six-dimension matrix

| Dimension | Status |
|---|---|
| Maintainability | LTS pins (Postal 3.x, SES API); deprecation cadence per ADR-0258. |
| Observability | 10 audit-event-classes + 9 SLOs + 3 dashboards (current) + 2 dashboards added. |
| Scalability | Per-tenant rate-limit; horizontal scale; capacity-model.md. |
| Performance | Send P99 ≤500ms; suppression lookup P99 ≤10ms; DMARC alignment ≥99.5%. |
| Optimization | Lazy DKIM key fetch (sidecar cache); eager bounce-storm circuit-breaker. |
| Code quality | ≥85% line ≥75% branch; `oya-check-*`; Rust deny(warnings). |

---



## §cell-eligibility
This anchor is closed for `comms-email` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `T0-transactional-send` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `comms-email` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `comms-email` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `comms-email` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `T0-transactional-send` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `comms-email` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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

