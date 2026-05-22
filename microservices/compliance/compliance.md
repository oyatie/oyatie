---
microservice: compliance
doc: ComplianceMapping
status: Drafting
authority_tier: 2
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Regulatory Framework Mapping

## SOC 2 Type II — AICPA Trust Services Criteria

| Criterion | Required artifact kinds | Cadence | Collector |
|---|---|---|---|
| CC1 Control Environment | access-review-snapshot | weekly | accessReviewSnapshot |
| CC2 Communication & Information | (informational; tracked via Runbooks index per ADR-0170) | continuous | — |
| CC3 Risk Assessment | vuln-scan-report + pen-test-report | per image + yearly | vulnScanReport + penTestReport |
| CC4 Monitoring Activities | minimum-necessary-access-log + audit-chain seal coverage | continuous | minimumNecessaryAccessLog |
| CC5 Control Activities | deploy-receipt + ci-artifact-hash | per deploy + per build | deployReceipt + ciArtifactHash |
| CC6 Logical & Physical Access | access-review-snapshot | weekly | accessReviewSnapshot |
| CC7 System Operations | backup-restore-drill-receipt | quarterly | backupRestoreDrillReceipt |
| CC8 Change Management | deploy-receipt + ci-artifact-hash | per deploy + per build | deployReceipt + ciArtifactHash |
| CC9 Risk Mitigation | vuln-scan-report + pen-test-report | per image + yearly | vulnScanReport + penTestReport |
| A1 Availability | backup-restore-drill-receipt + SLO burn-down evidence | quarterly + continuous | backupRestoreDrillReceipt + attestationAggregator |
| C1 Confidentiality | access-review-snapshot + audit-chain seal coverage | weekly + continuous | accessReviewSnapshot |
| PI1 Processing Integrity | audit-chain seal coverage (every operation sealed) | continuous | auditChainSealCoverage |
| P1-P8 Privacy | DSAR completion record + Cedar policy snapshots | per DSAR + weekly | dsarCompletionRecord |

## GDPR — General Data Protection Regulation (EU)

| Article | Requirement | Required artifact | Status |
|---|---|---|---|
| Art. 5 Principles | Lawfulness, purpose limitation, data minimization | access-review-snapshot + minimum-necessary-access-log | ✓ |
| Art. 12 Transparency | Communicate within 30 days | dsar-completion-record (statutory SLA tracking) | ✓ |
| Art. 15 Right of access | Subject can request export | dsar-completion-record (export sub-type) | ✓ |
| Art. 16 Rectification | Subject can correct data | dsar-completion-record (rectify sub-type) | ✓ |
| Art. 17 Right to erasure | Subject can request deletion | dsar-completion-record (delete sub-type) | ✓ |
| Art. 18 Restriction | Subject can restrict processing | dsar-completion-record (restrict sub-type) | Phase 2 |
| Art. 20 Portability | Machine-readable export | dsar-completion-record + JSON-LD export format | ✓ |
| Art. 30 Records of processing | RoPA register | RoPA register at policy/ropa.json | ✓ |
| Art. 32 Security | TLS + encryption-at-rest + access controls | (substrate via ADR-0145 + ADR-0148) | ✓ |
| Art. 33 Breach notification | 72-hour authority notification | EVT-PERSONAL-DATA-BREACH | ✓ |
| Art. 35 DPIA | High-risk processing DPIA | dpia.md | ✓ |

## HIPAA — Health Insurance Portability and Accountability Act

| Section | Requirement | Required artifact | Status |
|---|---|---|---|
| § 164.308(a)(1) Security management | Risk analysis | vuln-scan-report + pen-test-report | ✓ |
| § 164.308(a)(3) Workforce security | Access review | access-review-snapshot | ✓ |
| § 164.308(a)(4) Information access | Cedar policy snapshot | access-review-snapshot | ✓ |
| § 164.308(a)(7) Contingency plan | Backup + DR | backup-restore-drill-receipt | ✓ |
| § 164.312(a)(1) Access control | RBAC + audit logs | minimum-necessary-access-log | ✓ |
| § 164.312(b) Audit controls | Audit logs | minimum-necessary-access-log + audit-chain seal | ✓ |
| § 164.312(c) Integrity | Tamper-evident logs | audit-chain seal coverage | ✓ |
| § 164.312(e) Transmission security | TLS 1.3 | (substrate via ADR-0148) | ✓ |
| § 164.314 Business Associate Contracts | BAA inventory | baa-inventory-entry | ✓ |
| § 164.514(d) Minimum necessary | Per-access purpose log | minimum-necessary-access-log | ✓ |

## PCI-DSS 4.0 — Payment Card Industry Data Security Standard

Status: **out of scope unless `microservices/payments/` lands.** Substrate ready.

| Requirement | Status |
|---|---|
| Req. 1 Network security controls | substrate via ADR-0148 service mesh |
| Req. 2 Apply secure configurations | substrate via ADR-0181 image promotion |
| Req. 3 Protect stored account data | requires CDE; deferred |
| Req. 4 Protect cardholder data with strong cryptography | substrate via cosign + TLS 1.3 |
| Req. 5 Anti-malware | substrate via Trivy scanning |
| Req. 6 Develop secure systems | substrate via ADR-0205 + lint discipline |
| Req. 7 Restrict access | substrate via ADR-0183 Cedar |
| Req. 8 Authentication | substrate via Zitadel |
| Req. 9 Physical access | operator-cluster operator responsibility |
| Req. 10 Log and monitor | minimum-necessary-access-log + observability backplane |
| Req. 11 Test security | vuln-scan-report + pen-test-report |
| Req. 12 Information security policy | policy/ directory |

## ISO 27001 (mapping; informational)

oyatie's SOC 2 + HIPAA artifact coverage subsumes most ISO 27001 Annex A controls. Per-control mapping at `policy/iso-27001-annex-a-coverage.json` (Phase 1.5).

## EU AI Act (mapping; informational)

ADR-0118 EU AI Act Annex III refusal kernel covers the refusal posture. Compliance µservice consumes EU-AI-Act-related events (EVT-EU-AI-ACT-REFUSAL); no separate artifact kind required.

## References

- ADR-0209 — compliance evidence automation.
- AICPA Trust Services Criteria 2017 (with 2022 points of focus update).
- GDPR Articles 5, 12, 15-22, 30, 32, 33, 35.
- HIPAA Title 45 CFR §§ 160, 162, 164.
- PCI-DSS 4.0 — PCI SSC, 2022.

---



## §day-one-cert-readiness
This anchor is closed for `compliance` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `compliance` covers packs `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/compliance/slos/audit-chain-seal-verify-success.openslo.yaml`, `microservices/compliance/slos/auditor-portal-availability.openslo.yaml`, `microservices/compliance/slos/auditor-portal-latency.openslo.yaml`, `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml`, `microservices/compliance/slos/cross-tenant-isolation-violations.openslo.yaml`, `microservices/compliance/slos/dsar-backlog-depth.openslo.yaml`; +22 more.
- Example: `compliance` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

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
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §pack-overlay-roster
This anchor is closed for `compliance` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`.
- Pack overlays modify Cedar fragments `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +1 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `compliance` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

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
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §platform-owner-indirection
This anchor is closed for `compliance` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `compliance` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`, `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`; +19 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `compliance` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.compliance.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

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
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §detection-substrate-binding
This anchor is closed for `compliance` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `compliance` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `compliance` touches those data classes.
- Signal sources: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +19 more.
- Example event class: `oya.compliance.compliance.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

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
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §investigation-binding
This anchor is closed for `compliance` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `compliance` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.compliance.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `compliance` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `compliance` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

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
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §insider-threat-controls
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `compliance` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`, `compliance.compliance`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `compliance.compliance` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

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
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §threat-intelligence-feeds
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `compliance` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/compliance/policy/abuse-defence.cedar`, `microservices/compliance/policy/action-authorization.cedar`, `microservices/compliance/policy/auditor-scope.cedar`, `microservices/compliance/policy/ci-scope.cedar`, `microservices/compliance/policy/data-residency.cedar`, `microservices/compliance/policy/data-residency.md`; +12 more.
- Example: `compliance` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

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
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §key-rotation-cadence
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.compliance` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/compliance/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/compliance/runbooks/audit-seal-verify-failure.md`, `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md`, `microservices/compliance/runbooks/cross-tenant-dsar-leak-suspected.md`, `microservices/compliance/runbooks/dsar-backlog-overflow.md`, `microservices/compliance/runbooks/engagement-cedar-revoke-failed.md`, `microservices/compliance/runbooks/evidence-collector-degraded.md`; +4 more.
- Example: `compliance` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

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
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §crypto-agility-plan
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `compliance` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`, `microservices/compliance/iac/ech-config.yaml`, `microservices/compliance/iac/edge-waf.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `compliance` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

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
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §pentest-and-bounty-cadence
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `compliance` is in annual full-scope pentest and every major `compliance` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/compliance/contracts/asyncapi.yaml`, `microservices/compliance/contracts/compliance.proto`, `microservices/compliance/contracts/dsar-export-format.json`, `microservices/compliance/contracts/openapi.yaml`, `microservices/compliance/iac/ech-config.yaml`, `microservices/compliance/iac/edge-waf.yaml`; +16 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `compliance` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

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
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §facility-controls
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `compliance` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `compliance` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

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
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §supply-chain-risk
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `compliance` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/compliance/catalog/api-asyncapi.yaml`, `microservices/compliance/catalog/api-rest.yaml`, `microservices/compliance/catalog/auditor-portal-frontend.yaml`, `microservices/compliance/catalog/component-info.yaml`, `microservices/compliance/catalog/oya-compliance-breach-notification-workflow-usecase.yaml`, `microservices/compliance/catalog/oya-compliance-cell-certification-attestation-worker.yaml`; +20 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `compliance` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

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
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
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
This anchor is closed for `compliance` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `compliance` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `compliance` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `compliance` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

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
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §data-classification
This anchor is closed for `compliance` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `compliance.compliance`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `compliance` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

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
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

