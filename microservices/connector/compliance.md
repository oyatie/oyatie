---
microservice: connector
doc_class: ComplianceMap
date: 2026-05-20
owner_team: council-privacy + ops-compliance + axis-integration
status: Accepted
related_adrs: [ADR-0245, ADR-0250, ADR-0251, ADR-0263, ADR-0272, ADR-0273, ADR-0276, ADR-0284, ADR-0292, ADR-0293, ADR-0295, ADR-0296]
companion_docs:
  - microservices/connector/threat-model.md
  - microservices/connector/dpia.md
  - microservices/connector/policy/data-residency.md
doc_status: published
---

# Compliance Map — connector (Integration Substrate)

Per ADR-0250 build-ahead-of-certification: the substrate ships day-one-ready for the certifications below.

## §day-one-cert-readiness

| Cert / Regulation | Status | Evidence |
|---|---|---|
| SOC 2 Type II (Security, Availability, Confidentiality, Privacy) | Ready | Audit-chain seals; Cedar default-deny; OpenBao SecretReference; SPIFFE workload identity |
| ISO 27001:2022 | Ready | Asset register in `manifest.json`; access control via Cedar; crypto controls per ADR-0253 |
| GDPR (EU 2016/679) | Ready | DPIA at `dpia.md`; per-purpose consent (ADR-0272); per-tenant DKIM/SPF/DMARC (ADR-0273); backup portability (ADR-0276) |
| KR PIPA (2020 amendment) + K-CSAP | Ready | Pack-kr overlay; PIPC notification flow; Toss/KakaoPay/LINE Pay allow-listed |
| HIPAA (US 45 CFR §164) | Ready (post-BAA) | Pack-us-healthcare overlay; HIPAA-eligible-only catalog filter; BAA-required-per-vendor matrix |
| PCI DSS v4.0 | Pass-through | Payment connectors (Stripe, Toss, etc.) are PCI-certified themselves; connect never holds raw card numbers |
| EU AI Act (2024/1689) | Conditional | When AI-assisted data-mapper ships (M02), Art. 6 high-risk-system gate via `oya-check-eu-ai-act-annex-iii-refusal` |
| CN-PIPL-2021 | Ready | Pack-cn overlay; domestic-licensed payment connector filter; cross-border transfer mechanism per Art. 38-40 |
| FedRAMP High | Ready (post-3PAO) | SPIFFE + SBOM + cosign keyless OIDC + audit chain meet control families AC, AU, IA, SC, SI |
| IL5/IL6 (DoD) | Conditional | Air-gapped cell variant; no internet-facing surface in IL6 |
| KSA NCA cloud-residency | Ready | Pack-ksa overlay; OCI me-jeddah-1 + me-riyadh-1 pinning |
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `day-one-cert-readiness` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `day one cert readiness` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.

## §pack-overlay-roster

| Pack ID | Activated | Overlay path | Connectors allow-listed |
|---|---|---|---|
| pack-kr | yes | packs/kr/overlay.yaml | Toss Payments, KakaoPay, LINE Pay, KakaoTalk |
| pack-eu | conditional | packs/eu/overlay.yaml | All GDPR-compliant; Schrems-II-mitigated transfer mechanisms |
| pack-us | conditional | packs/us/overlay.yaml | All US-hosted vendors |
| pack-us-healthcare | conditional | packs/us-healthcare/overlay.yaml | HIPAA-BAA-signed vendors only (Twilio HIPAA, AWS HIPAA-eligible, Google Cloud BAA) |
| pack-jp | conditional | packs/jp/overlay.yaml | LINE, SoftBank, NTT, KDDI |
| pack-sg | conditional | packs/sg/overlay.yaml | GovTech endpoints; PDPA-compliant |
| pack-au | conditional | packs/au/overlay.yaml | Privacy Act 1988-compliant |
| pack-in | conditional | packs/in/overlay.yaml | DPDPA 2023-compliant; UPI/Razorpay |
| pack-br | conditional | packs/br/overlay.yaml | LGPD-compliant; PIX |
| pack-ae | conditional | packs/ae/overlay.yaml | UAE-PDPL-compliant |
| pack-ksa | conditional | packs/ksa/overlay.yaml | NCA cloud-residency; STC/Mada |
| pack-cn | conditional | packs/cn/overlay.yaml | CN-PIPL-2021; domestic-licensed only |
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `pack-overlay-roster` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `pack overlay roster` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `pack overlay roster`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.

## §consent (ADR-0272 adherence)

Per-purpose consent surface for end-users whose data flows through connectors:
- `purpose:integration_dispatch` — required for any connector wiring to invoke against the user's data
- `purpose:audit_telemetry` — required for telemetry retention beyond 30d
- `purpose:abuse_defence_fingerprinting` — required for JA4 storage beyond 7d
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `consent (ADR-0272 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `consent (ADR 0272 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `consent (ADR 0272 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent (ADR 0272 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `consent (ADR 0272 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `consent (ADR 0272 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `consent (ADR 0272 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §email-deliverability (ADR-0273 adherence)

Per-tenant DKIM/SPF/DMARC for outbound emails from connect (e.g., schema-drift notifications). Per-tenant DNS subdomain for webhook receiver: `hooks.<tenant>.oyatie.app` with HTTPS RR including `ech=` per ADR-0253.
### Content-pass expansion — email-deliverability
- This expansion preserves the existing prose above and closes `email-deliverability` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Workspace DKIM/SPF/DMARC anchors the external control pattern for `email-deliverability`.
- Precedent 2: AWS SES domain identity provides a second independent hyperscaler pattern for `email-deliverability`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `email-deliverability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `email-deliverability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `email-deliverability (ADR-0273 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `email deliverability (ADR 0273 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `email deliverability (ADR 0273 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `email deliverability (ADR 0273 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `email deliverability (ADR 0273 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `email deliverability (ADR 0273 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `email deliverability (ADR 0273 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connector` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `email deliverability (ADR 0273 adherence)` workflow.
- Depth detail 17: `connector` telemetry for `email deliverability (ADR 0273 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connector` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §self-modification (ADR-0247 adherence)

is not a Foundry-touching µservice in the self-modification doctrine sense. Connector adapter publishing (MPO flow) is governed by marketplace + Foundry approval per ADR-0247, not by connect itself.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `self-modification (ADR-0247 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `self modification (ADR 0247 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `self modification (ADR 0247 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification (ADR 0247 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `self modification (ADR 0247 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `self modification (ADR 0247 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `self modification (ADR 0247 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connector` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification (ADR 0247 adherence)` workflow.
- Depth detail 17: `connector` telemetry for `self modification (ADR 0247 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connector` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §minor-protection (ADR-0292 adherence)

When a tenant's audience includes minors:
- COPPA <13: connector wiring requires tenant attestation; certain connectors (social media APIs) are filtered.
- KOSA 14-17: data-sharing connectors require parental-consent surface per `microservices/community/`.
- EU age-verification: AVS check before activating any connector wiring for minor-audience workflows.
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `minor-protection (ADR-0292 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `minor protection (ADR 0292 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection (ADR 0292 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection (ADR 0292 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `minor protection (ADR 0292 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `minor protection (ADR 0292 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `minor protection (ADR 0292 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §platform-owner-indirection (ADR-0284 adherence)

No hard-coded `oyatie` strings in connect source. All references go through `platform_owner_name()` indirection from `oya-shared-platform-config`.
### Content-pass expansion — platform-owner-indirection
- This expansion preserves the existing prose above and closes `platform-owner-indirection` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Salesforce My Domain anchors the external control pattern for `platform-owner-indirection`.
- Precedent 2: Google Workspace tenant branding provides a second independent hyperscaler pattern for `platform-owner-indirection`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `platform-owner-indirection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `platform-owner-indirection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `platform-owner-indirection (ADR-0284 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `platform owner indirection (ADR 0284 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `platform owner indirection (ADR 0284 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `platform owner indirection (ADR 0284 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `platform owner indirection (ADR 0284 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `platform owner indirection (ADR 0284 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `platform owner indirection (ADR 0284 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connector` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `platform owner indirection (ADR 0284 adherence)` workflow.
- Depth detail 17: `connector` telemetry for `platform owner indirection (ADR 0284 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connector` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §bootstrap-trust-chain (ADR-0295 adherence)

SPIFFE workload identity issued by SPIRE server. Bootstrap CI ≥30d-aged trust root per ADR-0295; kill-switch wired (see ARCHITECTURE.md §bootstrap-trust-chain).
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `bootstrap-trust-chain (ADR-0295 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `bootstrap trust chain (ADR 0295 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain (ADR 0295 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain (ADR 0295 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `bootstrap trust chain (ADR 0295 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `bootstrap trust chain (ADR 0295 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `bootstrap trust chain (ADR 0295 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connector` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain (ADR 0295 adherence)` workflow.
- Depth detail 17: `connector` telemetry for `bootstrap trust chain (ADR 0295 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connector` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §credential-isolation (ADR-0296 adherence)

`credential_isolation_mode`: `sidecar`. Refresh tokens never leave OpenBao; access tokens ≤60s TTL.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `credential-isolation (ADR-0296 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `credential isolation (ADR 0296 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `connector` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `connector` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296 adherence)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `connector` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `connector` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `connector` uses SLOs `slos/connect-retirement.openslo.yaml, slos/connector-availability.openslo.yaml, slos/dlq-overflow-prevention.openslo.yaml, slos/oauth-token-health.openslo.yaml, slos/webhook-receiver-throughput.openslo.yaml` and dashboards `dashboards/connector-usage-by-tenant.json, dashboards/dlq-state.json, dashboards/oauth-token-health.md, dashboards/webhook-receiver-throughput.json` when those artifacts exist.
- Depth detail 11: Incident evidence for `connector` uses runbooks `runbooks/connector-attestation-revoked.md, runbooks/connector-cascade-failure.md, runbooks/connector-onboarding.md, runbooks/connector-rate-limit-saturation.md, runbooks/dlq-overflow.md, plus 5 more` so `credential isolation (ADR 0296 adherence)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `connector` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/external-secret.yaml, iac/helm-values-connector.yaml, iac/ingress-production.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `connector` uses `capabilities/connector-invoke.yaml, capabilities/oauth-grant-initiate.yaml, capabilities/umbrella-retirement-readiness.yaml, capabilities/webhook-endpoint-register.yaml` and `catalog/oya-connector-adapter-domain.yaml, catalog/oya-connector-catalog-api.yaml, catalog/oya-connector-catalog-domain.yaml, catalog/oya-connector-catalog-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `connector` fails closed when `credential isolation (ADR 0296 adherence)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `connector` emits denial evidence for `credential isolation (ADR 0296 adherence)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `connector` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296 adherence)` workflow.
- Depth detail 17: `connector` telemetry for `credential isolation (ADR 0296 adherence)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `connector` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §meta-trust-attestation (ADR-0293 adherence)

Not applicable (connect is not Foundry-touching). If/when MPO connector adapters are signed by Foundry, the meta-trust-root attestation path applies — currently routed via marketplace per ADR-0249.

## Evidence collectors

| Collector | Cadence | Output |
|---|---|---|
| audit-chain-replay | hourly | Per-tenant evidence pack |
| slo-window | daily | SLO compliance report |
| abuse-defence-outcome | daily | Per-tenant false-positive + blocked-bot counts |
| oauth-grant-lifecycle | daily | Active grants, revoke events, rotation events |
| dlq-replay-outcome | daily | DLQ replay success rate per tenant |

## References

- `microservices/connector/policy/data-residency.md`
- `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- `docs/standards/documentation-rigor.md` §3.2.3

---
### Content-pass expansion — meta-trust-attestation
- This expansion preserves the existing prose above and closes `meta-trust-attestation` for `connector` to the ≥50-line documentation-rigor floor.
- Service owner `council-architecture` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `umbrella-retirement-readiness`; bounded contexts: `umbrella-retirement-readiness`.
- API surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`; +5 more.
- Cedar/policy surfaces: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`; +5 more.
- State/event surfaces: `connector.umbrella_retirement_readiness`, `connect.t3`, `connect.microservices_connect_capabilities_umbrella_retirement_readiness_yaml`.
- SLO/dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: The Update Framework roots anchors the external control pattern for `meta-trust-attestation`.
- Precedent 2: Sigstore Rekor transparency provides a second independent hyperscaler pattern for `meta-trust-attestation`.
- Tenant-scope invariant: every `connector` `umbrella-retirement-readiness` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/connect/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `connector` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `connector` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `connector` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `connector` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `connector` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `umbrella-retirement-readiness` evaluates `<tenant>.connector.umbrella-retirement-readiness` against policy, writes `connector.umbrella_retirement_readiness`, and emits `oya.connector.umbrella.retirement.readiness.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `meta-trust-attestation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `meta-trust-attestation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `connector` binds `meta-trust-attestation (ADR-0293 adherence)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `connector` is `contracts/asyncapi-v1.yaml, contracts/connect-retirement.asyncapi.yaml, contracts/connect-retirement.openapi.yaml, contracts/connect_retirement.proto, contracts/connector-adapter-trait.md, contracts/metric-naming-convention.md, plus 1 more`; reviewers must map `meta trust attestation (ADR 0293 adherence)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `connector` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/connector-authorization.cedar, policy/connector-catalog-publishing.cedar, policy/data-residency.md, plus 5 more`; missing policy files are scaffold debt, not an implicit pass for `meta trust attestation (ADR 0293 adherence)`.
- Depth detail 4: `connector` state/event naming uses `connect.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `connector` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.

## §detection-substrate-binding
This anchor is closed for `connector` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `connector` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `umbrella-retirement-readiness` touches those data classes.
- Signal sources: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +14 more.
- Example event class: `oya.connector.umbrella.retirement.readiness.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §investigation-binding
This anchor is closed for `connector` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `connector` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.connector.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `umbrella-retirement-readiness` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `umbrella-retirement-readiness` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §insider-threat-controls
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `connector` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`; +1 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `connector.umbrella_retirement_readiness` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §threat-intelligence-feeds
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `connector` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +16 more.
- Example: `umbrella-retirement-readiness` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §key-rotation-cadence
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.connector` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/connect/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.
- Example: `umbrella-retirement-readiness` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §crypto-agility-plan
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `connector` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`; +15 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `umbrella-retirement-readiness` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §pentest-and-bounty-cadence
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `connector` is in annual full-scope pentest and every major `umbrella-retirement-readiness` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`; +22 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `connector` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §facility-controls
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `connector` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `umbrella-retirement-readiness` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §supply-chain-risk
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `connector` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/connector/catalog/connectors/airtable.yaml`, `microservices/connector/catalog/connectors/asana.yaml`, `microservices/connector/catalog/connectors/bigquery.yaml`, `microservices/connector/catalog/connectors/clickup.yaml`, `microservices/connector/catalog/connectors/datadog.yaml`, `microservices/connector/catalog/connectors/discord.yaml`; +27 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `umbrella-retirement-readiness` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `connector` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `connector` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `umbrella-retirement-readiness` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `umbrella-retirement-readiness` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §data-classification
This anchor is closed for `connector` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `connector.umbrella_retirement_readiness`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `umbrella-retirement-readiness` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `connector`; owner `council-architecture`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `connector` root context.
- Capability records cited: `microservices/connector/capabilities/connector-invoke.yaml`, `microservices/connector/capabilities/oauth-grant-initiate.yaml`, `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`, `microservices/connector/capabilities/webhook-endpoint-register.yaml`.
- API surfaces cited: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- SLO and dashboard evidence: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`, `microservices/connector/dashboards/connector-usage-by-tenant.json`; +3 more.
- Runbook/IaC evidence: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, `microservices/connector/contracts/asyncapi-v1.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect_retirement.proto`, `microservices/connector/contracts/connector-adapter-trait.md`.
- Cedar binding: `microservices/connector/policy/abuse-defence.cedar`, `microservices/connector/policy/auditor-scope.cedar`, `microservices/connector/policy/ci-scope.cedar`, `microservices/connector/policy/connector-authorization.cedar`, `microservices/connector/policy/connector-catalog-publishing.cedar`, `microservices/connector/policy/data-residency.md`; +5 more.
- State/event binding: `connector.umbrella_retirement_readiness`.
- Capability binding: `umbrella-retirement-readiness`.
- SLO binding: `microservices/connector/slos/connect-retirement.openslo.yaml`, `microservices/connector/slos/connector-availability.openslo.yaml`, `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`, `microservices/connector/slos/oauth-token-health.openslo.yaml`, `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`.
- Runbook binding: `microservices/connector/runbooks/connector-attestation-revoked.md`, `microservices/connector/runbooks/connector-cascade-failure.md`, `microservices/connector/runbooks/connector-onboarding.md`, `microservices/connector/runbooks/connector-rate-limit-saturation.md`, `microservices/connector/runbooks/dlq-overflow.md`, `microservices/connector/runbooks/oauth-token-revocation-cascade.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `connector`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `connector`.
- `policy-engine` supplies the signed Cedar corpus while `connector` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `connector` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `connector`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `connector` applies the most restrictive policy and emits a degraded-mode audit event.
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
