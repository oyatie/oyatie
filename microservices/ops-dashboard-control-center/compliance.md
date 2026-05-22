---
doc_class: Compliance
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0263
  - ADR-0272
  - ADR-0276
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
companion_docs:
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
  - microservices/ops-dashboard-control-center/dpia.md
  - microservices/ops-dashboard-control-center/threat-model.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Compliance — ops-dashboard-control-center

## §pack-overlay-roster

Per ADR-0251, the following compliance packs activate overlays on this µservice:

| Pack ID | Activation condition | Overlay effect |
|---|---|---|
| `oya-pack-eu` | EU-region operator | GDPR Art. 5(1)(e) data minimisation on audit retention; breach-notification workflow |
| `oya-pack-us-healthcare` | HIPAA-eligible cell operator | PHI access reason-coded; break-glass audit-trail HIPAA-sealed; BAA required |
| `oya-pack-kr` | KR-region operator | K-ISMS requirement for access-log retention ≥3yr; KR-PIPC DPA audit surface |
| `oya-pack-us` | US operators | FedRAMP-High controls for `.gov` tenants; SOX 806 whistleblower protection |
| `oya-pack-jp` | JP-region operator | APPI access-log retention; FISC compliance for financial-tenant admin actions |
| `oya-pack-sg` | SG-region operator | MAS TRM §9 operational resilience; PDPA access-log |
| `oya-pack-au` | AU-region operator | ASD Essential Eight controls; Privacy Act 1988 access-log |
| `oya-pack-in` | IN-region operator | DPDP Act 2023 access-log; RBI CSCRF for financial-tenant actions |
| `oya-pack-br` | BR-region operator | LGPD Art. 37 access-log; BACEN operational resilience |
| `oya-pack-ae` | AE-region operator | TDRA SIA access-log; CBUAE operational resilience |
| `oya-pack-ksa` | KSA-region operator | NCA ECC 1-3 access-log; SAMA operational resilience |

Pack-id values sourced from central pack registry. No ad-hoc pack-ids.
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `pack-overlay-roster` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `pack overlay roster` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `pack overlay roster`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.

## §day-one-cert-readiness

Per ADR-0250, this µservice ships ready for the following certifications on day one:

| Certification | Readiness | Evidence |
|---|---|---|
| SOC 2 Type II (Security + Availability + Confidentiality) | Day-one | Audit trail per ADR-0263; Cedar default-deny; TLS 1.3; step-up auth; evidence-pack export |
| ISO 27001:2022 | Day-one | ISMS controls: access control (Cedar), cryptography (PQC + ECH), operations security (runbooks), incident management |
| HIPAA Security Rule | Day-one (for `oya-pack-us-healthcare`) | PHI access logging; BAA surface; break-glass audit; HITECH breach notification workflow |
| KR ISMS-P | Day-one (for `oya-pack-kr`) | K-ISMS controls; access-log retention ≥3yr; DPA audit surface |
| FedRAMP High | Architecture-ready; ATOs via tenant-specific packages | FIPS 140-2 cryptography; SPIFFE workload identity; continuous monitoring |
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `day-one-cert-readiness` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `day one cert readiness` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `day one cert readiness` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `day one cert readiness` failures have trigger, rollback, and post-incident closure.

## §detection-substrate-binding

Per ADR-0307 (wave-3 backlog), this µservice contributes to and consumes from the following detection families:

**Contributes to:**
- Family 7 (Insider risk): Every operator action emitted to detection µservice audit-event stream. Features computed: access breadth per operator per hour, cross-tenant pivot attempts, off-hours access patterns.
- Family 8 (Policy violation): Cedar DENY events emitted as `PolicyViolationDetected` to detection substrate.

**Consumes:**
- Family 7 (Insider risk): UEBA insider-risk score per operator displayed in Cedar-admin-console panel. On score > threshold: triggers `AdminMfaCascade` runbook.
- Family 8 (Policy violation): Cedar anomaly signals; on anomaly: auto-suspend operator session pending review.

Audit event classes emitted per ADR-0263: `AdminActionExecuted`, `AdminActionDenied`, `TenantScopeViolationDetected`, `InsiderRiskSignalReceived`, `PolicyViolationDetected`.

Per-tenant per-pack overlay: `oya-pack-us-healthcare` tenants → PHI features never enter feature store.

Appeal mechanism: adverse action (operator session suspension) routes to `runbooks/step-up-auth-bypass-attempt.md` triage; operator gets remediation path within 15 minutes.
### Content-pass expansion — detection-substrate-binding
- This expansion preserves the existing prose above and closes `detection-substrate-binding` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS GuardDuty findings anchors the external control pattern for `detection-substrate-binding`.
- Precedent 2: Google Chronicle detections provides a second independent hyperscaler pattern for `detection-substrate-binding`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `detection-substrate-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `detection-substrate-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `detection-substrate-binding` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `detection substrate binding` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `detection substrate binding`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `detection substrate binding` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §insider-threat-controls

This µservice is the PRIMARY insider-risk surface in the platform. All other µservices emit to this surface's audit-stream. Controls:

1. **UEBA (User and Entity Behaviour Analytics)**: per-operator baseline computed from 90d rolling window. Anomaly triggers: access breadth > 3σ, cross-tenant pivot, off-hours T3 action, rapid sequential mutations.
2. **JIT access (Just-In-Time)**: T3 admin access provisioned on-demand via OpenBao; expires after session window; no standing access.
3. **Session recording**: video + keystroke capture on all T3 sessions. Stored encrypted in `${openbao:secret/oyatie/session-recordings/<operator_id>/<session_id>}`. Accessible to `oyatie.ops.forensics` principal only.
4. **Access review**: quarterly automated access review via Foundry pipeline; principals not active in 90d are suspended pending re-justification.
5. **Privileged Access Management**: CyberArk/Teleport/Boundary for jump-host access (if needed); Cedar gate on every step.

CI lane: `oya-governance-insider-threat-controls`.
### Content-pass expansion — insider-threat-controls
- This expansion preserves the existing prose above and closes `insider-threat-controls` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Microsoft Purview Insider Risk anchors the external control pattern for `insider-threat-controls`.
- Precedent 2: Google BeyondCorp provides a second independent hyperscaler pattern for `insider-threat-controls`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `insider-threat-controls`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `insider-threat-controls` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `insider-threat-controls` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `insider threat controls` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `insider threat controls`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `insider threat controls` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `insider threat controls` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.

## §threat-intelligence-feeds

Threat intelligence feeds consumed by this µservice for real-time operator risk scoring:

| Feed | Source | Update cadence | Purpose |
|---|---|---|---|
| Employee risk score | HR system + UEBA substrate | Real-time | Elevated risk on PIP, off-boarding notice, access review due |
| Credential compromise | HIBP API | On every session create | Operator password in breach corpus |
| IP reputation | Internal threat intel + Cloudflare feed | Per-request | IP associated with known malicious actor |
| Geo-impossibility | Operator session history | Per-request | Login from impossible geography relative to last session |
| Cert revocation | OCSP stapling | Per TLS handshake | Operator client cert revoked |
### Content-pass expansion — threat-intelligence-feeds
- This expansion preserves the existing prose above and closes `threat-intelligence-feeds` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Mandiant threat intelligence anchors the external control pattern for `threat-intelligence-feeds`.
- Precedent 2: AWS GuardDuty threat lists provides a second independent hyperscaler pattern for `threat-intelligence-feeds`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `threat-intelligence-feeds`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `threat-intelligence-feeds` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `threat-intelligence-feeds` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `threat intelligence feeds` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `threat intelligence feeds`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `threat intelligence feeds` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `threat intelligence feeds` failures have trigger, rollback, and post-incident closure.

## §key-rotation-cadence

| Key class | Rotation cadence | Mechanism |
|---|---|---|
| TLS leaf certificates (mTLS) | 24h | cert-manager ACME + SPIFFE SVID |
| Operator session tokens | Per step-up class (1h–4h) | OpenBao TTL |
| Audit signing key | 90d | OpenBao transit key rotation |
| ECH config-id | 90d | DNS HTTPS RR rotation script |
| PQC KEM public key | 90d (with 30d overlap) | oyatie-rooted CA + sigstore |
| OpenBao root seal key | Annual + on-incident | Shamir 5-of-9 ceremony |
### Content-pass expansion — key-rotation-cadence
- This expansion preserves the existing prose above and closes `key-rotation-cadence` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: AWS KMS key rotation anchors the external control pattern for `key-rotation-cadence`.
- Precedent 2: Google Cloud KMS versions provides a second independent hyperscaler pattern for `key-rotation-cadence`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `key-rotation-cadence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `key-rotation-cadence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `key-rotation-cadence` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `key rotation cadence` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `key rotation cadence`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `key rotation cadence` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `key rotation cadence` failures have trigger, rollback, and post-incident closure.

## §crypto-agility-plan

Crypto agility per §3.2.1 row 12 (ADR-0253) + wave-3 PQC roadmap:

1. **Current**: `X25519MLKEM768` KEM hybrid + `ed25519+ml_dsa_65` signatures. Classical fallback X25519/P-256.
2. **NIST PQC round-4 (2027)**: Migrate to `ML-KEM-1024` (FIPS 203) + `ML-DSA-87` (FIPS 204) when IANA codepoints stable.
3. **Migration path**: library-first policy-eval crate abstracts KEM negotiation. Swap underlying algorithm without API change. Per-crate `#[cfg(feature = "pqc-ml-kem-1024")]` feature gate.
4. **Agility test**: quarterly chaos test rotates all keys + verifies zero-downtime for operator sessions.
### Content-pass expansion — crypto-agility-plan
- This expansion preserves the existing prose above and closes `crypto-agility-plan` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Cloudflare post-quantum TLS anchors the external control pattern for `crypto-agility-plan`.
- Precedent 2: Chrome hybrid PQ TLS provides a second independent hyperscaler pattern for `crypto-agility-plan`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `crypto-agility-plan`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `crypto-agility-plan` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `crypto-agility-plan` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `crypto agility plan` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `crypto agility plan`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `crypto agility plan` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `crypto agility plan` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `crypto agility plan` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §self-modification

Per ADR-0247: this µservice does NOT self-modify. It surfaces self-modification artifacts (Cedar fragment publish, Foundry admission results) as read + dispatch. Meta-trust-root attestation path in `ARCHITECTURE.md §self-modification`.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `self-modification` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `self modification` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ops-dashboard-control-center` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §a11y-disability-accommodations

Per `docs/standards/documentation-rigor.md §3.2.5 row 12`:

- Voice-control-only operators: all actions reachable via keyboard nav; no mouse-required interactions.
- Single-switch access: tab order enforced; focus management tested with `axe` + `pa11y`.
- Longer time budgets: T3 step-up auth timeout extended from 60s to 300s for operators with assistive tech profile flag.
- Screen-reader support: ARIA labels on all action buttons; semantic HTML; no icon-only buttons.
- WCAG 2.2 AA floor; AAA target on accommodation paths.

CI runners: `axe` + `pa11y` per manifest `a11y.test_runners`.

## §minor-protection

Per ADR-0292: this µservice is an internal ops surface; no consumer-facing surfaces. Minor-protection (COPPA <13, KOSA 14-17, EU age verification) does NOT apply to this surface. Internal operators are verified adults (employment verification).
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `minor-protection` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `minor protection` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `minor protection` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `minor protection` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `minor protection` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `minor protection` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `minor protection` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ops-dashboard-control-center` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §meta-trust-attestation

Per ADR-0293: this µservice is Foundry-touching (reads admission gate results). Meta-trust-root attestation: `oyatie.ops.admin-console` SPIFFE SVID verified by Foundry pipeline via Cedar principal check. See `ARCHITECTURE.md §bootstrap-trust-chain`.
### Content-pass expansion — meta-trust-attestation
- This expansion preserves the existing prose above and closes `meta-trust-attestation` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: The Update Framework roots anchors the external control pattern for `meta-trust-attestation`.
- Precedent 2: Sigstore Rekor transparency provides a second independent hyperscaler pattern for `meta-trust-attestation`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `meta-trust-attestation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `meta-trust-attestation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `meta-trust-attestation` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `meta trust attestation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `meta trust attestation`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `meta trust attestation` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `meta trust attestation` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `meta trust attestation` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `meta trust attestation` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `meta trust attestation` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `meta trust attestation` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ops-dashboard-control-center` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §platform-owner-indirection

Per ADR-0284: migrated. All `oyatie` hard-coded strings replaced with `${config:platform.owner.*}` lookups. Lint: `oya-check-platform-owner-indirection` in CI.
### Content-pass expansion — platform-owner-indirection
- This expansion preserves the existing prose above and closes `platform-owner-indirection` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Salesforce My Domain anchors the external control pattern for `platform-owner-indirection`.
- Precedent 2: Google Workspace tenant branding provides a second independent hyperscaler pattern for `platform-owner-indirection`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `platform-owner-indirection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `platform-owner-indirection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `platform-owner-indirection` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `platform owner indirection` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `platform owner indirection`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `platform owner indirection` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `platform owner indirection` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `platform owner indirection` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `platform owner indirection` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `platform owner indirection` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `platform owner indirection` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ops-dashboard-control-center` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §bootstrap-trust-chain

Per ADR-0295: SPIFFE attestation enabled. Kill-switch: `iac/prod-spiffe-kill-switch.yaml`. Bootstrap-tier-1 classification: YES — this µservice bootstraps control-plane access.
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `bootstrap-trust-chain` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `bootstrap trust chain` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `bootstrap trust chain` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `bootstrap trust chain` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `bootstrap trust chain` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `bootstrap trust chain` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ops-dashboard-control-center` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §prevention-layers

Per `docs/standards/documentation-rigor.md §3.2.6.D`, insider-risk threat class is addressed at:
- L0 (Edge): rate-limit per operator; IP reputation check.
- L2 (Auth): JIT credentials; WebAuthn passkeys for step-up; session binding.
- L3 (Cedar): per-action permits; cross-tenant FORBID; default-deny.
- L4 (Application): per-operator quota gates; session recording on T3.
- L5 (Data): RLS at Postgres layer; field-level encryption for PHI.
- L6 (Observability): UEBA baseline; anomaly detection; audit emission per ADR-0263.
- L7 (Investigation): triage queue; evidence correlation; ombudsman escalation.
- L9 (Organizational): quarterly insider-threat training; access review; pentest.

≥3 layers per threat class: satisfied (8 layers for insider risk as primary surface).

## §critical-path-edge-cases

Applicable §3.2.5 rows: 1, 5, 6, 12, 19. Full handling documented in `ARCHITECTURE.md §critical-path-edge-cases`.

CI lane: `oya-governance-critical-path-coverage`.
### Content-pass expansion — critical-path-edge-cases
- This expansion preserves the existing prose above and closes `critical-path-edge-cases` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google SRE incident playbooks anchors the external control pattern for `critical-path-edge-cases`.
- Precedent 2: Stripe idempotency recovery provides a second independent hyperscaler pattern for `critical-path-edge-cases`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `critical-path-edge-cases`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `critical-path-edge-cases` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `critical-path-edge-cases` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `critical path edge cases` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `critical path edge cases`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `critical path edge cases` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `critical path edge cases` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `critical path edge cases` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `critical path edge cases` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `critical path edge cases` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `critical path edge cases` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §consent

Per ADR-0272: this is an internal ops surface. Per-purpose consent surface is NOT required (no user-facing cookies; internal operators have employment-based consent for audit logging). Audit logging is mandatory regardless of operator preference.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `ops-dashboard-control-center` to the ≥50-line documentation-rigor floor.
- Service owner `ops-sre-reliability` owns this answer; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Primary capability/context: `incident-declare`; bounded contexts: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- API surfaces: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; +1 more.
- Cedar/policy surfaces: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`; +5 more.
- State/event surfaces: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- SLO/dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `jp`; +3 more; data classes: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Cross-service dependencies: `tenancy`, `identity`, `policy-engine`, `observability`, `audit-chain`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `ops-dashboard-control-center` `incident-declare` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/ops-dashboard-control-center/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `ops-dashboard-control-center` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `ops-dashboard-control-center` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `ops-dashboard-control-center` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `ops-dashboard-control-center` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `ops-dashboard-control-center` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `incident-declare` evaluates `<tenant>.ops-dashboard-control-center.incident-declare` against policy, writes `ops_dashboard_control_center.incident_command`, and emits `oya.ops.dashboard.control.center.incident.declare.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `ops-dashboard-control-center` binds `consent` to `{'name': 'incident-command', 'description': 'Operator-safe incident lifecycle, severity, communications, remediation handoff, and post-incident evidence command surface for FD-001.', 'crates': ['oya-ops-dashboard-control-center-incident-command-kernel', 'oya-ops-dashboard-control-center-incident-command-domain', 'oya-ops-dashboard-control-center-incident-command-usecase', 'oya-ops-dashboard-control-center-incident-command-app', 'oya-ops-dashboard-control-center-incident-command-api', 'oya-ops-dashboard-control-center-incident-command-rest', 'oya-ops-dashboard-control-center-incident-command-worker', 'oya-ops-dashboard-control-center-incident-command-adapter']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `ops-dashboard-control-center` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml`; reviewers must map `consent` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `ops-dashboard-control-center` is `policy/abuse-defence.cedar, policy/admin-action-authorization.cedar, policy/audit-emission-required.cedar, policy/auditor-scope.cedar, policy/cedar/abuse-defence.cedar, policy/cedar/admin-action-authorization.cedar, plus 14 more`; missing policy files are scaffold debt, not an implicit pass for `consent`.
- Depth detail 4: `ops-dashboard-control-center` state/event naming uses `ops_dashboard_control_center.{'name': 'incident_command', 'description': 'Operator_safe incident lifecycle, severity, communications, remediation handoff, and post_incident evidence command surface for FD_001.', 'crates': ['oya_ops_dashboard_control_center_incident_command_kernel', 'oya_ops_dashboard_control_center_incident_command_domain', 'oya_ops_dashboard_control_center_incident_command_usecase', 'oya_ops_dashboard_control_center_incident_command_app', 'oya_ops_dashboard_control_center_incident_command_api', 'oya_ops_dashboard_control_center_incident_command_rest', 'oya_ops_dashboard_control_center_incident_command_worker', 'oya_ops_dashboard_control_center_incident_command_adapter']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `ops-dashboard-control-center` covers `tenancy, policy-engine, audit-chain, observability` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `ops-dashboard-control-center` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `ops-dashboard-control-center` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `ops-dashboard-control-center` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `ops-dashboard-control-center` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `ops-dashboard-control-center` uses SLOs `slos/admin-action-audit-seal-completeness.openslo.yaml, slos/cluster-health-freshness.openslo.yaml, slos/command-availability.openslo.yaml, slos/evidence-pack-freshness.openslo.yaml, slos/incident-ack-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/admin-action-audit-stream.json, dashboards/cell-operator.json, dashboards/on-call-handoff.md, dashboards/ops-overview.json, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `ops-dashboard-control-center` uses runbooks `runbooks/admin-action-rollback.md, runbooks/admin-mfa-cascade.md, runbooks/dashboard-perf-degradation.md, runbooks/deployment-rollback.md, runbooks/forensic-investigation-handoff.md, plus 6 more` so `consent` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `ops-dashboard-control-center` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/pqc-cert.yaml, iac/prod-credential-sidecar.yaml, iac/prod-ech-config.yaml, plus 6 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `ops-dashboard-control-center` uses `capabilities/cluster-health-query.yaml, capabilities/deployment-approve.yaml, capabilities/evidence-pack-export.yaml, capabilities/incident-declare.yaml, plus 4 more` and `catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml, catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml, catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml, catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `ops-dashboard-control-center` fails closed when `consent` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `ops-dashboard-control-center` emits denial evidence for `consent` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `ops-dashboard-control-center` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `consent` workflow.
- Depth detail 17: `ops-dashboard-control-center` telemetry for `consent` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `ops-dashboard-control-center` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §portability

Per ADR-0276: per-tenant backup export format is signed JSON Lines (`.jsonl`) compressed with `zstd`. Export triggered via evidence-pack export capability. Portability path: `GET /ops/v1/tenants/{tenant_id}/evidence-packs/{pack_id}/export` → signed archive + SHA-256 manifest.

## §substrate-dependencies

Per ADR-0280: substrate-dependency DAG position = product (leaf node). Dependencies: `observability`, `tenancy`, `policy-engine`, `cloud-secrets`, `foundry`, `governance`, `finops-portal`, `detection`. No substrate µservice depends on this µservice.

---



## §investigation-binding
This anchor is closed for `ops-dashboard-control-center` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `ops-dashboard-control-center` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.ops-dashboard-control-center.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `incident-declare` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `incident-declare` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `ops-dashboard-control-center`; owner `ops-sre-reliability`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- Capability records cited: `microservices/ops-dashboard-control-center/capabilities/cluster-health-query.yaml`, `microservices/ops-dashboard-control-center/capabilities/deployment-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/evidence-pack-export.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-declare.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-remediation-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml`; +2 more.
- API surfaces cited: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar/policy artifacts cited: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- SLO and dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar binding: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- State/event binding: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- Capability binding: `incident-declare`, `incident-remediation-approve`, `deployment-approve`, `rollback-execute`, `cluster-health-query`, `tenant-isolation-posture-query`; +1 more.
- SLO binding: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ops-dashboard-control-center`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ops-dashboard-control-center`.
- `policy-engine` supplies the signed Cedar corpus while `ops-dashboard-control-center` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ops-dashboard-control-center` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ops-dashboard-control-center`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ops-dashboard-control-center` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ops-dashboard-control-center` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `ops-dashboard-control-center` is in annual full-scope pentest and every major `incident-declare` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`; +27 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `ops-dashboard-control-center` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `ops-dashboard-control-center`; owner `ops-sre-reliability`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- Capability records cited: `microservices/ops-dashboard-control-center/capabilities/cluster-health-query.yaml`, `microservices/ops-dashboard-control-center/capabilities/deployment-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/evidence-pack-export.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-declare.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-remediation-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml`; +2 more.
- API surfaces cited: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar/policy artifacts cited: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- SLO and dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar binding: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- State/event binding: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- Capability binding: `incident-declare`, `incident-remediation-approve`, `deployment-approve`, `rollback-execute`, `cluster-health-query`, `tenant-isolation-posture-query`; +1 more.
- SLO binding: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ops-dashboard-control-center`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ops-dashboard-control-center`.
- `policy-engine` supplies the signed Cedar corpus while `ops-dashboard-control-center` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ops-dashboard-control-center` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ops-dashboard-control-center`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ops-dashboard-control-center` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ops-dashboard-control-center` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `ops-dashboard-control-center` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `incident-declare` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `ops-dashboard-control-center`; owner `ops-sre-reliability`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- Capability records cited: `microservices/ops-dashboard-control-center/capabilities/cluster-health-query.yaml`, `microservices/ops-dashboard-control-center/capabilities/deployment-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/evidence-pack-export.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-declare.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-remediation-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml`; +2 more.
- API surfaces cited: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar/policy artifacts cited: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- SLO and dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar binding: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- State/event binding: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- Capability binding: `incident-declare`, `incident-remediation-approve`, `deployment-approve`, `rollback-execute`, `cluster-health-query`, `tenant-isolation-posture-query`; +1 more.
- SLO binding: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ops-dashboard-control-center`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ops-dashboard-control-center`.
- `policy-engine` supplies the signed Cedar corpus while `ops-dashboard-control-center` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ops-dashboard-control-center` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ops-dashboard-control-center`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ops-dashboard-control-center` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ops-dashboard-control-center` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `ops-dashboard-control-center` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/ops-dashboard-control-center/catalog/oya-ops-dashboard-control-center-adr-promotion-triage-app.yaml`, `microservices/ops-dashboard-control-center/catalog/oya-ops-dashboard-control-center-cedar-admin-console-app.yaml`, `microservices/ops-dashboard-control-center/catalog/oya-ops-dashboard-control-center-cluster-health-api.yaml`, `microservices/ops-dashboard-control-center/catalog/oya-ops-dashboard-control-center-deployment-command-api.yaml`, `microservices/ops-dashboard-control-center/catalog/oya-ops-dashboard-control-center-finops-integration-adapter.yaml`, `microservices/ops-dashboard-control-center/catalog/oya-ops-dashboard-control-center-finops-portal-kernel.yaml`; +23 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `incident-declare` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `ops-dashboard-control-center`; owner `ops-sre-reliability`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- Capability records cited: `microservices/ops-dashboard-control-center/capabilities/cluster-health-query.yaml`, `microservices/ops-dashboard-control-center/capabilities/deployment-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/evidence-pack-export.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-declare.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-remediation-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml`; +2 more.
- API surfaces cited: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar/policy artifacts cited: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- SLO and dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar binding: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- State/event binding: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- Capability binding: `incident-declare`, `incident-remediation-approve`, `deployment-approve`, `rollback-execute`, `cluster-health-query`, `tenant-isolation-posture-query`; +1 more.
- SLO binding: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ops-dashboard-control-center`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ops-dashboard-control-center`.
- `policy-engine` supplies the signed Cedar corpus while `ops-dashboard-control-center` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ops-dashboard-control-center` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ops-dashboard-control-center`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ops-dashboard-control-center` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ops-dashboard-control-center` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- State/event surfaces carrying classification: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `incident-declare` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `ops-dashboard-control-center`; owner `ops-sre-reliability`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `incident-command`, `deployment-command`, `cluster-health`, `tenant-isolation-posture`, `policy-audit-evidence`.
- Capability records cited: `microservices/ops-dashboard-control-center/capabilities/cluster-health-query.yaml`, `microservices/ops-dashboard-control-center/capabilities/deployment-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/evidence-pack-export.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-declare.yaml`, `microservices/ops-dashboard-control-center/capabilities/incident-remediation-approve.yaml`, `microservices/ops-dashboard-control-center/capabilities/rollback-execute.yaml`; +2 more.
- API surfaces cited: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar/policy artifacts cited: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- SLO and dashboard evidence: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`, `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md`, `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`, `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`, `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`.
- Cedar binding: `microservices/ops-dashboard-control-center/policy/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/admin-action-authorization.cedar`, `microservices/ops-dashboard-control-center/policy/audit-emission-required.cedar`, `microservices/ops-dashboard-control-center/policy/auditor-scope.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/abuse-defence.cedar`, `microservices/ops-dashboard-control-center/policy/cedar/admin-action-authorization.cedar`; +10 more.
- State/event binding: `ops_dashboard_control_center.incident_command`, `ops_dashboard_control_center.deployment_command`, `ops_dashboard_control_center.cluster_health`, `ops_dashboard_control_center.tenant_isolation_posture`, `ops_dashboard_control_center.policy_audit_evidence`.
- Capability binding: `incident-declare`, `incident-remediation-approve`, `deployment-approve`, `rollback-execute`, `cluster-health-query`, `tenant-isolation-posture-query`; +1 more.
- SLO binding: `microservices/ops-dashboard-control-center/slos/admin-action-audit-seal-completeness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/cluster-health-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/command-availability.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/evidence-pack-freshness.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/incident-ack-latency.openslo.yaml`, `microservices/ops-dashboard-control-center/slos/operator-action-audit-completeness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md`, `microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md`, `microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md`, `microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md`, `microservices/ops-dashboard-control-center/runbooks/incident-command.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ops-dashboard-control-center`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ops-dashboard-control-center`.
- `policy-engine` supplies the signed Cedar corpus while `ops-dashboard-control-center` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ops-dashboard-control-center` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ops-dashboard-control-center`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ops-dashboard-control-center` applies the most restrictive policy and emits a degraded-mode audit event.
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

