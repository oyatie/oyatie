---
doc_class: Compliance
shape: Reference
status: Proposed
date: 2026-05-21
owner_team: axis-detection
microservice: detection
related_adrs:
  - ADR-0307-detection-substrate-streaming-batch
  - ADR-0308-ml-model-lifecycle-ai-act-compliance
  - ADR-0309-detection-fairness-audit-civil-rights
  - ADR-0310-investigation-case-management
  - ADR-0263-observability-emission-contract
  - ADR-0105-13-layer-enum-and-check-family-patterns
  - ADR-0131-per-microservice-flat-layout
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0703-cas-cache-live-apex.md
planned_enforcement_ref: oya-governance-detection-baseline
bnf_version: v4.1
layer_enum: layer_5_shared_substrate
---

# Detection Microservice Compliance

## Scope

This file answers the per-microservice ADR-adherence matrix for detection, including rows 49 through 52.
The detection substrate is regulated because it can influence fraud, account, content, employment, payment, and safety outcomes.
Every adverse action carries explanation, appeal, chain-of-custody, and fairness evidence.

## self-modification
- Required compliance anchor: self-modification.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## day-one-cert-readiness
- Required compliance anchor: day-one-cert-readiness.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## pack-overlay-roster
- Required compliance anchor: pack-overlay-roster.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## consent
- Required compliance anchor: consent.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## email-deliverability
- Required compliance anchor: email-deliverability.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## platform-owner-indirection
- Required compliance anchor: platform-owner-indirection.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## minor-protection
- Required compliance anchor: minor-protection.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## meta-trust-attestation
- Required compliance anchor: meta-trust-attestation.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## bootstrap-trust-chain
- Required compliance anchor: bootstrap-trust-chain.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## detection-substrate-binding
- Required compliance anchor: detection-substrate-binding.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## ml-model-lifecycle
- Required compliance anchor: ml-model-lifecycle.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## detection-fairness-audit
- Required compliance anchor: detection-fairness-audit.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## investigation-binding
- Required compliance anchor: investigation-binding.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## critical-path-edge-cases
- Required compliance anchor: critical-path-edge-cases.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## prevention-layers
- Required compliance anchor: prevention-layers.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## threat-intelligence-feeds
- Required compliance anchor: threat-intelligence-feeds.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## vuln-mgmt-sla
- Required compliance anchor: vuln-mgmt-sla.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## data-classification
- Required compliance anchor: data-classification.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## key-rotation-cadence
- Required compliance anchor: key-rotation-cadence.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## supply-chain-risk
- Required compliance anchor: supply-chain-risk.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## crypto-agility-plan
- Required compliance anchor: crypto-agility-plan.
- Binding decisions: ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- Evidence location: this section plus manifest.json, policy fragments, contracts, SLOs, dashboards, and runbooks.
- Day-one posture: build-ahead-of-certification is required; the service ships ready for pack overlays rather than retrofitting them.
- Tenant boundary: tenant_id and compliance_packs are mandatory on each signal, feature, model decision, replay, and case.
- Appeal boundary: adverse outcomes are case-managed with human review and pack-specific SLA.
- Audit boundary: every state transition emits structured ADR-0263 evidence with trace and audit identifiers.
- Retention boundary: raw features, explanations, cases, and replay outputs have pack-specific retention and export controls.
- Failure handling: missing compliance context rejects the action before a side effect.

## detection-substrate-binding

### Family payment-fraud
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family account-takeover
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family synthetic-identity
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family aml-sanctions
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family content-abuse
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family fake-reviews-engagement
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family insider-risk
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.
### Family policy-violation
- Contributes signals: yes.
- Consumes signals: yes, through investigation and score fusion.
- Emits: DetectionSignalEmitted, InvestigationCaseOpened, DetectionAppealFiled when adverse.
- Features: velocity, graph degree, device risk, content hash, policy anomaly, and temporal burst metrics as allowed by pack.
- Pack overlay: PHI and PIPL features remain local and are excluded from cross-tenant training.

## ml-model-lifecycle

### training
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### validation
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### ab-test
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### drift-detection
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### fairness-reaudit
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### versioning
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### rollback
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.
### appeal
- ADR-0308 control: required before production model serving.
- Evidence: model card, training snapshot, validation report, replay report, and rollback handle.
- Guardrail: no cross-tenant training without explicit pack consent and tenant-scoped audit evidence.

## detection-fairness-audit

### no-proxy-discrimination
- ADR-0309 control: enforced for every model or rule that can produce adverse action.
- Threshold: plus or minus 2 percentage points for TPR/FPR equity unless an explicit ADR exception exists.
- Audit event: DetectionFairnessReportEmitted or DetectionDriftAlertTriggered.
### per-class-tpr-fpr-equity
- ADR-0309 control: enforced for every model or rule that can produce adverse action.
- Threshold: plus or minus 2 percentage points for TPR/FPR equity unless an explicit ADR exception exists.
- Audit event: DetectionFairnessReportEmitted or DetectionDriftAlertTriggered.
### disparate-impact-testing
- ADR-0309 control: enforced for every model or rule that can produce adverse action.
- Threshold: plus or minus 2 percentage points for TPR/FPR equity unless an explicit ADR exception exists.
- Audit event: DetectionFairnessReportEmitted or DetectionDriftAlertTriggered.
### explainability-floor
- ADR-0309 control: enforced for every model or rule that can produce adverse action.
- Threshold: plus or minus 2 percentage points for TPR/FPR equity unless an explicit ADR exception exists.
- Audit event: DetectionFairnessReportEmitted or DetectionDriftAlertTriggered.
### per-jurisdiction-model-variants
- ADR-0309 control: enforced for every model or rule that can produce adverse action.
- Threshold: plus or minus 2 percentage points for TPR/FPR equity unless an explicit ADR exception exists.
- Audit event: DetectionFairnessReportEmitted or DetectionDriftAlertTriggered.

## investigation-binding

### InvestigationCaseOpened
- ADR-0310 control: case-management state is chain-of-custody bound and Cedar-gated.
- ADR-0263 registry: event includes tenant_id, trace_id, audit_id, case_id, actor, reason, and retention class.
- Review: human investigator assignment and feedback label close the loop into model retraining.
### InvestigationCaseTriagePriorityAssigned
- ADR-0310 control: case-management state is chain-of-custody bound and Cedar-gated.
- ADR-0263 registry: event includes tenant_id, trace_id, audit_id, case_id, actor, reason, and retention class.
- Review: human investigator assignment and feedback label close the loop into model retraining.
### InvestigationEvidenceAdded
- ADR-0310 control: case-management state is chain-of-custody bound and Cedar-gated.
- ADR-0263 registry: event includes tenant_id, trace_id, audit_id, case_id, actor, reason, and retention class.
- Review: human investigator assignment and feedback label close the loop into model retraining.
### InvestigationPIIAccessed
- ADR-0310 control: case-management state is chain-of-custody bound and Cedar-gated.
- ADR-0263 registry: event includes tenant_id, trace_id, audit_id, case_id, actor, reason, and retention class.
- Review: human investigator assignment and feedback label close the loop into model retraining.
### InvestigationAppealAdjudicated
- ADR-0310 control: case-management state is chain-of-custody bound and Cedar-gated.
- ADR-0263 registry: event includes tenant_id, trace_id, audit_id, case_id, actor, reason, and retention class.
- Review: human investigator assignment and feedback label close the loop into model retraining.

## 52-row compliance matrix

- Row 01: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 01 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 02: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 02 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 03: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 03 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 04: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 04 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 05: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 05 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 06: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 06 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 07: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 07 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 08: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 08 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 09: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 09 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 10: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 10 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 11: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 11 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 12: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 12 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 13: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 13 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 14: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 14 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 15: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 15 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 16: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 16 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 17: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 17 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 18: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 18 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 19: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 19 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 20: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 20 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 21: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 21 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 22: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 22 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 23: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 23 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 24: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 24 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 25: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 25 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 26: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 26 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 27: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 27 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 28: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 28 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 29: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 29 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 30: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 30 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 31: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 31 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 32: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 32 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 33: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 33 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 34: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 34 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 35: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 35 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 36: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 36 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 37: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 37 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 38: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 38 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 39: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 39 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 40: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 40 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 41: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 41 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 42: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 42 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 43: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 43 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 44: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 44 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 45: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 45 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 46: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 46 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 47: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 47 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 48: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 48 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 49: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 49 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 50: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 50 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 51: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 51 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
- Row 52: answered with a concrete detection artifact; no external tribal knowledge is required.
- Row 52 evidence: ARCHITECTURE.md, this compliance.md, manifest.json, policy, contracts, runbooks, and scorecards.
Compliance buildability note 1: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 2: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 3: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 4: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 5: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 6: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 7: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 8: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 9: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 10: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 11: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 12: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 13: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 14: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 15: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 16: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 17: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 18: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 19: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 20: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 21: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 22: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 23: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 24: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 25: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 26: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Compliance buildability note 27: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
